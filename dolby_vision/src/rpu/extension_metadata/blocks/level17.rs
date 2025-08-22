use anyhow::Result;
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

/// Up mapping metadata
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ExtMetadataBlockLevel17 {
    pub mid_boost: u8,
    pub highlight_stretch: u8,
    pub shadow_drop: u8,
    pub contrast_boost: u8,
    pub saturation_boost: u8,
    pub detail_boost: u8,
    pub chroma_indicator: u8,

    /// 12 bits
    pub intensity_indicator_pq: u16,

    /// 4 bits
    pub revision: u8,
}

impl ExtMetadataBlockLevel17 {
    pub(crate) fn parse(reader: &mut BsIoSliceReader) -> Result<ExtMetadataBlock> {
        let block = Self {
            mid_boost: reader.read::<8, u8>()?,
            highlight_stretch: reader.read::<8, u8>()?,
            shadow_drop: reader.read::<8, u8>()?,
            contrast_boost: reader.read::<8, u8>()?,
            saturation_boost: reader.read::<8, u8>()?,
            detail_boost: reader.read::<8, u8>()?,
            chroma_indicator: reader.read::<8, u8>()?,
            intensity_indicator_pq: reader.read::<12, u16>()?,
            revision: reader.read::<4, u8>()?,
        };

        Ok(ExtMetadataBlock::Level17(block))
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        self.validate()?;

        writer.write::<8, u8>(self.mid_boost)?;
        writer.write::<8, u8>(self.highlight_stretch)?;
        writer.write::<8, u8>(self.shadow_drop)?;
        writer.write::<8, u8>(self.contrast_boost)?;
        writer.write::<8, u8>(self.saturation_boost)?;
        writer.write::<8, u8>(self.detail_boost)?;
        writer.write::<8, u8>(self.chroma_indicator)?;
        writer.write::<12, u16>(self.intensity_indicator_pq)?;
        writer.write::<4, u8>(self.revision)?;

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel17 {
    fn level(&self) -> u8 {
        17
    }

    fn bytes_size(&self) -> u64 {
        9
    }

    fn required_bits(&self) -> u64 {
        72
    }
}
