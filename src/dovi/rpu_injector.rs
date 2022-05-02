use std::fs::File;
use std::io::{stdout, BufReader, BufWriter, Write};
use std::path::PathBuf;

use anyhow::{bail, ensure, Result};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use hevc_parser::hevc::*;
use hevc_parser::io::{processor, IoProcessor};
use hevc_parser::HevcParser;
use processor::{HevcProcessor, HevcProcessorOpts};

use utilities_dovi::parse_rpu_file;

use crate::commands::InjectRpuArgs;

use super::{
    get_aud, input_from_either, is_st2094_40_sei, CliOptions, DoviRpu, IoFormat, OUT_NAL_HEADER,
};

pub struct RpuInjector {
    input: PathBuf,
    rpu_in: PathBuf,
    no_add_aud: bool,
    options: CliOptions,

    rpus: Option<Vec<DoviRpu>>,

    writer: BufWriter<File>,
    progress_bar: ProgressBar,
    already_checked_for_rpu: bool,

    frames: Vec<Frame>,
    nals: Vec<NALUnit>,
    mismatched_length: bool,

    last_slice_indices: Vec<usize>,
    last_frame_index: u64,
    nals_parsed: usize,
    last_metadata_written: Option<Vec<u8>>,
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
            rpus: None,

            writer,
            progress_bar,
            already_checked_for_rpu: false,

            frames: Vec::new(),
            nals: Vec::new(),
            mismatched_length: false,

            last_slice_indices: Vec::new(),
            last_frame_index: 0,
            nals_parsed: 0,
            last_metadata_written: None,
        };

        println!("Parsing RPU file...");
        stdout().flush().ok();

        injector.rpus = parse_rpu_file(&injector.rpu_in)?;

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
        if let Some(ref mut rpus) = self.rpus {
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

            println!("Computing frame indices..");
            stdout().flush().ok();

            let pb_indices = ProgressBar::new(self.frames.len() as u64);
            pb_indices.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:60.cyan} {percent}%"),
            );

            self.last_slice_indices = self
                .frames
                .par_iter()
                .map(|f| {
                    let index = find_last_slice_nal_index(&self.nals, f);

                    pb_indices.inc(1);

                    index
                })
                .collect();

            pb_indices.finish_and_clear();

            ensure!(self.frames.len() == self.last_slice_indices.len());

            println!("Rewriting file with interleaved RPU NALs..");
            stdout().flush().ok();

            self.progress_bar = super::initialize_progress_bar(&IoFormat::Raw, &self.input)?;

            let chunk_size = 100_000;

            let mut processor =
                HevcProcessor::new(IoFormat::Raw, HevcProcessorOpts::default(), chunk_size);

            let file = File::open(&self.input)?;
            let mut reader = Box::new(BufReader::with_capacity(chunk_size, file));

            // First frame AUD
            if !self.no_add_aud {
                let first_decoded_frame = self
                    .frames
                    .iter()
                    .find(|f| f.decoded_number == self.last_frame_index)
                    .unwrap();
                self.writer.write_all(&get_aud(first_decoded_frame))?;
            }

            processor.process_io(&mut reader, self)?;
        }

        Ok(())
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
            if let Some(ref mut rpus) = self.rpus {
                for (cur_index, nal) in nals.iter().enumerate() {
                    // On new frame, write AUD
                    if !self.no_add_aud {
                        // Skip existing AUDs
                        if nal.nal_type == NAL_AUD {
                            continue;
                        }

                        if self.last_frame_index != nal.decoded_frame_index {
                            let decoded_frame = self
                                .frames
                                .iter()
                                .find(|f| f.decoded_number == nal.decoded_frame_index)
                                .unwrap();
                            self.writer.write_all(&get_aud(decoded_frame))?;

                            self.last_frame_index = decoded_frame.decoded_number;
                        }
                    }

                    if self.options.drop_hdr10plus
                        && nal.nal_type == NAL_SEI_PREFIX
                        && is_st2094_40_sei(&chunk[nal.start..nal.end])?
                    {
                        continue;
                    }

                    if nal.nal_type != NAL_UNSPEC62 {
                        // Skip writing existing RPUs, only one allowed
                        self.writer.write_all(OUT_NAL_HEADER)?;
                        self.writer.write_all(&chunk[nal.start..nal.end])?;
                    }

                    let global_index = self.nals_parsed + cur_index;

                    // Slice before interleaved RPU
                    if self.last_slice_indices.contains(&global_index) {
                        // We can unwrap because parsed indices are the same
                        let rpu_index = self
                            .last_slice_indices
                            .iter()
                            .position(|i| i == &global_index)
                            .unwrap();

                        // If we have a RPU for index, write it
                        // Otherwise, write the same data as previous
                        if rpu_index < rpus.len() {
                            let dovi_rpu = &mut rpus[rpu_index];
                            let data = dovi_rpu.write_hevc_unspec62_nalu()?;

                            self.writer.write_all(OUT_NAL_HEADER)?;
                            self.writer.write_all(&data)?;

                            self.last_metadata_written = Some(data);
                        } else if self.mismatched_length {
                            if let Some(data) = &self.last_metadata_written {
                                self.writer.write_all(OUT_NAL_HEADER)?;
                                self.writer.write_all(data)?;
                            }
                        }
                    }
                }

                self.nals_parsed += nals.len();
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
            // Second pass
            self.writer.flush()?;
        }

        self.progress_bar.finish_and_clear();

        Ok(())
    }
}

fn find_last_slice_nal_index(nals: &[NALUnit], frame: &Frame) -> usize {
    let slice_nals = frame.nals.iter().enumerate().filter(|(_idx, nal)| {
        matches!(
            nal.nal_type,
            NAL_TRAIL_R
                | NAL_TRAIL_N
                | NAL_TSA_N
                | NAL_TSA_R
                | NAL_STSA_N
                | NAL_STSA_R
                | NAL_BLA_W_LP
                | NAL_BLA_W_RADL
                | NAL_BLA_N_LP
                | NAL_IDR_W_RADL
                | NAL_IDR_N_LP
                | NAL_CRA_NUT
                | NAL_RADL_N
                | NAL_RADL_R
                | NAL_RASL_N
                | NAL_RASL_R
        )
    });

    // Assuming the slices are decoded in order, the highest index is the last slice NAL
    let last_slice = slice_nals
        .enumerate()
        .max_by_key(|(_idx1, (idx2, _))| *idx2)
        .unwrap();

    let last_slice_index = last_slice.0;
    let last_slice_global_index = last_slice.1 .0;
    let last_slice_nal = last_slice.1 .1;

    // Use last non EOS/EOB NALU
    let non_eos_eob_nal_count = frame
        .nals
        .iter()
        .filter(|nal| !matches!(nal.nal_type, NAL_EOS_NUT | NAL_EOB_NUT))
        .count();

    let last_nal_offset = last_slice_index + non_eos_eob_nal_count - last_slice_global_index - 1;

    if let Some(first_slice_index) = nals.iter().position(|n| {
        n.decoded_frame_index == frame.decoded_number && last_slice_nal.nal_type == n.nal_type
    }) {
        first_slice_index + last_nal_offset
    } else {
        panic!("Could not find a NAL for frame {}", frame.decoded_number);
    }
}
