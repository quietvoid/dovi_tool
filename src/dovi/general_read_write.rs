use std::io::{stdout, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::{fs::File, path::Path};

use anyhow::{bail, Result};
use indicatif::ProgressBar;

use hevc_parser::hevc::{NALUnit, NAL_SEI_PREFIX, NAL_UNSPEC62, NAL_UNSPEC63};
use hevc_parser::io::{processor, IoFormat, IoProcessor};
use hevc_parser::HevcParser;
use processor::{HevcProcessor, HevcProcessorOpts};

use super::{convert_encoded_from_opts, is_st2094_40_sei, CliOptions, WriteStartCodePreset};

pub struct DoviProcessor {
    input: PathBuf,
    options: CliOptions,
    rpu_nals: Vec<RpuNal>,

    payload_count: usize,
    previous_frame_index: u64,
    previous_rpu_index: u64,

    progress_bar: ProgressBar,
    dovi_writer: DoviWriter,
}

pub struct DoviWriter {
    bl_writer: Option<BufWriter<File>>,
    el_writer: Option<BufWriter<File>>,
    rpu_writer: Option<BufWriter<File>>,
    sl_writer: Option<BufWriter<File>>,
}

#[derive(Debug)]
pub struct RpuNal {
    decoded_index: usize,
    presentation_number: usize,
    data: Vec<u8>,
}

impl DoviWriter {
    pub fn new(
        bl_out: Option<&Path>,
        el_out: Option<&Path>,
        rpu_out: Option<&Path>,
        single_layer_out: Option<&Path>,
    ) -> DoviWriter {
        let chunk_size = 100_000;
        let bl_writer = bl_out.map(|bl_out| {
            BufWriter::with_capacity(
                chunk_size,
                File::create(bl_out).expect("Can't create file for BL"),
            )
        });

        let el_writer = el_out.map(|el_out| {
            BufWriter::with_capacity(
                chunk_size,
                File::create(el_out).expect("Can't create file for EL"),
            )
        });

        let rpu_writer = rpu_out.map(|rpu_out| {
            BufWriter::with_capacity(
                chunk_size,
                File::create(rpu_out).expect("Can't create file for RPU"),
            )
        });

        let sl_writer = single_layer_out.map(|single_layer_out| {
            BufWriter::with_capacity(
                chunk_size,
                File::create(single_layer_out).expect("Can't create file for SL output"),
            )
        });

        DoviWriter {
            bl_writer,
            el_writer,
            rpu_writer,
            sl_writer,
        }
    }
}

impl DoviProcessor {
    pub fn new(
        options: CliOptions,
        input: PathBuf,
        dovi_writer: DoviWriter,
        progress_bar: ProgressBar,
    ) -> DoviProcessor {
        DoviProcessor {
            input,
            options,
            rpu_nals: Vec::new(),
            payload_count: 0,
            previous_frame_index: 0,
            previous_rpu_index: 0,
            progress_bar,
            dovi_writer,
        }
    }

    pub fn read_write_from_io(&mut self, format: &IoFormat) -> Result<()> {
        let chunk_size = 100_000;

        let processor_opts = HevcProcessorOpts {
            parse_nals: true,
            ..Default::default()
        };
        let mut processor = HevcProcessor::new(format.clone(), processor_opts, chunk_size);

        let stdin = std::io::stdin();
        let mut reader = Box::new(stdin.lock()) as Box<dyn BufRead>;

        if let IoFormat::Raw = format {
            let file = File::open(&self.input)?;
            reader = Box::new(BufReader::with_capacity(100_000, file));
        }

        processor.process_io(&mut reader, self)
    }

    pub fn write_nals(&mut self, chunk: &[u8], nals: &[NALUnit]) -> Result<()> {
        for (i, nal) in nals.iter().enumerate() {
            if self.options.drop_hdr10plus
                && nal.nal_type == NAL_SEI_PREFIX
                && is_st2094_40_sei(&chunk[nal.start..nal.end])?
            {
                continue;
            }

            // Skip duplicate NALUs if they are after a first RPU for the frame
            if self.previous_rpu_index > 0
                && nal.nal_type == NAL_UNSPEC62
                && nal.decoded_frame_index == self.previous_rpu_index
            {
                println!(
                    "Warning: Unexpected RPU NALU found for frame {}. Discarding.",
                    self.previous_rpu_index
                );

                continue;
            }

            // First NAL of stream, or frame
            let first_nal_of_frame =
                if i == 0 && self.payload_count == 0 && self.previous_frame_index == 0 {
                    true
                } else if self.previous_frame_index != nal.decoded_frame_index {
                    self.previous_frame_index = nal.decoded_frame_index;

                    true
                } else {
                    false
                };

            if let Some(ref mut sl_writer) = self.dovi_writer.sl_writer {
                if nal.nal_type == NAL_UNSPEC63 && self.options.discard_el {
                    continue;
                }

                if nal.nal_type == NAL_UNSPEC62
                    && (self.options.mode.is_some() || self.options.edit_config.is_some())
                {
                    let modified_data =
                        convert_encoded_from_opts(&self.options, &chunk[nal.start..nal.end])?;

                    NALUnit::write_with_preset(
                        sl_writer,
                        &modified_data,
                        self.options.start_code.into(),
                        nal.nal_type,
                        first_nal_of_frame,
                    )?;

                    continue;
                }

                NALUnit::write_with_preset(
                    sl_writer,
                    &chunk[nal.start..nal.end],
                    self.options.start_code.into(),
                    nal.nal_type,
                    first_nal_of_frame,
                )?;

                continue;
            }

            match nal.nal_type {
                NAL_UNSPEC63 => {
                    if let Some(ref mut el_writer) = self.dovi_writer.el_writer {
                        // Can't know for EL, always size 4
                        NALUnit::write_with_preset(
                            el_writer,
                            &chunk[nal.start + 2..nal.end],
                            WriteStartCodePreset::Four.into(),
                            nal.nal_type,
                            false,
                        )?;
                    }
                }
                NAL_UNSPEC62 => {
                    self.previous_rpu_index = nal.decoded_frame_index;
                    let rpu_data = &chunk[nal.start..nal.end];

                    // No mode: Copy
                    // Mode 0: Parse, untouched
                    // Mode 1: to MEL
                    // Mode 2: to 8.1
                    // Mode 3: 5 to 8.1
                    if self.options.mode.is_some() || self.options.edit_config.is_some() {
                        let modified_data = convert_encoded_from_opts(&self.options, rpu_data)?;

                        if let Some(ref mut _rpu_writer) = self.dovi_writer.rpu_writer {
                            // RPU for x265, remove 0x7C01
                            self.rpu_nals.push(RpuNal {
                                decoded_index: self.rpu_nals.len(),
                                presentation_number: 0,
                                data: modified_data[2..].to_owned(),
                            });
                        } else if let Some(ref mut el_writer) = self.dovi_writer.el_writer {
                            // RPU should never be first NAL
                            NALUnit::write_with_preset(
                                el_writer,
                                &modified_data,
                                self.options.start_code.into(),
                                nal.nal_type,
                                false,
                            )?;
                        }
                    } else if let Some(ref mut _rpu_writer) = self.dovi_writer.rpu_writer {
                        // RPU for x265, remove 0x7C01
                        self.rpu_nals.push(RpuNal {
                            decoded_index: self.rpu_nals.len(),
                            presentation_number: 0,
                            data: rpu_data[2..].to_vec(),
                        });
                    } else if let Some(ref mut el_writer) = self.dovi_writer.el_writer {
                        // RPU should never be first NAL
                        NALUnit::write_with_preset(
                            el_writer,
                            rpu_data,
                            self.options.start_code.into(),
                            nal.nal_type,
                            false,
                        )?;
                    }
                }
                _ => {
                    if let Some(ref mut bl_writer) = self.dovi_writer.bl_writer {
                        NALUnit::write_with_preset(
                            bl_writer,
                            &chunk[nal.start..nal.end],
                            self.options.start_code.into(),
                            nal.nal_type,
                            first_nal_of_frame,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    fn flush_writer(&mut self, parser: &HevcParser) -> Result<()> {
        if let Some(ref mut bl_writer) = self.dovi_writer.bl_writer {
            bl_writer.flush()?;
        }

        if let Some(ref mut el_writer) = self.dovi_writer.el_writer {
            el_writer.flush()?;
        }

        // Reorder RPUs to display output order
        if let Some(ref mut rpu_writer) = self.dovi_writer.rpu_writer {
            let frames = parser.ordered_frames();

            if frames.is_empty() {
                bail!("No frames parsed!");
            }

            print!("Reordering metadata... ");
            stdout().flush().ok();

            // Sort by matching frame POC
            self.rpu_nals.sort_by_cached_key(|rpu| {
                let matching_index = frames
                    .iter()
                    .position(|f| rpu.decoded_index == f.decoded_number as usize);

                if let Some(i) = matching_index {
                    frames[i].presentation_number
                } else {
                    panic!(
                        "Missing frame/slices for metadata! Decoded index {}",
                        rpu.decoded_index
                    );
                }
            });

            // Set presentation number to new index
            self.rpu_nals
                .iter_mut()
                .enumerate()
                .for_each(|(idx, rpu)| rpu.presentation_number = idx);

            println!("Done.");

            // Write data to file
            for rpu in self.rpu_nals.iter_mut() {
                // RPU file is always 4 bytes start code
                NALUnit::write_with_preset(
                    rpu_writer,
                    &rpu.data,
                    WriteStartCodePreset::Four.into(),
                    NAL_UNSPEC62,
                    true,
                )?;
            }

            rpu_writer.flush()?;
        }

        Ok(())
    }
}

impl IoProcessor for DoviProcessor {
    fn input(&self) -> &std::path::PathBuf {
        &self.input
    }

    fn update_progress(&mut self, delta: u64) {
        self.progress_bar.inc(delta);
    }

    fn process_nals(&mut self, _parser: &HevcParser, nals: &[NALUnit], chunk: &[u8]) -> Result<()> {
        self.write_nals(chunk, nals)?;
        self.payload_count += 1;

        Ok(())
    }

    fn finalize(&mut self, parser: &HevcParser) -> Result<()> {
        self.progress_bar.finish_and_clear();
        self.flush_writer(parser)
    }
}
