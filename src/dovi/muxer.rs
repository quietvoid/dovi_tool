use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write, stdout};
use std::path::PathBuf;

use anyhow::{Result, bail};
use indicatif::ProgressBar;
use itertools::Itertools;

use hevc_parser::HevcParser;
use hevc_parser::io::{FrameBuffer, IoProcessor, NalBuffer, processor};
use hevc_parser::{NALUStartCode, hevc::*};
use processor::{HevcProcessor, HevcProcessorOpts};

use crate::commands::MuxArgs;

use super::hdr10plus_utils::prefix_sei_removed_hdr10plus_nalu;
use super::{CliOptions, IoFormat, StartCodePreset, convert_encoded_from_opts};

const EL_NALU_PREFIX: &[u8] = &[0x7E, 0x01];

pub struct Muxer {
    input: PathBuf,
    format: IoFormat,
    progress_bar: ProgressBar,

    no_add_aud: bool,
    eos_before_el: bool,
    options: CliOptions,

    frame_buffer: FrameBuffer,

    el_processor: HevcProcessor,
    el_handler: ElHandler,
    el_reader: Box<dyn BufRead>,
}

pub struct ElHandler {
    input: PathBuf,
    writer: BufWriter<File>,
    buffered_frames: VecDeque<FrameBuffer>,

    options: CliOptions,
}

impl Muxer {
    pub fn from_args(args: MuxArgs, mut cli_options: CliOptions) -> Result<Self> {
        let MuxArgs {
            bl,
            el,
            output,
            no_add_aud,
            eos_before_el,
            discard,
        } = args;

        cli_options.discard_el = discard;

        let bl_format = hevc_parser::io::format_from_path(&bl)?;
        let el_format = hevc_parser::io::format_from_path(&el)?;

        if bl_format == IoFormat::Matroska {
            bail!("Muxer: Matroska input is unsupported");
        }
        if el_format != IoFormat::Raw {
            bail!("Muxer: Invalid EL file format: must be raw HEVC bitstream");
        }

        let output = match output {
            Some(path) => path,
            None => {
                if cli_options.discard_el {
                    PathBuf::from("BL_RPU.hevc")
                } else {
                    PathBuf::from("BL_EL_RPU.hevc")
                }
            }
        };

        let chunk_size = 100_000;

        let writer =
            BufWriter::with_capacity(chunk_size, File::create(output).expect("Can't create file"));

        let el_file = File::open(&el)?;
        let el_reader = Box::new(BufReader::with_capacity(chunk_size, el_file));

        let el_opts = HevcProcessorOpts {
            buffer_frame: true,
            ..Default::default()
        };
        let el_handler = ElHandler {
            input: el,
            writer,
            buffered_frames: VecDeque::new(),
            options: cli_options.clone(),
        };

        let progress_bar = super::initialize_progress_bar(&bl_format, &bl)?;

        Ok(Self {
            input: bl,
            format: bl_format,
            progress_bar,

            no_add_aud,
            eos_before_el,
            options: cli_options,

            frame_buffer: FrameBuffer {
                frame_number: 0,
                nals: Vec::with_capacity(16),
            },
            el_processor: HevcProcessor::new(IoFormat::Raw, el_opts, chunk_size),
            el_handler,
            el_reader,
        })
    }

    pub fn mux_el(args: MuxArgs, cli_options: CliOptions) -> Result<()> {
        let mut muxer = Muxer::from_args(args, cli_options)?;
        muxer.interleave_el()
    }

    fn interleave_el(&mut self) -> Result<()> {
        println!("Rewriting file with interleaved EL NALUs..");
        stdout().flush().ok();

        let chunk_size = 100_000;

        let mut processor = HevcProcessor::new(
            self.format.clone(),
            HevcProcessorOpts::default(),
            chunk_size,
        );

        let file_path = if let IoFormat::RawStdin = self.format {
            None
        } else {
            Some(self.input.clone())
        };

        processor.process_file(self, file_path)
    }
}

impl IoProcessor for Muxer {
    fn input(&self) -> &PathBuf {
        &self.input
    }

    fn update_progress(&mut self, delta: u64) {
        self.progress_bar.inc(delta);
    }

