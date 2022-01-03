use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::{ensure, Result};

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

use crate::rpu::dovi_rpu::DoviRpu;

use super::{extension_metadata::blocks, vdr_dm_data::CmVersion};
use blocks::*;

const OUT_NAL_HEADER: &[u8] = &[0, 0, 0, 1];

/// Generic generation config struct.
#[derive(Debug)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct GenerateConfig {
    /// Content mapping version
    /// Optional, defaults to v4.0
    #[cfg_attr(feature = "serde_feature", serde(default = "CmVersion::v40"))]
    pub cm_version: CmVersion,

    /// Number of RPU frames to generate.
    /// Required only when no shots are specified.
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub length: usize,

    /// Mastering display min luminance, as 12 bit PQ code.
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub source_min_pq: Option<u16>,

    /// Mastering display max luminance, as 12 bit PQ code.
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub source_max_pq: Option<u16>,

    /// Active area offsets.
    /// Defaults to zero offsets, should be present in RPU
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub level5: ExtMetadataBlockLevel5,

    /// ST2086/HDR10 fallback metadata.
    /// Required for deserialization.
    /// Defaults to 1000,0.0001
    pub level6: ExtMetadataBlockLevel6,

    /// List of metadata blocks to use for every RPU generated.
    ///
    /// Per-shot or per-frame metadata replaces the default
    /// metadata blocks if there are conflicts.
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub default_metadata_blocks: Vec<ExtMetadataBlock>,

    /// List of shots to generate.
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub shots: Vec<VideoShot>,
}

/// Struct defining a video shot.
/// A shot is a group of frames that share the same metadata.
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct VideoShot {
    /// Optional (unused) ID of the shot.
    /// Only XML generation provides this.
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub id: String,

    /// Frame start offset of the shot.
    /// Used as a sorting key for the shots.
    pub start: usize,

    /// Number of frames contained in the shot.
    pub duration: usize,

    /// List of metadata blocks.
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub metadata_blocks: Vec<ExtMetadataBlock>,

    /// List of per-frame metadata edits.
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub frame_edits: Vec<ShotFrameEdit>,
}

/// Struct to represent a list of metadata edits for a specific frame.
#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct ShotFrameEdit {
    /// Frame offset within the parent shot.
    pub edit_offset: usize,

    /// List of metadata blocks to use.
    pub metadata_blocks: Vec<ExtMetadataBlock>,
}

impl GenerateConfig {
    pub fn generate_rpu_list(&self) -> Result<Vec<DoviRpu>> {
        let rpu = DoviRpu::profile81_config(self)?;
        let mut list = Vec::with_capacity(self.length);

        let shots_length: usize = self.shots.iter().map(|s| s.duration).sum();

        ensure!(
            self.length == shots_length,
            format!(
                "Config length is not the same as shots total duration. Config: {}, Shots: {}",
                self.length, shots_length
            )
        );

        for shot in &self.shots {
            let end = shot.duration;

            for i in 0..end {
                let mut frame_rpu = rpu.clone();

                if let Some(ref mut vdr_dm_data) = frame_rpu.vdr_dm_data {
                    if i == 0 {
                        vdr_dm_data.set_scene_cut(true);
                    }

                    // Set metadata for this shot
                    for block in &shot.metadata_blocks {
                        vdr_dm_data.replace_metadata_block(block.clone())?;
                    }

                    let frame_edit = shot.frame_edits.iter().find(|e| e.edit_offset == i);

                    // Set different metadata for this frame
                    if let Some(edit) = frame_edit {
                        for block in &edit.metadata_blocks {
                            vdr_dm_data.replace_metadata_block(block.clone())?;
                        }
                    }
                }

                list.push(frame_rpu)
            }
        }

        Ok(list)
    }

    pub fn encode_option_rpus(rpus: &mut Vec<Option<DoviRpu>>) -> Vec<Vec<u8>> {
        let encoded_rpus = rpus
            .iter_mut()
            .filter_map(|e| e.as_mut())
            .map(|e| e.write_hevc_unspec62_nalu())
            .filter_map(Result::ok)
            .collect();

        encoded_rpus
    }

    pub fn encode_rpus(rpus: &mut Vec<DoviRpu>) -> Vec<Vec<u8>> {
        let encoded_rpus = rpus
            .iter_mut()
            .map(|e| e.write_hevc_unspec62_nalu())
            .filter_map(Result::ok)
            .collect();

        encoded_rpus
    }

