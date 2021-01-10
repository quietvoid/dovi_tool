use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use ansi_term::Colour::Red;
use indicatif::ProgressBar;
use nom::{bytes::complete::take_until, IResult};
use std::io::Read;

use super::rpu::parse_dovi_rpu;
use super::Format;

const NAL_START_CODE: &[u8] = &[0, 0, 1];
const HEADER_LEN: usize = 3;

pub struct DoviReader {
    out_nal_header: Vec<u8>,
    mode: Option<u8>,
    nalus: Vec<NalUnit>,
}

pub struct DoviWriter {
    bl_writer: Option<BufWriter<File>>,
    el_writer: Option<BufWriter<File>>,
    rpu_writer: Option<BufWriter<File>>,
}

pub struct NalUnit {
    chunk_type: ChunkType,
    start: usize,
    end: usize,
}

pub enum ChunkType {
    BLChunk,
    ELChunk,
    RPUChunk,
}

impl DoviWriter {
    pub fn new(
        bl_out: Option<&PathBuf>,
        el_out: Option<&PathBuf>,
        rpu_out: Option<&PathBuf>,
    ) -> DoviWriter {
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
            nalus: Vec::with_capacity(2048),
        }
    }

    pub fn take_until_nal(data: &[u8]) -> IResult<&[u8], &[u8]> {
        take_until(NAL_START_CODE)(data)
    }

    pub fn get_offsets(data: &[u8]) -> Vec<usize> {
        let mut consumed = 0;
        let mut offsets = Vec::with_capacity(256);

        loop {
            match Self::take_until_nal(&data[consumed..]) {
                Ok(nal) => {
                    // Byte count before the NAL is the offset
                    consumed += nal.1.len();

                    offsets.push(consumed);

                    // nom consumes the tag, so add it back
                    consumed += HEADER_LEN;
                }
                _ => return offsets,
            }
        }
    }

    pub fn read_write_from_io(
        &mut self,
        format: &Format,
        input: &PathBuf,
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
        let mut end: Vec<u8> = Vec::with_capacity(10_000);

        let mut consumed = 0;

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

            let mut offsets = Self::get_offsets(&chunk);

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

            self.nalus.clear();
            self.parse_offsets(&chunk, &offsets, last);
            self.write_nalus(&chunk, dovi_writer, &self.nalus)?;

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

        if let Some(ref mut bl_writer) = dovi_writer.bl_writer {
            bl_writer.flush()?;
        }

        if let Some(ref mut el_writer) = dovi_writer.el_writer {
            el_writer.flush()?;
        }

        if let Some(ref mut rpu_writer) = dovi_writer.rpu_writer {
            rpu_writer.flush()?;
        }

        Ok(())
    }

    pub fn parse_offsets(&mut self, chunk: &[u8], offsets: &[usize], last: usize) {
        let count = offsets.len();
        for (index, offset) in offsets.iter().enumerate() {
            let size = if offset == &last {
                chunk.len() - offset
            } else {
                let size = if index == count - 1 {
                    last - offset
                } else {
                    offsets[index + 1] - offset
                };

                match &chunk[offset + size - 1..offset + size + 3] {
                    [0, 0, 0, 1] => size - 1,
                    _ => size,
                }
            };

            let nal_type = chunk[offset + 3] >> 1;

            let chunk_type = match nal_type {
                62 => ChunkType::RPUChunk,
                63 => ChunkType::ELChunk,
                _ => ChunkType::BLChunk,
            };

            let start = match chunk_type {
                ChunkType::ELChunk => offset + 5,
                _ => offset + 3,
            };

            let end = offset + size;

            self.nalus.push(NalUnit {
                chunk_type,
                start,
                end,
            });
        }
    }

    pub fn write_nalus(
        &self,
        chunk: &[u8],
        dovi_writer: &mut DoviWriter,
        nalus: &[NalUnit],
    ) -> Result<(), std::io::Error> {
        for nalu in nalus {
            match nalu.chunk_type {
                ChunkType::BLChunk => {
                    if let Some(ref mut bl_writer) = dovi_writer.bl_writer {
                        bl_writer.write_all(&self.out_nal_header)?;
                        bl_writer.write_all(&chunk[nalu.start..nalu.end])?;
                    }
                }
                ChunkType::ELChunk => {
                    if let Some(ref mut el_writer) = dovi_writer.el_writer {
                        el_writer.write_all(&self.out_nal_header)?;
                        el_writer.write_all(&chunk[nalu.start..nalu.end])?;
                    }
                }
                ChunkType::RPUChunk => {
                    if let Some(ref mut rpu_writer) = dovi_writer.rpu_writer {
                        rpu_writer.write_all(&self.out_nal_header)?;
                    } else if let Some(ref mut el_writer) = dovi_writer.el_writer {
                        el_writer.write_all(&self.out_nal_header)?;
                    }

                    // No mode: Copy
                    // Mode 0: Parse, untouched
                    // Mode 1: to MEL
                    // Mode 2: to 8.1
                    if let Some(mode) = self.mode {
                        match parse_dovi_rpu(&chunk[nalu.start..nalu.end]) {
                            Ok(mut dovi_rpu) => {
                                let modified_data = dovi_rpu.write_rpu_data(mode);

                                if let Some(ref mut rpu_writer) = dovi_writer.rpu_writer {
                                    // RPU for x265, remove 0x7C01
                                    rpu_writer.write_all(&modified_data[2..])?;
                                } else if let Some(ref mut el_writer) = dovi_writer.el_writer {
                                    el_writer.write_all(&modified_data)?;
                                }
                            }
                            Err(e) => panic!("{}", Red.paint(e)),
                        }
                    } else if let Some(ref mut rpu_writer) = dovi_writer.rpu_writer {
                        // RPU for x265, remove 0x7C01
                        rpu_writer.write_all(&chunk[nalu.start + 2..nalu.end])?;
                    } else if let Some(ref mut el_writer) = dovi_writer.el_writer {
                        el_writer.write_all(&chunk[nalu.start..nalu.end])?;
                    }
                }
            }
        }

        Ok(())
    }
}
