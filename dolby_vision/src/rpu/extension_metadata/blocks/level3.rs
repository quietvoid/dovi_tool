use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

/// Level 1 offsets.
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct ExtMetadataBlockLevel3 {
    pub min_pq_offset: u16,
    pub max_pq_offset: u16,
    pub avg_pq_offset: u16,
}

impl ExtMetadataBlockLevel3 {
    pub fn parse(reader: &mut BitVecReader) -> ExtMetadataBlock {
        ExtMetadataBlock::Level3(Self {
            min_pq_offset: reader.get_n(12),
            max_pq_offset: reader.get_n(12),
            avg_pq_offset: reader.get_n(12),
        })
    }

    pub fn write(&self, writer: &mut BitVecWriter) {
        writer.write_n(&self.min_pq_offset.to_be_bytes(), 12);
        writer.write_n(&self.max_pq_offset.to_be_bytes(), 12);
        writer.write_n(&self.avg_pq_offset.to_be_bytes(), 12);
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel3 {
    fn level(&self) -> u8 {
        3
    }

    fn bytes_size(&self) -> u64 {
        5
    }

    fn required_bits(&self) -> u64 {
        36
    }
}
