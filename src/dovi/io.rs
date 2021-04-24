use std::io::{BufRead, BufReader, BufWriter, Write};
use std::{fs::File, path::Path};

use ansi_term::Colour::Red;
use indicatif::ProgressBar;
use std::io::Read;

use super::rpu::parse_dovi_rpu;
use super::Format;

use hevc_parser::hevc::NALUnit;
use hevc_parser::hevc::{NAL_UNSPEC62, NAL_UNSPEC63};
use hevc_parser::HevcParser;

pub struct DoviReader {
    out_nal_header: Vec<u8>,
    mode: Option<u8>,

    rpu_nals: Vec<RpuNal>,
}

pub struct DoviWriter {
    bl_writer: Option<BufWriter<File>>,
    el_writer: Option<BufWriter<File>>,
    rpu_writer: Option<BufWriter<File>>,
}

#[derive(Debug)]
pub struct RpuNal {
    decoded_index: usize,
    presentation_number: usize,
    data: Vec<u8>,
}

impl DoviWriter {
    pub fn new(bl_out: Option<&Path>, el_out: Option<&Path>, rpu_out: Option<&Path>) -> DoviWriter {
        let chunk_size = 100_000;
        let bl_writer = if let Some(bl_out) = bl_out {
            Some(BufWriter::with_capacity(
                chunk_size,
                File::create(bl_out).expect("Can't create file"),
            ))
        } else {
            None
        };

        let el_writer = if let Some(el_out) = el_out {
            Some(BufWriter::with_capacity(
                chunk_size,
                File::create(el_out).expect("Can't create file"),
            ))
        } else {
            None
        };

        let rpu_writer = if let Some(rpu_out) = rpu_out {
            Some(BufWriter::with_capacity(
                chunk_size,
                File::create(rpu_out).expect("Can't create file"),
            ))
        } else {
            None
        };

        DoviWriter {
            bl_writer,
            el_writer,
            rpu_writer,
        }
    }
}

impl DoviReader {
    pub fn new(mode: Option<u8>) -> DoviReader {
        DoviReader {
            out_nal_header: vec![0, 0, 0, 1],
            mode,
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
            if read_bytes == 0 {
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
            }

            consumed += read_bytes;

            if consumed >= 100_000_000 {
                if let Some(pb) = pb {
                    pb.inc(1);
                    consumed = 0;
                }
            }
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
            match nal.nal_type {
                NAL_UNSPEC63 => {
                    if let Some(ref mut el_writer) = dovi_writer.el_writer {
                        el_writer.write_all(&self.out_nal_header)?;
                        el_writer.write_all(&chunk[nal.start + 2..nal.end])?;
                    }
                }
                NAL_UNSPEC62 => {
                    if let Some(ref mut el_writer) = dovi_writer.el_writer {
                        el_writer.write_all(&self.out_nal_header)?;
                    }

                    // No mode: Copy
                    // Mode 0: Parse, untouched
                    // Mode 1: to MEL
                    // Mode 2: to 8.1
                    if let Some(mode) = self.mode {
                        match parse_dovi_rpu(&chunk[nal.start..nal.end]) {
                            Ok(mut dovi_rpu) => {
                                let modified_data = dovi_rpu.write_rpu_data(mode);

                                if let Some(ref mut _rpu_writer) = dovi_writer.rpu_writer {
                                    self.rpu_nals.push(RpuNal {
                                        decoded_index: self.rpu_nals.len(),
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
                        self.rpu_nals.push(RpuNal {
                            decoded_index: self.rpu_nals.len(),
                            presentation_number: 0,
                            data: chunk[nal.start + 2..nal.end].to_vec(),
                        });
                    } else if let Some(ref mut el_writer) = dovi_writer.el_writer {
                        el_writer.write_all(&chunk[nal.start..nal.end])?;
                    }
                }
                _ => {
                    if let Some(ref mut bl_writer) = dovi_writer.bl_writer {
                        bl_writer.write_all(&self.out_nal_header)?;
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

            // Sort by matching frame POC
            self.rpu_nals.sort_by_key(|rpu| {
                let matching_index = frames
                    .iter()
                    .position(|f| rpu.decoded_index == f.decoded_number as usize)
                    .unwrap();

                frames[matching_index].presentation_number
            });

            // Set presentation number to new index
            self.rpu_nals
                .iter_mut()
                .enumerate()
                .for_each(|(idx, rpu)| rpu.presentation_number = idx);

            // Write data to file
            for rpu in self.rpu_nals.iter_mut() {
                rpu_writer.write_all(&self.out_nal_header)?;
                rpu_writer.write_all(&rpu.data)?;
            }

            rpu_writer.flush()?;
        }

        Ok(())
    }
}
