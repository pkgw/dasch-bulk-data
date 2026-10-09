use binary_serde::{BinarySerde, Endianness};
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::{
    error::Error, fs::File, io::{BufRead, BufReader, Read, Seek, SeekFrom}, path::Path,
};

pub mod gscbin;
pub mod phot_data;
pub mod plates_dump;
pub mod refcat_dump;

/// Sample code to parse an AWS DynamoDB dump file, doing nothing but counting
/// the number of records.
///
/// As written this function just tests that we can parse everything in the file
/// without crashing.
pub fn parse_dump_file<'a, T: Deserialize<'a>, P: AsRef<Path>>(
    path: P,
) -> Result<usize, Box<dyn Error>> {
    let mut n = 0;
    let f = File::open(path)?;
    let gz = GzDecoder::new(f);
    let buf = BufReader::new(gz);

    for maybe_line in buf.lines() {
        let line = maybe_line?;

        // So gross! Our dump lines have the form `{"Item":{...}}`, but the
        // serde_dynamo Item deserializer doesn't want to see that wrapper.
        // We're going straight from text to an Item, though, so there isn't an
        // intermediate data structure that we can futz with. But ... the format
        // of every line is identical, so we can strip out the item wrapper at a
        // textual level. Fun.
        if !line.starts_with("{\"Item\":{") {
            return Err("item wrapper 1 failure".into());
        }

        if !line.ends_with("}}") {
            return Err("item wrapper 2 failure".into());
        }

        let noitem_line = &line[8..line.len() - 1];

        let item: serde_dynamo::Item = serde_json::from_str(noitem_line).expect("parse item");
        let _: T = serde_dynamo::from_item(item).expect("parse object");
        n += 1;
    }

    Ok(n)
}

/// Sample code to load lightcurve detection data for a specific source.
/// 
/// This is basically easy. In the refcat database, every source record has a
/// unique ref_number and a gsc_bin_index number (derived from its catalog RA
/// and Dec). If there are any detections of the source, it has a photometry
/// record giving a file offset and record count.
/// 
/// The name of the file to load is derived from the gsc_bin_index number,
/// rounded down to the nearest multiple of 1024. We seek into that file using
/// the cataloged byte offset, and read the cataloged number of records.
/// 
/// *However*, this only handles the detections. Any proper lightcurve analysis
/// also needs to factor in nondetections. The data loaded here must be combined
/// with the data from [`load_limit_records`] to assemble a complete
/// lightcurve.
pub fn load_lightcurve_detection_records(
    gsc_bin_index: u32,
    ref_number: u64,
    phot_offset: u32,
    phot_count: u32,
) -> Result<Vec<phot_data::MagRecord>, Box<dyn Error>> {
    let file_number = 1024 * (gsc_bin_index >> 10);
    let path = format!("{file_number}.dat");

    let mut file = File::open(&path)?;
    file.seek(SeekFrom::Start(phot_offset as u64))?;

    let mut buf = vec![0; phot_data::MagRecord::SERIALIZED_SIZE];
    let mut recs = Vec::new();

    for _ in 0..phot_count {
        file.read_exact(&mut buf)?;
        let rec = phot_data::MagRecord::binary_deserialize(&buf, Endianness::Little).unwrap();

        if rec.ref_number != ref_number {
            return Err("file consistency failure".into());
        }

        recs.push(rec);
    }

    Ok(recs)
}

/// Sample code to load upper-limit data for a given sky position.
/// 
/// This information might be interesting on its own, but you also need to use
/// these data if you're extracting the lightcurve of a DASCH source, so that
/// you know about nondetections as well as detections.
/// 
/// The limit data are binned using a level-2 GSC binning, which is *not* the
/// same as the binning used for the lightcurve source data. So we must compute
/// the bin index on the fly --- it would not be correct to use the
/// gsc_bin_index from one of the refcat tables.
/// 
/// The GSC bins are indexed in a simple file called `lim_aws.idx`. It stores a
/// byte offset and byte length used to look up the per-bin records in the main
/// data file.
/// 
/// For DASCH lightcurve analysis, you should first look up the detections, then
/// read the upper limit data. For any unique combination of (series_id,
/// plate_number, solution_number) that does *not* have a lightcurve detection,
/// you should record an upper limit data point.
pub fn load_limit_records<P1: AsRef<Path>, P2: AsRef<Path>>(
    ra_deg: f64,
    dec_deg: f64,
    aws_index_path: P1,
    data_path: P2,
) -> Result<Vec<phot_data::LimitsPlateRecord>, Box<dyn Error>> {
    // It is not efficient to compute the binning lookup table from scratch
    // every time. Real code should precompute this and store it.
    let bin2 = gscbin::GscBinning::new2();

    let dec_bin = bin2.get_dec_bin(dec_deg);
    let total_bin = bin2.get_total_bin(dec_bin, ra_deg);

    // Look up the bin in the index

    let rec = {
        let mut f_index = File::open(aws_index_path)?;
        f_index.seek(SeekFrom::Start((total_bin * phot_data::LimitsIndexRecord::SERIALIZED_SIZE) as u64))?;
        let mut buf = [0u8; phot_data::LimitsIndexRecord::SERIALIZED_SIZE];
        f_index.read_exact(&mut buf)?;
        phot_data::LimitsIndexRecord::binary_deserialize(&buf, Endianness::Little).map_err(|e| format!("deserialize failed: {e}"))?
    };

    // Now we can fetch the actual payload

    let mut f_data = File::open(data_path)?;
    f_data.seek(SeekFrom::Start(rec.byte_offset))?;

    let mut buf = vec![0; phot_data::LimitsPlateRecord::SERIALIZED_SIZE];
    let count = rec.byte_length / phot_data::LimitsPlateRecord::SERIALIZED_SIZE as u64;
    let mut recs = Vec::new();

    for _ in 0..count {
        f_data.read_exact(&mut buf)?;
        let rec = phot_data::LimitsPlateRecord::binary_deserialize(&buf, Endianness::Little).unwrap();
        recs.push(rec);
    }

    Ok(recs)
}