    pub fn write_rpus(&self, path: &Path) -> Result<()> {
        let mut writer =
            BufWriter::with_capacity(100_000, File::create(path).expect("Can't create file"));

        let rpus = self.generate_rpu_list()?;

        for rpu in &rpus {
            let encoded_rpu = rpu.write_hevc_unspec62_nalu()?;

            writer.write_all(OUT_NAL_HEADER)?;

            // Remove 0x7C01
            writer.write_all(&encoded_rpu[2..])?;
        }

        writer.flush()?;

        Ok(())
    }
}

impl Default for GenerateConfig {
    fn default() -> Self {
        Self {
            cm_version: CmVersion::V40,
            length: Default::default(),
            source_min_pq: Default::default(),
            source_max_pq: Default::default(),
            default_metadata_blocks: Default::default(),
            level5: Default::default(),
            level6: ExtMetadataBlockLevel6 {
                max_display_mastering_luminance: 1000,
                min_display_mastering_luminance: 1,
                max_content_light_level: 0,
                max_frame_average_light_level: 0,
            },
            shots: Default::default(),
        }
    }
}

impl VideoShot {
    pub fn copy_metadata_from_shot(
        &mut self,
        other_shot: &VideoShot,
        level_block_list: Option<&[u8]>,
    ) {
        // Add blocks to shot metadata
        let new_shot_blocks: Vec<ExtMetadataBlock> = if let Some(block_list) = level_block_list {
            other_shot
                .metadata_blocks
                .iter()
                .filter(|b| !block_list.contains(&b.level()))
                .cloned()
                .collect()
        } else {
            other_shot.metadata_blocks.clone()
        };

        self.metadata_blocks.extend(new_shot_blocks);

        // Add blocks to existing frame edits for the same offsets
        for frame_edit in &mut self.frame_edits {
            let new_frame_edit = other_shot
                .frame_edits
                .iter()
                .find(|e| e.edit_offset == frame_edit.edit_offset);

            if let Some(other_edit) = new_frame_edit {
                let new_edit_blocks: Vec<ExtMetadataBlock> =
                    if let Some(block_list) = level_block_list {
                        other_edit
                            .metadata_blocks
                            .iter()
                            .filter(|b| !block_list.contains(&b.level()))
                            .cloned()
                            .collect()
                    } else {
                        other_edit.metadata_blocks.clone()
                    };

                frame_edit.metadata_blocks.extend(new_edit_blocks);
            }
        }

        // Add extra frame edits but don't replace
        let existing_edit_offsets: Vec<usize> =
            self.frame_edits.iter().map(|e| e.edit_offset).collect();

        // Filter out unwanted blocks and add new edits
        let added_frame_edits = other_shot
            .frame_edits
            .iter()
            .filter(|e| !existing_edit_offsets.contains(&e.edit_offset))
            .cloned()
            .map(|mut frame_edit| {
                if let Some(block_list) = level_block_list {
                    frame_edit
                        .metadata_blocks
                        .retain(|b| !block_list.contains(&b.level()));
                }

                frame_edit
            });

        self.frame_edits.extend(added_frame_edits);
    }
}

#[cfg(all(test, feature = "xml"))]
mod tests {
    use anyhow::Result;
    use std::path::PathBuf;

    use crate::{
        rpu::{extension_metadata::blocks::ExtMetadataBlock, vdr_dm_data::CmVersion},
        xml::{CmXmlParser, XmlParserOpts},
    };

