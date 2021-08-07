use serde::{Deserialize, Serialize};
use std::fs::File;
use std::{collections::HashMap, path::PathBuf};

use super::{
    encode_rpus, parse_rpu_file, rpu::vdr_dm_data::ExtMetadataBlockLevel5, write_rpu_file, DoviRpu,
};

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
    pub fn edit(input: PathBuf, json_path: PathBuf, rpu_out: Option<PathBuf>) {
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

        let json_file = File::open(&editor.json_path).unwrap();
        let mut config: EditConfig = serde_json::from_reader(&json_file).unwrap();

        println!("{:#?}", config);

        editor.rpus = if let Some(rpus) = parse_rpu_file(&editor.input) {
            Some(rpus.into_iter().map(Some).collect())
        } else {
            None
        };

        if let Some(ref mut rpus) = editor.rpus {
            config.execute(rpus);

            let mut data = encode_rpus(rpus);

            if let Some(ref mut to_duplicate) = config.duplicate {
                to_duplicate.sort_by_key(|meta| meta.offset);
                to_duplicate.reverse();
            }

            if let Some(to_duplicate) = &config.duplicate {
                config.duplicate_metadata(to_duplicate, &mut data);
            }

            println!("Final metadata length: {}", data.len());

            match write_rpu_file(&editor.rpu_out, data) {
                Ok(_) => (),
                Err(e) => panic!("{:?}", e),
            }
        }
    }
}

impl EditConfig {
    fn execute(&self, rpus: &mut Vec<Option<DoviRpu>>) {
        // Drop metadata frames
        if let Some(ranges) = &self.remove {
            self.remove_frames(ranges, rpus);
        }

        // Convert with mode
        if self.mode > 0 {
            self.convert_with_mode(rpus);
        }

        if let Some(active_area) = &self.active_area {
            active_area.execute(rpus);
        }

        if self.min_pq.is_some() || self.max_pq.is_some() {
            self.change_source_levels(rpus);
        }
    }

    fn convert_with_mode(&self, rpus: &mut Vec<Option<DoviRpu>>) {
        println!("Converting with mode {}...", self.mode);
        rpus.iter_mut()
            .filter_map(|e| e.as_mut())
            .for_each(|rpu| rpu.convert_with_mode(self.mode));
    }

    fn range_string_to_tuple(range: &str) -> (usize, usize) {
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

            result
        } else {
            panic!("Invalid edit range");
        }
    }

    fn remove_frames(&self, ranges: &Vec<String>, rpus: &mut Vec<Option<DoviRpu>>) {
        let mut amount = 0;

        ranges.iter().for_each(|range| {
            if range.contains('-') {
                let (start, end) = EditConfig::range_string_to_tuple(range);
                assert!(end < rpus.len());

                amount += end - start + 1;
                rpus[start..=end].iter_mut().for_each(|e| *e = None);
            } else if let Ok(index) = range.parse::<usize>() {
                assert!(index < rpus.len());

                amount += 1;

                rpus[index] = None;
            }
        });

        println!("Removed {} metadata frames.", amount);
    }

    fn duplicate_metadata(&self, to_duplicate: &Vec<DuplicateMetadata>, data: &mut Vec<Vec<u8>>) {
        println!("Duplicating metadata. Initial metadata len {}", data.len());

        to_duplicate.iter().for_each(|meta| {
            assert!(meta.source < data.len() && meta.offset < data.len());

            let source = data[meta.source].clone();
            data.splice(
                meta.offset..meta.offset,
                std::iter::repeat(source).take(meta.length),
            );
        });
    }

    fn change_source_levels(&self, rpus: &mut Vec<Option<DoviRpu>>) {
        rpus.iter_mut().filter_map(|e| e.as_mut()).for_each(|rpu| {
            rpu.modified = true;

            if let Some(ref mut vdr_dm_data) = rpu.vdr_dm_data {
                vdr_dm_data.change_source_levels(self.min_pq, self.max_pq)
            }
        });
    }
}

impl ActiveArea {
    fn execute(&self, rpus: &mut Vec<Option<DoviRpu>>) {
        if self.crop {
            self.crop(rpus);
        }

        if let Some(edits) = &self.edits {
            if !edits.is_empty() {
                self.do_edits(edits, rpus);
            }
        }
    }

    fn crop(&self, rpus: &mut Vec<Option<DoviRpu>>) {
        println!("Cropping...");
        rpus.iter_mut()
            .filter_map(|e| e.as_mut())
            .for_each(|rpu| rpu.crop());
    }

    fn do_edits(&self, edits: &HashMap<String, u16>, rpus: &mut Vec<Option<DoviRpu>>) {
        if let Some(presets) = &self.presets {
            println!("Editing active area offsets...");

            edits.iter().for_each(|edit| {
                let (start, end) = EditConfig::range_string_to_tuple(edit.0);
                let preset_id = *edit.1;

                if end as usize > rpus.len() {
                    panic!("Invalid range: {} > {} available RPUs", end, rpus.len());
                }

                if let Some(active_area_offsets) = presets.iter().find(|e| e.id == preset_id) {
                    rpus[start..=end]
                        .iter_mut()
                        .filter_map(|e| e.as_mut())
                        .for_each(|rpu| {
                            let (left, right, top, bottom) = (
                                active_area_offsets.left,
                                active_area_offsets.right,
                                active_area_offsets.top,
                                active_area_offsets.bottom,
                            );

                            if let Some(block) = ExtMetadataBlockLevel5::get_mut(rpu) {
                                block.set_offsets(left, right, top, bottom);
                            }
                        });
                } else {
                    panic!("Invalid preset ID: {}", preset_id);
                }
            });
        }
    }
}
