use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

/// Statistical analysis of the frame: min, max, avg brightness.
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct ExtMetadataBlockLevel1 {
    pub min_pq: u16,
    pub max_pq: u16,
    pub avg_pq: u16,
}

impl ExtMetadataBlockLevel1 {
    pub fn parse(reader: &mut BitVecReader) -> ExtMetadataBlock {
        ExtMetadataBlock::Level1(Self {
            min_pq: reader.get_n(12),
            max_pq: reader.get_n(12),
            avg_pq: reader.get_n(12),
        })
    }

    pub fn write(&self, writer: &mut BitVecWriter) {
        writer.write_n(&self.min_pq.to_be_bytes(), 12);
        writer.write_n(&self.max_pq.to_be_bytes(), 12);
        writer.write_n(&self.avg_pq.to_be_bytes(), 12);
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel1 {
    fn level(&self) -> u8 {
        1
    }

    fn bytes_size(&self) -> u64 {
        5
    }

    fn required_bits(&self) -> u64 {
        36
    }
}
