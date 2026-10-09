use flate2::read::GzDecoder;
use serde::Deserialize;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

pub mod phot_data;
pub mod plates_dump;
pub mod refcat_dump;

/// Parse an AWS DynamoDB dump file, doing nothing but counting the number of
/// records.
///
/// The purpose of this function is just to test that we can parse everything in
/// the file without crashing.
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
