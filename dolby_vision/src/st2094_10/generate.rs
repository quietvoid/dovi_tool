use anyhow::Result;

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

use super::ST2094_10Meta;

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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Serialize, Deserialize))]
pub struct Level2Metadata {
    pub target_nits: Option<u16>,

    /// Normalized to 0-4095
    pub target_max_pq: Option<u16>,

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
    pub fn update_from_config(&mut self, config: &GenerateConfig) -> Result<()> {
        self.set_level2_from_config(config)?;
        self.set_level5_from_config(config);
        self.set_level6_from_config(config);

        self.num_ext_blocks = self.ext_metadata_blocks.len() as u64;
        self.sort_extension_blocks();

        Ok(())
    }

    fn set_level2_from_config(&mut self, config: &GenerateConfig) -> Result<()> {
        // Either single default target or multiple
        if config.target_nits.is_some() {
            let l2 = Level2Metadata {
                target_nits: config.target_nits,
                ..Default::default()
            };

            self.add_level2_metadata(&l2)?;
        } else if let Some(l2_targets) = &config.level2 {
            for l2 in l2_targets {
                self.add_level2_metadata(l2)?;
            }
        }

        Ok(())
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

        self.add_level5_metadata(left, right, top, bottom);
    }

    fn set_level6_from_config(&mut self, config: &GenerateConfig) {
        if let Some(level6_config) = &config.level6 {
            self.add_level6_metadata(level6_config);
        }
    }
}

impl Default for Level2Metadata {
    fn default() -> Self {
        Self {
            target_nits: None,
            target_max_pq: None,
            trim_slope: 2048,
            trim_offset: 2048,
            trim_power: 2048,
            trim_chroma_weight: 2048,
            trim_saturation_gain: 2048,
            ms_weight: 2048,
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
