use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::Result;

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

use crate::rpu::dovi_rpu::DoviRpu;

use super::{extension_metadata::blocks, vdr_dm_data::CmVersion};
use blocks::*;

const OUT_NAL_HEADER: &[u8] = &[0, 0, 0, 1];

#[derive(Debug)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct GenerateConfig {
    pub cm_version: CmVersion,
    pub length: usize,

    /// Optional, specifies a L2 block for this target
    pub target_nits: Option<u16>,

    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub source_min_pq: Option<u16>,
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub source_max_pq: Option<u16>,

    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub shots: Vec<VideoShot>,

    /// Defaults to zero offsets, should be present in RPU
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub level5: ExtMetadataBlockLevel5,

    /// Defaults to 1000,0.0001
    pub level6: ExtMetadataBlockLevel6,
}

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct VideoShot {
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub id: String,

    pub start: usize,
    pub duration: usize,

    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub metadata_blocks: Vec<ExtMetadataBlock>,

    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub frame_edits: Vec<ShotFrameEdit>,
}

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct ShotFrameEdit {
    pub edit_offset: usize,
    pub metadata_blocks: Vec<ExtMetadataBlock>,
}

impl GenerateConfig {
    pub fn generate_rpu_list(&self) -> Result<Vec<DoviRpu>> {
        let rpu = DoviRpu::profile81_config(self)?;
        let mut list = Vec::with_capacity(self.length);

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
            target_nits: Default::default(),
            source_min_pq: Default::default(),
            source_max_pq: Default::default(),
            shots: Default::default(),
            level5: Default::default(),
            level6: ExtMetadataBlockLevel6 {
                max_display_mastering_luminance: 1000,
                min_display_mastering_luminance: 1,
                max_content_light_level: 0,
                max_frame_average_light_level: 0,
            },
        }
    }
}

#[cfg(test)]
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
