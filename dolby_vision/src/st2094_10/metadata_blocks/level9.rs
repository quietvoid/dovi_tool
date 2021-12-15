use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

///  Creative intent trim passes per target display peak brightness
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct ExtMetadataBlockLevel9 {
    pub source_primary_index: u8,
}

impl ExtMetadataBlockLevel9 {
    pub fn parse(reader: &mut BitVecReader) -> ExtMetadataBlock {
        ExtMetadataBlock::Level9(Self {
            source_primary_index: reader.get_n(8),
        })
    }

    pub fn write(&self, writer: &mut BitVecWriter) {
        writer.write_n(&self.source_primary_index.to_be_bytes(), 8);
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel9 {
    fn level(&self) -> u8 {
        9
    }

    fn bytes_size(&self) -> u64 {
        1
    }

    fn required_bits(&self) -> u64 {
        8
    }
}
