use std::io::{stdout, BufRead, BufReader, BufWriter, Write};
use std::{fs::File, path::Path};

use ansi_term::Colour::Red;
use indicatif::ProgressBar;
use std::io::Read;

use super::rpu::parse_dovi_rpu;
use super::{Format, RpuOptions, OUT_NAL_HEADER};

use hevc_parser::hevc::NALUnit;
use hevc_parser::hevc::{NAL_UNSPEC62, NAL_UNSPEC63};
use hevc_parser::HevcParser;

pub struct DoviReader {
    options: RpuOptions,
    rpu_nals: Vec<RpuNal>,
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

impl DoviReader {
    pub fn new(options: RpuOptions) -> DoviReader {
        DoviReader {
            options,
            rpu_nals: Vec::new(),
        }
    }

    pub fn read_write_from_io(
        &mut self,
        format: &Format,
        input: &Path,
        pb: Option<&ProgressBar>,
        dovi_writer: &mut DoviWriter,
    ) -> Result<(), std::io::Error> {
        //BufReader & BufWriter
        let stdin = std::io::stdin();
        let mut reader = Box::new(stdin.lock()) as Box<dyn BufRead>;

        if let Format::Raw = format {
            let file = File::open(input)?;
            reader = Box::new(BufReader::with_capacity(100_000, file));
        }

        let chunk_size = 100_000;

        let mut main_buf = vec![0; 100_000];
        let mut sec_buf = vec![0; 50_000];

        let mut chunk = Vec::with_capacity(chunk_size);
        let mut end: Vec<u8> = Vec::with_capacity(100_000);

        let mut consumed = 0;

        let mut parser = HevcParser::default();

        let mut offsets = Vec::with_capacity(2048);
        let parse_nals = dovi_writer.rpu_writer.is_some();

        while let Ok(n) = reader.read(&mut main_buf) {
            let mut read_bytes = n;
            if read_bytes == 0 && end.is_empty() && chunk.is_empty() {
                break;
            }

            if *format == Format::RawStdin {
                chunk.extend_from_slice(&main_buf[..read_bytes]);

                loop {
                    match reader.read(&mut sec_buf) {
                        Ok(num) => {
                            if num > 0 {
                                read_bytes += num;

                                chunk.extend_from_slice(&sec_buf[..num]);

                                if read_bytes >= chunk_size {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        Err(e) => panic!("{:?}", e),
                    }
                }
            } else if read_bytes < chunk_size {
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

            let nals: Vec<NALUnit> = parser.split_nals(&chunk, &offsets, last, parse_nals);
            self.write_nals(&chunk, dovi_writer, &nals)?;

            chunk.clear();

            if !end.is_empty() {
                chunk.extend_from_slice(&end);
                end.clear();
            }

            consumed += read_bytes;

            if consumed >= 100_000_000 {
                if let Some(pb) = pb {
                    pb.inc(1);
                    consumed = 0;
                }
            }
        }

        if let Some(pb) = pb {
            pb.finish_and_clear();
        }

        parser.finish();

        self.flush_writer(&parser, dovi_writer)?;

        Ok(())
    }

    pub fn write_nals(
        &mut self,
        chunk: &[u8],
        dovi_writer: &mut DoviWriter,
        nals: &[NALUnit],
    ) -> Result<(), std::io::Error> {
        for nal in nals {
            if let Some(ref mut sl_writer) = dovi_writer.sl_writer {
                if nal.nal_type == NAL_UNSPEC63 && self.options.discard_el {
                    continue;
                }

                sl_writer.write_all(OUT_NAL_HEADER)?;

                if nal.nal_type == NAL_UNSPEC62 {
                    if let Some(mode) = self.options.mode {
                        match parse_dovi_rpu(&chunk[nal.start..nal.end]) {
                            Ok(mut dovi_rpu) => {
                                dovi_rpu.convert_with_mode(mode);

                                if self.options.crop {
                                    dovi_rpu.crop();
                                }

                                let modified_data = dovi_rpu.write_rpu_data();
                                sl_writer.write_all(&modified_data)?;

                                continue;
                            }
                            Err(e) => panic!("{}", Red.paint(e)),
                        }
                    }
                }

                sl_writer.write_all(&chunk[nal.start..nal.end])?;

                continue;
            }

            match nal.nal_type {
                NAL_UNSPEC63 => {
                    if let Some(ref mut el_writer) = dovi_writer.el_writer {
                        el_writer.write_all(OUT_NAL_HEADER)?;
                        el_writer.write_all(&chunk[nal.start + 2..nal.end])?;
                    }
                }
                NAL_UNSPEC62 => {
                    if let Some(ref mut el_writer) = dovi_writer.el_writer {
                        el_writer.write_all(OUT_NAL_HEADER)?;
                    }

                    // No mode: Copy
                    // Mode 0: Parse, untouched
                    // Mode 1: to MEL
                    // Mode 2: to 8.1
                    if let Some(mode) = self.options.mode {
                        match parse_dovi_rpu(&chunk[nal.start..nal.end]) {
                            Ok(mut dovi_rpu) => {
                                dovi_rpu.convert_with_mode(mode);

                                if self.options.crop {
                                    dovi_rpu.crop();
                                }

                                let modified_data = dovi_rpu.write_rpu_data();

                                if let Some(ref mut _rpu_writer) = dovi_writer.rpu_writer {
                                    // RPU for x265, remove 0x7C01
                                    self.rpu_nals.push(RpuNal {
                                        decoded_index: nal.decoded_frame_index as usize,
                                        presentation_number: 0,
                                        data: modified_data[2..].to_vec(),
                                    });
                                } else if let Some(ref mut el_writer) = dovi_writer.el_writer {
                                    el_writer.write_all(&modified_data)?;
                                }
                            }
                            Err(e) => panic!("{}", Red.paint(e)),
                        }
                    } else if let Some(ref mut _rpu_writer) = dovi_writer.rpu_writer {
                        // RPU for x265, remove 0x7C01
                        self.rpu_nals.push(RpuNal {
                            decoded_index: nal.decoded_frame_index as usize,
                            presentation_number: 0,
                            data: chunk[nal.start + 2..nal.end].to_vec(),
                        });
                    } else if let Some(ref mut el_writer) = dovi_writer.el_writer {
                        el_writer.write_all(&chunk[nal.start..nal.end])?;
                    }
                }
                _ => {
                    if let Some(ref mut bl_writer) = dovi_writer.bl_writer {
                        bl_writer.write_all(OUT_NAL_HEADER)?;
                        bl_writer.write_all(&chunk[nal.start..nal.end])?;
                    }
                }
            }
        }

        Ok(())
    }

    fn flush_writer(
        &mut self,
        parser: &HevcParser,
        dovi_writer: &mut DoviWriter,
    ) -> Result<(), std::io::Error> {
        if let Some(ref mut bl_writer) = dovi_writer.bl_writer {
            bl_writer.flush()?;
        }

        if let Some(ref mut el_writer) = dovi_writer.el_writer {
            el_writer.flush()?;
        }

        // Reorder RPUs to display output order
        if let Some(ref mut rpu_writer) = dovi_writer.rpu_writer {
            let frames = parser.ordered_frames();

            if frames.is_empty() {
                panic!("No frames parsed!");
            }

            print!("Reordering metadata... ");
            stdout().flush().ok();

            // Remove duplicates because there should only be one RPU NALU per frame
            self.rpu_nals
                .dedup_by(|a, b| a.decoded_index == b.decoded_index);

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
                rpu_writer.write_all(OUT_NAL_HEADER)?;
                rpu_writer.write_all(&rpu.data)?;
            }

            rpu_writer.flush()?;
        }

        Ok(())
    }
}
