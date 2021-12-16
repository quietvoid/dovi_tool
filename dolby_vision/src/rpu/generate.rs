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
    pub target_nits: Option<u16>,

    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub source_min_pq: Option<u16>,
    #[cfg_attr(feature = "serde_feature", serde(default))]
    pub source_max_pq: Option<u16>,

    pub shots: Vec<VideoShot>,

    /// Defaults to zero offsets, should be present in RPU
    pub level5: ExtMetadataBlockLevel5,

    /// Defaults to 1000,0.0001
    pub level6: ExtMetadataBlockLevel6,
}

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct VideoShot {
    pub id: String,
    pub start: usize,

    // Optional, only if dynamic length
    pub full_length: bool,
    pub duration: usize,

    pub metadata_blocks: Vec<ExtMetadataBlock>,
    pub frame_edits: Vec<ShotFrameEdit>,
}

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct ShotFrameEdit {
    pub edit_offset: usize,
    pub metadata_blocks: Vec<ExtMetadataBlock>,
}

impl GenerateConfig {
    pub fn write_rpus(&self, path: &Path) -> Result<()> {
        let mut writer =
            BufWriter::with_capacity(100_000, File::create(path).expect("Can't create file"));

        let rpu = DoviRpu::profile81_config(self)?;

        for shot in &self.shots {
            let end = shot.duration;

            for i in 0..end {
                let mut frame_rpu = rpu.clone();

                if let Some(ref mut vdr_dm_data) = frame_rpu.vdr_dm_data {
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

                let encoded_rpu = frame_rpu.write_hevc_unspec62_nalu()?;

                writer.write_all(OUT_NAL_HEADER)?;

                // Remove 0x7C01
                writer.write_all(&encoded_rpu[2..])?;
            }
        }

        println!("Generated metadata for {} frames", self.length);

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
