use std::borrow::Cow;
use std::fs::File;
use std::io::{stdout, BufWriter, Write};
use std::ops::Range;
use std::path::PathBuf;

use anyhow::Result;
use dolby_vision::rpu::extension_metadata::blocks::{ExtMetadataBlock, ExtMetadataBlockLevel5};
use itertools::Itertools;
use serde::ser::SerializeSeq;
use serde::Serializer;

use dolby_vision::rpu::utils::parse_rpu_file;
use serde_json::json;

use crate::commands::{ExportArgs, ExportData};
use crate::dovi::input_from_either;

use super::DoviRpu;

pub struct Exporter {
    input: PathBuf,
    data: Vec<(ExportData, Option<PathBuf>)>,
}

impl Exporter {
    pub fn export(args: ExportArgs) -> Result<()> {
        let ExportArgs {
            input,
            input_pos,
            data,
            output,
        } = args;

        let input = input_from_either("editor", input, input_pos)?;
        let mut exporter = Exporter { input, data };

        if exporter.data.is_empty() {
            exporter.data.push((ExportData::All, output));
        }

        exporter.data.dedup_by_key(|(k, _)| *k);

        println!("Parsing RPU file...");
        stdout().flush().ok();

        let rpus = parse_rpu_file(&exporter.input)?;
        exporter.execute(&rpus)?;

        println!("Done.");

        Ok(())
    }

    fn execute(&self, rpus: &[DoviRpu]) -> Result<()> {
        for (data, maybe_output) in &self.data {
            let out_path = if let Some(out_path) = maybe_output {
                Cow::Borrowed(out_path)
            } else {
                Cow::Owned(PathBuf::from(data.default_output_file()))
            };

            let writer_buf_len = if matches!(data, ExportData::All) {
                100_000
            } else {
                1000
            };
            let mut writer = BufWriter::with_capacity(
                writer_buf_len,
                File::create(out_path.as_path()).expect("Can't create file"),
            );

            match data {
                ExportData::All => {
                    println!("Exporting serialized RPU list...");

                    let mut ser = serde_json::Serializer::new(&mut writer);
                    let mut seq = ser.serialize_seq(Some(rpus.len()))?;

                    for rpu in rpus {
                        seq.serialize_element(&rpu)?;
                    }
                    seq.end()?;
                }
                ExportData::Scenes => {
                    println!("Exporting scenes list...");

                    let scene_refresh_indices = rpus
                        .iter()
                        .enumerate()
                        .filter(|(_, rpu)| {
                            rpu.vdr_dm_data
                                .as_ref()
                                .is_some_and(|vdr| vdr.scene_refresh_flag == 1)
                        })
                        .map(|e| e.0);
                    for i in scene_refresh_indices {
                        writeln!(&mut writer, "{i}")?;
                    }
                }
                ExportData::Level5 => {
                    self.export_level5_config(rpus, &mut writer)?;
                }
            }

            writer.flush()?;
        }

        Ok(())
    }

    fn export_level5_config<W: Write>(&self, rpus: &[DoviRpu], writer: &mut W) -> Result<()> {
        println!("Exporting L5 metadata config...");

        let default_l5 = ExtMetadataBlockLevel5::default();

        let l5_groups = rpus.iter().enumerate().group_by(|(_, rpu)| {
            rpu.vdr_dm_data
                .as_ref()
                .and_then(|vdr| {
                    vdr.get_block(5).and_then(|b| match b {
                        ExtMetadataBlock::Level5(b) => Some(b),
                        _ => None,
                    })
                })
                .unwrap_or(&default_l5)
        });
        let l5_indices = l5_groups
            .into_iter()
            .map(|(k, group)| (k, group.take(1).map(|(i, _)| i).next().unwrap()));

        let mut l5_presets =
            Vec::<&ExtMetadataBlockLevel5>::with_capacity(l5_indices.size_hint().0);
        let mut l5_edits = Vec::<(Range<usize>, usize)>::new();

        for (k, start_index) in l5_indices {
            if !l5_presets.iter().any(|l5| *l5 == k) {
                l5_presets.push(k);
            }

            if let Some(last_edit) = l5_edits.last_mut() {
                last_edit.0.end = start_index - 1;
            }

            let preset_idx = l5_presets.iter().position(|l5| *l5 == k).unwrap();
            l5_edits.push((start_index..start_index, preset_idx));
        }

        // Set last edit end index
        if let Some(last_edit) = l5_edits.last_mut() {
            last_edit.0.end = rpus.len() - 1;
        }

        let l5_presets = l5_presets
            .iter()
            .enumerate()
            .map(|(id, l5)| {
                json!({
                    "id": id,
                    "left": l5.active_area_left_offset,
                    "right": l5.active_area_right_offset,
                    "top": l5.active_area_top_offset,
                    "bottom": l5.active_area_bottom_offset
                })
            })
            .collect::<Vec<_>>();
        let l5_edits = l5_edits.iter().map(|(edit_range, id)| {
            (
                format!("{}-{}", edit_range.start, edit_range.end),
                json!(id),
            )
        });
        let l5_edits = serde_json::Value::Object(l5_edits.collect());

        let edit_config = json!({
            "crop": true,
            "presets": l5_presets,
            "edits": l5_edits,
        });
        serde_json::to_writer_pretty(writer, &edit_config)?;

        Ok(())
    }
}
