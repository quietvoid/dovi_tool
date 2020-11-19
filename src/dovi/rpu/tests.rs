use std::fs::File;
use std::{io::Read, path::PathBuf};

use super::parse_dovi_rpu;

#[test]
fn fel_test() {
    let input_file = PathBuf::from("./assets/fel_rpu.bin");
    let mut f = File::open(input_file).unwrap();
    let metadata = f.metadata().unwrap();

    let mut original_data = vec![0; metadata.len() as usize];
    f.read(&mut original_data).unwrap();

    let mut dovi_rpu = parse_dovi_rpu(&original_data, 0);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data[2..], parsed_data.as_slice());
}

#[test]
fn mel_test() {
    let input_file = PathBuf::from("./assets/mel_rpu.bin");
    let mut f = File::open(input_file).unwrap();
    let metadata = f.metadata().unwrap();

    let mut original_data = vec![0; metadata.len() as usize];
    f.read(&mut original_data).unwrap();

    let mut dovi_rpu = parse_dovi_rpu(&original_data, 0);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data[2..], parsed_data.as_slice());
}

#[test]
fn fel_orig_test() {
    let input_file = PathBuf::from("./assets/fel_orig.bin");
    let mut f = File::open(input_file).unwrap();
    let metadata = f.metadata().unwrap();

    let mut original_data = vec![0; metadata.len() as usize];
    f.read(&mut original_data).unwrap();

    let mut dovi_rpu = parse_dovi_rpu(&original_data, 0);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data[2..], parsed_data.as_slice());
}

#[test]
fn fel_to_mel_test() {
    let input_file = PathBuf::from("./assets/fel_to_mel1.bin");
    let mut f = File::open(input_file).unwrap();
    let metadata = f.metadata().unwrap();

    let mut original_data = vec![0; metadata.len() as usize];
    f.read(&mut original_data).unwrap();

    let mut dovi_rpu = parse_dovi_rpu(&original_data, 0);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data[2..], parsed_data.as_slice());
}
