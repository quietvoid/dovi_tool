use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use super::ExtMetadataBlock;

/// Active area of the picture (letterbox, aspect ratio)
#[repr(C)]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct ExtMetadataBlockLevel5 {
    pub active_area_left_offset: u16,
    pub active_area_right_offset: u16,
    pub active_area_top_offset: u16,
    pub active_area_bottom_offset: u16,
}

impl ExtMetadataBlockLevel5 {
    pub fn parse(reader: &mut BitVecReader) -> ExtMetadataBlock {
        ExtMetadataBlock::Level5(Self {
            active_area_left_offset: reader.get_n(13),
            active_area_right_offset: reader.get_n(13),
            active_area_top_offset: reader.get_n(13),
            active_area_bottom_offset: reader.get_n(13),
        })
    }

    pub fn write(&self, writer: &mut BitVecWriter) {
        writer.write_n(&self.active_area_left_offset.to_be_bytes(), 13);
        writer.write_n(&self.active_area_right_offset.to_be_bytes(), 13);
        writer.write_n(&self.active_area_top_offset.to_be_bytes(), 13);
        writer.write_n(&self.active_area_bottom_offset.to_be_bytes(), 13);
    }

    pub fn get_offsets(&self) -> (u16, u16, u16, u16) {
        (
            self.active_area_left_offset,
            self.active_area_right_offset,
            self.active_area_top_offset,
            self.active_area_bottom_offset,
        )
    }

    pub fn get_offsets_vec(&self) -> Vec<u16> {
        vec![
            self.active_area_left_offset,
            self.active_area_right_offset,
            self.active_area_top_offset,
            self.active_area_bottom_offset,
        ]
    }

    pub fn set_offsets(&mut self, left: u16, right: u16, top: u16, bottom: u16) {
        self.active_area_left_offset = left;
        self.active_area_right_offset = right;
        self.active_area_top_offset = top;
        self.active_area_bottom_offset = bottom;
    }

    pub fn crop(&mut self) {
        self.active_area_left_offset = 0;
        self.active_area_right_offset = 0;
        self.active_area_top_offset = 0;
        self.active_area_bottom_offset = 0;
    }
}
