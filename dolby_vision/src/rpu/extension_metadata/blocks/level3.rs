use anyhow::{Result, ensure};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo, MAX_12_BIT_VALUE};

/// Level 1 offsets.
#[repr(C)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct ExtMetadataBlockLevel3 {
    pub min_pq_offset: u16,
    pub max_pq_offset: u16,
    pub avg_pq_offset: u16,
}

impl ExtMetadataBlockLevel3 {
    pub(crate) fn parse(reader: &mut BsIoSliceReader) -> Result<ExtMetadataBlock> {
        Ok(ExtMetadataBlock::Level3(Self {
            min_pq_offset: reader.read::<12, u16>()?,
            max_pq_offset: reader.read::<12, u16>()?,
            avg_pq_offset: reader.read::<12, u16>()?,
        }))
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        self.validate()?;

        writer.write::<12, u16>(self.min_pq_offset)?;
        writer.write::<12, u16>(self.max_pq_offset)?;
        writer.write::<12, u16>(self.avg_pq_offset)?;

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
