use anyhow::Result;
use bitvec_helpers::{bitslice_reader::BitSliceReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

/// Metadata level present in CM v4.0
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde_feature", serde(default))]
pub struct ExtMetadataBlockLevel254 {
    pub dm_mode: u8,
    pub dm_version_index: u8,
}

impl ExtMetadataBlockLevel254 {
    pub(crate) fn parse(reader: &mut BitSliceReader) -> Result<ExtMetadataBlock> {
        Ok(ExtMetadataBlock::Level254(Self {
            dm_mode: reader.get_n(8)?,
            dm_version_index: reader.get_n(8)?,
        }))
    }

    pub fn write(&self, writer: &mut BitVecWriter) -> Result<()> {
        writer.write_n(&self.dm_mode.to_be_bytes(), 8);
        writer.write_n(&self.dm_version_index.to_be_bytes(), 8);

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
