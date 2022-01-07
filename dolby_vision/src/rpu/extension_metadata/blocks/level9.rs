use anyhow::{ensure, Result};
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

use crate::rpu::extension_metadata::MasteringDisplayPrimaries;

use super::{ColorPrimaries, ExtMetadataBlock, ExtMetadataBlockInfo};

/// Source/mastering display color primaries
///
/// This block can have varying byte lengths: 1 or 17
/// Depending on the length, the fields parsed default to zero and may not be set.
/// Up to (including):
///     - 1: source_primary_index
///     - 17: source_primary_{red,green,blue,white}_{x,y}
#[repr(C)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize))]
pub struct ExtMetadataBlockLevel9 {
    pub length: u64,
    pub source_primary_index: u8,

    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub source_primary_red_x: u16,
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub source_primary_red_y: u16,
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub source_primary_green_x: u16,
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub source_primary_green_y: u16,
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub source_primary_blue_x: u16,
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub source_primary_blue_y: u16,
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub source_primary_white_x: u16,
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub source_primary_white_y: u16,
}

impl ExtMetadataBlockLevel9 {
    pub fn parse(reader: &mut BitVecReader, length: u64) -> ExtMetadataBlock {
        let mut block = Self {
            length,
            source_primary_index: reader.get_n(8),
            ..Default::default()
        };

        if length > 1 {
            block.source_primary_red_x = reader.get_n(16);
            block.source_primary_red_y = reader.get_n(16);
            block.source_primary_green_x = reader.get_n(16);
            block.source_primary_green_y = reader.get_n(16);
            block.source_primary_blue_x = reader.get_n(16);
            block.source_primary_blue_y = reader.get_n(16);
            block.source_primary_white_x = reader.get_n(16);
            block.source_primary_white_y = reader.get_n(16);
        }

        ExtMetadataBlock::Level9(block)
    }

    pub fn write(&self, writer: &mut BitVecWriter) -> Result<()> {
        self.validate()?;

        writer.write_n(&self.source_primary_index.to_be_bytes(), 8);

        if self.length > 1 {
            writer.write_n(&self.source_primary_red_x.to_be_bytes(), 16);
            writer.write_n(&self.source_primary_red_y.to_be_bytes(), 16);
            writer.write_n(&self.source_primary_green_x.to_be_bytes(), 16);
            writer.write_n(&self.source_primary_green_y.to_be_bytes(), 16);
            writer.write_n(&self.source_primary_blue_x.to_be_bytes(), 16);
            writer.write_n(&self.source_primary_blue_y.to_be_bytes(), 16);
            writer.write_n(&self.source_primary_white_x.to_be_bytes(), 16);
            writer.write_n(&self.source_primary_white_y.to_be_bytes(), 16);
        }

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if self.length > 1 {
            // Custom primaries required
            ensure!(self.source_primary_index == 255);
            ensure!(self.source_primary_red_x > 0);
            ensure!(self.source_primary_red_y > 0);
            ensure!(self.source_primary_green_x > 0);
            ensure!(self.source_primary_green_y > 0);
            ensure!(self.source_primary_blue_x > 0);
            ensure!(self.source_primary_blue_y > 0);
            ensure!(self.source_primary_white_x > 0);
            ensure!(self.source_primary_white_y > 0);
        } else {
            // Should be a preset primary between 0-8?
            // But not custom primaries
            ensure!(self.source_primary_index != 255);
        }

        Ok(())
    }

    pub fn set_from_primaries(&mut self, primaries: &ColorPrimaries) {
        self.source_primary_red_x = primaries.red_x;
        self.source_primary_red_y = primaries.red_y;
        self.source_primary_green_x = primaries.green_x;
        self.source_primary_green_y = primaries.green_y;
        self.source_primary_blue_x = primaries.blue_x;
        self.source_primary_blue_y = primaries.blue_y;
        self.source_primary_white_x = primaries.white_x;
        self.source_primary_white_y = primaries.white_y;
    }

    pub fn default_dci_p3() -> ExtMetadataBlockLevel9 {
        Self {
            length: 1,
            source_primary_index: MasteringDisplayPrimaries::DCIP3D65 as u8,
            ..Default::default()
        }
    }
}

impl ExtMetadataBlockInfo for ExtMetadataBlockLevel9 {
    fn level(&self) -> u8 {
        9
    }

    fn bytes_size(&self) -> u64 {
        self.length
    }

    fn required_bits(&self) -> u64 {
        match self.length {
            1 => 8,
            17 => 136,
            _ => unreachable!(),
        }
    }

    fn sort_key(&self) -> (u8, u16) {
        (self.level(), self.source_primary_index as u16)
    }
}

impl Default for ExtMetadataBlockLevel9 {
    /// DCI-P3 D65 preset
    fn default() -> Self {
        Self {
            length: 1,
            source_primary_index: MasteringDisplayPrimaries::DCIP3D65 as u8,
            source_primary_red_x: 0,
            source_primary_red_y: 0,
            source_primary_green_x: 0,
            source_primary_green_y: 0,
            source_primary_blue_x: 0,
            source_primary_blue_y: 0,
            source_primary_white_x: 0,
            source_primary_white_y: 0,
        }
    }
}

#[cfg(feature = "serde_feature")]
impl Serialize for ExtMetadataBlockLevel9 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let name = "ExtMetadataBlockLevel9";
        let fields_count = match self.length {
            1 => 2,
            17 => 10,
            _ => unreachable!(),
        };

        let mut state = serializer.serialize_struct(name, fields_count)?;

        state.serialize_field("length", &self.length)?;
        state.serialize_field("source_primary_index", &self.source_primary_index)?;

        if self.length > 1 {
            state.serialize_field("source_primary_red_x", &self.source_primary_red_x)?;
            state.serialize_field("source_primary_red_y", &self.source_primary_red_y)?;
            state.serialize_field("source_primary_green_x", &self.source_primary_green_x)?;
            state.serialize_field("source_primary_green_y", &self.source_primary_green_y)?;
            state.serialize_field("source_primary_blue_x", &self.source_primary_blue_x)?;
            state.serialize_field("source_primary_blue_y", &self.source_primary_blue_y)?;
            state.serialize_field("source_primary_white_x", &self.source_primary_white_x)?;
            state.serialize_field("source_primary_white_y", &self.source_primary_white_y)?;
        }

        state.end()
    }
}
