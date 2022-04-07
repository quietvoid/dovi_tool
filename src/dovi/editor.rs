use std::fs::File;
use std::{collections::HashMap, path::PathBuf};

use anyhow::{bail, ensure, Result};
use dolby_vision::rpu::extension_metadata::blocks::{
    ExtMetadataBlock, ExtMetadataBlockLevel11, ExtMetadataBlockLevel5, ExtMetadataBlockLevel6,
    ExtMetadataBlockLevel9,
};
use dolby_vision::rpu::extension_metadata::MasteringDisplayPrimaries;
use dolby_vision::rpu::extension_metadata::{CmV40DmData, DmData};
use dolby_vision::rpu::generate::GenerateConfig;
use serde::{Deserialize, Serialize};

use super::{parse_rpu_file, write_rpu_file, DoviRpu};

pub struct Editor {
    input: PathBuf,
    json_path: PathBuf,
    rpu_out: PathBuf,

    rpus: Option<Vec<Option<DoviRpu>>>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct EditConfig {
    #[serde(default)]
    mode: u8,

    #[serde(default)]
    convert_to_cmv4: bool,

    #[serde(default)]
    remove_mapping: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    active_area: Option<ActiveArea>,

    #[serde(skip_serializing_if = "Option::is_none")]
    remove: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    duplicate: Option<Vec<DuplicateMetadata>>,

    #[serde(default)]
    min_pq: Option<u16>,

    #[serde(default)]
    max_pq: Option<u16>,

    level6: Option<ExtMetadataBlockLevel6>,
    level9: Option<MasteringDisplayPrimaries>,
    level11: Option<ExtMetadataBlockLevel11>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
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

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ActiveAreaOffsets {
    id: u16,
    left: u16,
    right: u16,
    top: u16,
    bottom: u16,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct DuplicateMetadata {
    source: usize,
    offset: usize,
    length: usize,
}

impl Editor {
    pub fn edit(input: PathBuf, json_path: PathBuf, rpu_out: Option<PathBuf>) -> Result<()> {
        let out_path = if let Some(out_path) = rpu_out {
            out_path
        } else {
            PathBuf::from(format!(
                "{}{}",
                input.file_stem().unwrap().to_str().unwrap(),
                "_modified.bin"
            ))
        };

        let mut editor = Editor {
            input,
            json_path,
            rpu_out: out_path,
            rpus: None,
        };

        let json_file = File::open(&editor.json_path)?;
        let mut config: EditConfig = serde_json::from_reader(&json_file)?;

        println!("{:#?}", config);

        editor.rpus =
            parse_rpu_file(&editor.input)?.map(|rpus| rpus.into_iter().map(Some).collect());

        // Override to CM v4.0
        if !config.convert_to_cmv4 {
            config.convert_to_cmv4 = config.level11.is_some() || config.level9.is_some();
        }

        if let Some(ref mut rpus) = editor.rpus {
            config.execute(rpus)?;

            let mut data = GenerateConfig::encode_option_rpus(rpus);

            if let Some(ref mut to_duplicate) = config.duplicate {
                to_duplicate.sort_by_key(|meta| meta.offset);
                to_duplicate.reverse();
            }

            if let Some(to_duplicate) = &config.duplicate {
                config.duplicate_metadata(to_duplicate, &mut data)?;
            }

            println!("Final metadata length: {}", data.len());

            write_rpu_file(&editor.rpu_out, data)?;
        }

        Ok(())
    }
}

impl EditConfig {
    fn execute(&self, rpus: &mut [Option<DoviRpu>]) -> Result<()> {
        // Drop metadata frames
        if let Some(ranges) = &self.remove {
            self.remove_frames(ranges, rpus)?;
        }

        if self.convert_to_cmv4 {
            self.add_cmv4_dm_data(rpus)?;
        }

        // Convert with mode
        if self.mode > 0 {
            self.convert_with_mode(rpus)?;
        }

        if let Some(active_area) = &self.active_area {
            active_area.execute(rpus)?;
        }

        if self.min_pq.is_some() || self.max_pq.is_some() {
            self.change_source_levels(rpus);
        }

        if self.remove_mapping {
            self.remove_mapping(rpus);
        }

        if let Some(l6) = &self.level6 {
            self.set_level6_metadata(rpus, l6)?;
        }

        if let Some(l9) = &self.level9 {
            self.set_level9_metadata(rpus, l9.clone())?;
        }

        if let Some(l11) = &self.level11 {
            self.set_level11_metadata(rpus, l11)?;
        }

        Ok(())
    }

    fn convert_with_mode(&self, rpus: &mut [Option<DoviRpu>]) -> Result<()> {
        println!("Converting with mode {}...", self.mode);
        let list = rpus.iter_mut().filter_map(|e| e.as_mut());

        for rpu in list {
            rpu.convert_with_mode(self.mode)?;
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

        println!("Removed {} metadata frames.", amount);

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
                meta.source < data.len() && meta.offset < data.len(),
                "invalid duplicate: {:?}",
                meta
            );

            let source = data[meta.source].clone();
            data.splice(
                meta.offset..meta.offset,
                std::iter::repeat(source).take(meta.length),
            );
        }

        Ok(())
    }

    fn change_source_levels(&self, rpus: &mut [Option<DoviRpu>]) {
        rpus.iter_mut().filter_map(|e| e.as_mut()).for_each(|rpu| {
            rpu.modified = true;

            if let Some(ref mut vdr_dm_data) = rpu.vdr_dm_data {
                vdr_dm_data.change_source_levels(self.min_pq, self.max_pq)
            }
        });
    }

    fn set_level6_metadata(
        &self,
        rpus: &mut [Option<DoviRpu>],
        level6: &ExtMetadataBlockLevel6,
    ) -> Result<()> {
        for rpu in rpus.iter_mut().filter_map(|e| e.as_mut()) {
            rpu.modified = true;

            if let Some(ref mut vdr_dm_data) = rpu.vdr_dm_data {
                vdr_dm_data.replace_metadata_block(ExtMetadataBlock::Level6(level6.clone()))?;
            }
        }

        Ok(())
    }

    fn add_cmv4_dm_data(&self, rpus: &mut [Option<DoviRpu>]) -> Result<()> {
        for rpu in rpus.iter_mut().filter_map(|e| e.as_mut()) {
            rpu.modified = true;

            if let Some(ref mut vdr_dm_data) = rpu.vdr_dm_data {
                if vdr_dm_data.cmv40_metadata.is_none() {
                    vdr_dm_data.cmv40_metadata =
                        Some(DmData::V40(CmV40DmData::new_with_l254_402()));

                    // Defaults
                    vdr_dm_data.add_metadata_block(ExtMetadataBlock::Level9(
                        ExtMetadataBlockLevel9::default_dci_p3(),
                    ))?;
                    vdr_dm_data.add_metadata_block(ExtMetadataBlock::Level11(
                        ExtMetadataBlockLevel11::default_reference_cinema(),
                    ))?;
                }
            }
        }

        Ok(())
    }

    fn set_level9_metadata(
        &self,
        rpus: &mut [Option<DoviRpu>],
        primaries: MasteringDisplayPrimaries,
    ) -> Result<()> {
        let primary_index = primaries as u8;

        let level9 = ExtMetadataBlockLevel9 {
            length: 1,
            source_primary_index: primary_index,
            ..Default::default()
        };

        for rpu in rpus.iter_mut().filter_map(|e| e.as_mut()) {
            rpu.modified = true;

            if let Some(ref mut vdr_dm_data) = rpu.vdr_dm_data {
                vdr_dm_data.replace_metadata_block(ExtMetadataBlock::Level9(level9.clone()))?;
            }
        }

        Ok(())
    }

    fn set_level11_metadata(
        &self,
        rpus: &mut [Option<DoviRpu>],
        level11: &ExtMetadataBlockLevel11,
    ) -> Result<()> {
        for rpu in rpus.iter_mut().filter_map(|e| e.as_mut()) {
            rpu.modified = true;

            if let Some(ref mut vdr_dm_data) = rpu.vdr_dm_data {
                vdr_dm_data.replace_metadata_block(ExtMetadataBlock::Level11(level11.clone()))?;
            }
        }

        Ok(())
    }

    fn remove_mapping(&self, rpus: &mut [Option<DoviRpu>]) {
        println!("Removing polynomial/MMR mapping...");
        let list = rpus.iter_mut().filter_map(|e| e.as_mut());

        for rpu in list {
            rpu.remove_mapping();
        }
    }
}

impl ActiveArea {
    fn execute(&self, rpus: &mut [Option<DoviRpu>]) -> Result<()> {
        if self.crop {
            self.crop(rpus)?;
        }

        if let Some(drop_opt) = &self.drop_l5 {
            self.drop_specific_l5(drop_opt, rpus)?;
        }

        if let Some(edits) = &self.edits {
            if !edits.is_empty() {
                self.do_edits(edits, rpus)?;
            }
        }

        Ok(())
    }

    fn crop(&self, rpus: &mut [Option<DoviRpu>]) -> Result<()> {
        println!("Cropping...");
        for rpu in rpus.iter_mut().filter_map(|e| e.as_mut()) {
            rpu.crop()?;
        }

        Ok(())
    }

    fn do_edits(&self, edits: &HashMap<String, u16>, rpus: &mut [Option<DoviRpu>]) -> Result<()> {
        if let Some(presets) = &self.presets {
            println!("Editing active area offsets...");

            for edit in edits {
                // Allow passing "all" instead of a range
                let (start, end) = if edit.0.to_lowercase() == "all" {
                    (0, rpus.len() - 1)
                } else {
                    EditConfig::range_string_to_tuple(edit.0)?
                };

                let preset_id = *edit.1;

                if end as usize > rpus.len() {
                    bail!("Invalid range: {} > {} available RPUs", end, rpus.len());
                }

                if let Some(active_area_offsets) = presets.iter().find(|e| e.id == preset_id) {
                    for rpu in rpus[start..=end].iter_mut().filter_map(|e| e.as_mut()) {
                        rpu.modified = true;

                        let (left, right, top, bottom) = (
                            active_area_offsets.left,
                            active_area_offsets.right,
                            active_area_offsets.top,
                            active_area_offsets.bottom,
                        );

                        if let Some(ref mut vdr_dm_data) = rpu.vdr_dm_data {
                            vdr_dm_data.replace_metadata_block(ExtMetadataBlock::Level5(
                                ExtMetadataBlockLevel5::from_offsets(left, right, top, bottom),
                            ))?;
                        }
                    }
                } else {
                    bail!("Invalid preset ID: {}", preset_id);
                }
            }
        }

        Ok(())
    }

    fn drop_specific_l5(&self, drop_opt: &str, rpus: &mut [Option<DoviRpu>]) -> Result<()> {
        let param = drop_opt.to_lowercase();

        println!("Dropping L5 metadata with opt '{}'", param);

        rpus.iter_mut().filter_map(|e| e.as_mut()).for_each(|rpu| {
            if let Some(ref mut vdr_dm_data) = rpu.vdr_dm_data {
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
        });

        Ok(())
    }
}
