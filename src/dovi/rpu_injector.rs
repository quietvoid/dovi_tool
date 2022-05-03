use std::fs::File;
use std::io::{stdout, BufReader, BufWriter, Write};
use std::path::PathBuf;

use anyhow::{bail, Result};
use indicatif::{ProgressBar, ProgressStyle};

use hevc_parser::io::{processor, FrameBuffer, IoProcessor, NalBuffer};
use hevc_parser::HevcParser;
use hevc_parser::{hevc::*, NALUStartCode};
use processor::{HevcProcessor, HevcProcessorOpts};

use utilities_dovi::parse_rpu_file;

use crate::commands::InjectRpuArgs;

use super::{input_from_either, is_st2094_40_sei, CliOptions, DoviRpu, IoFormat};

pub struct RpuInjector {
    input: PathBuf,
    rpu_in: PathBuf,
    no_add_aud: bool,
    options: CliOptions,

    rpus: Vec<DoviRpu>,

    writer: BufWriter<File>,
    progress_bar: ProgressBar,
    already_checked_for_rpu: bool,

    frames: Vec<Frame>,
    nals: Vec<NALUnit>,
    mismatched_length: bool,

    frame_buffer: FrameBuffer,
    last_metadata_written: Option<NalBuffer>,
}

impl RpuInjector {
    pub fn from_args(args: InjectRpuArgs, cli_options: CliOptions) -> Result<RpuInjector> {
        let InjectRpuArgs {
            input,
            input_pos,
            rpu_in,
            output,
            no_add_aud,
        } = args;

        let input = input_from_either("inject-rpu", input, input_pos)?;

        let output = match output {
            Some(path) => path,
            None => PathBuf::from("injected_output.hevc"),
        };

        let chunk_size = 100_000;
        let progress_bar = super::initialize_progress_bar(&IoFormat::Raw, &input)?;

        let writer = BufWriter::with_capacity(
            chunk_size,
            File::create(&output).expect("Can't create file"),
        );

        let mut injector = RpuInjector {
            input,
            rpu_in,
            no_add_aud,
            options: cli_options,
            rpus: Vec::new(),

            writer,
            progress_bar,
            already_checked_for_rpu: false,

            frames: Vec::new(),
            nals: Vec::new(),
            mismatched_length: false,

            frame_buffer: FrameBuffer {
                frame_number: 0,
                nals: Vec::with_capacity(16),
            },
            last_metadata_written: None,
        };

        println!("Parsing RPU file...");
        stdout().flush().ok();

        // Assumes parsing returns on error
        injector.rpus = parse_rpu_file(&injector.rpu_in)?.unwrap();

        Ok(injector)
    }

    pub fn inject_rpu(args: InjectRpuArgs, cli_options: CliOptions) -> Result<()> {
        let input = input_from_either("inject-rpu", args.input.clone(), args.input_pos.clone())?;
        let format = hevc_parser::io::format_from_path(&input)?;

        if let IoFormat::Raw = format {
            let mut injector = RpuInjector::from_args(args, cli_options)?;

            injector.process_input()?;
            injector.interleave_rpu_nals()
        } else {
            bail!("RpuInjector: Must be a raw HEVC bitstream file")
        }
    }

    fn process_input(&mut self) -> Result<()> {
        println!("Processing input video for frame order info...");
        stdout().flush().ok();

        let chunk_size = 100_000;

        let mut processor =
            HevcProcessor::new(IoFormat::Raw, HevcProcessorOpts::default(), chunk_size);

        let file = File::open(&self.input)?;
        let mut reader = Box::new(BufReader::with_capacity(100_000, file));

        processor.process_io(&mut reader, self)?;

        Ok(())
    }

    fn interleave_rpu_nals(&mut self) -> Result<()> {
        let rpus = &self.rpus;

        self.mismatched_length = if self.frames.len() != rpus.len() {
            println!(
                "\nWarning: mismatched lengths. video {}, RPU {}",
                self.frames.len(),
                rpus.len()
            );

            if rpus.len() < self.frames.len() {
                println!("Metadata will be duplicated at the end to match video length\n");
            } else {
                println!("Metadata will be skipped at the end to match video length\n");
            }

            true
        } else {
            false
        };

        let pb_indices = ProgressBar::new(self.frames.len() as u64);
        pb_indices.set_style(
            ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:60.cyan} {percent}%"),
        );

        pb_indices.finish_and_clear();

        println!("Rewriting file with interleaved RPU NALs..");
        stdout().flush().ok();

        self.progress_bar = super::initialize_progress_bar(&IoFormat::Raw, &self.input)?;

        let chunk_size = 100_000;

        let mut processor =
            HevcProcessor::new(IoFormat::Raw, HevcProcessorOpts::default(), chunk_size);

        let file = File::open(&self.input)?;
        let mut reader = Box::new(BufReader::with_capacity(chunk_size, file));

        processor.process_io(&mut reader, self)?;

