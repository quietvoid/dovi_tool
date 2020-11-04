use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use indicatif::{ProgressBar, ProgressStyle};
use std::io::Read;

pub enum Format {
    Raw,
    RawStdin,
    Matroska,
}

pub struct Parser {
    format: Format,
    input: PathBuf,
    output: Option<PathBuf>,
    verify: bool,
    force_single_profile: bool,
}

#[derive(Debug)]
pub struct NalUnit {
    offset: usize,
    nal_type: u8,
    size: usize,
}

impl Parser {
    pub fn new(
        format: Format,
        input: PathBuf,
        output: Option<PathBuf>,
        verify: bool,
        force_single_profile: bool,
    ) -> Self {
        Self {
            format,
            input,
            output,
            verify,
            force_single_profile,
        }
    }

    pub fn process_file(&self) {
        let pb: ProgressBar;
        let bytes_count;

        if let Format::RawStdin = self.format {
            pb = ProgressBar::hidden();
        } else {
            let file = File::open(&self.input).expect("No file found");

            //Info for indicatif ProgressBar
            let file_meta = file.metadata();
            bytes_count = file_meta.unwrap().len() / 100_000_000;

            if self.verify {
                pb = ProgressBar::hidden();
            } else {
                pb = ProgressBar::new(bytes_count);
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template("[{elapsed_precise}] {bar:60.cyan} {percent}%"),
                );
            }
        }

        let result = match self.format {
            Format::Matroska => panic!("unsupported"),
            _ => self.parse_raw_hevc(Some(&pb)),
        };
    }

    pub fn parse_raw_hevc(&self, pb: Option<&ProgressBar>) {
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

        let chunk_size = 1024 * 1024 * 1;
        let mut read_chunk = [0; 1024 * 1024 * 1];
        let mut chunk = Vec::with_capacity(chunk_size);

        let mut end: Vec<u8> = Vec::with_capacity(512 * 512);

        let mut bl_writer =
            BufWriter::new(File::create(PathBuf::from("BL.hevc")).expect("Can't create file"));
        let mut el_writer =
            BufWriter::new(File::create(PathBuf::from("EL.hevc")).expect("Can't create file"));

        while let Ok(n) = reader.read(&mut read_chunk) {
            if n == 0 {
                break;
            } else if n < chunk_size {
                chunk.extend(&read_chunk[..n]);
            } else {
                chunk.extend(&read_chunk);
            }

            let mut offsets: Vec<usize> = chunk
                .windows(3)
                .enumerate()
                .filter_map(|(index, v)| if v == header { Some(index) } else { None })
                .collect();

            let last = if n < chunk_size {
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

                    el_writer.write_all(&out_header).expect("Failed writing");
                    el_writer.write_all(&data).expect("Failed writing");
                } else {
                    let data = &chunk[offset + 3..offset + size];

                    bl_writer.write_all(&out_header).expect("Failed writing");
                    bl_writer.write_all(&data).expect("Failed writing");
                }
            }

            if end.len() > 0 {
                chunk = end.clone();
            } else {
                chunk.clear();
            }

            end.clear();
        }

        bl_writer.flush().unwrap();
        el_writer.flush().unwrap();
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
