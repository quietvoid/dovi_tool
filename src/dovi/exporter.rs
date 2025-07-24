use std::borrow::Cow;
use std::fs::File;
use std::io::{BufWriter, Write, stdout};
use std::ops::Range;
use std::path::PathBuf;

use anyhow::Result;
use csv::WriterBuilder;
use dolby_vision::rpu::extension_metadata::blocks::{ExtMetadataBlock, ExtMetadataBlockLevel5};
use itertools::Itertools;
use serde::ser::Error;
use serde::{Serialize, Serializer};
use serde_json::json;

use dolby_vision::rpu::utils::parse_rpu_file;

use crate::commands::{ExportArgs, ExportData, ExportLevel, LevelsOutputFormat};
use crate::dovi::{
    editor::{ActiveArea, ActiveAreaOffsets, EditConfig},
    input_from_either,
};

use super::DoviRpu;

pub struct Exporter {
    input: PathBuf,
    data: Vec<(ExportData, Option<PathBuf>)>,
    levels_format: LevelsOutputFormat,
    levels: Vec<(ExportLevel, Option<PathBuf>)>,
}

#[derive(Serialize)]

struct JsonRecord<'a> {
    frame: usize,
    #[serde(flatten, serialize_with = "serialize_level_block")]
    block: &'a ExtMetadataBlock,
}

struct CsvHeaders<'a> {
    block: &'a ExtMetadataBlock,
}

#[derive(Serialize)]
struct CsvRecord<'a> {
    frame: usize,
    #[serde(serialize_with = "serialize_level_block")]
    block: &'a ExtMetadataBlock,
}

impl Exporter {
    pub fn export(args: ExportArgs) -> Result<()> {
        let ExportArgs {
            input,
            input_pos,
            data,
            output,
            levels_format,
            levels,
        } = args;

        let input = input_from_either("editor", input, input_pos)?;
        let mut exporter = Exporter {
            input,
            data,
            levels_format,
            levels,
        };

        if exporter.data.is_empty() && exporter.levels.is_empty() {
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
        if !self.data.is_empty() {
            self.export_data(rpus)?;
        }

        if !self.levels.is_empty() {
            self.export_levels(rpus)?;
        }

        Ok(())
    }

    fn export_data(&self, rpus: &[DoviRpu]) -> Result<()> {
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
                    ser.collect_seq(rpus)?;
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

        let l5_groups = rpus.iter().enumerate().chunk_by(|(_, rpu)| {
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
            if !l5_presets.contains(&k) {
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
            .map(|(id, l5)| ActiveAreaOffsets::new(id as u16, l5))
            .collect::<Vec<_>>();
        let l5_edits = l5_edits
            .into_iter()
            .map(|(edit_range, id)| {
                (
                    format!("{}-{}", edit_range.start, edit_range.end),
                    id as u16,
                )
            })
            .collect();

        let edit_config = EditConfig::from_active_area(ActiveArea {
            crop: true,
            presets: Some(l5_presets),
            edits: Some(l5_edits),
            ..Default::default()
        });
        serde_json::to_writer_pretty(writer, &edit_config)?;

        Ok(())
    }

    fn export_levels(&self, rpus: &[DoviRpu]) -> Result<()> {
        println!(
            "Exporting extension metadata levels {}...",
            self.levels.iter().map(|e| e.0.level()).join(", ")
        );

        for (export_level, maybe_output) in &self.levels {
            let out_path = if let Some(out_path) = maybe_output {
                Cow::Borrowed(out_path)
            } else {
                Cow::Owned(PathBuf::from(
                    export_level.default_output_file(self.levels_format),
                ))
            };

            let writer_buf_len = 10_000;
            let mut writer = BufWriter::with_capacity(
                writer_buf_len,
                File::create(out_path.as_path()).expect("Can't create file"),
            );

            let vdr_dm_data_list = rpus.iter().enumerate().filter_map(|(idx, rpu)| {
                let blocks = rpu
                    .vdr_dm_data
                    .as_ref()
                    .map(|vdr_dm_data| vdr_dm_data.level_blocks_iter(export_level.level()));

                blocks.map(|list| (idx, list))
            });

            match self.levels_format {
                LevelsOutputFormat::Json => {
                    let mut ser = serde_json::Serializer::new(&mut writer);
                    ser.collect_seq(vdr_dm_data_list.flat_map(|(frame, blocks)| {
                        blocks.map(move |block| JsonRecord { frame, block })
                    }))?;
                }
                LevelsOutputFormat::Csv => {
                    let mut writer = WriterBuilder::new().has_headers(false).from_writer(writer);
                    let mut headers_serialized = false;

                    for (frame, blocks) in vdr_dm_data_list {
                        for block in blocks {
                            if !headers_serialized {
                                let value = CsvHeaders { block };
                                writer.serialize(value)?;
                                headers_serialized = true;
                            }

                            writer.serialize(CsvRecord { frame, block })?;
                        }
                    }

                    writer.flush()?;
                }
            }
        }

        Ok(())
    }
}

fn serialize_level_block<S: Serializer>(
    block: &ExtMetadataBlock,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    block.serialize_inner(serializer)
}

impl<'a> Serialize for CsvHeaders<'a> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self.block {
            ExtMetadataBlock::Level1(b) => json!(b),
            ExtMetadataBlock::Level2(b) => json!(b),
            ExtMetadataBlock::Level3(b) => json!(b),
            ExtMetadataBlock::Level4(b) => json!(b),
            ExtMetadataBlock::Level5(b) => json!(b),
            ExtMetadataBlock::Level6(b) => json!(b),
            ExtMetadataBlock::Level8(b) => json!(b),
            ExtMetadataBlock::Level9(b) => json!(b),
            ExtMetadataBlock::Level10(b) => json!(b),
            ExtMetadataBlock::Level11(b) => json!(b),
            ExtMetadataBlock::Level15(b) => json!(b),
            ExtMetadataBlock::Level253(b) => json!(b),
            ExtMetadataBlock::Level254(b) => json!(b),
            ExtMetadataBlock::Level255(b) => json!(b),
            ExtMetadataBlock::Reserved(b) => json!(b),
        };

        value
            .as_object()
            .ok_or_else(|| S::Error::custom("Failed serializing headers"))
            .and_then(|value| {
                let headers = std::iter::once("frame").chain(value.keys().map(String::as_str));
                serializer.collect_seq(headers)
            })
    }
}
