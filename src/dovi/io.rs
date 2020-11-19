use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use indicatif::ProgressBar;
use std::io::Read;

use super::rpu::parse_dovi_rpu;
use super::Format;

pub struct DoviReader {
    nal_header: Vec<u8>,
    out_nal_header: Vec<u8>,
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
        let chunk_size = 1024 * 1024 * 4;
        let bl_writer = if let Some(bl_out) = bl_out {
            Some(BufWriter::with_capacity(
                chunk_size * 2,
                File::create(bl_out).expect("Can't create file"),
            ))
        } else {
            None
        };

        let el_writer = if let Some(el_out) = el_out {
            Some(BufWriter::with_capacity(
                chunk_size * 2,
                File::create(el_out).expect("Can't create file"),
            ))
        } else {
            None
        };

        let rpu_writer = if let Some(rpu_out) = rpu_out {
            Some(BufWriter::with_capacity(
                chunk_size * 2,
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
    pub fn new() -> DoviReader {
        DoviReader {
            nal_header: vec![0, 0, 1],
            out_nal_header: vec![0, 0, 0, 1],
        }
    }

    pub fn read_write_from_io(
        &self,
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
            reader = Box::new(BufReader::with_capacity(1024 * 1024 * 4, file));
        }

        let chunk_size = 1024 * 1024 * 4;

        let mut main_buf = [0; 1024 * 1024 * 4];
        let mut sec_buf = [0; 256 * 256 * 2];

        let mut chunk = Vec::with_capacity(chunk_size);
        let mut end: Vec<u8> = Vec::with_capacity(512 * 512);

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

            let mut offsets: Vec<usize> = chunk
                .windows(3)
                .enumerate()
                .filter_map(|(index, v)| {
                    if v == self.nal_header {
                        Some(index)
                    } else {
                        None
                    }
                })
                .collect();

            if offsets.is_empty() {
                continue;
            }

            let last = if read_bytes < chunk_size {
                *offsets.last().unwrap()
            } else {
                let last = offsets.pop().unwrap();

                end.extend(&chunk[last..]);

                last
            };

            let nalus = self.parse_offsets(&chunk, &offsets, last);

            self.write_nalus(&chunk, dovi_writer, &nalus)?;

            if !end.is_empty() {
                chunk = end.clone();
            } else {
                chunk.clear();
            }

            end.clear();

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

    pub fn parse_offsets(&self, chunk: &[u8], offsets: &[usize], last: usize) -> Vec<NalUnit> {
        let mut nalus: Vec<NalUnit> = Vec::new();

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

            nalus.push(NalUnit {
                chunk_type,
                start,
                end,
            });
        }

        nalus
    }

    pub fn write_nalus(
        &self,
        chunk: &[u8],
        dovi_writer: &mut DoviWriter,
        nalus: &Vec<NalUnit>,
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

                        if false {
                            let modified_data = parse_dovi_rpu(&chunk[nalu.start..nalu.end]);

                            rpu_writer.write_all(&modified_data)?;
                        } else {
                            rpu_writer.write_all(&chunk[nalu.start + 2..nalu.end])?;
                        }
                    } else if let Some(ref mut el_writer) = dovi_writer.el_writer {
                        el_writer.write_all(&self.out_nal_header)?;
                        el_writer.write_all(&chunk[nalu.start..nalu.end])?;
                    }
                }
            }
        }

        Ok(())
    }
}
