//! Demonstrate how to read a local copy of a refcat database dump.
//!
//! You should not use this code for anything besides exploratory testing! For
//! almost all realistic use cases, the proper way to use the database dump file
//! will be to load it into a true database of your own (SQLite, PostgreSQL,
//! etc.) and query that database. As written, this code has to read and
//! decompress the full dump file from the start in order to find any record,
//! which is wildly inefficient.

use clap::Parser;
use dasch_backend;
use flate2::read::GzDecoder;
use std::{error::Error, fs::File, io::{BufRead, BufReader}, path::PathBuf};

#[derive(Parser, Debug)]
struct Args {
    /// Path to the gzipped refcat JSON-lines file
    #[arg(short, long)]
    dump_file: PathBuf,

    /// Refnum (ID) of the source to look up.
    #[arg(short, long)]
    refnum: u64,
}


fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let f = File::open(&args.dump_file)?;
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

        let item: serde_dynamo::Item = serde_json::from_str(noitem_line)?;
        let rec: dasch_backend::refcat_dump::RefcatItemRecord = serde_dynamo::from_item(item)?;

        if rec.ref_number == args.refnum {
            println!("{} ra = {} deg", args.refnum, rec.ra);
            println!("{} dec = {} deg", args.refnum, rec.dec);
            println!("{} gsc_bin_index = {}", args.refnum, rec.gsc_bin_index);

            if let Some(p) = rec.phot {
                let file_number = 1024 * (rec.gsc_bin_index >> 10);
                println!("{} phot_data_filename = {}.dat (based on gsc_bin_index)", args.refnum, file_number);
                println!("{} phot_data_byte_offset = {}", args.refnum, p.offset);
                println!("{} phot_data_record_count = {}", args.refnum, p.count);
            } else {
                println!("{} has no detections in DASCH", args.refnum);
            }

            return Ok(());
        }
    }

    Err(format!("source refnum (ID) {} not found in the entire dump file", args.refnum).into())
}