    #[test]
    fn config_with_frame_edits() -> Result<()> {
        let lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let assets_path = lib_path.parent().unwrap();

        let opts = XmlParserOpts {
            canvas_width: Some(3840),
            canvas_height: Some(2160),
        };
        let parser = CmXmlParser::parse_file(&assets_path.join("assets/tests/cmv4_0_2.xml"), opts)?;

        let config = parser.config;
        assert_eq!(config.cm_version, CmVersion::V40);

        let rpus = config.generate_rpu_list()?;

        assert_eq!(rpus.len(), 259);

        // SHOT 1
        let shot1_rpu = &rpus[0];
        let shot1_vdr_dm_data = &shot1_rpu.vdr_dm_data.as_ref().unwrap();
        assert_eq!(shot1_vdr_dm_data.scene_refresh_flag, 1);

        // L1, L5, L6 in CMv2.9
        assert_eq!(shot1_vdr_dm_data.metadata_blocks(1).unwrap().len(), 3);

        // L3, L9, L11, L254 in CMv4.0
        assert_eq!(shot1_vdr_dm_data.metadata_blocks(3).unwrap().len(), 4);

        if let ExtMetadataBlock::Level1(level1) = shot1_vdr_dm_data.get_block(1).unwrap() {
            assert_eq!(level1.min_pq, 0);
            assert_eq!(level1.max_pq, 2828);
            assert_eq!(level1.avg_pq, 1120);
        }

        if let ExtMetadataBlock::Level3(level3) = shot1_vdr_dm_data.get_block(3).unwrap() {
            assert_eq!(level3.min_pq_offset, 2048);
            assert_eq!(level3.max_pq_offset, 2048);
            assert_eq!(level3.avg_pq_offset, 2048);
        }

        if let ExtMetadataBlock::Level5(level5) = shot1_vdr_dm_data.get_block(5).unwrap() {
            assert_eq!(level5.get_offsets(), (240, 240, 0, 0));
        }

        if let ExtMetadataBlock::Level6(level6) = shot1_vdr_dm_data.get_block(6).unwrap() {
            assert_eq!(level6.min_display_mastering_luminance, 1);
            assert_eq!(level6.max_display_mastering_luminance, 1000);
            assert_eq!(level6.max_content_light_level, 3948);
            assert_eq!(level6.max_frame_average_light_level, 120);
        }

        if let ExtMetadataBlock::Level9(level9) = shot1_vdr_dm_data.get_block(9).unwrap() {
            assert_eq!(level9.source_primary_index, 255);
        }

        if let ExtMetadataBlock::Level11(level11) = shot1_vdr_dm_data.get_block(11).unwrap() {
            assert_eq!(level11.content_type, 1);
            assert_eq!(level11.whitepoint, 0);
            assert_eq!(level11.reference_mode_flag, true);
        }

        // SHOT 2
        let shot2_rpu = &rpus[120];
        let shot2_vdr_dm_data = &shot2_rpu.vdr_dm_data.as_ref().unwrap();
        assert_eq!(shot2_vdr_dm_data.scene_refresh_flag, 1);

        // L1, 3*L2, L5, L6 in CMv2.9
        assert_eq!(shot2_vdr_dm_data.metadata_blocks(1).unwrap().len(), 6);

        // L3, 2*L8, L9, L11, L254 in CMv4.0
        assert_eq!(shot2_vdr_dm_data.metadata_blocks(3).unwrap().len(), 6);

        if let ExtMetadataBlock::Level1(level1) = shot2_vdr_dm_data.get_block(1).unwrap() {
            assert_eq!(level1.min_pq, 0);
            assert_eq!(level1.max_pq, 2081);
            assert_eq!(level1.avg_pq, 1229);
        }

        assert_eq!(shot2_vdr_dm_data.level_blocks_iter(2).count(), 3);
        let mut shot2_level2_iter = shot2_vdr_dm_data.level_blocks_iter(2);

        if let ExtMetadataBlock::Level2(shot2_l2) = shot2_level2_iter.next().unwrap() {
            assert_eq!(shot2_l2.target_max_pq, 2081);
            assert_eq!(shot2_l2.trim_slope, 2013);
            assert_eq!(shot2_l2.trim_offset, 2016);
            assert_eq!(shot2_l2.trim_power, 1339);
            assert_eq!(shot2_l2.trim_chroma_weight, 2048);
            assert_eq!(shot2_l2.trim_saturation_gain, 2048);
            assert_eq!(shot2_l2.ms_weight, 2048);
        }

        if let ExtMetadataBlock::Level2(shot2_l2) = shot2_level2_iter.next().unwrap() {
            assert_eq!(shot2_l2.target_max_pq, 2851);
            assert_eq!(shot2_l2.trim_slope, 2059);
            assert_eq!(shot2_l2.trim_offset, 2048);
            assert_eq!(shot2_l2.trim_power, 1955);
            assert_eq!(shot2_l2.trim_chroma_weight, 2048);
            assert_eq!(shot2_l2.trim_saturation_gain, 2048);
            assert_eq!(shot2_l2.ms_weight, 2048);
        }

        if let ExtMetadataBlock::Level2(shot2_l2) = shot2_level2_iter.next().unwrap() {
            assert_eq!(shot2_l2.target_max_pq, 3079);
            assert_eq!(shot2_l2.trim_slope, 2049);
            assert_eq!(shot2_l2.trim_offset, 2048);
            assert_eq!(shot2_l2.trim_power, 2047);
            assert_eq!(shot2_l2.trim_chroma_weight, 2048);
            assert_eq!(shot2_l2.trim_saturation_gain, 2048);
            assert_eq!(shot2_l2.ms_weight, 2048);
        }

        if let ExtMetadataBlock::Level5(level5) = shot2_vdr_dm_data.get_block(5).unwrap() {
            assert_eq!(level5.get_offsets(), (480, 480, 0, 0));
        }

        if let ExtMetadataBlock::Level6(level6) = shot2_vdr_dm_data.get_block(6).unwrap() {
            assert_eq!(level6.min_display_mastering_luminance, 1);
            assert_eq!(level6.max_display_mastering_luminance, 1000);
            assert_eq!(level6.max_content_light_level, 3948);
            assert_eq!(level6.max_frame_average_light_level, 120);
        }

        assert_eq!(shot2_vdr_dm_data.level_blocks_iter(8).count(), 2);
        let mut shot2_level8_iter = shot2_vdr_dm_data.level_blocks_iter(8);

        if let ExtMetadataBlock::Level8(shot2_l8) = shot2_level8_iter.next().unwrap() {
            assert_eq!(shot2_l8.target_display_index, 1);
            assert_eq!(shot2_l8.trim_slope, 2048);
            assert_eq!(shot2_l8.trim_offset, 2048);
            assert_eq!(shot2_l8.trim_power, 2048);
            assert_eq!(shot2_l8.trim_chroma_weight, 2048);
            assert_eq!(shot2_l8.trim_saturation_gain, 2048);
            assert_eq!(shot2_l8.ms_weight, 2048);
        }
        if let ExtMetadataBlock::Level8(shot2_l8) = shot2_level8_iter.next().unwrap() {
            assert_eq!(shot2_l8.target_display_index, 48);
            assert_eq!(shot2_l8.trim_slope, 2048);
            assert_eq!(shot2_l8.trim_offset, 2048);
            assert_eq!(shot2_l8.trim_power, 2048);
            assert_eq!(shot2_l8.trim_chroma_weight, 2048);
            assert_eq!(shot2_l8.trim_saturation_gain, 2048);
            assert_eq!(shot2_l8.ms_weight, 2048);
        }

        if let ExtMetadataBlock::Level9(level9) = shot2_vdr_dm_data.get_block(9).unwrap() {
            assert_eq!(level9.source_primary_index, 255);
        }

        if let ExtMetadataBlock::Level11(level11) = shot2_vdr_dm_data.get_block(11).unwrap() {
            assert_eq!(level11.content_type, 1);
            assert_eq!(level11.whitepoint, 0);
            assert_eq!(level11.reference_mode_flag, true);
        }

        // SHOT 3
        let shot3_rpu = &rpus[219];
        let shot3_vdr_dm_data = &shot3_rpu.vdr_dm_data.as_ref().unwrap();
        assert_eq!(shot3_vdr_dm_data.scene_refresh_flag, 1);

        // L1, L5, L6 in CMv2.9
        assert_eq!(shot3_vdr_dm_data.metadata_blocks(1).unwrap().len(), 3);

        // L3, L9, L11, L254 in CMv4.0
        assert_eq!(shot3_vdr_dm_data.metadata_blocks(3).unwrap().len(), 4);

        if let ExtMetadataBlock::Level1(level1) = shot3_vdr_dm_data.get_block(1).unwrap() {
            assert_eq!(level1.min_pq, 0);
            assert_eq!(level1.max_pq, 2875);
            assert_eq!(level1.avg_pq, 819);
        }

        if let ExtMetadataBlock::Level3(level3) = shot3_vdr_dm_data.get_block(3).unwrap() {
            assert_eq!(level3.min_pq_offset, 2048);
            assert_eq!(level3.max_pq_offset, 1871);
            assert_eq!(level3.avg_pq_offset, 2048);
        }

        if let ExtMetadataBlock::Level5(level5) = shot3_vdr_dm_data.get_block(5).unwrap() {
            assert_eq!(level5.get_offsets(), (480, 480, 0, 0));
        }

        if let ExtMetadataBlock::Level6(level6) = shot3_vdr_dm_data.get_block(6).unwrap() {
            assert_eq!(level6.min_display_mastering_luminance, 1);
            assert_eq!(level6.max_display_mastering_luminance, 1000);
            assert_eq!(level6.max_content_light_level, 3948);
            assert_eq!(level6.max_frame_average_light_level, 120);
        }

        if let ExtMetadataBlock::Level9(level9) = shot3_vdr_dm_data.get_block(9).unwrap() {
            assert_eq!(level9.source_primary_index, 255);
        }

        if let ExtMetadataBlock::Level11(level11) = shot3_vdr_dm_data.get_block(11).unwrap() {
            assert_eq!(level11.content_type, 1);
            assert_eq!(level11.whitepoint, 0);
            assert_eq!(level11.reference_mode_flag, true);
        }

        // Frame edit in shot 3, offset 10 = 229
        let shot3_edit_rpu = &rpus[229];
        let shot3_edit_vdr_dm_data = &shot3_edit_rpu.vdr_dm_data.as_ref().unwrap();
        assert_eq!(shot3_edit_vdr_dm_data.scene_refresh_flag, 0);

        // L1, L2, L5, L6 in CMv2.9
        assert_eq!(shot3_edit_vdr_dm_data.metadata_blocks(1).unwrap().len(), 4);

        // L3, L8, L9, L11, L254 in CMv4.0
        assert_eq!(shot3_edit_vdr_dm_data.metadata_blocks(3).unwrap().len(), 5);

        if let ExtMetadataBlock::Level1(level1) = shot3_edit_vdr_dm_data.get_block(1).unwrap() {
            assert_eq!(level1.min_pq, 0);
            assert_eq!(level1.max_pq, 2081);
            assert_eq!(level1.avg_pq, 1229);
        }

        assert_eq!(shot3_edit_vdr_dm_data.level_blocks_iter(2).count(), 1);
        let mut shot3_edit_level2_iter = shot3_edit_vdr_dm_data.level_blocks_iter(2);

        if let ExtMetadataBlock::Level2(shot3_edit_l2) = shot3_edit_level2_iter.next().unwrap() {
            assert_eq!(shot3_edit_l2.target_max_pq, 2081);
            assert_eq!(shot3_edit_l2.trim_slope, 2013);
            assert_eq!(shot3_edit_l2.trim_offset, 2016);
            assert_eq!(shot3_edit_l2.trim_power, 1339);
            assert_eq!(shot3_edit_l2.trim_chroma_weight, 2048);
            assert_eq!(shot3_edit_l2.trim_saturation_gain, 2048);
            assert_eq!(shot3_edit_l2.ms_weight, 2048);
        }

        if let ExtMetadataBlock::Level3(level3) = shot3_edit_vdr_dm_data.get_block(3).unwrap() {
            assert_eq!(level3.min_pq_offset, 2048);
            assert_eq!(level3.max_pq_offset, 1871);
            assert_eq!(level3.avg_pq_offset, 2048);
        }

        if let ExtMetadataBlock::Level5(level5) = shot3_edit_vdr_dm_data.get_block(5).unwrap() {
            assert_eq!(level5.get_offsets(), (480, 480, 0, 0));
        }

        if let ExtMetadataBlock::Level6(level6) = shot3_edit_vdr_dm_data.get_block(6).unwrap() {
            assert_eq!(level6.min_display_mastering_luminance, 1);
            assert_eq!(level6.max_display_mastering_luminance, 1000);
            assert_eq!(level6.max_content_light_level, 3948);
            assert_eq!(level6.max_frame_average_light_level, 120);
        }

        assert_eq!(shot3_edit_vdr_dm_data.level_blocks_iter(8).count(), 1);
        let mut shot3_edit_level8_iter = shot3_edit_vdr_dm_data.level_blocks_iter(8);

        if let ExtMetadataBlock::Level8(shot3_edit_l8) = shot3_edit_level8_iter.next().unwrap() {
            assert_eq!(shot3_edit_l8.target_display_index, 1);
            assert_eq!(shot3_edit_l8.trim_slope, 2068);
            assert_eq!(shot3_edit_l8.trim_offset, 2048);
            assert_eq!(shot3_edit_l8.trim_power, 2048);
            assert_eq!(shot3_edit_l8.trim_chroma_weight, 2048);
            assert_eq!(shot3_edit_l8.trim_saturation_gain, 2048);
            assert_eq!(shot3_edit_l8.ms_weight, 2048);
        }

        if let ExtMetadataBlock::Level9(level9) = shot3_edit_vdr_dm_data.get_block(9).unwrap() {
            assert_eq!(level9.source_primary_index, 255);
        }

        if let ExtMetadataBlock::Level11(level11) = shot3_edit_vdr_dm_data.get_block(11).unwrap() {
            assert_eq!(level11.content_type, 1);
            assert_eq!(level11.whitepoint, 0);
            assert_eq!(level11.reference_mode_flag, true);
        }

        Ok(())
    }
}
