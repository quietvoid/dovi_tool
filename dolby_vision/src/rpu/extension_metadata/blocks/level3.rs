use anyhow::{ensure, Result};
use bitvec_helpers::{bitslice_reader::BitSliceReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo, MAX_12_BIT_VALUE};

/// Level 1 offsets.
#[repr(C)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde_feature", serde(default))]
pub struct ExtMetadataBlockLevel3 {
    pub min_pq_offset: u16,
    pub max_pq_offset: u16,
    pub avg_pq_offset: u16,
}

impl ExtMetadataBlockLevel3 {
    pub(crate) fn parse(reader: &mut BitSliceReader) -> Result<ExtMetadataBlock> {
        Ok(ExtMetadataBlock::Level3(Self {
            min_pq_offset: reader.get_n(12)?,
            max_pq_offset: reader.get_n(12)?,
            avg_pq_offset: reader.get_n(12)?,
        }))
    }

    pub fn write(&self, writer: &mut BitVecWriter) -> Result<()> {
        self.validate()?;

        writer.write_n(&self.min_pq_offset.to_be_bytes(), 12);
        writer.write_n(&self.max_pq_offset.to_be_bytes(), 12);
        writer.write_n(&self.avg_pq_offset.to_be_bytes(), 12);

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(self.min_pq_offset <= MAX_12_BIT_VALUE);
        ensure!(self.max_pq_offset <= MAX_12_BIT_VALUE);
        ensure!(self.avg_pq_offset <= MAX_12_BIT_VALUE);

        Ok(())
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

impl Default for ExtMetadataBlockLevel3 {
    fn default() -> Self {
        Self {
            min_pq_offset: 2048,
            max_pq_offset: 2048,
            avg_pq_offset: 2048,
        }
    }
}
