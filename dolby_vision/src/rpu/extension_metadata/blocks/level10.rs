use anyhow::{ensure, Result};
use bitvec_helpers::{bitslice_reader::BitSliceReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde")]
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

use super::{level6::MAX_PQ_LUMINANCE, ColorPrimaries, ExtMetadataBlock, ExtMetadataBlockInfo};

pub const PRESET_TARGET_DISPLAYS: &[u8] = &[1, 16, 18, 21, 27, 28, 37, 38, 42, 48, 49];

/// Custom target display information
///
/// This block can have varying byte lengths: 5 or 21
/// Depending on the length, the fields parsed default to zero and may not be set.
/// Up to (including):
///     - 5: target_primary_index
///     - 21: target_primary_{red,green,blue,white}_{x,y}
#[repr(C)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct ExtMetadataBlockLevel10 {
    pub length: u64,
    pub target_display_index: u8,
    pub target_max_pq: u16,
    pub target_min_pq: u16,
    pub target_primary_index: u8,

    pub target_primary_red_x: u16,
    pub target_primary_red_y: u16,
    pub target_primary_green_x: u16,
    pub target_primary_green_y: u16,
    pub target_primary_blue_x: u16,
    pub target_primary_blue_y: u16,
    pub target_primary_white_x: u16,
    pub target_primary_white_y: u16,
}

impl ExtMetadataBlockLevel10 {
    pub(crate) fn parse(reader: &mut BitSliceReader, length: u64) -> Result<ExtMetadataBlock> {
        let mut block = Self {
            length,
            target_display_index: reader.get_n(8)?,
            target_max_pq: reader.get_n(12)?,
            target_min_pq: reader.get_n(12)?,
            target_primary_index: reader.get_n(8)?,
            ..Default::default()
        };

        if length > 5 {
            block.target_primary_red_x = reader.get_n(16)?;
            block.target_primary_red_y = reader.get_n(16)?;
            block.target_primary_green_x = reader.get_n(16)?;
            block.target_primary_green_y = reader.get_n(16)?;
            block.target_primary_blue_x = reader.get_n(16)?;
            block.target_primary_blue_y = reader.get_n(16)?;
            block.target_primary_white_x = reader.get_n(16)?;
            block.target_primary_white_y = reader.get_n(16)?;
        }

        Ok(ExtMetadataBlock::Level10(block))
    }

    pub fn write(&self, writer: &mut BitVecWriter) -> Result<()> {
        self.validate()?;

        writer.write_n(&self.target_display_index, 8);
        writer.write_n(&self.target_max_pq, 12);
        writer.write_n(&self.target_min_pq, 12);
        writer.write_n(&self.target_primary_index, 8);

        if self.length > 5 {
            writer.write_n(&self.target_primary_red_x, 16);
            writer.write_n(&self.target_primary_red_y, 16);
            writer.write_n(&self.target_primary_green_x, 16);
            writer.write_n(&self.target_primary_green_y, 16);
            writer.write_n(&self.target_primary_blue_x, 16);
            writer.write_n(&self.target_primary_blue_y, 16);
            writer.write_n(&self.target_primary_white_x, 16);
            writer.write_n(&self.target_primary_white_y, 16);
        }

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(!PRESET_TARGET_DISPLAYS.contains(&self.target_display_index));
        ensure!(self.target_max_pq <= MAX_PQ_LUMINANCE);
        ensure!(self.target_min_pq <= MAX_PQ_LUMINANCE);

        if self.length > 5 {
            ensure!(self.target_primary_index == 255);
            ensure!(self.target_primary_red_x > 0);
            ensure!(self.target_primary_red_y > 0);
            ensure!(self.target_primary_green_x > 0);
            ensure!(self.target_primary_green_y > 0);
            ensure!(self.target_primary_blue_x > 0);
            ensure!(self.target_primary_blue_y > 0);
            ensure!(self.target_primary_white_x > 0);
            ensure!(self.target_primary_white_y > 0);
        } else {
            ensure!(self.target_primary_index != 255);
        }

        Ok(())
    }

    pub fn set_from_primaries(&mut self, primaries: &ColorPrimaries) {
        self.target_primary_red_x = primaries.red_x;
        self.target_primary_red_y = primaries.red_y;
        self.target_primary_green_x = primaries.green_x;
        self.target_primary_green_y = primaries.green_y;
        self.target_primary_blue_x = primaries.blue_x;
        self.target_primary_blue_y = primaries.blue_y;
        self.target_primary_white_x = primaries.white_x;
        self.target_primary_white_y = primaries.white_y;
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel10 {
    fn level(&self) -> u8 {
        10
    }

    fn bytes_size(&self) -> u64 {
        self.length
    }

    fn required_bits(&self) -> u64 {
        match self.length {
            5 => 40,
            21 => 168,
            _ => unreachable!(),
        }
    }

    fn sort_key(&self) -> (u8, u16) {
        (self.level(), self.target_display_index as u16)
    }
}

impl Default for ExtMetadataBlockLevel10 {
    fn default() -> Self {
        Self {
            length: 5,
            target_display_index: 20,
            target_max_pq: 2081,
            target_min_pq: 0,
            target_primary_index: 2,
            target_primary_red_x: 0,
            target_primary_red_y: 0,
            target_primary_green_x: 0,
            target_primary_green_y: 0,
            target_primary_blue_x: 0,
            target_primary_blue_y: 0,
            target_primary_white_x: 0,
            target_primary_white_y: 0,
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for ExtMetadataBlockLevel10 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let name = "ExtMetadataBlockLevel9";
        let fields_count = match self.length {
            5 => 5,
            21 => 13,
            _ => unreachable!(),
        };

        let mut state = serializer.serialize_struct(name, fields_count)?;

        state.serialize_field("length", &self.length)?;
        state.serialize_field("target_display_index", &self.target_display_index)?;
        state.serialize_field("target_max_pq", &self.target_max_pq)?;
        state.serialize_field("target_min_pq", &self.target_min_pq)?;
        state.serialize_field("target_primary_index", &self.target_primary_index)?;

        if self.length > 5 {
            state.serialize_field("target_primary_red_x", &self.target_primary_red_x)?;
            state.serialize_field("target_primary_red_y", &self.target_primary_red_y)?;
            state.serialize_field("target_primary_green_x", &self.target_primary_green_x)?;
            state.serialize_field("target_primary_green_y", &self.target_primary_green_y)?;
            state.serialize_field("target_primary_blue_x", &self.target_primary_blue_x)?;
            state.serialize_field("target_primary_blue_y", &self.target_primary_blue_y)?;
            state.serialize_field("target_primary_white_x", &self.target_primary_white_x)?;
            state.serialize_field("target_primary_white_y", &self.target_primary_white_y)?;
        }

        state.end()
    }
}
