use anyhow::Result;
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

/// Metadata level present in CM v4.0
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct ExtMetadataBlockLevel254 {
    pub dm_mode: u8,
    pub dm_version_index: u8,
}

impl ExtMetadataBlockLevel254 {
    pub(crate) fn parse(reader: &mut BsIoSliceReader) -> Result<ExtMetadataBlock> {
        Ok(ExtMetadataBlock::Level254(Self {
            dm_mode: reader.get_n(8)?,
            dm_version_index: reader.get_n(8)?,
        }))
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        writer.write_n(&self.dm_mode, 8)?;
        writer.write_n(&self.dm_version_index, 8)?;

        Ok(())
    }

    pub fn cmv402_default() -> ExtMetadataBlockLevel254 {
        ExtMetadataBlockLevel254 {
            dm_mode: 0,
            dm_version_index: 2,
        }
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel254 {
    fn level(&self) -> u8 {
        254
    }

    fn bytes_size(&self) -> u64 {
        2
    }

    fn required_bits(&self) -> u64 {
        16
    }
}
