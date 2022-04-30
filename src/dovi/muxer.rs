use std::collections::VecDeque;
use std::fs::File;
use std::io::{stdout, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use anyhow::{bail, Result};
use indicatif::ProgressBar;
use itertools::Itertools;

use hevc_parser::hevc::*;
use hevc_parser::io::{processor, FrameBuffer, IoProcessor, NalBuffer};
use hevc_parser::HevcParser;
use processor::{HevcProcessor, HevcProcessorOpts};

use super::{
    convert_encoded_from_opts, get_aud, is_st2094_40_sei, CliOptions, IoFormat, OUT_NAL_HEADER,
};

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
    buffers: VecDeque<FrameBuffer>,

    options: CliOptions,
}

impl Muxer {
    pub fn mux_el(
        bl: PathBuf,
        el: PathBuf,
        output: Option<PathBuf>,
        no_add_aud: bool,
        eos_before_el: bool,
        cli_options: CliOptions,
    ) -> Result<()> {
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

        let writer = BufWriter::with_capacity(
            chunk_size,
            File::create(&output).expect("Can't create file"),
        );

        let el_file = File::open(&el)?;
        let el_reader = Box::new(BufReader::with_capacity(chunk_size, el_file));

        let el_opts = HevcProcessorOpts { buffer_frame: true };
        let el_handler = ElHandler {
            input: el,
            writer,
            buffers: VecDeque::new(),
            options: cli_options,
        };

        let progress_bar = super::initialize_progress_bar(&bl_format, &bl)?;

        let mut muxer = Muxer {
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
        };

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

        let stdin = std::io::stdin();
        let mut reader = Box::new(stdin.lock()) as Box<dyn BufRead>;

        if let IoFormat::Raw = self.format {
            let file = File::open(&self.input)?;
            reader = Box::new(BufReader::with_capacity(100_000, file));
        }

        processor.process_io(&mut reader, self)?;

        Ok(())
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
            // Skip ST2094-40 SEI if desired
            if self.options.drop_hdr10plus
                && nal.nal_type == NAL_SEI_PREFIX
                && is_st2094_40_sei(&chunk[nal.start..nal.end])?
            {
                continue;
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

                    self.el_handler.writer.write_all(&get_aud(previous_frame))?;
                }

                // Write BL frame
                self.write_bl_frame()?;

                // Process EL, read if possibly incomplete frame
                if self.el_handler.buffers.len() < 2 {
                    self.el_processor
                        .parse_nalus(&mut self.el_reader, &mut self.el_handler)?;
                }

                // Write EL frame if complete
                if self.el_handler.buffers.len() > 1 {
                    self.el_handler.write_next_frame()?;
                }

                // Write remaining EOS/EOB
                if !self.eos_before_el {
                    Muxer::write_buffers(
                        &mut self.el_handler.writer,
                        self.frame_buffer.nals.iter(),
                    )?;
                }

                self.frame_buffer.frame_number = nal.decoded_frame_index;
            }

            // Buffer original BL NALUs
            if nal.nal_type != NAL_UNSPEC62 && nal.nal_type != NAL_UNSPEC63 {
                // Skip AUD NALUs if we're adding them
                if !self.no_add_aud && nal.nal_type == NAL_AUD {
                    continue;
                }

                self.frame_buffer.nals.push(NalBuffer {
                    nal_type: nal.nal_type,
                    start_code: nal.start_code,
                    data: chunk[nal.start..nal.end].to_vec(),
                });
            }
        }

        Ok(())
    }

    fn finalize(&mut self, parser: &HevcParser) -> Result<()> {
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

                self.el_handler.writer.write_all(&get_aud(last_frame))?;
            }

            // Write last BL frame
            self.write_bl_frame()?;

            if self.el_handler.buffers.len() == 1 {
                // Maybe incomplete last frame
                self.el_processor
                    .parse_nalus(&mut self.el_reader, &mut self.el_handler)?;

                // Write last EL frame
                self.el_handler.write_next_frame()?;
            } else if let Some(last_frame) = self.el_handler.buffers.back() {
                bail!(
                    "Mismatched BL/EL frame count. Expected {} frames, got {} frames in EL",
                    total_frames,
                    last_frame.frame_number
                );
            }

            // Write remaining EOS/EOB
            if !self.eos_before_el {
                Muxer::write_buffers(&mut self.el_handler.writer, self.frame_buffer.nals.iter())?;
            }
        }

        self.el_handler.writer.flush()?;

        Ok(())
    }
}

impl IoProcessor for ElHandler {
    fn input(&self) -> &PathBuf {
        &self.input
    }

    fn update_progress(&mut self, _delta: u64) {}

    fn process_nals(&mut self, _parser: &HevcParser, nals: &[NALUnit], chunk: &[u8]) -> Result<()> {
        let by_frame = nals.iter().group_by(|nal| nal.decoded_frame_index);
        for (frame_number, frame_nals) in &by_frame {
            let nal_buffers = frame_nals
                .filter(|nal| !self.options.discard_el || matches!(nal.nal_type, NAL_UNSPEC62)) // discard everything but RPU
                .map(|nal| {
                    let data = &chunk[nal.start..nal.end];
                    let buf = if nal.nal_type != NAL_UNSPEC62 {
                        let mut vec = Vec::from(EL_NALU_PREFIX);
                        vec.extend(data);

                        vec
                    } else if let Some(_mode) = self.options.mode {
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
                .buffers
                .iter_mut()
                .find(|fb| fb.frame_number == frame_number);

            if let Some(existing_frame) = existing_frame {
                existing_frame.nals.extend(nal_buffers);
            } else {
                let frame_buffer = FrameBuffer {
                    frame_number,
                    nals: nal_buffers.collect(),
                };

                self.buffers.push_back(frame_buffer);
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
                .filter(|nb| !matches!(nb.nal_type, NAL_EOS_NUT | NAL_EOB_NUT));

            Muxer::write_buffers(&mut self.el_handler.writer, nals_to_write)?;

            self.frame_buffer
                .nals
                .retain(|nb| matches!(nb.nal_type, NAL_EOS_NUT | NAL_EOB_NUT))
        } else {
            Muxer::write_buffers(&mut self.el_handler.writer, self.frame_buffer.nals.iter())?;

            self.frame_buffer.nals.clear();
        }

        Ok(())
    }

    fn write_buffers<'a>(
        writer: &mut dyn Write,
        nal_buffers: impl Iterator<Item = &'a NalBuffer>,
    ) -> Result<()> {
        for nal_buf in nal_buffers {
            writer.write_all(OUT_NAL_HEADER)?;
            writer.write_all(&nal_buf.data)?;
        }

        Ok(())
    }
}

impl ElHandler {
    fn write_next_frame(&mut self) -> Result<()> {
        if let Some(frame_buffer) = self.buffers.pop_front() {
            for nal_buf in &frame_buffer.nals {
                self.writer.write_all(OUT_NAL_HEADER)?;
                self.writer.write_all(&nal_buf.data)?;
            }
        }

        Ok(())
    }
}
