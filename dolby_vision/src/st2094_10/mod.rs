use anyhow::{bail, ensure, Result};
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use crate::utils::nits_to_pq;

pub mod generate;
pub mod metadata_blocks;

pub use metadata_blocks::*;

use generate::*;

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct ST2094_10Meta {
    pub num_ext_blocks: u64,
    pub ext_metadata_blocks: Vec<ExtMetadataBlock>,
}

impl ST2094_10Meta {
    pub fn parse_itu_t35(data: Vec<u8>) -> Result<ST2094_10Meta> {
        let _meta = ST2094_10Meta::default();
        let mut reader = BitVecReader::new(data);

        let itu_t_t35_country_code: u8 = reader.get_n(8);
        let itu_t_t35_terminal_provider_code: u16 = reader.get_n(16);

        ensure!(itu_t_t35_country_code == 0xB5);
        ensure!(itu_t_t35_terminal_provider_code == 0x3B);

        let _itu_t_t35_terminal_provider_oriented_code: u32 = reader.get_n(32);
        let _data_type_code: u8 = reader.get_n(8);

        let app_identifier: u64 = reader.get_ue();
        let app_version: u64 = reader.get_ue();

        println!("App {} version {}", app_identifier, app_version);

        todo!()
    }

    pub fn parse(reader: &mut BitVecReader) -> Result<ST2094_10Meta> {
        let mut meta = ST2094_10Meta {
            num_ext_blocks: reader.get_ue(),
            ..Default::default()
        };

        if meta.num_ext_blocks > 0 {
            while !reader.is_aligned() {
                ensure!(!reader.get(), "dm_alignment_zero_bit != 0");
            }

            for _ in 0..meta.num_ext_blocks {
                let ext_metadata_block = ext_metadata_block(reader)?;
                meta.ext_metadata_blocks.push(ext_metadata_block);
            }
        }

        Ok(meta)
    }

    pub fn write(&self, writer: &mut BitVecWriter) {
        writer.write_ue(self.num_ext_blocks);

        if self.num_ext_blocks > 0 {
            // dm_alignment_zero_bit
            while !writer.is_aligned() {
                writer.write(false);
            }

            for ext_metadata_block in &self.ext_metadata_blocks {
                let remaining_bits = (8 * ext_metadata_block.length()) - ext_metadata_block.bits();

                writer.write_ue(ext_metadata_block.length());
                writer.write_n(&ext_metadata_block.level().to_be_bytes(), 8);

                ext_metadata_block.write(writer);

                // ext_dm_alignment_zero_bit
                (0..remaining_bits).for_each(|_| writer.write(false));
            }
        }
    }

    pub fn sort_extension_blocks(&mut self) {
        self.ext_metadata_blocks.sort_by_key(|ext| ext.sort_key());
    }

    pub fn update_extension_block_info(&mut self) {
        self.num_ext_blocks = self.ext_metadata_blocks.len() as u64;
        self.sort_extension_blocks();
    }

    pub fn add_level1_metadata(&mut self, min_pq: u16, max_pq: u16, avg_pq: u16) {
        let ext_metadata_block_level1 = level1::ExtMetadataBlockLevel1 {
            min_pq,
            max_pq,
            avg_pq,
        };

        self.ext_metadata_blocks
            .push(ExtMetadataBlock::Level1(ext_metadata_block_level1));
        self.update_extension_block_info();
    }

    pub fn add_level2_metadata(&mut self, level2_config: &Level2Metadata) -> Result<()> {
        let target_max_pq = if let Some(target_nits) = level2_config.target_nits {
            (nits_to_pq(target_nits) * 4095.0).round() as u16
        } else if let Some(target_pq) = level2_config.target_max_pq {
            target_pq
        } else {
            bail!("Level2 metadata: peak brightness of the target display is required");
        };

        let ext_metadata_block_level2 = level2::ExtMetadataBlockLevel2 {
            target_max_pq,
            trim_slope: level2_config.trim_slope,
            trim_offset: level2_config.trim_offset,
            trim_power: level2_config.trim_power,
            trim_chroma_weight: level2_config.trim_chroma_weight,
            trim_saturation_gain: level2_config.trim_saturation_gain,
            ms_weight: level2_config.ms_weight,
        };

        self.ext_metadata_blocks
            .push(ExtMetadataBlock::Level2(ext_metadata_block_level2));
        self.update_extension_block_info();

        Ok(())
    }

    pub fn add_level3_metadata(
        &mut self,
        min_pq_offset: u16,
        max_pq_offset: u16,
        avg_pq_offset: u16,
    ) {
        let ext_metadata_block_level3 = level3::ExtMetadataBlockLevel3 {
            min_pq_offset,
            max_pq_offset,
            avg_pq_offset,
        };

        self.ext_metadata_blocks
            .push(ExtMetadataBlock::Level3(ext_metadata_block_level3));
        self.update_extension_block_info();
    }

    pub fn set_level5_metadata(&mut self, left: u16, right: u16, top: u16, bottom: u16) {
        self.ext_metadata_blocks.retain(|b| b.level() != 5);

        // Add it back with zero offsets
        self.add_level5_metadata(left, right, top, bottom);
    }

    pub fn add_level5_metadata(&mut self, left: u16, right: u16, top: u16, bottom: u16) {
        let ext_metadata_block_level5 = level5::ExtMetadataBlockLevel5 {
            active_area_left_offset: left,
            active_area_right_offset: right,
            active_area_top_offset: top,
            active_area_bottom_offset: bottom,
        };

        self.ext_metadata_blocks
            .push(ExtMetadataBlock::Level5(ext_metadata_block_level5));
        self.update_extension_block_info();
    }

    pub fn set_level6_metadata(&mut self, level6_config: &Level6Metadata) {
        self.ext_metadata_blocks.retain(|b| b.level() != 6);

        // Add it back with zero offsets
        self.add_level6_metadata(level6_config);
    }

    pub fn add_level6_metadata(&mut self, level6_config: &Level6Metadata) {
        let ext_metadata_block_level6 = level6::ExtMetadataBlockLevel6 {
            max_display_mastering_luminance: level6_config.max_display_mastering_luminance,
            min_display_mastering_luminance: level6_config.min_display_mastering_luminance,
            max_content_light_level: level6_config.max_content_light_level,
            max_frame_average_light_level: level6_config.max_frame_average_light_level,
        };

        self.ext_metadata_blocks
            .push(ExtMetadataBlock::Level6(ext_metadata_block_level6));
        self.update_extension_block_info();
    }
}