    fn process_nals(&mut self, parser: &HevcParser, nals: &[NALUnit], chunk: &[u8]) -> Result<()> {
        for nal in nals {
            let mut nalu_data_override = None;

            // Skip ST2094-40 SEI if desired
            if self.options.drop_hdr10plus && nal.nal_type == NAL_SEI_PREFIX {
                let (has_st2094_40, data) = prefix_sei_removed_hdr10plus_nalu(chunk, nal)?;

                // Drop NALUs containing only one SEI message
                if has_st2094_40 && data.is_none() {
                    continue;
                } else {
                    nalu_data_override = data;
                }
            }

            // First NALU of new frame
            // Write previous frame buffer
            if self.frame_buffer.frame_number != nal.decoded_frame_index {
                if !self.no_add_aud {
                    let maybe_frame_gop = parser
                        .processed_frames()
                        .iter()
                        .find(|f| f.decoded_number == self.frame_buffer.frame_number);

                    let maybe_frame_existing = parser
                        .ordered_frames()
                        .iter()
                        .find(|f| f.decoded_number == self.frame_buffer.frame_number);

                    let previous_frame = if let Some(f) = maybe_frame_gop {
                        f
                    } else if let Some(f) = maybe_frame_existing {
                        f
                    } else {
                        bail!("No previous frame found");
                    };

                    self.frame_buffer.nals.insert(
                        0,
                        NalBuffer {
                            nal_type: NAL_AUD,
                            start_code: NALUStartCode::Length4,
                            data: hevc_parser::utils::aud_for_frame(previous_frame, None)?,
                        },
                    );
                }

                // Write BL frame
                self.write_bl_frame()?;

                // Process EL, read if possibly incomplete frame
                if self.el_handler.buffered_frames.len() < 2 {
                    self.el_processor
                        .parse_nalus(&mut self.el_reader, &mut self.el_handler)?;
                }

                // Write EL frame if complete
                if self.el_handler.buffered_frames.len() > 1 {
                    self.el_handler.write_next_frame()?;
                }

                // Write remaining EOS/EOB
                if !self.eos_before_el {
                    Muxer::write_buffers(
                        &mut self.el_handler.writer,
                        self.frame_buffer.nals.iter().enumerate(),
                        self.options.start_code,
                        false,
                    )?;
                }

                self.frame_buffer.frame_number = nal.decoded_frame_index;
                self.frame_buffer.nals.clear();
            }

            // Buffer original BL NALUs
            if nal.nal_type != NAL_UNSPEC62 && nal.nal_type != NAL_UNSPEC63 {
                // Skip AUD NALUs if we're adding them
                if !self.no_add_aud && nal.nal_type == NAL_AUD {
                    continue;
                }

                // Override in case of modified multi-message SEI
                let final_chunk_data = if let Some(data) = nalu_data_override {
                    data
                } else {
                    chunk[nal.start..nal.end].to_vec()
                };

                self.frame_buffer.nals.push(NalBuffer {
                    nal_type: nal.nal_type,
                    start_code: nal.start_code,
                    data: final_chunk_data,
                });
            }
        }

        Ok(())
    }

    fn finalize(&mut self, parser: &HevcParser) -> Result<()> {
        let mut error = None;

        let ordered_frames = parser.ordered_frames();
        let total_frames = ordered_frames.len();

        // Last slice wasn't considered (no AUD/EOS NALU at the end)
        if (self.frame_buffer.frame_number as usize) != total_frames
            && !self.frame_buffer.nals.is_empty()
        {
            if !self.no_add_aud {
                let last_frame = ordered_frames
                    .iter()
                    .find(|f| f.decoded_number == self.frame_buffer.frame_number)
                    .unwrap();

                self.frame_buffer.nals.insert(
                    0,
                    NalBuffer {
                        nal_type: NAL_AUD,
                        start_code: NALUStartCode::Length4,
                        data: hevc_parser::utils::aud_for_frame(last_frame, None)?,
                    },
                );
            }

            // Write last BL frame
            self.write_bl_frame()?;

            // Finalize EL, maybe incomplete last frame
            self.el_processor
                .parse_nalus(&mut self.el_reader, &mut self.el_handler)?;

            // Write last EL frame
            self.el_handler.write_next_frame()?;

            // There should be no more frames if the BL/EL have the same number
            if let Some(last_frame) = self.el_handler.buffered_frames.back() {
                // Do not bail here to avoid incomplete processing
                error = Some(format!(
                    "Mismatched BL/EL frame count. Expected {} frames, got {} (or more) frames in EL.\nThe EL will be trimmed to the BL length.",
                    total_frames,
                    last_frame.frame_number + 1
                ));
            }

            // Write remaining EOS/EOB
            if !self.eos_before_el {
                Muxer::write_buffers(
                    &mut self.el_handler.writer,
                    self.frame_buffer.nals.iter().enumerate(),
                    self.options.start_code,
                    false,
                )?;
            }

            self.frame_buffer.nals.clear();
        }

        self.el_handler.writer.flush()?;

        self.progress_bar.finish_and_clear();

        if let Some(err) = error {
            // Should still error to have correct status code
            bail!(err);
        } else {
            Ok(())
        }
    }
}

