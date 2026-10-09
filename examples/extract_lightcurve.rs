//! Demonstrate how to extract DASCH lightcurve data for a cataloged source.
//!
//! Unlike some of the other examples, this code is fast as-is, because the
//! DASCH data files are designed to ensure that precisely this operation is
//! speedy in the APIs. If you want to extract your own lightcurves from the
//! bulk data files, your code should basically follow this structure. You will
//! probably want to look at *many* more of the columns available for each
//! lightcurve point, though.

use binary_serde::{BinarySerde, Endianness};
use clap::Parser;
use dasch_bulk_data::{
    gscbin::GscBinning,
    phot_data::{LimitsIndexRecord, LimitsPlateRecord, MagRecord},
};
use std::{
    collections::{HashMap, hash_map::Entry},
    error::Error,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};

#[derive(Parser, Debug)]
struct Args {
    /// Refnum (ID) of the source to look up.
    #[arg(short, long)]
    refnum: u64,

    /// The source's RA in degrees
    #[arg(long)]
    ra_deg: f64,

    /// The source's dec in degrees
    #[arg(long)]
    dec_deg: f64,

    /// The name of the file containing the source's photometry data
    #[arg(long)]
    photdata_file: Option<PathBuf>,

    /// The byte offset for the source's photometry data given by the refcat
    #[arg(long)]
    photdata_offset: Option<u64>,

    /// The number of records in the source's photometry data given by the refcat
    #[arg(long)]
    photdata_count: Option<u64>,

    /// The path to the limiting-mag index file
    #[arg(long)]
    lim_index_path: PathBuf,

    /// The path to the limiting-mag data file
    #[arg(long)]
    lim_data_path: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Key {
    series_id: u8,
    plate_num: u32,
    sol_num: u8,
}

#[derive(Clone, Debug)]
struct MiniDetection {
    jd: f64,
    magcal_magdep: f64,
    ra_deg: f64,
    dec_deg: f64,
}

#[derive(Clone, Debug)]
struct MiniUpperLimit {
    jd: f64,
    lim_mag: f64,
}

#[derive(Clone, Debug)]
enum MiniLightcurvePoint {
    Detection(MiniDetection),
    UpperLimit(MiniUpperLimit),
}

impl MiniLightcurvePoint {
    fn jd(&self) -> f64 {
        match self {
            MiniLightcurvePoint::Detection(d) => d.jd,
            MiniLightcurvePoint::UpperLimit(u) => u.jd,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // The buffer for the results:

    let mut points = HashMap::new();

    // Load the lightcurve detections, if there are any. For many sources there
    // are not!

    match (
        args.photdata_file,
        args.photdata_offset,
        args.photdata_count,
    ) {
        (Some(p_file), Some(p_offset), Some(p_count)) => {
            let mut file = File::open(&p_file)?;
            file.seek(SeekFrom::Start(p_offset))?;

            let mut buf = vec![0; MagRecord::SERIALIZED_SIZE];

            for _ in 0..p_count {
                file.read_exact(&mut buf)?;
                let rec = MagRecord::binary_deserialize(&buf, Endianness::Little).unwrap();

                if rec.ref_number != args.refnum {
                    return Err("file consistency failure".into());
                }

                let key = Key {
                    series_id: rec.series_id,
                    plate_num: rec.plate_number,
                    sol_num: rec.solution_number,
                };

                let datum = MiniLightcurvePoint::Detection(MiniDetection {
                    jd: rec.date_jd,
                    magcal_magdep: rec.magcal_magdep as f64,
                    ra_deg: rec.ra_deg,
                    dec_deg: rec.dec_deg,
                });

                points.insert(key, datum);
            }
        }

        (None, None, None) => {}

        _ => {
            return Err("either supply all three --photdata-* arguments, or none".into());
        }
    }

    // Load the upper limits. To do this we need to look up its index, based on
    // the binning index of the source in the coarse "level 2" GSC binning.

    let bin2 = GscBinning::new2();
    let dec_bin = bin2.get_dec_bin(args.dec_deg);
    let gsc2_bin_index = bin2.get_total_bin(dec_bin, args.ra_deg);

    let lim_bin_rec = {
        let mut f_index = File::open(&args.lim_index_path)?;
        f_index.seek(SeekFrom::Start(
            (gsc2_bin_index * LimitsIndexRecord::SERIALIZED_SIZE) as u64,
        ))?;
        let mut buf = [0u8; LimitsIndexRecord::SERIALIZED_SIZE];
        f_index.read_exact(&mut buf)?;
        LimitsIndexRecord::binary_deserialize(&buf, Endianness::Little)
            .map_err(|e| format!("deserialize failed: {e}"))?
    };

    // Now we can fetch the actual limit-data payload, and load any needed
    // limits.

    {
        let mut f_data = File::open(args.lim_data_path)?;
        f_data.seek(SeekFrom::Start(lim_bin_rec.byte_offset))?;

        let mut buf = vec![0; LimitsPlateRecord::SERIALIZED_SIZE];
        let count = lim_bin_rec.byte_length / LimitsPlateRecord::SERIALIZED_SIZE as u64;

        for _ in 0..count {
            f_data.read_exact(&mut buf)?;
            let rec = LimitsPlateRecord::binary_deserialize(&buf, Endianness::Little).unwrap();

            let key = Key {
                series_id: rec.series_id as u8,
                plate_num: rec.plate_number,
                sol_num: rec.solution_number as u8,
            };

            let e = points.entry(key);

            match e {
                Entry::Occupied(_) => {
                    // This plate/solution has a detection. If you want to know
                    // the mosaic_number associated with that data point, you
                    // have to pull it out of the limit record here.
                }

                Entry::Vacant(v) => {
                    // No detection, at least as far as the pipeline's
                    // source-matching efforts can tell. Fill in the upper limit.
                    v.insert(MiniLightcurvePoint::UpperLimit(MiniUpperLimit {
                        jd: rec.geo_jd,
                        lim_mag: rec.limiting_mag_local,
                    }));
                }
            }
        }
    }

    // Finally, sort and emit

    let mut outputs: Vec<_> = points.drain().collect();
    outputs.sort_by(|a, b| a.1.jd().partial_cmp(&b.1.jd()).unwrap());

    for (key, datum) in outputs {
        match datum {
            MiniLightcurvePoint::Detection(d) => {
                println!(
                    "{:.7} {:2} {:5} {} det {} {} {}",
                    d.jd, key.series_id, key.plate_num, key.sol_num, d.magcal_magdep, d.ra_deg, d.dec_deg
                );
            }

            MiniLightcurvePoint::UpperLimit(u) => {
                println!(
                    "{:.7} {:2} {:5} {} lim {}",
                    u.jd, key.series_id, key.plate_num, key.sol_num, u.lim_mag
                );
            }
        }
    }

    Ok(())
}
