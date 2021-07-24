use std::fs::File;
use std::io::{stdout, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

//use crate::dovi::get_aud;

use super::{input_format, parse_rpu_file, DoviRpu, Format, OUT_NAL_HEADER};

use hevc_parser::hevc::*;
use hevc_parser::HevcParser;
use indicatif::{ProgressBar, ProgressStyle};

pub struct RpuInjector {
    input: PathBuf,
    rpu_in: PathBuf,
    output: PathBuf,

    rpus: Option<Vec<DoviRpu>>,
}

impl RpuInjector {
    pub fn inject_rpu(input: PathBuf, rpu_in: PathBuf, output: Option<PathBuf>) {
        match input_format(&input) {
            Ok(format) => {
                if let Format::Raw = format {
                    let output = match output {
                        Some(path) => path,
                        None => PathBuf::from("injected_output.hevc"),
                    };

                    let mut injector = RpuInjector::new(input, rpu_in, output);
                    let mut parser = HevcParser::default();

                    injector.process_input(&mut parser, format);
                    parser.finish();

                    let frames = parser.ordered_frames();
                    let nals = parser.get_nals();

                    match injector.interleave_rpu_nals(&nals, frames) {
                        Ok(_) => (),
                        Err(e) => panic!("{}", e),
                    }
                } else {
                    panic!("unsupported format");
                }
            }
            Err(msg) => println!("{}", msg),
        }
    }

    fn process_input(&self, parser: &mut HevcParser, format: Format) {
        println!("Processing input video for frame order info...");
        stdout().flush().ok();

        let pb = super::initialize_progress_bar(&format, &self.input);

        //BufReader & BufWriter
        let file = File::open(&self.input).unwrap();
        let mut reader = Box::new(BufReader::with_capacity(100_000, file));

        let chunk_size = 100_000;

        let mut main_buf = vec![0; 100_000];

        let mut chunk = Vec::with_capacity(chunk_size);
        let mut end: Vec<u8> = Vec::with_capacity(chunk_size);

        let mut consumed = 0;

        let mut offsets = Vec::with_capacity(2048);

        while let Ok(n) = reader.read(&mut main_buf) {
            let read_bytes = n;
            if read_bytes == 0 {
                break;
            }

            if read_bytes < chunk_size {
                chunk.extend_from_slice(&main_buf[..read_bytes]);
            } else {
                chunk.extend_from_slice(&main_buf);
            }

            parser.get_offsets(&chunk, &mut offsets);

            if offsets.is_empty() {
                continue;
            }

            let last = if read_bytes < chunk_size {
                *offsets.last().unwrap()
            } else {
                let last = offsets.pop().unwrap();

                end.clear();
                end.extend_from_slice(&chunk[last..]);

                last
            };

            parser.split_nals(&chunk, &offsets, last, true);

            chunk.clear();

            if !end.is_empty() {
                chunk.extend_from_slice(&end);
            }

            consumed += read_bytes;

            if consumed >= 100_000_000 {
                pb.inc(1);
                consumed = 0;
            }
        }

        pb.finish_and_clear();
    }

    pub fn new(input: PathBuf, rpu_in: PathBuf, output: PathBuf) -> RpuInjector {
        let mut injector = RpuInjector {
            input,
            rpu_in,
            output,
            rpus: None,
        };

        injector.rpus = parse_rpu_file(&injector.rpu_in);

        injector
    }

    fn interleave_rpu_nals(
        &mut self,
        nals: &[NALUnit],
        frames: &[Frame],
    ) -> Result<(), std::io::Error> {
        if let Some(ref mut rpus) = self.rpus {
            if frames.len() != rpus.len() {
                panic!("Number of frames of input and RPU file are different!");
            }

            println!("Computing frame indices..");
            stdout().flush().ok();

            let pb_indices = ProgressBar::new(frames.len() as u64);
            pb_indices.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:60.cyan} {percent}%"),
            );

            let last_slice_indices: Vec<usize> = frames
                .iter()
                .map(|f| {
                    let index = find_last_slice_nal_index(nals, f);

                    pb_indices.inc(1);

                    index
                })
                .collect();

            pb_indices.finish_and_clear();

            assert_eq!(frames.len(), last_slice_indices.len());

            println!("Rewriting file with interleaved RPU NALs..");
            stdout().flush().ok();

            let pb = super::initialize_progress_bar(&Format::Raw, &self.input);
            let mut parser = HevcParser::default();

            let chunk_size = 100_000;

            let mut main_buf = vec![0; 100_000];

            let mut chunk = Vec::with_capacity(chunk_size);
            let mut end: Vec<u8> = Vec::with_capacity(chunk_size);

            //BufReader & BufWriter
            let file = File::open(&self.input).unwrap();
            let mut reader = Box::new(BufReader::with_capacity(100_000, file));
            let mut writer = BufWriter::with_capacity(
                chunk_size,
                File::create(&self.output).expect("Can't create file"),
            );

            let mut consumed = 0;
            let mut offsets = Vec::with_capacity(2048);

            let mut nals_parsed = 0;

            // AUDs
            //let first_decoded_index = frames.iter().position(|f| f.decoded_number == 0).unwrap();
            //writer.write_all(&get_aud(&frames[first_decoded_index]))?;

            while let Ok(n) = reader.read(&mut main_buf) {
                let read_bytes = n;
                if read_bytes == 0 {
                    break;
                }

                if read_bytes < chunk_size {
                    chunk.extend_from_slice(&main_buf[..read_bytes]);
                } else {
                    chunk.extend_from_slice(&main_buf);
                }

                parser.get_offsets(&chunk, &mut offsets);

                if offsets.is_empty() {
                    continue;
                }

                let last = if read_bytes < chunk_size {
                    *offsets.last().unwrap()
                } else {
                    let last = offsets.pop().unwrap();

                    end.clear();
                    end.extend_from_slice(&chunk[last..]);

                    last
                };

                let nals = parser.split_nals(&chunk, &offsets, last, true);

                for (cur_index, nal) in nals.iter().enumerate() {
                    // AUDs
                    //if nal.nal_type == NAL_AUD {
                    //    continue;
                    //}

                    writer.write_all(OUT_NAL_HEADER)?;
                    writer.write_all(&chunk[nal.start..nal.end])?;

                    let global_index = nals_parsed + cur_index;

                    // Slice before interleaved RPU
                    if last_slice_indices.contains(&global_index) {
                        let rpu_index = last_slice_indices
                            .iter()
                            .position(|i| i == &global_index)
                            .unwrap();
                        let dovi_rpu = &mut rpus[rpu_index];
                        let data = dovi_rpu.write_rpu_data();

                        writer.write_all(OUT_NAL_HEADER)?;
                        writer.write_all(&data)?;

                        // AUDs
                        //if rpu_index < rpus.len() - 1 {
                        //    writer.write_all(&get_aud(&frames[rpu_index]))?;
                        //}
                    }
                }

                nals_parsed += nals.len();

                chunk.clear();

                if !end.is_empty() {
                    chunk.extend_from_slice(&end);
                }

                consumed += read_bytes;

                if consumed >= 100_000_000 {
                    pb.inc(1);
                    consumed = 0;
                }
            }

            parser.finish();

            writer.flush()?;

            pb.finish_and_clear();
        }

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

    // Use the last nal because there might be suffix NALs (EL or SEI suffix)
    let last_nal_offset = last_slice_index + frame.nals.len() - last_slice_global_index - 1;

    if let Some(first_slice_index) = nals.iter().position(|n| {
        n.decoded_frame_index == frame.decoded_number && last_slice_nal.nal_type == n.nal_type
    }) {
        first_slice_index + last_nal_offset
    } else {
        panic!("Could not find a NAL for frame {}", frame.decoded_number);
    }
}
