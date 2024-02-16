use anyhow::{ensure, Result};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

/// cbindgen:ignore
pub const MAX_PQ_LUMINANCE: u16 = 10_000;

/// ST2086/HDR10 metadata fallback
#[repr(C)]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ExtMetadataBlockLevel6 {
    pub max_display_mastering_luminance: u16,
    pub min_display_mastering_luminance: u16,
    pub max_content_light_level: u16,
    pub max_frame_average_light_level: u16,
}

impl ExtMetadataBlockLevel6 {
    pub(crate) fn parse(reader: &mut BsIoSliceReader) -> Result<ExtMetadataBlock> {
        Ok(ExtMetadataBlock::Level6(Self {
            max_display_mastering_luminance: reader.get_n(16)?,
            min_display_mastering_luminance: reader.get_n(16)?,
            max_content_light_level: reader.get_n(16)?,
            max_frame_average_light_level: reader.get_n(16)?,
        }))
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        self.validate()?;

        writer.write_n(&self.max_display_mastering_luminance, 16)?;
        writer.write_n(&self.min_display_mastering_luminance, 16)?;
        writer.write_n(&self.max_content_light_level, 16)?;
        writer.write_n(&self.max_frame_average_light_level, 16)?;

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(self.max_display_mastering_luminance <= MAX_PQ_LUMINANCE);
        ensure!(self.min_display_mastering_luminance <= MAX_PQ_LUMINANCE);
        ensure!(self.max_content_light_level <= MAX_PQ_LUMINANCE);
        ensure!(self.max_frame_average_light_level <= MAX_PQ_LUMINANCE);

        Ok(())
    }

    pub fn source_meta_from_l6(&self) -> (u16, u16) {
        let mdl_min = self.min_display_mastering_luminance;
        let mdl_max = self.max_display_mastering_luminance;

        let source_min_pq = if mdl_min <= 10 {
            7
        } else if mdl_min == 50 {
            62
        } else {
            0
        };

        let source_max_pq = match mdl_max {
            1000 => 3079,
            2000 => 3388,
            4000 => 3696,
            10000 => 4095,
            _ => 3079,
        };

        (source_min_pq, source_max_pq)
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel6 {
    fn level(&self) -> u8 {
        6
    }

    fn bytes_size(&self) -> u64 {
        8
    }

    fn required_bits(&self) -> u64 {
        64
    }
}
