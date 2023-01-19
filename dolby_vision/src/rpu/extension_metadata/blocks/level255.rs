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
            dm_run_mode: reader.get_n(8)?,
            dm_run_version: reader.get_n(8)?,
            dm_debug0: reader.get_n(8)?,
            dm_debug1: reader.get_n(8)?,
            dm_debug2: reader.get_n(8)?,
            dm_debug3: reader.get_n(8)?,
        }))
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        writer.write_n(&self.dm_run_mode, 8)?;
        writer.write_n(&self.dm_run_version, 8)?;
        writer.write_n(&self.dm_debug0, 8)?;
        writer.write_n(&self.dm_debug1, 8)?;
        writer.write_n(&self.dm_debug2, 8)?;
        writer.write_n(&self.dm_debug3, 8)?;

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
