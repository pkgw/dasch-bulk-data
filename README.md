# dasch-backend

This repository provides a minimal skeleton of Rust code that can parse the
files that are provided to people who obtain copies of the [DASCH DR7]
bulk-access data files.

[DASCH DR7]: https://dasch.cfa.harvard.edu/dr7/

Even if you don't use Rust, you can examine the code here to understand the file
formats and how to make use of them.


## Examples

The `examples/` directory contains several example programs that show how to
use the bulk-data files. **However**, these programs are meant for instruction
only — they will be devastatingly inefficient for the sorts of analyses that
would motivate you to download the bulk data files in the first place

### The Plates Database Dump

The example program `print_plate_info` scans through the plates database dump
file (`plates.json.gz`) and prints out a tiny amount of information about a
specific plate. Run it like this:

```
cargo run --example print_plate_info -- --dump-file ./plates.json.gz --plate-id a10000
```

To understand the structure of the data contained in the plates database,
examine `src/plates_dump.rs` and cross-reference with the DR7 documentation.

### The Reference Catalog Dump

The example program `print_source_info` scans through the a reference catalog
database dump file (`refcat_apass.json.gz` or `refcat_atlas.json.gz`) and prints
out a small amount of information about a specified source. Run it like this:

```
cargo run --example print_source_info -- --dump-file ./refcat_apass.json.gz --refnum 409523462451513
```

... where the source of interest is identified by its reference number. This
will be excruciatingly slow unless you pick a source from the very beginning of
the file.

To understand the structure of the data contained in the refcat databases,
examine `src/refcat_dump.rs` and cross-reference with the DR7 documentation.

### The Lightcurve Data

The example program `extract_lightcurve` shows how to extract a lightcurve from
the bulk-data files. Unlike the programs above, it is quite fast, but it needs
to be given information that is printed out by `print_source_info`.

Run it like this:

```
cargo run --example extract_lightcurve -- \
  --refnum 409523462451513 \
  --ra-deg 148.144214 \
  --dec-deg=-45.253474 \
  --lim-index-path apass_lim_aws.idx \
  --lim-data-path apass_limiting.dat \
  --photdata-file apass_mags/24473600.dat \
  --photdata-offset 65898560 \
  --photdata-count 131
```

This program reads binary records out of the specified photometry data file. To
understand the contents of these records, examine `src/phot_data.rs` and
cross-reference with the DR7 documentation.

This program also reads binary records from the refcat-wide limiting magnitude
data file, since any proper lightcurve analysis must account for nondetections
as well as detections. The structures of this file and its associated index are
simple, but you should consult the source code to this example to understand how
to use them.

For some purposes, you might wish to make use of these limiting-magnitude files
on their own, without needing the bulk lightcurve data.
