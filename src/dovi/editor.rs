use serde::{Deserialize, Serialize};
use std::fs::File;
use std::{collections::HashMap, path::PathBuf};

use super::{parse_rpu_file, rpu::vdr_dm_data::ExtMetadataBlockLevel5, write_rpu_file, DoviRpu};

pub struct Editor {
    input: PathBuf,
    json_path: PathBuf,
    rpu_out: PathBuf,

    rpus: Option<Vec<DoviRpu>>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct EditConfig {
    #[serde(default)]
    mode: u8,

    #[serde(skip_serializing_if = "Option::is_none")]
    active_area: Option<ActiveArea>,

    #[serde(default)]
    p5_to_p81: bool,
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
        let config: EditConfig = serde_json::from_reader(&json_file).unwrap();

        println!("{:#?}", config);

        editor.rpus = parse_rpu_file(&editor.input);

        if let Some(ref mut rpus) = editor.rpus {
            config.execute(rpus);

            match write_rpu_file(&editor.rpu_out, rpus) {
                Ok(_) => (),
                Err(e) => panic!("{:?}", e),
            }
        }
    }
}

impl EditConfig {
    fn execute(&self, rpus: &mut Vec<DoviRpu>) {
        // Convert with mode
        if self.mode > 0 && !self.p5_to_p81 {
            self.convert_with_mode(rpus);
        } else if self.p5_to_p81 {
            self.convert_p5_to_p81(rpus);
        }

        if let Some(active_area) = &self.active_area {
            active_area.execute(rpus);
        }
    }

    fn convert_with_mode(&self, rpus: &mut Vec<DoviRpu>) {
        println!("Converting with mode {}...", self.mode);
        rpus.iter_mut()
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

    fn convert_p5_to_p81(&self, rpus: &mut Vec<DoviRpu>) {
        println!("Converting from profile 5 to profile 8.1 (experimental)");
        rpus.iter_mut().for_each(|rpu| rpu.p5_to_p81());
    }
}

impl ActiveArea {
    fn execute(&self, rpus: &mut Vec<DoviRpu>) {
        if self.crop {
            self.crop(rpus);
        }

        if let Some(edits) = &self.edits {
            if !edits.is_empty() {
                self.do_edits(edits, rpus);
            }
        }
    }

    fn crop(&self, rpus: &mut Vec<DoviRpu>) {
        println!("Cropping...");
        rpus.iter_mut().for_each(|rpu| rpu.crop());
    }

    fn do_edits(&self, edits: &HashMap<String, u16>, rpus: &mut Vec<DoviRpu>) {
        if let Some(presets) = &self.presets {
            println!("Editing active area offsets...");

            edits.iter().for_each(|edit| {
                let (start, end) = EditConfig::range_string_to_tuple(edit.0);
                let preset_id = *edit.1;

                if end as usize > rpus.len() {
                    panic!("Invalid range: {} > {} available RPUs", start, rpus.len());
                }

                if let Some(active_area_offsets) = presets.iter().find(|e| e.id == preset_id) {
                    rpus[start..=end].iter_mut().for_each(|rpu| {
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
