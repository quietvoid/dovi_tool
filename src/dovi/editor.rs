use std::fs::File;
use std::{collections::HashMap, path::PathBuf};

use anyhow::{bail, ensure, Result};
use dolby_vision::st2094_10::generate::Level6Metadata;
use dolby_vision::st2094_10::ExtMetadataBlock;
use serde::{Deserialize, Serialize};

use super::{encode_rpus, parse_rpu_file, write_rpu_file, DoviRpu};

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

    level6: Option<Level6Metadata>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ActiveArea {
    #[serde(default)]
    crop: bool,

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

        if let Some(ref mut rpus) = editor.rpus {
            config.execute(rpus)?;

            let mut data = encode_rpus(rpus);

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
    fn execute(&self, rpus: &mut Vec<Option<DoviRpu>>) -> Result<()> {
        // Drop metadata frames
        if let Some(ranges) = &self.remove {
            self.remove_frames(ranges, rpus)?;
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

        if let Some(l6) = &self.level6 {
            self.set_level6_metadata(rpus, l6);
        }

        Ok(())
    }

    fn convert_with_mode(&self, rpus: &mut Vec<Option<DoviRpu>>) -> Result<()> {
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

    fn remove_frames(&self, ranges: &[String], rpus: &mut Vec<Option<DoviRpu>>) -> Result<()> {
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

    fn change_source_levels(&self, rpus: &mut Vec<Option<DoviRpu>>) {
        rpus.iter_mut().filter_map(|e| e.as_mut()).for_each(|rpu| {
            rpu.modified = true;

            if let Some(ref mut vdr_dm_data) = rpu.vdr_dm_data {
                vdr_dm_data.change_source_levels(self.min_pq, self.max_pq)
            }
        });
    }

    fn set_level6_metadata(&self, rpus: &mut Vec<Option<DoviRpu>>, l6: &Level6Metadata) {
        rpus.iter_mut().filter_map(|e| e.as_mut()).for_each(|rpu| {
            rpu.modified = true;

            if let Some(ref mut vdr_dm_data) = rpu.vdr_dm_data {
                let level6_block = vdr_dm_data
                    .st2094_10_metadata
                    .ext_metadata_blocks
                    .iter_mut()
                    .find(|e| matches!(e, ExtMetadataBlock::Level6(_)));

                if let Some(ExtMetadataBlock::Level6(ref mut block)) = level6_block {
                    block.set_fields_from_generate_l6(l6);
                } else {
                    vdr_dm_data.st2094_10_metadata.add_level6_metadata(l6);
                }
            }
        });
    }
}

impl ActiveArea {
    fn execute(&self, rpus: &mut Vec<Option<DoviRpu>>) -> Result<()> {
        if self.crop {
            self.crop(rpus);
        }

        if let Some(edits) = &self.edits {
            if !edits.is_empty() {
                self.do_edits(edits, rpus)?;
            }
        }

        Ok(())
    }

    fn crop(&self, rpus: &mut Vec<Option<DoviRpu>>) {
        println!("Cropping...");
        rpus.iter_mut()
            .filter_map(|e| e.as_mut())
            .for_each(|rpu| rpu.crop());
    }

    fn do_edits(
        &self,
        edits: &HashMap<String, u16>,
        rpus: &mut Vec<Option<DoviRpu>>,
    ) -> Result<()> {
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
                    rpus[start..=end]
                        .iter_mut()
                        .filter_map(|e| e.as_mut())
                        .for_each(|rpu| {
                            rpu.modified = true;

                            let (left, right, top, bottom) = (
                                active_area_offsets.left,
                                active_area_offsets.right,
                                active_area_offsets.top,
                                active_area_offsets.bottom,
                            );

                            if let Some(block) = rpu.get_level5_block_mut() {
                                block.set_offsets(left, right, top, bottom);
                            } else if let Some(ref mut dm_data) = rpu.vdr_dm_data {
                                dm_data
                                    .st2094_10_metadata
                                    .add_level5_metadata(left, right, top, bottom);
                            }
                        });
                } else {
                    bail!("Invalid preset ID: {}", preset_id);
                }
            }
        }

        Ok(())
    }
}
