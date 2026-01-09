use std::fs::File;
use std::io::{Write, stdout};
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Serialize};

use dolby_vision::rpu::extension_metadata::blocks::{
    ExtMetadataBlock, ExtMetadataBlockLevel5, ExtMetadataBlockLevel6, ExtMetadataBlockLevel9,
    ExtMetadataBlockLevel11, ExtMetadataBlockLevel255,
};
use dolby_vision::rpu::extension_metadata::{
    CmV40DmData, MasteringDisplayPrimaries, WithExtMetadataBlocks,
};
use dolby_vision::rpu::generate::GenerateConfig;

use dolby_vision::rpu::utils::parse_rpu_file;

use super::{DoviRpu, input_from_either, write_rpu_file};
use crate::commands::EditorArgs;

pub struct Editor {
    input: PathBuf,
    json_file: PathBuf,
    rpu_out: PathBuf,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct EditConfig {
    #[serde(default)]
    mode: u8,

    #[serde(default)]
    remove_cmv4: bool,

    #[serde(default)]
    remove_mapping: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    min_pq: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_pq: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    active_area: Option<ActiveArea>,

    #[serde(skip_serializing_if = "Option::is_none")]
    remove: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    duplicate: Option<Vec<DuplicateMetadata>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    scene_cuts: Option<HashMap<String, bool>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    level6: Option<ExtMetadataBlockLevel6>,
    #[serde(skip_serializing_if = "Option::is_none")]
    level9: Option<MasteringDisplayPrimaries>,
    #[serde(skip_serializing_if = "Option::is_none")]
    level11: Option<ExtMetadataBlockLevel11>,
    #[serde(skip_serializing_if = "Option::is_none")]
    level255: Option<ExtMetadataBlockLevel255>,

    #[serde(skip_serializing_if = "Option::is_none")]
    source_rpu: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rpu_levels: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allow_cmv4_transfer: Option<bool>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ActiveArea {
    #[serde(default)]
    crop: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    drop_l5: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    presets: Option<Vec<ActiveAreaOffsets>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    edits: Option<HashMap<String, u16>>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ActiveAreaOffsets {
    id: u16,
    left: u16,
    right: u16,
    top: u16,
    bottom: u16,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct DuplicateMetadata {
    source: usize,
    offset: usize,
    length: usize,
}

impl Editor {
    pub fn from_args(args: EditorArgs) -> Result<Self> {
        let EditorArgs {
            input,
            input_pos,
            json_file,
            rpu_out,
        } = args;

        let input = input_from_either("editor", input, input_pos)?;

        let out_path = if let Some(out_path) = rpu_out {
            out_path
        } else {
            PathBuf::from(format!(
                "{}{}",
                input.file_stem().unwrap().to_str().unwrap(),
                "_modified.bin"
            ))
        };

        Ok(Self {
            input,
            json_file,
            rpu_out: out_path,
        })
    }

    pub fn edit(args: EditorArgs) -> Result<()> {
        let editor = Editor::from_args(args)?;

        let mut config: EditConfig = EditConfig::from_path(&editor.json_file)?;

        println!("EditConfig {}", serde_json::to_string_pretty(&config)?);

        println!("Parsing RPU file...");
        stdout().flush().ok();

        let mut rpus: Vec<Option<DoviRpu>> = parse_rpu_file(&editor.input)?
            .into_iter()
            .map(Some)
            .collect();

        config.execute(&mut rpus)?;

        let mut warned = false;
        let mut data = GenerateConfig::encode_option_rpus(&rpus)
            .enumerate()
            .filter_map(|(i, res)| {
                if !warned && let Err(err) = &res {
                    warned = true;
                    println!("Failed writing invalid RPU: Index {i}\n  {err:#}");
                }

                res.ok()
            })
            .collect();

        if let Some(to_duplicate) = config.duplicate.as_mut() {
            to_duplicate.sort_by_key(|meta| meta.offset);
            to_duplicate.reverse();
        }

        if let Some(to_duplicate) = &config.duplicate {
            config.duplicate_metadata(to_duplicate, &mut data)?;
        }

        println!("Final metadata length: {}", data.len());

        write_rpu_file(&editor.rpu_out, data)?;

        Ok(())
    }
}

impl EditConfig {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let json_file = File::open(path)?;
        let config: EditConfig = serde_json::from_reader(&json_file)?;

        Ok(config)
    }

    fn execute(&self, rpus: &mut [Option<DoviRpu>]) -> Result<()> {
        // Drop metadata frames
        if let Some(ranges) = &self.remove {
            self.remove_frames(ranges, rpus)?;
        }

        if self.remove_cmv4 {
            println!("Removing CMv4.0 metadata...");
        }

        if self.mode > 0 {
            println!("Converting with mode {}...", self.mode);
        }

        if self.remove_mapping {
            println!("Removing polynomial/MMR mapping...");
        }

        if let Some(active_area) = &self.active_area {
            if active_area.crop {
                println!("Cropping...");
            }

            if let Some(drop_opt) = &active_area.drop_l5 {
                println!(
                    "Dropping L5 metadata with opt '{}'",
                    drop_opt.to_lowercase()
                );
            }
        }

        if self.scene_cuts.is_some() {
            println!("Editing scene cuts...")
        }

        for rpu in rpus.iter_mut().filter_map(|e| e.as_mut()) {
            self.execute_single_rpu(rpu)?;
        }

        if let Some(edits) = &self.scene_cuts {
            self.set_scene_cuts(rpus, edits)?;
        }

        // Specific ranges only, requires complete list
        if let Some(active_area) = &self.active_area {
            active_area.execute(rpus)?;
        }

        if let Some(source_rpu_path) = &self.source_rpu {
            let source_rpus = parse_rpu_file(source_rpu_path)?;
            self.replace_from_rpus(rpus, &source_rpus)?;
        }

        Ok(())
    }

    pub fn execute_single_rpu(&self, rpu: &mut DoviRpu) -> Result<()> {
        if self.remove_cmv4 {
            rpu.remove_cmv40_extension_metadata()?;
        }

        if self.mode > 0 {
            rpu.convert_with_mode(self.mode)?;
        }

        if self.min_pq.is_some() || self.max_pq.is_some() {
            self.change_source_levels(rpu);
        }

        if self.remove_mapping {
            rpu.remove_mapping();
        }

        if let Some(l6) = &self.level6 {
            self.set_level6_metadata(rpu, l6)?;
        }

        if let Some(l9) = &self.level9 {
            self.set_level9_metadata(rpu, l9)?;
        }

        if let Some(l11) = &self.level11 {
            self.set_level11_metadata(rpu, l11)?;
        }

        if let Some(l255) = &self.level255 {
            self.set_level255_metadata(rpu, l255)?;
        }

        if let Some(edits) = &self.scene_cuts {
            self.set_scene_cuts_single_rpu(rpu, edits)?;
        }

        if let Some(active_area) = &self.active_area {
            active_area.execute_single_rpu(rpu)?;
        }

        Ok(())
    }

    fn range_string_to_tuple(range: &str) -> Result<(usize, usize)> {
        let mut result = (0, 0);

        if range.contains('-') {
            let mut split = range.split('-');

            if let Some(first) = split.next() {
                if let Ok(first_num) = first.parse() {
                    result.0 = first_num;
                }
            }

            if let Some(second) = split.next() {
                if let Ok(second_num) = second.parse() {
                    result.1 = second_num;
                }
            }

            Ok(result)
        } else {
            bail!("Invalid edit range")
        }
    }

    fn remove_frames(&self, ranges: &[String], rpus: &mut [Option<DoviRpu>]) -> Result<()> {
        let mut amount = 0;

        for range in ranges {
            if range.contains('-') {
                let (start, end) = EditConfig::range_string_to_tuple(range)?;
                ensure!(end < rpus.len(), "invalid end range {}", end);

                amount += end - start + 1;
                rpus[start..=end].iter_mut().for_each(|e| *e = None);
            } else if let Ok(index) = range.parse::<usize>() {
                ensure!(
                    index < rpus.len(),
                    "invalid frame index to remove {}",
                    index
                );

                amount += 1;

                rpus[index] = None;
            }
        }

        println!("Removed {amount} metadata frames.");

        Ok(())
    }

    fn duplicate_metadata(
        &self,
        to_duplicate: &[DuplicateMetadata],
        data: &mut Vec<Vec<u8>>,
    ) -> Result<()> {
        println!("Duplicating metadata. Initial metadata len {}", data.len());

        for meta in to_duplicate {
            ensure!(
                meta.source < data.len() && meta.offset <= data.len(),
                "invalid duplicate: {:?}",
                meta
            );

            let source = data[meta.source].clone();
            data.splice(
                meta.offset..meta.offset,
                std::iter::repeat_n(source, meta.length),
            );
        }

        Ok(())
    }

    fn change_source_levels(&self, rpu: &mut DoviRpu) {
        rpu.modified = true;

        if let Some(vdr_dm_data) = rpu.vdr_dm_data.as_mut() {
            vdr_dm_data.change_source_levels(self.min_pq, self.max_pq)
        }
    }

    fn set_level6_metadata(
        &self,
        rpu: &mut DoviRpu,
        level6: &ExtMetadataBlockLevel6,
    ) -> Result<()> {
        rpu.modified = true;

        if let Some(vdr_dm_data) = rpu.vdr_dm_data.as_mut() {
            vdr_dm_data.replace_metadata_block(ExtMetadataBlock::Level6(level6.clone()))?;
        }

        Ok(())
    }

    fn set_level9_metadata(
        &self,
        rpu: &mut DoviRpu,
        primaries: &MasteringDisplayPrimaries,
    ) -> Result<()> {
        let primary_index = *primaries as u8;

        let level9 = ExtMetadataBlockLevel9 {
            length: 1,
            source_primary_index: primary_index,
            ..Default::default()
        };

        if let Some(vdr_dm_data) = rpu.vdr_dm_data.as_mut() {
            rpu.modified = true;

            vdr_dm_data.replace_metadata_block(ExtMetadataBlock::Level9(level9))?;
        }

        Ok(())
    }

    fn set_level11_metadata(
        &self,
        rpu: &mut DoviRpu,
        level11: &ExtMetadataBlockLevel11,
    ) -> Result<()> {
        if let Some(vdr_dm_data) = rpu.vdr_dm_data.as_mut() {
            rpu.modified = true;
            vdr_dm_data.replace_metadata_block(ExtMetadataBlock::Level11(level11.clone()))?;
        }

        Ok(())
    }

    fn set_scene_cuts_single_rpu(
        &self,
        rpu: &mut DoviRpu,
        edits: &HashMap<String, bool>,
    ) -> Result<()> {
        // Allow passing "all" instead of a range
        // Do "all" presets before specific ranges
        for edit in edits {
            if edit.0.to_lowercase() == "all" {
                if let Some(vdr_dm_data) = rpu.vdr_dm_data.as_mut() {
                    rpu.modified = true;
                    vdr_dm_data.set_scene_cut(*edit.1);
                }
            }
        }

        Ok(())
    }

    fn set_scene_cuts(
        &self,
        rpus: &mut [Option<DoviRpu>],
        edits: &HashMap<String, bool>,
    ) -> Result<()> {
        let edits = edits.iter().filter(|e| e.0.to_lowercase() != "all");

        for edit in edits {
            let (start, end) = EditConfig::range_string_to_tuple(edit.0)?;

            if end > rpus.len() {
                bail!("Invalid range: {} > {} available RPUs", end, rpus.len());
            }

            for rpu in rpus[start..=end].iter_mut().filter_map(|e| e.as_mut()) {
                if let Some(vdr_dm_data) = rpu.vdr_dm_data.as_mut() {
                    rpu.modified = true;
                    vdr_dm_data.set_scene_cut(*edit.1)
                }
            }
        }

        Ok(())
    }

    fn set_level255_metadata(
        &self,
        rpu: &mut DoviRpu,
        level255: &ExtMetadataBlockLevel255,
    ) -> Result<()> {
        rpu.modified = true;

        if let Some(vdr_dm_data) = rpu.vdr_dm_data.as_mut() {
            vdr_dm_data.replace_metadata_block(ExtMetadataBlock::Level255(level255.clone()))?;
        }

        Ok(())
    }

    fn replace_from_rpus(
        &self,
        rpus: &mut [Option<DoviRpu>],
        source_rpus: &Vec<DoviRpu>,
    ) -> Result<()> {
        println!("Replacing metadata levels from second RPU...");
        ensure!(rpus.len() == source_rpus.len());

        let zip_iter = rpus.iter_mut().filter_map(|e| e.as_mut()).zip(source_rpus);

        let levels = self
            .rpu_levels
            .as_ref()
            .expect("Levels to replace must be present");

        let allow_cmv4_transfer = self.allow_cmv4_transfer.unwrap_or(false)
            && levels
                .iter()
                .any(|l| CmV40DmData::ALLOWED_BLOCK_LEVELS.contains(l));

        for (dst_rpu, src_rpu) in zip_iter {
            dst_rpu.replace_levels_from_rpu_cmv40(src_rpu, levels, allow_cmv4_transfer)?;
        }

        Ok(())
    }
}

impl ActiveArea {
    fn execute(&self, rpus: &mut [Option<DoviRpu>]) -> Result<()> {
        if let Some(edits) = &self.edits {
            if !edits.is_empty() {
                self.do_edits(edits, rpus)?;
            }
        }

        Ok(())
    }

    fn execute_single_rpu(&self, rpu: &mut DoviRpu) -> Result<()> {
        if self.crop {
            rpu.crop()?;
        }

        if let Some(drop_opt) = &self.drop_l5 {
            self.drop_specific_l5(&drop_opt.to_lowercase(), rpu)?;
        }

        // Allow passing "all" instead of a range
        // Do "all" presets before specific ranges
        if let (Some(presets), Some(edits)) = (&self.presets, &self.edits) {
            for edit in edits {
                let preset_id = *edit.1;

                if edit.0.to_lowercase() == "all" {
                    if let Some(active_area_offsets) = presets.iter().find(|e| e.id == preset_id) {
                        self.set_offsets(rpu, active_area_offsets)?;
                    } else {
                        bail!("Invalid preset ID: {}", preset_id);
                    }
                }
            }
        }

        Ok(())
    }

    fn do_edits(&self, edits: &HashMap<String, u16>, rpus: &mut [Option<DoviRpu>]) -> Result<()> {
        if let Some(presets) = &self.presets {
            println!("Editing active area offsets...");

            let specific_edits = edits.iter().filter(|e| e.0.to_lowercase() != "all");

            for edit in specific_edits {
                let (start, end) = EditConfig::range_string_to_tuple(edit.0)?;
                let preset_id = *edit.1;

                if end > rpus.len() {
                    bail!("Invalid range: {} > {} available RPUs", end, rpus.len());
                }

                if let Some(active_area_offsets) = presets.iter().find(|e| e.id == preset_id) {
                    for rpu in rpus[start..=end].iter_mut().filter_map(|e| e.as_mut()) {
                        self.set_offsets(rpu, active_area_offsets)?;
                    }
                } else {
                    bail!("Invalid preset ID: {}", preset_id);
                }
            }
        }

        Ok(())
    }

    fn set_offsets(
        &self,
        rpu: &mut DoviRpu,
        active_area_offsets: &ActiveAreaOffsets,
    ) -> Result<()> {
        rpu.modified = true;

        let (left, right, top, bottom) = (
            active_area_offsets.left,
            active_area_offsets.right,
            active_area_offsets.top,
            active_area_offsets.bottom,
        );

        if let Some(vdr_dm_data) = rpu.vdr_dm_data.as_mut() {
            vdr_dm_data.replace_metadata_block(ExtMetadataBlock::Level5(
                ExtMetadataBlockLevel5::from_offsets(left, right, top, bottom),
            ))?;
        }

        Ok(())
    }

    fn drop_specific_l5(&self, param: &str, rpu: &mut DoviRpu) -> Result<()> {
        if let Some(vdr_dm_data) = rpu.vdr_dm_data.as_mut() {
            let drop_it = if param == "zeroes" {
                let level5_block = vdr_dm_data.get_block(5);

                if let Some(ExtMetadataBlock::Level5(m)) = level5_block {
                    m.active_area_left_offset == 0
                        && m.active_area_right_offset == 0
                        && m.active_area_top_offset == 0
                        && m.active_area_bottom_offset == 0
                } else {
                    false
                }
            } else {
                param == "all"
            };

            if drop_it {
                rpu.modified = true;

                vdr_dm_data.remove_metadata_level(5);
            }
        }

        Ok(())
    }
}
