use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use crate::st2094_10::generate::Level6Metadata;

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

/// ST2086 metadata fallback
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct ExtMetadataBlockLevel6 {
    pub max_display_mastering_luminance: u16,
    pub min_display_mastering_luminance: u16,
    pub max_content_light_level: u16,
    pub max_frame_average_light_level: u16,
}

impl ExtMetadataBlockLevel6 {
    pub fn parse(reader: &mut BitVecReader) -> ExtMetadataBlock {
        ExtMetadataBlock::Level6(Self {
            max_display_mastering_luminance: reader.get_n(16),
            min_display_mastering_luminance: reader.get_n(16),
            max_content_light_level: reader.get_n(16),
            max_frame_average_light_level: reader.get_n(16),
        })
    }

    pub fn write(&self, writer: &mut BitVecWriter) {
        writer.write_n(&self.max_display_mastering_luminance.to_be_bytes(), 16);
        writer.write_n(&self.min_display_mastering_luminance.to_be_bytes(), 16);
        writer.write_n(&self.max_content_light_level.to_be_bytes(), 16);
        writer.write_n(&self.max_frame_average_light_level.to_be_bytes(), 16);
    }

    pub fn set_fields_from_generate_l6(&mut self, meta: &Level6Metadata) {
        self.max_display_mastering_luminance = meta.max_display_mastering_luminance;
        self.min_display_mastering_luminance = meta.min_display_mastering_luminance;
        self.max_content_light_level = meta.max_content_light_level;
        self.max_frame_average_light_level = meta.max_frame_average_light_level;
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
