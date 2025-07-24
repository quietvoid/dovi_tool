use anyhow::{Result, ensure};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

/// Consumer look metadata, Precision Rendering/Detail
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ExtMetadataBlockLevel15 {
    pub confidence: u8,
    pub precision_rendering_strength: u8,
    pub d_local_contrast: u8,
    pub d_brightness: u8,
    pub d_saturation_plus_one: u8,
    pub d_contrast_plus_one: u8,

    pub confidence_no_pr: u8,
    pub d_brightness_no_pr: u8,
    pub d_saturation_plus_one_no_pr: u8,
    pub d_contrast_plus_one_no_pr: u8,

    /// 4 bits
    pub revision: u8,
    /// 4 bits
    pub reserved: u8,
}

impl ExtMetadataBlockLevel15 {
    pub(crate) fn parse(reader: &mut BsIoSliceReader) -> Result<ExtMetadataBlock> {
        let block = Self {
            confidence: reader.read::<8, u8>()?,
            precision_rendering_strength: reader.read::<8, u8>()?,
            d_local_contrast: reader.read::<8, u8>()?,
            d_brightness: reader.read::<8, u8>()?,
            d_saturation_plus_one: reader.read::<8, u8>()?,
            d_contrast_plus_one: reader.read::<8, u8>()?,
            confidence_no_pr: reader.read::<8, u8>()?,
            d_brightness_no_pr: reader.read::<8, u8>()?,
            d_saturation_plus_one_no_pr: reader.read::<8, u8>()?,
            d_contrast_plus_one_no_pr: reader.read::<8, u8>()?,
            revision: reader.read::<4, u8>()?,
            reserved: reader.read::<4, u8>()?,
        };

        Ok(ExtMetadataBlock::Level15(block))
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        self.validate()?;

        writer.write::<8, u8>(self.confidence)?;
        writer.write::<8, u8>(self.precision_rendering_strength)?;
        writer.write::<8, u8>(self.d_local_contrast)?;
        writer.write::<8, u8>(self.d_brightness)?;
        writer.write::<8, u8>(self.d_saturation_plus_one)?;
        writer.write::<8, u8>(self.d_contrast_plus_one)?;
        writer.write::<8, u8>(self.confidence_no_pr)?;
        writer.write::<8, u8>(self.d_brightness_no_pr)?;
        writer.write::<8, u8>(self.d_saturation_plus_one_no_pr)?;
        writer.write::<8, u8>(self.d_contrast_plus_one_no_pr)?;
        writer.write::<4, u8>(self.revision)?;
        writer.write::<4, u8>(self.reserved)?;

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(self.reserved == 0);

        Ok(())
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel15 {
    fn level(&self) -> u8 {
        15
    }

    fn bytes_size(&self) -> u64 {
        11
    }

    fn required_bits(&self) -> u64 {
        11 * 8
    }
}
