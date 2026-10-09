//! Dealing with the DASCH photometry data.

use binary_serde::BinarySerde;

/// A record in the binary "magfiles" that store compiled DASCH photometry data.
/// We cannot rearrange any fields here -- this struct captures the format used
/// by the legacy C code.
#[derive(BinarySerde, Debug, PartialEq)]
pub struct MagRecord {
    pub marker: u32,
    pub version: u32,
    pub ref_number: u64,
    pub x_image: f64,
    pub y_image: f64,
    pub mag_iso: f64,
    pub ra_deg: f64,
    pub dec_deg: f64,
    pub date_jd: f64,
    pub flux_iso: f64,
    pub mag_aper: f64,
    pub mag_auto: f64,
    pub kron_radius: f64,
    pub background: f64,
    pub flux_max_adu: f64,
    pub theta_j2000: f64,
    pub ellipticity: f64,
    pub iso_area_sqdeg: f64,
    pub fwhm_pix: f64,
    pub fwhm_deg: f64,
    pub plate_center_dist_deg: f64,
    pub blended_mag: f64,
    pub drad_rms2: f64,
    pub ra_cat_corrected: f64,
    pub dec_cat_corrected: f64,
    pub magcal_iso: f32,
    pub magcal_iso_rms: f32,
    pub magcal_local: f32,
    pub magcal_local_rms: f32,
    pub limiting_mag_local: f32,
    pub magcal_local_error: f32,
    pub magcor_local: f32,
    pub extinction: f32,
    pub magcal_magdep: f32,
    pub magcal_magdep_rms: f32,
    pub pm_ra_masyr: f32,
    pub pm_dec_masyr: f32,
    pub time_accuracy_days: f32,
    pub gsc_bin_index: u32,
    pub plate_number: u32,
    pub sextractor_number: u32,
    pub version_id: u32,
    pub aflags: u32,
    pub a2flags: u32,
    pub bflags: u32,
    pub b2flags: u32,
    pub iso_areas: [u32; 8],
    pub npoints_local: u32,
    pub reject_flag: u32,
    pub magdep_bin: i32,
    pub pass_bits: u16,
    pub local_bin_index: u16,
    pub mask_index: u16,
    pub series_id: u8,
    pub exposure_number: i8,
    pub solution_number: u8,
    pub spatial_bin: u8,
    pub catalog_number: u8,
    pub padding: [u8; 5],
}

#[derive(BinarySerde, Debug, PartialEq)]
pub struct LimitsPlateRecord {
    pub rec_type: u32,
    pub rec_version: u32,
    pub limiting_mag_local: f64,
    pub geo_jd: f64,
    pub series_id: u32,
    pub plate_number: u32,
    pub mosaic_number: u32,
    pub solution_number: u32,
    pub version: u32,
    _unused: u32,
}

#[derive(BinarySerde, Debug, PartialEq)]
pub struct LimitsIndexRecord {
    pub byte_offset: u64,
    pub byte_length: u64,
}

// / Given a sky position, retrieve the limiting-magnitude data associated with
// / it.
// /
// / The return value will be a buffer of the binary data, which can be unpacked
// / into LimitsPlateRecord values.
//pub async fn get_limiting_records(
//    refcat: &str,
//    ra_deg: f64,
//    dec_deg: f64,
//    s3: &aws_sdk_s3::Client,
//    bin2: &GscBinning,
//) -> Result<Vec<u8>, Error> {
//    let dec_bin = bin2.get_dec_bin(dec_deg);
//    let total_bin = bin2.get_total_bin(dec_bin, ra_deg);
//
//    // Look up this bin in the index
//
//    let start_byte = total_bin * LimitsIndexRecord::SERIALIZED_SIZE;
//    let s3_key = format!("dasch-dr7-phot-{}/lim_aws.idx", refcat);
//    let data = s3
//        .get_object()
//        .bucket(USER_BUCKET)
//        .key(&s3_key)
//        .range(format!(
//            "bytes={}-{}",
//            start_byte,
//            start_byte + LimitsIndexRecord::SERIALIZED_SIZE - 1
//        ))
//        .send()
//        .await?
//        .body
//        .collect()
//        .await?
//        .to_vec();
//
//    let rec = LimitsIndexRecord::binary_deserialize(&data, Endianness::Little)
//        .map_err(|e| -> Error { format!("deserialize failed: {e}").into() })?;
//
//    // Now we can fetch the actual payload
//
//    let s3_key = format!("dasch-dr7-phot-{}/limiting.dat", refcat);
//    Ok(s3
//        .get_object()
//        .bucket(USER_BUCKET)
//        .key(&s3_key)
//        .range(format!(
//            "bytes={}-{}",
//            rec.byte_offset,
//            rec.byte_offset + rec.byte_length - 1
//        ))
//        .send()
//        .await?
//        .body
//        .collect()
//        .await?
//        .to_vec())
//}
