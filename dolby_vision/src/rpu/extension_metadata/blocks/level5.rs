use anyhow::{ensure, Result};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{ExtMetadataBlock, ExtMetadataBlockInfo};

const MAX_RESOLUTION_13_BITS: u16 = 8191;

/// Active area of the picture (letterbox, aspect ratio)
#[repr(C)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ExtMetadataBlockLevel5 {
    pub active_area_left_offset: u16,
    pub active_area_right_offset: u16,
    pub active_area_top_offset: u16,
    pub active_area_bottom_offset: u16,
}

impl ExtMetadataBlockLevel5 {
    pub(crate) fn parse(reader: &mut BsIoSliceReader) -> Result<ExtMetadataBlock> {
        Ok(ExtMetadataBlock::Level5(Self {
            active_area_left_offset: reader.get_n(13)?,
            active_area_right_offset: reader.get_n(13)?,
            active_area_top_offset: reader.get_n(13)?,
            active_area_bottom_offset: reader.get_n(13)?,
        }))
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        self.validate()?;

        writer.write_n(&self.active_area_left_offset, 13)?;
        writer.write_n(&self.active_area_right_offset, 13)?;
        writer.write_n(&self.active_area_top_offset, 13)?;
        writer.write_n(&self.active_area_bottom_offset, 13)?;

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(self.active_area_left_offset <= MAX_RESOLUTION_13_BITS);
        ensure!(self.active_area_right_offset <= MAX_RESOLUTION_13_BITS);
        ensure!(self.active_area_top_offset <= MAX_RESOLUTION_13_BITS);
        ensure!(self.active_area_bottom_offset <= MAX_RESOLUTION_13_BITS);

        Ok(())
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

    pub fn from_offsets(left: u16, right: u16, top: u16, bottom: u16) -> Self {
        ExtMetadataBlockLevel5 {
            active_area_left_offset: left,
            active_area_right_offset: right,
            active_area_top_offset: top,
            active_area_bottom_offset: bottom,
        }
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel5 {
    fn level(&self) -> u8 {
        5
    }

    fn bytes_size(&self) -> u64 {
        7
    }

    fn required_bits(&self) -> u64 {
        52
    }
}
