use bitvec::prelude::*;

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

use super::{ext_metadata_blocks::*, ST2094_10Meta};

#[derive(Default, Debug)]
#[cfg_attr(feature = "serde_feature", derive(Serialize, Deserialize))]
pub struct GenerateConfig {
    pub length: u64,
    pub target_nits: Option<u16>,

    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub source_min_pq: Option<u16>,

    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub source_max_pq: Option<u16>,

    pub level2: Option<Vec<Level2Metadata>>,
    pub level5: Option<Level5Metadata>,
    pub level6: Option<Level6Metadata>,
}

#[derive(Default, Debug, Clone)]
pub struct Level1Metadata {
    pub min_pq: u16,
    pub max_pq: u16,
    pub avg_pq: u16,
}

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Serialize, Deserialize))]
pub struct Level2Metadata {
    pub target_nits: u16,

    #[cfg_attr(feature = "serde_feature", serde(default = "default_trim"))]
    pub trim_slope: u16,
    #[cfg_attr(feature = "serde_feature", serde(default = "default_trim"))]
    pub trim_offset: u16,
    #[cfg_attr(feature = "serde_feature", serde(default = "default_trim"))]
    pub trim_power: u16,
    #[cfg_attr(feature = "serde_feature", serde(default = "default_trim"))]
    pub trim_chroma_weight: u16,
    #[cfg_attr(feature = "serde_feature", serde(default = "default_trim"))]
    pub trim_saturation_gain: u16,
    #[cfg_attr(feature = "serde_feature", serde(default = "default_trim_neg"))]
    pub ms_weight: i16,
}

#[derive(Default, Debug, Clone)]
pub struct Level3Metadata {
    pub min_pq_offset: u16,
    pub max_pq_offset: u16,
    pub avg_pq_offset: u16,
}

#[derive(Default, Debug)]
#[cfg_attr(feature = "serde_feature", derive(Serialize, Deserialize))]
pub struct Level5Metadata {
    pub active_area_left_offset: u16,
    pub active_area_right_offset: u16,
    pub active_area_top_offset: u16,
    pub active_area_bottom_offset: u16,
}

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Serialize, Deserialize))]
pub struct Level6Metadata {
    pub max_display_mastering_luminance: u16,
    pub min_display_mastering_luminance: u16,
    pub max_content_light_level: u16,
    pub max_frame_average_light_level: u16,
}

impl ST2094_10Meta {
    pub fn update_from_config(&mut self, config: &GenerateConfig) {
        self.set_level2_from_config(config);
        self.set_level5_from_config(config);
        self.set_level6_from_config(config);

        self.num_ext_blocks = self.ext_metadata_blocks.len() as u64;
        self.sort_extension_blocks();
    }

    fn set_level2_from_config(&mut self, config: &GenerateConfig) {
        // Either single default target or multiple
        if let Some(target_nits) = config.target_nits {
            self.add_level2_metadata(target_nits, 2048, 2048, 2048, 2048, 2048, 2048);
        } else if let Some(l2_targets) = &config.level2 {
            for l2 in l2_targets {
                self.add_level2_metadata(
                    l2.target_nits,
                    l2.trim_slope,
                    l2.trim_offset,
                    l2.trim_power,
                    l2.trim_chroma_weight,
                    l2.trim_saturation_gain,
                    l2.ms_weight,
                );
            }
        }
    }

    fn set_level5_from_config(&mut self, config: &GenerateConfig) {
        let (left, right, top, bottom) = if let Some(level5_config) = &config.level5 {
            (
                level5_config.active_area_left_offset,
                level5_config.active_area_right_offset,
                level5_config.active_area_top_offset,
                level5_config.active_area_bottom_offset,
            )
        } else {
            (0, 0, 0, 0)
        };

        let ext_metadata_block_level5 = ExtMetadataBlockLevel5 {
            block_info: BlockInfo {
                ext_block_length: 7,
                ext_block_level: 5,
                remaining: BitVec::from_bitslice(bits![Msb0, u8; 0; 4]),
            },
            active_area_left_offset: left,
            active_area_right_offset: right,
            active_area_top_offset: top,
            active_area_bottom_offset: bottom,
        };

        self.ext_metadata_blocks
            .push(ExtMetadataBlock::Level5(ext_metadata_block_level5))
    }

    fn set_level6_from_config(&mut self, config: &GenerateConfig) {
        if let Some(level6_config) = &config.level6 {
            let ext_metadata_block_level6 = ExtMetadataBlockLevel6 {
                block_info: BlockInfo {
                    ext_block_length: 8,
                    ext_block_level: 6,
                    ..Default::default()
                },
                max_display_mastering_luminance: level6_config.max_display_mastering_luminance,
                min_display_mastering_luminance: level6_config.min_display_mastering_luminance,
                max_content_light_level: level6_config.max_content_light_level,
                max_frame_average_light_level: level6_config.max_frame_average_light_level,
            };

            self.ext_metadata_blocks
                .push(ExtMetadataBlock::Level6(ext_metadata_block_level6))
        }
    }
}

impl Level5Metadata {
    pub fn get_offsets(&self) -> (u16, u16, u16, u16) {
        (
            self.active_area_left_offset,
            self.active_area_right_offset,
            self.active_area_top_offset,
            self.active_area_bottom_offset,
        )
    }
}

#[cfg(feature = "serde_feature")]
fn default_trim() -> u16 {
    2048
}

#[cfg(feature = "serde_feature")]
fn default_trim_neg() -> i16 {
    2048
}
