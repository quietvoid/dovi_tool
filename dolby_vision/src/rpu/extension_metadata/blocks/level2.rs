use anyhow::{Result, ensure};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::utils::nits_to_pq_12_bit;

use super::{ExtMetadataBlock, ExtMetadataBlockInfo, MAX_12_BIT_VALUE};

/// Creative intent trim passes per target display peak brightness
#[repr(C)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
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
    pub(crate) fn parse(reader: &mut BsIoSliceReader) -> Result<ExtMetadataBlock> {
        let mut level2 = Self {
            target_max_pq: reader.read::<12, u16>()?,
            trim_slope: reader.read::<12, u16>()?,
            trim_offset: reader.read::<12, u16>()?,
            trim_power: reader.read::<12, u16>()?,
            trim_chroma_weight: reader.read::<12, u16>()?,
            trim_saturation_gain: reader.read::<12, u16>()?,
            ms_weight: reader.read::<13, i16>()?,
        };

        if level2.ms_weight > MAX_12_BIT_VALUE as i16 {
            level2.ms_weight = level2.ms_weight.wrapping_sub(8192);
        }

        Ok(ExtMetadataBlock::Level2(level2))
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        self.validate()?;

        writer.write::<12, u16>(self.target_max_pq)?;
        writer.write::<12, u16>(self.trim_slope)?;
        writer.write::<12, u16>(self.trim_offset)?;
        writer.write::<12, u16>(self.trim_power)?;
        writer.write::<12, u16>(self.trim_chroma_weight)?;
        writer.write::<12, u16>(self.trim_saturation_gain)?;
        writer.write::<13, i16>(self.ms_weight)?;

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
            target_max_pq: nits_to_pq_12_bit(target_nits),
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
