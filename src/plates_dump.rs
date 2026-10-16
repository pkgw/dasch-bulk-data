//! Type definitions for deserializing the DASCH DR7 plates database dump.
//!
//! The contents of this dump power the [exposures query API][1].
//!
//! [1]: https://dasch.cfa.harvard.edu/dr7/web-apis/#post-dasch-dr7-queryexps
//!
//! The dump comes as a single gzipped JSON-lines file, where every line is a
//! record in the table serialized in the [DynamoDB typed JSON format][2].
//!
//! [2]: https://aws.amazon.com/blogs/database/exploring-amazon-dynamodb-sdk-clients/

use serde::Deserialize;

/// A top-level record in the plates database.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlatesRecord {
    pub astrometry: Option<PlatesAstrometryRecord>,
    pub mosaic: Option<PlatesMosaicRecord>,
    pub photometry: Option<PlatesPhotometryData>,
    pub plate_id: String,
    pub plate_number: usize,
    pub series: String,
}

/// Information about a plate's astrometric solution.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlatesAstrometryRecord {
    #[serde(default, with = "serde_bytes")]
    pub result_id: Vec<u8>,

    #[serde(default, with = "serde_bytes")]
    // should be Option<>, but not sure how to nest the custom deserializer
    pub b01_header_gz: Vec<u8>,
    pub n_solutions: Option<usize>,
    pub rotation_delta: Option<isize>,
    pub exposures: Vec<Option<PlatesExposureRecord>>,
}

/// Information about an exposure associated with a plate.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlatesExposureRecord {
    /// The exposure number. Zero-based.
    pub number: i8,

    /// The DR7 breakdown of these values is: 429,565 "imWCS"; 155,760
    /// "Logbook"; 26,443 None; 12,531 "" (empty string); 5,707 "Catalog". If
    /// the value is None, ra_deg and dec_deg are always None as well. If it is
    /// the empty string, there are 10,107 cases where the RA and Dec are None,
    /// and 2,423 where they are both Some(0), and one case where they are both
    /// -99. It seems overwhelmingly likely that those zero RA/Decs are all
    /// invalid, so in other words, either a None or an empty string here
    /// implies that we don't have valid RA/Dec info.
    pub center_source: Option<String>,
    pub ra_deg: Option<f64>,
    pub dec_deg: Option<f64>,

    /// The DR7 breakdown of these values is: 603,925 "Logbook"; 26,599
    /// "Catalog"; 112 None. There is one case where the source is "Catalog" but
    /// the midpoint date is None (misc10078) and one case where the source is
    /// None but the midpoint date is defined (ctio00403). Otherwise, a None
    /// here implies a None for midpoint_date, and Some implies Some.
    pub date_source: Option<String>,

    /// These are in ISO8601 format. These are all in UTC, but only some of the
    /// records end with an explicit "Z"; the rest have no explicit timezone
    /// offset indicator. Some records do not include the seconds specifier. A
    /// few hundred records have "seconds" fields of `60.0`, which most parsers
    /// will reject. (Technically you can have a this during a negative leap
    /// second, but the instances of 60.0 in this database are clearly mistakes
    /// due to rounding in whatever code generated the date strings.)
    pub midpoint_date: Option<String>,

    /// Estimated accuracy of the midpoint date, in days.
    pub date_acc_days: Option<f64>,

    /// Exposure duration in minutes.
    pub dur_min: Option<f64>,
}

/// Information about a mosaic (DASCH scanned image) associated with a plate.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlatesMosaicRecord {
    #[serde(default, with = "serde_bytes")]
    pub result_id: Vec<u8>,

    #[serde(rename = "b01OrigFileMD5", with = "serde_bytes")]
    pub b01_orig_file_md5: Vec<u8>,
    pub b01_orig_file_size: u64,

    #[serde(rename = "b01DataMD5", with = "serde_bytes")]
    pub b01_data_md5: Vec<u8>,

    #[serde(rename = "b16OrigFileMD5", with = "serde_bytes")]
    pub b16_orig_file_md5: Vec<u8>,
    pub b16_orig_file_size: u64,

    #[serde(rename = "b16DataMD5", with = "serde_bytes")]
    pub b16_data_md5: Vec<u8>,

    pub b01_height: usize,
    pub b01_width: usize,
    pub creation_date: String,
    pub mos_num: i8,
    pub scan_num: i8,
    pub legacy_comment: Option<String>,
    pub legacy_rotation: u16,
    pub legacy_bucket: String,
    pub s3_key_template: String,
}

/// High-level information about the photometric solutions associated
/// with a plate.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlatesPhotometryData {
    pub median_colorterm_apass: Option<f32>,
    pub median_colorterm_atlas: Option<f32>,
    pub n_magdep_apass: Option<u8>,
    pub n_magdep_atlas: Option<u8>,
    pub n_solutions_apass: Option<u8>,
    pub n_solutions_atlas: Option<u8>,

    #[serde(default, with = "serde_bytes")]
    // should be Option<>, but not sure how to nest the custom deserializer
    pub result_id_apass: Vec<u8>,

    #[serde(default, with = "serde_bytes")]
    // should be Option<>, but not sure how to nest the custom deserializer
    pub result_id_atlas: Vec<u8>,
}
