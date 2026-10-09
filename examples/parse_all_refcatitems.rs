//! Demonstrate that we can parse a plate dump file without crashing.

use dasch_backend;
use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {

    for arg in env::args_os().skip(1) {
        let n = dasch_backend::parse_dump_file::<dasch_backend::refcat_dump::RefcatItemRecord, _>(&arg)?;
        println!("{} records in {}", n, arg.display());
    }

    Ok(())
}