        Ok(())
    }

    fn get_rpu_and_index_to_insert(
        frames: &[Frame],
        rpus: &[DoviRpu],
        frame_buffer: &FrameBuffer,
        mismatched_length: bool,
        last_metadata: &Option<NalBuffer>,
    ) -> Result<(usize, NalBuffer)> {
        let existing_frame = frames
            .iter()
            .find(|f| f.decoded_number == frame_buffer.frame_number);

        // If we have a RPU buffered frame, write it
        // Otherwise, write the same data as previous
        let rpu_nb = if let Some(frame) = existing_frame {
            if let Some(ref mut dovi_rpu) = rpus.get(frame.presentation_number as usize) {
                let rpu_data = dovi_rpu.write_hevc_unspec62_nalu()?;

                Some(NalBuffer {
                    nal_type: NAL_UNSPEC62,
                    start_code: NALUStartCode::Length4,
                    data: rpu_data,
                })
            } else {
                bail!(
                    "No RPU found for presentation frame {}",
                    frame.presentation_number
                );
            }
        } else if mismatched_length {
            last_metadata.clone()
        } else {
            None
        };

        if let Some(rpu_nb) = rpu_nb {
            let insert_index = frame_buffer.nals.iter().rposition(|nb| {
                let not_eos_eob = !matches!(nb.nal_type, NAL_EOS_NUT | NAL_EOB_NUT);
                let is_el_nalu = matches!(nb.nal_type, NAL_UNSPEC63);
                let is_slice = NALUnit::is_type_slice(nb.nal_type);

                not_eos_eob && (is_slice || is_el_nalu)
            });

            if let Some(idx) = insert_index {
                // + 1 since we want the RPU after
                Ok((idx + 1, rpu_nb))
            } else {
                bail!(
                    "No slice or UNSPEC63 NALUs in decoded frame {}. Cannot insert RPU.",
                    frame_buffer.frame_number
                );
            }
        } else {
            bail!(
                "No RPU data to write for decoded frame {}",
                frame_buffer.frame_number
            );
        }
    }
}

impl IoProcessor for RpuInjector {
    fn input(&self) -> &PathBuf {
        &self.input
    }

    fn update_progress(&mut self, delta: u64) {
        if !self.already_checked_for_rpu {
            self.already_checked_for_rpu = true;
        }

        self.progress_bar.inc(delta);
    }

    fn process_nals(&mut self, _parser: &HevcParser, nals: &[NALUnit], chunk: &[u8]) -> Result<()> {
        // Second pass
        if !self.frames.is_empty() && !self.nals.is_empty() {
            let rpus = &self.rpus;

            for nal in nals {
                // Ignore HDR10+
                if self.options.drop_hdr10plus
                    && nal.nal_type == NAL_SEI_PREFIX
                    && is_st2094_40_sei(&chunk[nal.start..nal.end])?
                {
                    continue;
                }

                if self.frame_buffer.frame_number != nal.decoded_frame_index {
                    // On new frame, write AUD
                    if !self.no_add_aud {
                        // Skip existing AUDs
                        if nal.nal_type == NAL_AUD {
                            continue;
                        }

                        if self.frame_buffer.frame_number != nal.decoded_frame_index {
                            // Find existing frame for the current buffered frame
                            let buffered_frame = self
                                .frames
                                .iter()
                                .find(|f| f.decoded_number == self.frame_buffer.frame_number)
                                .unwrap();

                            self.writer.write_all(&hevc_parser::utils::aud_for_frame(
                                buffered_frame,
                                Some(NALUStartCode::Length4),
                            ))?;
                        }
                    }

                    let (idx, rpu_nb) = Self::get_rpu_and_index_to_insert(
                        &self.frames,
                        rpus,
                        &self.frame_buffer,
                        self.mismatched_length,
                        &self.last_metadata_written,
                    )?;

                    self.last_metadata_written = Some(rpu_nb.clone());
                    self.frame_buffer.nals.insert(idx, rpu_nb);

                    // Write NALUs for the frame
                    for nal_buf in &self.frame_buffer.nals {
                        self.writer.write_all(NALUStartCode::Length4.slice())?;
                        self.writer.write_all(&nal_buf.data)?;
                    }

                    self.frame_buffer.frame_number = nal.decoded_frame_index;
                    self.frame_buffer.nals.clear();
                }

                // Ignore existing RPU
                if nal.nal_type != NAL_UNSPEC62 {
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
        } else if !self.already_checked_for_rpu && nals.iter().any(|e| e.nal_type == NAL_UNSPEC62) {
            self.already_checked_for_rpu = true;
            println!("\nWarning: Input file already has RPUs, they will be replaced.");
        }

        Ok(())
    }

    fn finalize(&mut self, parser: &HevcParser) -> Result<()> {
        // First pass
        if self.frames.is_empty() && self.nals.is_empty() {
            self.frames = parser.ordered_frames().clone();
            self.nals = parser.get_nals().clone();
        } else {
            let ordered_frames = parser.ordered_frames();
            let total_frames = ordered_frames.len();

            // Last slice wasn't considered (no AUD/EOS NALU at the end)
            if (self.frame_buffer.frame_number as usize) != total_frames
                && !self.frame_buffer.nals.is_empty()
            {
                let rpus = &self.rpus;

                if !self.no_add_aud {
                    let last_frame = ordered_frames
                        .iter()
                        .find(|f| f.decoded_number == self.frame_buffer.frame_number)
                        .unwrap();

                    self.writer.write_all(&hevc_parser::utils::aud_for_frame(
                        last_frame,
                        Some(NALUStartCode::Length4),
                    ))?;
                }

                let (idx, rpu_nb) = Self::get_rpu_and_index_to_insert(
                    &self.frames,
                    rpus,
                    &self.frame_buffer,
                    self.mismatched_length,
                    &self.last_metadata_written,
                )?;

                self.last_metadata_written = Some(rpu_nb.clone());
                self.frame_buffer.nals.insert(idx, rpu_nb);

                // Write NALUs for the last frame
                for nal_buf in &self.frame_buffer.nals {
                    self.writer.write_all(NALUStartCode::Length4.slice())?;
                    self.writer.write_all(&nal_buf.data)?;
                }

                self.frame_buffer.nals.clear();
            }

            // Second pass
            self.writer.flush()?;
        }

        self.progress_bar.finish_and_clear();

        Ok(())
    }
}
