//! Dump an entire photometry data file in Rust's debugging format.

use binary_serde::{BinarySerde, Endianness};
use clap::Parser;
use dasch_backend::phot_data::MagRecord;
use std::{
    error::Error,
    fs::File,
    io::{ErrorKind, Read, Seek, SeekFrom},
    path::PathBuf,
};

#[derive(Parser, Debug)]
struct Args {
    /// The name of the file to dump
    #[arg(short, long)]
    photdata_file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut file = File::open(&args.photdata_file)?;

    // Skip the big header record
    file.seek(SeekFrom::Start(12320))?;

    let mut buf = vec![0; MagRecord::SERIALIZED_SIZE];

    loop {
        let r = file.read_exact(&mut buf);

        if let Err(ref e) = r {
            if e.kind() == ErrorKind::UnexpectedEof {
                break;
            }
        }

        let _ = r?;
        let rec = MagRecord::binary_deserialize(&buf, Endianness::Little).unwrap();
        println!("{rec:?}");
    }

    Ok(())
}
