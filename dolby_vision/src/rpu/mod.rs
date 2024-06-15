use crc::{Crc, Table, CRC_32_MPEG_2};

pub mod dovi_rpu;
pub mod extension_metadata;
pub mod generate;
pub mod profiles;
pub mod rpu_data_header;
pub mod rpu_data_mapping;
pub mod rpu_data_nlq;
pub mod vdr_dm_data;

pub mod utils;

static CRC32_INSTANCE: Crc<u32, Table<16>> = Crc::<u32, Table<16>>::new(&CRC_32_MPEG_2);

pub const NUM_COMPONENTS: usize = 3;

pub(crate) const MMR_MAX_COEFFS: usize = 7;
pub(crate) const NLQ_NUM_PIVOTS: usize = 2;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConversionMode {
    Lossless = 0,
    ToMel,
    To81,
    To84,
    To81MappingPreserved,
}

#[inline(always)]
fn compute_crc32(data: &[u8]) -> u32 {
    CRC32_INSTANCE.checksum(data)
}

impl From<u8> for ConversionMode {
    fn from(mode: u8) -> ConversionMode {
        match mode {
            0 => ConversionMode::Lossless,
            1 => ConversionMode::ToMel,
            2 | 3 => ConversionMode::To81,
            4 => ConversionMode::To84,
            _ => ConversionMode::Lossless,
        }
    }
}

impl std::fmt::Display for ConversionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionMode::Lossless => write!(f, "Lossless"),
            ConversionMode::ToMel => write!(f, "To MEL"),
            ConversionMode::To81 => write!(f, "To 8.1"),
            ConversionMode::To84 => write!(f, "To 8.4"),
            ConversionMode::To81MappingPreserved => {
                write!(f, "To 8.1, preserving the mapping metadata")
            }
        }
    }
}

impl Default for ConversionMode {
    fn default() -> Self {
        Self::Lossless
    }
}
