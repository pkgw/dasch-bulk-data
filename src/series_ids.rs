//! Some stuff about plates, exposures, mosaics, etc, as well as the
//! `mosaic_package` endpoint.

use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
};

/// Mapping from numerical series ID to series identifier. This captures the
/// DASCH SQL table `photometry.photseries`. Valid IDs range from 1 to 99,
/// inclusive.
pub const PLATE_SERIES_BY_ID: &[&str] = &[
    "INVALID", // 0
    "a",       // 1
    "ac",      // 2
    "adh",     // 3
    "al",      // 4
    "am",      // 5
    "b",       // 6
    "bm",      // 7
    "br",      // 8
    "c",       // 9
    "d",       // 10
    "dnb",     // 11
    "dnr",     // 12
    "dny",     // 13
    "dsb",     // 14
    "dsr",     // 15
    "dsy",     // 16
    "fa",      // 17
    "h",       // 18
    "i",       // 19
    "ir",      // 20
    "j",       // 21
    "ma",      // 22
    "mb",      // 23
    "mc",      // 24
    "md",      // 25
    "meteor",  // 26
    "mf",      // 27
    "na",      // 28
    "r",       // 29
    "rb",      // 30
    "rh",      // 31
    "rl",      // 32
    "sb",      // 33
    "sh",      // 34
    "x",       // 35
    "ctio",    // 36
    "ab",      // 37
    "acmisc",  // 38
    "aco",     // 39
    "ai",      // 40
    "ak",      // 41
    "an",      // 42
    "ax",      // 43
    "ay",      // 44
    "bc",      // 45
    "bi",      // 46
    "bo",      // 47
    "ca",      // 48
    "darnor",  // 49
    "darsou",  // 50
    "ee",      // 51
    "er",      // 52
    "hsl",     // 53
    "jdar",    // 54
    "misc",    // 55
    "m",       // 56
    "n",       // 57
    "nviews",  // 58
    "oa",      // 59
    "o",       // 60
    "p",       // 61
    "qa",      // 62
    "qb",      // 63
    "q",       // 64
    "ra",      // 65
    "ro",      // 66
    "rp",      // 67
    "s",       // 68
    "sp",      // 69
    "sq",      // 70
    "t",       // 71
    "vq",      // 72
    "wa",      // 73
    "w",       // 74
    "y",       // 75
    "z",       // 76
    "solar",   // 77
    "ww",      // 78
    "ayroe",   // 79
    "yb",      // 80
    "pz",      // 81
    "me",      // 82
    "pas",     // 83
    "hale",    // 84
    "lwla",    // 85
    "ka",      // 86
    "kb",      // 87
    "kc",      // 88
    "kd",      // 89
    "ke",      // 90
    "kf",      // 91
    "kg",      // 92
    "kge",     // 93
    "kh",      // 94
    "ad",      // 95
    "e",       // 96
    "u",       // 97
    "v",       // 98
    "poss",    // 99
];

// An inversion of PLATE_SERIES_BY_ID
pub static PLATE_ID_BY_SERIES: Lazy<HashMap<String, u8>> = Lazy::new(|| {
    PLATE_SERIES_BY_ID[1..]
        .iter()
        .enumerate()
        .map(|t| ((**t.1).to_owned(), (t.0 + 1) as u8))
        .collect()
});
