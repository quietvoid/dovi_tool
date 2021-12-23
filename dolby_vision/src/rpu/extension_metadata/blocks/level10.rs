use anyhow::{ensure, Result};
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

use super::{level6::MAX_PQ_LUMINANCE, ExtMetadataBlock, ExtMetadataBlockInfo};

pub const PRESET_TARGET_DISPLAYS: &[u8] = &[1, 16, 18, 21, 27, 28, 37, 38, 42, 48, 49];

/// Custom target display information
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct ExtMetadataBlockLevel10 {
    pub target_display_index: u8,
    pub target_max_pq: u16,
    pub target_min_pq: u16,
    pub target_primary_index: u8,
}

impl ExtMetadataBlockLevel10 {
    pub fn parse(reader: &mut BitVecReader) -> ExtMetadataBlock {
        ExtMetadataBlock::Level10(Self {
            target_display_index: reader.get_n(8),
            target_max_pq: reader.get_n(12),
            target_min_pq: reader.get_n(12),
            target_primary_index: reader.get_n(8),
        })
    }

    pub fn write(&self, writer: &mut BitVecWriter) -> Result<()> {
        self.validate()?;

        writer.write_n(&self.target_display_index.to_be_bytes(), 8);
        writer.write_n(&self.target_max_pq.to_be_bytes(), 12);
        writer.write_n(&self.target_min_pq.to_be_bytes(), 12);
        writer.write_n(&self.target_primary_index.to_be_bytes(), 8);

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(!PRESET_TARGET_DISPLAYS.contains(&self.target_display_index));
        ensure!(self.target_max_pq <= MAX_PQ_LUMINANCE);
        ensure!(self.target_min_pq <= MAX_PQ_LUMINANCE);

        Ok(())
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel10 {
    fn level(&self) -> u8 {
        10
    }

    fn bytes_size(&self) -> u64 {
        5
    }

    fn required_bits(&self) -> u64 {
        40
    }
}
