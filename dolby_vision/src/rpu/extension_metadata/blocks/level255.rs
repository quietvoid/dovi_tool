use anyhow::Result;
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

/// Metadata level optionally present in CM v2.9.
/// Different display modes (calibration/verify/bypass), debugging
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct ExtMetadataBlockLevel255 {
    pub dm_run_mode: u8,
    pub dm_run_version: u8,
    pub dm_debug0: u8,
    pub dm_debug1: u8,
    pub dm_debug2: u8,
    pub dm_debug3: u8,
}

impl ExtMetadataBlockLevel255 {
    pub(crate) fn parse(reader: &mut BsIoSliceReader) -> Result<ExtMetadataBlock> {
        Ok(ExtMetadataBlock::Level255(Self {
            dm_run_mode: reader.read::<8, u8>()?,
            dm_run_version: reader.read::<8, u8>()?,
            dm_debug0: reader.read::<8, u8>()?,
            dm_debug1: reader.read::<8, u8>()?,
            dm_debug2: reader.read::<8, u8>()?,
            dm_debug3: reader.read::<8, u8>()?,
        }))
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        writer.write::<8, u8>(self.dm_run_mode)?;
        writer.write::<8, u8>(self.dm_run_version)?;
        writer.write::<8, u8>(self.dm_debug0)?;
        writer.write::<8, u8>(self.dm_debug1)?;
        writer.write::<8, u8>(self.dm_debug2)?;
        writer.write::<8, u8>(self.dm_debug3)?;

        Ok(())
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel255 {
    fn level(&self) -> u8 {
        255
    }

    fn bytes_size(&self) -> u64 {
        6
    }

    fn required_bits(&self) -> u64 {
        48
    }
}
