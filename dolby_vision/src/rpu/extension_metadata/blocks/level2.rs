use anyhow::{ensure, Result};
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

use crate::utils::nits_to_pq;

use super::{ExtMetadataBlock, ExtMetadataBlockInfo, MAX_12_BIT_VALUE};

/// Creative intent trim passes per target display peak brightness
#[repr(C)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct ExtMetadataBlockLevel2 {
    pub target_max_pq: u16,
    pub trim_slope: u16,
    pub trim_offset: u16,
    pub trim_power: u16,
    pub trim_chroma_weight: u16,
    pub trim_saturation_gain: u16,
    pub ms_weight: i16,
}

impl ExtMetadataBlockLevel2 {
    pub fn parse(reader: &mut BitVecReader) -> ExtMetadataBlock {
        let mut level2 = Self {
            target_max_pq: reader.get_n(12),
            trim_slope: reader.get_n(12),
            trim_offset: reader.get_n(12),
            trim_power: reader.get_n(12),
            trim_chroma_weight: reader.get_n(12),
            trim_saturation_gain: reader.get_n(12),
            ms_weight: reader.get_n::<u16>(13) as i16,
        };

        if level2.ms_weight > MAX_12_BIT_VALUE as i16 {
            level2.ms_weight = level2.ms_weight.wrapping_sub(8192);
        }

        ExtMetadataBlock::Level2(level2)
    }

    pub fn write(&self, writer: &mut BitVecWriter) -> Result<()> {
        self.validate()?;

        writer.write_n(&self.target_max_pq.to_be_bytes(), 12);
        writer.write_n(&self.trim_slope.to_be_bytes(), 12);
        writer.write_n(&self.trim_offset.to_be_bytes(), 12);
        writer.write_n(&self.trim_power.to_be_bytes(), 12);
        writer.write_n(&self.trim_chroma_weight.to_be_bytes(), 12);
        writer.write_n(&self.trim_saturation_gain.to_be_bytes(), 12);
        writer.write_n(&self.ms_weight.to_be_bytes(), 13);

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(self.target_max_pq <= MAX_12_BIT_VALUE);
        ensure!(self.trim_slope <= MAX_12_BIT_VALUE);
        ensure!(self.trim_offset <= MAX_12_BIT_VALUE);
        ensure!(self.trim_power <= MAX_12_BIT_VALUE);
        ensure!(self.trim_chroma_weight <= MAX_12_BIT_VALUE);
        ensure!(self.trim_saturation_gain <= MAX_12_BIT_VALUE);
        ensure!(self.ms_weight >= -1 && self.ms_weight <= (MAX_12_BIT_VALUE as i16));

        Ok(())
    }

    pub fn from_nits(target_nits: u16) -> ExtMetadataBlockLevel2 {
        ExtMetadataBlockLevel2 {
            target_max_pq: (nits_to_pq(target_nits.into()) * 4095.0).round() as u16,
            ..Default::default()
        }
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel2 {
    fn level(&self) -> u8 {
        2
    }

    fn bytes_size(&self) -> u64 {
        11
    }

    fn required_bits(&self) -> u64 {
        85
    }

    fn sort_key(&self) -> (u8, u16) {
        (self.level(), self.target_max_pq)
    }
}

impl Default for ExtMetadataBlockLevel2 {
    fn default() -> Self {
        Self {
            target_max_pq: 2081,
            trim_slope: 2048,
            trim_offset: 2048,
            trim_power: 2048,
            trim_chroma_weight: 2048,
            trim_saturation_gain: 2048,
            ms_weight: 2048,
        }
    }
}
