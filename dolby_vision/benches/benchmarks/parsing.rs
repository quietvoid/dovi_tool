use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use criterion::{Criterion, criterion_group};
use dolby_vision::rpu::dovi_rpu::DoviRpu;

const RPU_FILES: &[&str] = &[
    "profile5.bin",
    "profile8.bin",
    "fel_orig.bin",
    "mel_variable_l8_length13.bin",
    "cmv40_full_rpu.bin",
    "unordered_l8_blocks.bin",
];

fn get_bytes<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let mut buf = Vec::with_capacity(500);
    File::open(path).unwrap().read_to_end(&mut buf).unwrap();

    buf
}

pub fn parse_single_unspec62_nalu(data: &[u8]) {
    DoviRpu::parse_unspec62_nalu(data).unwrap();
}

fn parse_single_unspec62_nalu_benchmark(c: &mut Criterion) {
    let lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let assets_path = lib_path.parent().unwrap().join("assets/tests");

    let mut group = c.benchmark_group("parse_single_unspec62_nalu");

    for file in RPU_FILES {
        let bytes = get_bytes(assets_path.join(file));

        group.bench_function(*file, |b| b.iter(|| parse_single_unspec62_nalu(&bytes)));
    }
}

criterion_group!(parse_rpus, parse_single_unspec62_nalu_benchmark);
