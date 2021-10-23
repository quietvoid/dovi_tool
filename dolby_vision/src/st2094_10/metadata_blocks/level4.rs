use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use super::ExtMetadataBlock;

#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct ExtMetadataBlockLevel4 {
    pub anchor_pq: u16,
    pub anchor_power: u16,
}

impl ExtMetadataBlockLevel4 {
    pub fn parse(reader: &mut BitVecReader) -> ExtMetadataBlock {
        ExtMetadataBlock::Level4(Self {
            anchor_pq: reader.get_n(12),
            anchor_power: reader.get_n(12),
        })
    }

    pub fn write(&self, writer: &mut BitVecWriter) {
        writer.write_n(&self.anchor_pq.to_be_bytes(), 12);
        writer.write_n(&self.anchor_power.to_be_bytes(), 12);
    }
}
