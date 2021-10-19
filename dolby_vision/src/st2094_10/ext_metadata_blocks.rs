use anyhow::{ensure, Result};
use bitvec::prelude::*;
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use super::generate::Level6Metadata;

#[derive(Debug)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub enum ExtMetadataBlock {
    Level1(ExtMetadataBlockLevel1),
    Level2(ExtMetadataBlockLevel2),
    Level3(ExtMetadataBlockLevel3),
    Level4(ExtMetadataBlockLevel4),
    Level5(ExtMetadataBlockLevel5),
    Level6(ExtMetadataBlockLevel6),
    Reserved(ReservedExtMetadataBlock),
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct BlockInfo {
    pub ext_block_length: u64,
    pub ext_block_level: u8,

    #[cfg_attr(
        feature = "serde_feature",
        serde(serialize_with = "crate::utils::bitvec_ser_bits")
    )]
    pub remaining: BitVec<Msb0, u8>,
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct ExtMetadataBlockLevel1 {
    pub block_info: BlockInfo,
    pub min_pq: u16,
    pub max_pq: u16,
    pub avg_pq: u16,
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct ExtMetadataBlockLevel2 {
    pub block_info: BlockInfo,
    pub target_max_pq: u16,
    pub trim_slope: u16,
    pub trim_offset: u16,
    pub trim_power: u16,
    pub trim_chroma_weight: u16,
    pub trim_saturation_gain: u16,
    pub ms_weight: i16,
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct ExtMetadataBlockLevel3 {
    pub block_info: BlockInfo,
    pub min_pq_offset: u16,
    pub max_pq_offset: u16,
    pub avg_pq_offset: u16,
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct ExtMetadataBlockLevel4 {
    pub block_info: BlockInfo,
    pub anchor_pq: u16,
    pub anchor_power: u16,
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct ExtMetadataBlockLevel5 {
    pub block_info: BlockInfo,
    pub active_area_left_offset: u16,
    pub active_area_right_offset: u16,
    pub active_area_top_offset: u16,
    pub active_area_bottom_offset: u16,
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct ExtMetadataBlockLevel6 {
    pub block_info: BlockInfo,
    pub max_display_mastering_luminance: u16,
    pub min_display_mastering_luminance: u16,
    pub max_content_light_level: u16,
    pub max_frame_average_light_level: u16,
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct ReservedExtMetadataBlock {
    pub block_info: BlockInfo,
}

impl ExtMetadataBlock {
    pub fn parse(reader: &mut BitVecReader) -> Result<ExtMetadataBlock> {
        let mut block_info = BlockInfo {
            ext_block_length: reader.get_ue(),
            ext_block_level: reader.get_n(8),
            ..Default::default()
        };

        let ext_block_len_bits = 8 * block_info.ext_block_length;
        let mut ext_block_use_bits = 0;

        let mut ext_metadata_block = match block_info.ext_block_level {
            1 => {
                ensure!(
                    block_info.ext_block_length == 5,
                    "level 1 block should have length 5"
                );

                let block = ExtMetadataBlockLevel1 {
                    min_pq: reader.get_n(12),
                    max_pq: reader.get_n(12),
                    avg_pq: reader.get_n(12),
                    ..Default::default()
                };

                ext_block_use_bits += 36;

                ExtMetadataBlock::Level1(block)
            }
            2 => {
                ensure!(
                    block_info.ext_block_length == 11,
                    "level 2 block should have length 11"
                );

                let block = ExtMetadataBlockLevel2 {
                    target_max_pq: reader.get_n(12),
                    trim_slope: reader.get_n(12),
                    trim_offset: reader.get_n(12),
                    trim_power: reader.get_n(12),
                    trim_chroma_weight: reader.get_n(12),
                    trim_saturation_gain: reader.get_n(12),
                    ms_weight: reader.get_n::<u16>(13) as i16,
                    ..Default::default()
                };

                ext_block_use_bits += 85;

                ExtMetadataBlock::Level2(block)
            }
            3 => {
                ensure!(
                    block_info.ext_block_length == 2,
                    "level 3 block should have length 2"
                );

                let block = ExtMetadataBlockLevel3 {
                    min_pq_offset: reader.get_n(12),
                    max_pq_offset: reader.get_n(12),
                    avg_pq_offset: reader.get_n(12),
                    ..Default::default()
                };

                ext_block_use_bits += 36;

                ExtMetadataBlock::Level3(block)
            }
            4 => {
                ensure!(
                    block_info.ext_block_length == 3,
                    "level 4 block should have length 4"
                );

                let block = ExtMetadataBlockLevel4 {
                    anchor_pq: reader.get_n(12),
                    anchor_power: reader.get_n(12),
                    ..Default::default()
                };

                ext_block_use_bits += 24;

                ExtMetadataBlock::Level4(block)
            }
            5 => {
                ensure!(
                    block_info.ext_block_length == 7,
                    "level 5 block should have length 7"
                );

                let block = ExtMetadataBlockLevel5 {
                    active_area_left_offset: reader.get_n(13),
                    active_area_right_offset: reader.get_n(13),
                    active_area_top_offset: reader.get_n(13),
                    active_area_bottom_offset: reader.get_n(13),
                    ..Default::default()
                };

                ext_block_use_bits += 52;

                ExtMetadataBlock::Level5(block)
            }
            6 => {
                ensure!(
                    block_info.ext_block_length == 8,
                    "level 6 block should have length 8"
                );

                let block = ExtMetadataBlockLevel6 {
                    max_display_mastering_luminance: reader.get_n(16),
                    min_display_mastering_luminance: reader.get_n(16),
                    max_content_light_level: reader.get_n(16),
                    max_frame_average_light_level: reader.get_n(16),
                    ..Default::default()
                };

                ext_block_use_bits += 64;

                ExtMetadataBlock::Level6(block)
            }
            _ => {
                ensure!(
                    false,
                    "Reserved metadata block found, please open an issue."
                );

                let block = ReservedExtMetadataBlock::default();
                ExtMetadataBlock::Reserved(block)
            }
        };

        while ext_block_use_bits < ext_block_len_bits {
            block_info.remaining.push(reader.get());
            ext_block_use_bits += 1;
        }

        match ext_metadata_block {
            ExtMetadataBlock::Level1(ref mut b) => b.block_info = block_info,
            ExtMetadataBlock::Level2(ref mut b) => b.block_info = block_info,
            ExtMetadataBlock::Level3(ref mut b) => b.block_info = block_info,
            ExtMetadataBlock::Level4(ref mut b) => b.block_info = block_info,
            ExtMetadataBlock::Level5(ref mut b) => b.block_info = block_info,
            ExtMetadataBlock::Level6(ref mut b) => b.block_info = block_info,
            ExtMetadataBlock::Reserved(ref mut b) => b.block_info = block_info,
        }

        Ok(ext_metadata_block)
    }

    pub fn write(&self, writer: &mut BitVecWriter) {
        let block_info = match self {
            ExtMetadataBlock::Level1(b) => &b.block_info,
            ExtMetadataBlock::Level2(b) => &b.block_info,
            ExtMetadataBlock::Level3(b) => &b.block_info,
            ExtMetadataBlock::Level4(b) => &b.block_info,
            ExtMetadataBlock::Level5(b) => &b.block_info,
            ExtMetadataBlock::Level6(b) => &b.block_info,
            ExtMetadataBlock::Reserved(b) => &b.block_info,
        };

        writer.write_ue(block_info.ext_block_length);
        writer.write_n(&block_info.ext_block_level.to_be_bytes(), 8);

        match self {
            ExtMetadataBlock::Level1(block) => {
                writer.write_n(&block.min_pq.to_be_bytes(), 12);
                writer.write_n(&block.max_pq.to_be_bytes(), 12);
                writer.write_n(&block.avg_pq.to_be_bytes(), 12);
            }
            ExtMetadataBlock::Level2(block) => {
                writer.write_n(&block.target_max_pq.to_be_bytes(), 12);
                writer.write_n(&block.trim_slope.to_be_bytes(), 12);
                writer.write_n(&block.trim_offset.to_be_bytes(), 12);
                writer.write_n(&block.trim_power.to_be_bytes(), 12);
                writer.write_n(&block.trim_chroma_weight.to_be_bytes(), 12);
                writer.write_n(&block.trim_saturation_gain.to_be_bytes(), 12);

                writer.write_n(&block.ms_weight.to_be_bytes(), 13);
            }
            ExtMetadataBlock::Level3(block) => {
                writer.write_n(&block.min_pq_offset.to_be_bytes(), 12);
                writer.write_n(&block.max_pq_offset.to_be_bytes(), 12);
                writer.write_n(&block.avg_pq_offset.to_be_bytes(), 12);
            }
            ExtMetadataBlock::Level4(block) => {
                writer.write_n(&block.anchor_pq.to_be_bytes(), 12);
                writer.write_n(&block.anchor_power.to_be_bytes(), 12);
            }
            ExtMetadataBlock::Level5(block) => {
                writer.write_n(&block.active_area_left_offset.to_be_bytes(), 13);
                writer.write_n(&block.active_area_right_offset.to_be_bytes(), 13);
                writer.write_n(&block.active_area_top_offset.to_be_bytes(), 13);
                writer.write_n(&block.active_area_bottom_offset.to_be_bytes(), 13);
            }
            ExtMetadataBlock::Level6(block) => {
                writer.write_n(&block.max_display_mastering_luminance.to_be_bytes(), 16);
                writer.write_n(&block.min_display_mastering_luminance.to_be_bytes(), 16);
                writer.write_n(&block.max_content_light_level.to_be_bytes(), 16);
                writer.write_n(&block.max_frame_average_light_level.to_be_bytes(), 16);
            }
            ExtMetadataBlock::Reserved(_) => {
                // Copy the data
                block_info.remaining.iter().for_each(|b| writer.write(*b));
            }
        }

        // Write zero bytes until aligned
        match self {
            ExtMetadataBlock::Reserved(_) => (),
            _ => block_info
                .remaining
                .iter()
                .for_each(|_| writer.write(false)),
        }
    }
}

impl ExtMetadataBlockLevel5 {
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

impl ExtMetadataBlockLevel6 {
    pub fn set_fields_from_generate_l6(&mut self, meta: &Level6Metadata) {
        self.max_display_mastering_luminance = meta.max_display_mastering_luminance;
        self.min_display_mastering_luminance = meta.min_display_mastering_luminance;
        self.max_content_light_level = meta.max_content_light_level;
        self.max_frame_average_light_level = meta.max_frame_average_light_level;
    }
}
