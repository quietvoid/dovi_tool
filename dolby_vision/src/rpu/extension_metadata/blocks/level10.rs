use anyhow::{Result, bail, ensure};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};

use super::{ColorPrimaries, ExtMetadataBlock, ExtMetadataBlockInfo, level6::MAX_PQ_LUMINANCE};

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

#[derive(PartialEq, Eq)]
pub(crate) enum Level10Length {
    PresetPrimaries = 5,
    CustomPrimaries = 21,
}

impl ExtMetadataBlockLevel10 {
    pub(crate) fn parse(
        reader: &mut BsIoSliceReader,
        block_length: u64,
    ) -> Result<ExtMetadataBlock> {
        let length = Level10Length::try_from(block_length)?;

        let mut block = Self {
            length: block_length,
            target_display_index: reader.read::<8, u8>()?,
            target_max_pq: reader.read::<12, u16>()?,
            target_min_pq: reader.read::<12, u16>()?,
            target_primary_index: reader.read::<8, u8>()?,
            ..Default::default()
        };

        if length == Level10Length::CustomPrimaries {
            block.target_primary_red_x = reader.read::<16, u16>()?;
            block.target_primary_red_y = reader.read::<16, u16>()?;
            block.target_primary_green_x = reader.read::<16, u16>()?;
            block.target_primary_green_y = reader.read::<16, u16>()?;
            block.target_primary_blue_x = reader.read::<16, u16>()?;
            block.target_primary_blue_y = reader.read::<16, u16>()?;
            block.target_primary_white_x = reader.read::<16, u16>()?;
            block.target_primary_white_y = reader.read::<16, u16>()?;
        }

        Ok(ExtMetadataBlock::Level10(block))
    }

    pub(crate) fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        let length = Level10Length::try_from(self.length)?;

        writer.write::<8, u8>(self.target_display_index)?;
        writer.write::<12, u16>(self.target_max_pq)?;
        writer.write::<12, u16>(self.target_min_pq)?;
        writer.write::<8, u8>(self.target_primary_index)?;

        if length == Level10Length::CustomPrimaries {
            writer.write::<16, u16>(self.target_primary_red_x)?;
            writer.write::<16, u16>(self.target_primary_red_y)?;
            writer.write::<16, u16>(self.target_primary_green_x)?;
            writer.write::<16, u16>(self.target_primary_green_y)?;
            writer.write::<16, u16>(self.target_primary_blue_x)?;
            writer.write::<16, u16>(self.target_primary_blue_y)?;
            writer.write::<16, u16>(self.target_primary_white_x)?;
            writer.write::<16, u16>(self.target_primary_white_y)?;
        }

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        let length = Level10Length::try_from(self.length)?;

        ensure!(!PRESET_TARGET_DISPLAYS.contains(&self.target_display_index));
        ensure!(self.target_max_pq <= MAX_PQ_LUMINANCE);
        ensure!(self.target_min_pq <= MAX_PQ_LUMINANCE);

        if length == Level10Length::CustomPrimaries {
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
        let Ok(length) = Level10Length::try_from(self.length) else {
            // validation is performed before this is called
            unreachable!();
        };

        length.required_bits()
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
        let name = "ExtMetadataBlockLevel10";
        let length = Level10Length::try_from(self.length).map_err(serde::ser::Error::custom)?;
        let fields_count = length.serialize_fields_count();

        let mut state = serializer.serialize_struct(name, fields_count)?;

        state.serialize_field("length", &self.length)?;
        state.serialize_field("target_display_index", &self.target_display_index)?;
        state.serialize_field("target_max_pq", &self.target_max_pq)?;
        state.serialize_field("target_min_pq", &self.target_min_pq)?;
        state.serialize_field("target_primary_index", &self.target_primary_index)?;

        if length == Level10Length::CustomPrimaries {
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

impl Level10Length {
    const fn required_bits(&self) -> u64 {
        match self {
            Self::PresetPrimaries => 40,
            Self::CustomPrimaries => 168,
        }
    }

    #[cfg(feature = "serde")]
    const fn serialize_fields_count(&self) -> usize {
        match self {
            Self::PresetPrimaries => 5,
            Self::CustomPrimaries => 13,
        }
    }
}

impl TryFrom<u64> for Level10Length {
    type Error = anyhow::Error;

    fn try_from(value: u64) -> Result<Self> {
        let gate = match value {
            5 => Self::PresetPrimaries,
            21 => Self::CustomPrimaries,
            _ => bail!("invalid level 10 length"),
        };

        Ok(gate)
    }
}