impl IoProcessor for ElHandler {
    fn input(&self) -> &PathBuf {
        &self.input
    }

    fn update_progress(&mut self, _delta: u64) {}

    fn process_nals(&mut self, _parser: &HevcParser, nals: &[NALUnit], chunk: &[u8]) -> Result<()> {
        let by_frame = nals.iter().chunk_by(|nal| nal.decoded_frame_index);
        for (frame_number, frame_nals) in &by_frame {
            let nal_buffers = frame_nals
                .filter(|nal| !self.options.discard_el || matches!(nal.nal_type, NAL_UNSPEC62)) // discard everything but RPU
                .map(|nal| {
                    let data = &chunk[nal.start..nal.end];
                    let buf = if nal.nal_type != NAL_UNSPEC62 {
                        let mut vec = Vec::from(EL_NALU_PREFIX);
                        vec.extend(data);

                        vec
                    } else if self.options.mode.is_some() || self.options.edit_config.is_some() {
                        convert_encoded_from_opts(&self.options, data).unwrap()
                    } else {
                        Vec::from(data)
                    };

                    NalBuffer {
                        nal_type: nal.nal_type,
                        start_code: nal.start_code,
                        data: buf,
                    }
                });

            // Existing incomplete frame
            let existing_frame = self
                .buffered_frames
                .iter_mut()
                .find(|fb| fb.frame_number == frame_number);

            if let Some(existing_frame) = existing_frame {
                existing_frame.nals.extend(nal_buffers);
            } else {
                let frame_buffer = FrameBuffer {
                    frame_number,
                    nals: nal_buffers.collect(),
                };

                self.buffered_frames.push_back(frame_buffer);
            }
        }

        Ok(())
    }

    fn finalize(&mut self, _parser: &HevcParser) -> Result<()> {
        Ok(())
    }
}

impl Muxer {
    fn write_bl_frame(&mut self) -> Result<()> {
        if !self.eos_before_el {
            // Default behaviour, EOS/EOB after EL is written

            let nals_to_write = self
                .frame_buffer
                .nals
                .iter()
                .filter(|nb| !matches!(nb.nal_type, NAL_EOS_NUT | NAL_EOB_NUT))
                .enumerate();

            Muxer::write_buffers(
                &mut self.el_handler.writer,
                nals_to_write,
                self.options.start_code,
                true,
            )?;

            self.frame_buffer
                .nals
                .retain(|nb| matches!(nb.nal_type, NAL_EOS_NUT | NAL_EOB_NUT))
        } else {
            Muxer::write_buffers(
                &mut self.el_handler.writer,
                self.frame_buffer.nals.iter().enumerate(),
                self.options.start_code,
                true,
            )?;

            self.frame_buffer.nals.clear();
        }

        Ok(())
    }

    fn write_buffers<'a>(
        writer: &mut dyn Write,
        nal_buffers: impl Iterator<Item = (usize, &'a NalBuffer)>,
        preset: StartCodePreset,
        frame_start: bool,
    ) -> Result<()> {
        for (i, nal_buf) in nal_buffers {
            // First if we didn't write an AUD
            let first_nal = i == 0 && frame_start && nal_buf.nal_type != NAL_AUD;

            NALUnit::write_with_preset(writer, &nal_buf.data, preset, nal_buf.nal_type, first_nal)?;
        }

        Ok(())
    }
}

impl ElHandler {
    fn write_next_frame(&mut self) -> Result<()> {
        if let Some(frame_buffer) = self.buffered_frames.pop_front() {
            for nal_buf in frame_buffer.nals {
                let nal_type = if nal_buf.nal_type != NAL_UNSPEC62 {
                    NAL_UNSPEC63
                } else {
                    NAL_UNSPEC62
                };

                // Ignore nal type since it's wrapped in UNSPEC63
                // Annex B: Always size 3 start code unless RPU
                NALUnit::write_with_preset(
                    &mut self.writer,
                    &nal_buf.data,
                    self.options.start_code,
                    nal_type,
                    false,
                )?;
            }
        }

        Ok(())
    }
}
