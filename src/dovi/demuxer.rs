use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use indicatif::{ProgressBar, ProgressStyle};
use std::io::Read;

use super::rpu::parse_dovi_rpu;
use super::Format;

pub struct Demuxer {
    format: Format,
    input: PathBuf,
    bl_out: PathBuf,
    el_out: PathBuf,
}

impl Demuxer {
    pub fn new(format: Format, input: PathBuf, bl_out: PathBuf, el_out: PathBuf) -> Self {
        Self {
            format,
            input,
            bl_out,
            el_out,
        }
    }

    pub fn process_input(&self) {
        let pb: ProgressBar;
        let bytes_count;

        if let Format::RawStdin = self.format {
            pb = ProgressBar::hidden();
        } else {
            let file = File::open(&self.input).expect("No file found");

            //Info for indicatif ProgressBar
            let file_meta = file.metadata();
            bytes_count = file_meta.unwrap().len() / 100_000_000;

            pb = ProgressBar::new(bytes_count);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:60.cyan} {percent}%"),
            );
        }

        match self.format {
            Format::Matroska => panic!("unsupported"),
            _ => self.demux_raw_hevc(Some(&pb)),
        };

        pb.finish_and_clear();
    }

    pub fn demux_raw_hevc(&self, pb: Option<&ProgressBar>) {
        //BufReader & BufWriter
        let stdin = std::io::stdin();
        let mut reader = Box::new(stdin.lock()) as Box<dyn BufRead>;

        if let Format::Raw = self.format {
            let file = File::open(&self.input).expect("No file found");
            reader = Box::new(BufReader::with_capacity(1024 * 1024 * 4, file));
        }

        let header: Vec<u8> = vec![0, 0, 1];
        let out_header: Vec<u8> = vec![0, 0, 0, 1];
        let el_nal_types = [62, 63];

        let chunk_size = 1024 * 1024 * 4;

        let mut main_buf = [0; 1024 * 1024 * 4];
        let mut sec_buf = [0; 256 * 256 * 2];

        let mut chunk = Vec::with_capacity(chunk_size);
        let mut end: Vec<u8> = Vec::with_capacity(512 * 512);

        let mut bl_writer = BufWriter::with_capacity(
            chunk_size * 2,
            File::create(&self.bl_out).expect("Can't create file"),
        );
        let mut el_writer = BufWriter::with_capacity(
            chunk_size * 2,
            File::create(&self.el_out).expect("Can't create file"),
        );

        let mut rpu_writer = BufWriter::with_capacity(
            chunk_size * 2,
            File::create(PathBuf::from("RPU.bin")).expect("Can't create file"),
        );

        let mut consumed = 0;

        while let Ok(n) = reader.read(&mut main_buf) {
            let mut read_bytes = n;
            if read_bytes == 0 {
                break;
            }

            if self.format == Format::RawStdin {
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
                .filter_map(|(index, v)| if v == header { Some(index) } else { None })
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

                if el_nal_types.contains(&nal_type) {
                    let data = if nal_type == 63 {
                        &chunk[offset + 5..offset + size]
                    } else {
                        &chunk[offset + 3..offset + size]
                    };

                    if nal_type == 62 {
                        let data = parse_dovi_rpu(&data[2..]);

                        rpu_writer.write_all(&out_header).expect("Failed writing");
                        rpu_writer.write_all(&data).expect("Failed writing");
                        rpu_writer.flush();

                        std::process::exit(0);
                    }

                    el_writer.write_all(&out_header).expect("Failed writing");
                    el_writer.write_all(&data).expect("Failed writing");
                } else {
                    let data = &chunk[offset + 3..offset + size];

                    bl_writer.write_all(&out_header).expect("Failed writing");
                    bl_writer.write_all(&data).expect("Failed writing");
                }
            }

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

        bl_writer.flush().unwrap();
        el_writer.flush().unwrap();
        rpu_writer.flush().unwrap();
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Format::Matroska => write!(f, "Matroska file"),
            Format::Raw => write!(f, "HEVC file"),
            Format::RawStdin => write!(f, "HEVC pipe"),
        }
    }
}
