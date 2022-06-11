use std::convert::TryInto;

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

pub const PREDEFINED_COLORSPACE_PRIMARIES: &[[f64; 8]] = &[
    [0.68, 0.32, 0.265, 0.69, 0.15, 0.06, 0.3127, 0.329], //  0, DCI-P3 D65
    [0.64, 0.33, 0.30, 0.60, 0.15, 0.06, 0.3127, 0.329],  //  1, BT.709
    [0.708, 0.292, 0.170, 0.797, 0.131, 0.046, 0.3127, 0.329], //  2, BT.2020
    [0.63, 0.34, 0.31, 0.595, 0.155, 0.07, 0.3127, 0.329], //  3, BT.601 NTSC / SMPTE-C
    [0.64, 0.33, 0.29, 0.60, 0.15, 0.06, 0.3127, 0.329],  //  4, BT.601 PAL / BT.470 BG
    [0.68, 0.32, 0.265, 0.69, 0.15, 0.06, 0.314, 0.351],  //  5, DCI-P3
    [0.7347, 0.2653, 0.0, 1.0, 0.0001, -0.077, 0.32168, 0.33767], //  6, ACES
    [0.73, 0.28, 0.14, 0.855, 0.10, -0.05, 0.3127, 0.329], //  7, S-Gamut
    [0.766, 0.275, 0.225, 0.80, 0.089, -0.087, 0.3127, 0.329], //  8, S-Gamut-3.Cine
];

#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct ColorPrimaries {
    pub red_x: u16,
    pub red_y: u16,
    pub green_x: u16,
    pub green_y: u16,
    pub blue_x: u16,
    pub blue_y: u16,
    pub white_x: u16,
    pub white_y: u16,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub enum MasteringDisplayPrimaries {
    #[cfg_attr(feature = "serde_feature", serde(alias = "DCI-P3 D65"))]
    DCIP3D65 = 0,
    #[cfg_attr(feature = "serde_feature", serde(alias = "BT.709"))]
    BT709,
    #[cfg_attr(feature = "serde_feature", serde(alias = "BT.2020"))]
    BT2020,
    #[cfg_attr(feature = "serde_feature", serde(alias = "SMPTE-C"))]
    SMPTEC,
    #[cfg_attr(feature = "serde_feature", serde(alias = "BT.601"))]
    BT601,
    #[cfg_attr(feature = "serde_feature", serde(alias = "DCI-P3"))]
    DCIP3,
    ACES,
    #[cfg_attr(feature = "serde_feature", serde(alias = "S-Gamut"))]
    SGamut,
    #[cfg_attr(feature = "serde_feature", serde(alias = "S-Gamut-3.Cine"))]
    SGamut3Cine,
}

impl ColorPrimaries {
    pub fn from_array_int(primaries: &[u16; 8]) -> ColorPrimaries {
        Self {
            red_x: primaries[0],
            red_y: primaries[1],
            green_x: primaries[2],
            green_y: primaries[3],
            blue_x: primaries[4],
            blue_y: primaries[5],
            white_x: primaries[6],
            white_y: primaries[7],
        }
    }

    pub fn from_array_float(primaries: &[f64; 8]) -> ColorPrimaries {
        // Float to integer primaries
        let primaries_int = f64_to_integer_primaries(primaries);

        Self::from_array_int(&primaries_int)
    }

    pub fn from_enum(primary: MasteringDisplayPrimaries) -> ColorPrimaries {
        Self::from_array_float(&PREDEFINED_COLORSPACE_PRIMARIES[primary as usize])
    }
}

/// Assumes a list of size 8, otherwise panics
pub fn f64_to_integer_primaries(primaries: &[f64]) -> [u16; 8] {
    const SCALE: f64 = 1.0 / 32767.0;

    primaries
        .iter()
        .map(|v| (v / SCALE).round() as u16)
        .collect::<Vec<u16>>()
        .try_into()
        .unwrap()
}
