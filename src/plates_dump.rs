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
    pub center_source: Option<String>,
    pub date_acc_days: Option<f64>,
    pub date_source: Option<String>,
    pub dec_deg: Option<f64>,
    pub dur_min: Option<f64>,
    pub midpoint_date: Option<String>,
    pub number: i8,
    pub ra_deg: Option<f64>,
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
