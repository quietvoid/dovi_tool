use anyhow::{Result, ensure};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

/// Creative environment metadata
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ExtMetadataBlockLevel18 {
    pub surround_luminance_pq: u16,
    pub min_preserved_luminance_pq: u16,
    pub adaptation_luminance_pq: u16,
    pub max_preserved_luminance_pq: u16,

    /// 4 bits
    pub revision: u8,
    /// 4 bits
    pub reserved: u8,
}

impl ExtMetadataBlockLevel18 {
    pub(crate) fn parse(reader: &mut BsIoSliceReader) -> Result<ExtMetadataBlock> {
        let block = Self {
            surround_luminance_pq: reader.read::<12, u16>()?,
            min_preserved_luminance_pq: reader.read::<12, u16>()?,
            adaptation_luminance_pq: reader.read::<12, u16>()?,
            max_preserved_luminance_pq: reader.read::<12, u16>()?,
            revision: reader.read::<4, u8>()?,
            reserved: reader.read::<4, u8>()?,
        };

        Ok(ExtMetadataBlock::Level18(block))
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        self.validate()?;

        writer.write::<12, u16>(self.surround_luminance_pq)?;
        writer.write::<12, u16>(self.min_preserved_luminance_pq)?;
        writer.write::<12, u16>(self.adaptation_luminance_pq)?;
        writer.write::<12, u16>(self.max_preserved_luminance_pq)?;
        writer.write::<4, u8>(self.revision)?;
        writer.write::<4, u8>(self.reserved)?;

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(self.reserved == 0);

        Ok(())
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel18 {
    fn level(&self) -> u8 {
        18
    }

    fn bytes_size(&self) -> u64 {
        7
    }

    fn required_bits(&self) -> u64 {
        56
    }
}
