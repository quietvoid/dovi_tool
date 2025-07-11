use anyhow::{Result, ensure};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo, MAX_12_BIT_VALUE};

/// Something about temporal stability
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ExtMetadataBlockLevel4 {
    pub anchor_pq: u16,
    pub anchor_power: u16,
}

impl ExtMetadataBlockLevel4 {
    pub(crate) fn parse(reader: &mut BsIoSliceReader) -> Result<ExtMetadataBlock> {
        Ok(ExtMetadataBlock::Level4(Self {
            anchor_pq: reader.read::<12, u16>()?,
            anchor_power: reader.read::<12, u16>()?,
        }))
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        self.validate()?;

        writer.write::<12, u16>(self.anchor_pq)?;
        writer.write::<12, u16>(self.anchor_power)?;

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(self.anchor_pq <= MAX_12_BIT_VALUE);
        ensure!(self.anchor_power <= MAX_12_BIT_VALUE);

        Ok(())
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel4 {
    fn level(&self) -> u8 {
        4
    }

    fn bytes_size(&self) -> u64 {
        3
    }

    fn required_bits(&self) -> u64 {
        24
    }
}
