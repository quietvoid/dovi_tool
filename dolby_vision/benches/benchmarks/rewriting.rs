use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use criterion::{criterion_group, Criterion};
use dolby_vision::rpu::dovi_rpu::DoviRpu;

const RPU_FILES: &[&str] = &["fel_orig.bin", "mel_variable_l8_length13.bin"];

fn get_bytes<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let mut buf = Vec::with_capacity(500);
    File::open(path).unwrap().read_to_end(&mut buf).unwrap();

    buf
}

pub fn rewrite_single_unspec62_nalu(data: &[u8]) {
    let mut rpu = DoviRpu::parse_unspec62_nalu(data).unwrap();
    rpu.convert_with_mode(2).unwrap();

    rpu.write_hevc_unspec62_nalu().unwrap();
}

fn rewrite_single_unspec62_nalu_benchmark(c: &mut Criterion) {
    let lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let assets_path = lib_path.parent().unwrap().join("assets/tests");

    let mut group = c.benchmark_group("rewrite_single_unspec62_nalu");

    for file in RPU_FILES {
        let bytes = get_bytes(assets_path.join(file));

        group.bench_function(*file, |b| b.iter(|| rewrite_single_unspec62_nalu(&bytes)));
    }
}

criterion_group!(rewrite_rpus, rewrite_single_unspec62_nalu_benchmark);
