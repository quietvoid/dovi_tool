use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

///  Creative intent trim passes per target display peak brightness
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct ExtMetadataBlockLevel254 {
    pub dm_mode: u8,
    pub dm_version_index: u8,
}

impl ExtMetadataBlockLevel254 {
    pub fn parse(reader: &mut BitVecReader) -> ExtMetadataBlock {
        ExtMetadataBlock::Level254(Self {
            dm_mode: reader.get_n(8),
            dm_version_index: reader.get_n(8),
        })
    }

    pub fn write(&self, writer: &mut BitVecWriter) {
        writer.write_n(&self.dm_mode.to_be_bytes(), 8);
        writer.write_n(&self.dm_version_index.to_be_bytes(), 8);
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel254 {
    fn level(&self) -> u8 {
        254
    }

    fn bytes_size(&self) -> u64 {
        2
    }

    fn required_bits(&self) -> u64 {
        16
    }
}
