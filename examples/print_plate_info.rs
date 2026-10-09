//! Demonstrate how to read a local copy of the plate database dump.
//!
//! You should not use this code for anything besides exploratory testing. For
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
    /// Path to the gzipped plate-data JSON-lines file
    #[arg(short, long)]
    dump_file: PathBuf,

    /// Name of the plate to look up
    #[arg(short, long)]
    plate_id: String,
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
        let rec: dasch_backend::plates_dump::PlatesRecord = serde_dynamo::from_item(item)?;

        if rec.plate_id == args.plate_id {
            println!("{} mosaic? {}", args.plate_id, if rec.mosaic.is_none() { "no" } else { "yes" });
            println!("{} astrometry? {}", args.plate_id, if rec.astrometry.is_none() { "no" } else { "yes" });
            println!("{} photometry? {}", args.plate_id, if rec.photometry.is_none() { "no" } else { "yes" });
            return Ok(());
        }
    }

    Err(format!("plate ID {} not found in the entire dump file", args.plate_id).into())
}
