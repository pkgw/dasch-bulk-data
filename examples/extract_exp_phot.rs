//! Demonstrate how to extract photometry data for (part of) a single exposure,
//! including uncataloged sources (which might possibly be transients, but are
//! probably defects).
//!
//! The fundamental layout of the DASCH photometry data files makes this a
//! *very* inefficient operation, no matter what language you're using. If you
//! want to analyze the photometry on an exposure-by-exposure (or
//! plate-by-plate) basis, you should expect to need to reprocess the
//! DASCH-provided files to reorganize the data in a way that allows for more
//! efficient access patterns.

use binary_serde::{BinarySerde, Endianness};
use clap::Parser;
use dasch_bulk_data::{
    phot_data::{MagRecord},
    series_ids::PLATE_ID_BY_SERIES,
};
use std::{
    error::Error,
    fs::File,
    io::{ErrorKind, Read, Seek, SeekFrom},
    path::PathBuf,
};

#[derive(Parser, Debug)]
struct Args {
    /// Series of the exposure to look up.
    #[arg(long)]
    series: String,

    /// Number of the exposure to look up.
    #[arg(long)]
    plate_num: u32,

    /// Solution number of the exposure to look up.
    #[arg(long)]
    sol_num: u8,

    /// The GSC bin index to query.
    #[arg(short, long)]
    gsc_bin_index: u32,

    /// The name of the photometry "mega-index" file
    #[arg(short, long)]
    mega_index_file: PathBuf,

    /// The name of the photometry data file
    #[arg(long)]
    photdata_file: PathBuf,
}

/// This is pretty dumb, but lets use reuse the binary-serde machinery.
#[derive(BinarySerde)]
struct Offset(pub u32);

impl Offset {
    /// This might also be dumb, but yes, we treat the stored data as u32s, but
    /// then we convert to i32. That way we get a panic if a size is too big.
    pub fn decode(buf: &[u8]) -> i32 {
        Self::binary_deserialize(&buf[..4], Endianness::Little)
            .unwrap()
            .0 as i32
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let series_id = *PLATE_ID_BY_SERIES
        .get(&args.series)
        .ok_or_else(|| format!("unrecognized plate series `{}`", args.series))?;

    // Use the mega-index to determine our offsets

    let (ofs1, ofs2) = {
        let mut file = File::open(&args.mega_index_file)?;
        file.seek(SeekFrom::Start(args.gsc_bin_index as u64 * 4))?;

        let mut buf = [0; 8];
        file.read_exact(&mut buf)?;

        (Offset::decode(&buf[..4]), Offset::decode(&buf[4..]))
    };

    // If ofs2 < ofs1, that must mean that the tranche of interest extends to
    // the end of its associated photdata file. rec_count will only be
    // meaningful if read_to_end is false.

    let read_to_end = ofs2 < ofs1;

    let rec_count = if read_to_end {
        0
    } else {
        (ofs2 - ofs1) as usize / MagRecord::SERIALIZED_SIZE
    };

    // Now process the actual phot data

    let mut file = File::open(&args.photdata_file)?;
    file.seek(SeekFrom::Start(ofs1 as u64))?;

    let mut buf = vec![0; MagRecord::SERIALIZED_SIZE];
    let mut n = 0;

    loop {
        // Read data, maybe. We have two stopping conditions depending on
        // read_to_end.

        if !read_to_end && n >= rec_count {
            break;
        }

        let r = file.read_exact(&mut buf);

        if let Err(ref e) = r {
            if read_to_end && e.kind() == ErrorKind::UnexpectedEof {
                break;
            }
        }

        let _ = r?;
        n += 1;

        // OK, we've got a new record. Is it relevant to our plate? This is why
        // this operation is so inefficient: we have to read the data basically
        // every single exposure that touches this sky location. If a plate has
        // multiple exposures and you want to look up all of the sources on a
        // particular physical patch of the plate, you have to do a set of scans
        // for every set of RA/dec coordinates mapping to that physical
        // location.

        let rec = MagRecord::binary_deserialize(&buf, Endianness::Little).unwrap();

        if rec.series_id != series_id || rec.plate_number != args.plate_num || rec.solution_number != args.sol_num {
            continue;
        }

        // Report. Uncataloged sources have ref_number = 0.

        println!("{} {} {} {} {} {}", rec.ref_number, rec.x_image, rec.y_image, rec.ra_deg, rec.dec_deg, rec.magcal_magdep);
    }

    Ok(())
}
