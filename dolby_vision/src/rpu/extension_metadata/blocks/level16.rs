use anyhow::{Result, ensure};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

/// Local tone mapping metadata
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ExtMetadataBlockLevel16 {
    pub revision: u8,
    pub count: usize,

    pub params: Vec<Level16Params>,
}

#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Level16Params {
    pub contrast_target: u8,
    pub precision_rendering_strength: u8,
    pub d_local_contrast: u8,
    pub max_d_brightness: u8,
    pub max_d_saturation_plus_one: u8,
}

impl ExtMetadataBlockLevel16 {
    pub(crate) fn parse(reader: &mut BsIoSliceReader) -> Result<ExtMetadataBlock> {
        let mut block = Self {
            revision: reader.read::<4, u8>()?,
            count: reader.read::<4, u8>()? as usize,
            ..Default::default()
        };

        block.params.reserve(block.count);

        for _ in 0..block.count {
            block.params.push(Level16Params {
                contrast_target: reader.read::<8, u8>()?,
                precision_rendering_strength: reader.read::<8, u8>()?,
                d_local_contrast: reader.read::<8, u8>()?,
                max_d_brightness: reader.read::<8, u8>()?,
                max_d_saturation_plus_one: reader.read::<8, u8>()?,
            });
        }

        Ok(ExtMetadataBlock::Level16(block))
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        self.validate()?;

        writer.write::<4, u8>(self.revision)?;
        writer.write::<4, u8>(self.count as u8)?;

        for params in &self.params {
            writer.write::<8, u8>(params.contrast_target)?;
            writer.write::<8, u8>(params.precision_rendering_strength)?;
            writer.write::<8, u8>(params.d_local_contrast)?;
            writer.write::<8, u8>(params.max_d_brightness)?;
            writer.write::<8, u8>(params.max_d_saturation_plus_one)?;
        }

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(self.count == self.params.len());

        Ok(())
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel16 {
    fn level(&self) -> u8 {
        16
    }

    fn bytes_size(&self) -> u64 {
        1 + (self.count as u64 * 5)
    }

    fn required_bits(&self) -> u64 {
        self.bytes_size() * 8
    }
}
