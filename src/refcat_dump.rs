//! Type definitions for deserializing a DASCH DR7 refcat database dump.
//! 
//! The contents of this dump power the [catalog query API][1] and
//! are also needed by the [lightcurve API][2].
//! 
//! [1]: https://dasch.cfa.harvard.edu/dr7/web-apis/#post-dasch-dr7-querycat
//! [2]: https://dasch.cfa.harvard.edu/dr7/web-apis/#post-dasch-dr7-lightcurve
//! 
//! The dump comes as a single gzipped JSON-lines file, where every line is a
//! record in the table serialized in the [DynamoDB typed JSON format][3].
//! 
//! [3]: https://aws.amazon.com/blogs/database/exploring-amazon-dynamodb-sdk-clients/

use serde::Deserialize;

/// A record in a reference catalog, i.e. a star.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RefcatItemRecord {
    pub gsc_bin_index: u32,
    pub ref_number: u64,
    pub ra: f64,
    pub dec: f64,
    pub stdmag: f32,
    pub color: f32,
    #[serde(rename = "raPM")]
    pub ra_pm: f32,
    #[serde(rename = "decPM")]
    pub dec_pm: f32,
    #[serde(rename = "raSigmaPM")]
    pub ra_sigma_pm: f32,
    #[serde(rename = "decSigmaPM")]
    pub dec_sigma_pm: f32,
    pub class: u8,
    pub v_flag: u8,
    pub mag_flag: u8,

    /// Extra per-source photometry information
    pub phot: Option<RefcatPhotRecord>,
}

/// Information about the lightcurve database information associated with a star.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RefcatPhotRecord {
    pub count: u32,

    /// The largest individual phot files are around 400 MB, so offsets into
    /// them can fit into a u32.
    pub offset: u32,
}
