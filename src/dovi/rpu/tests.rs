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

    let parsed_data = parse_dovi_rpu(&original_data, 0);

    assert_eq!(&original_data[2..], parsed_data.as_slice());
}

#[test]
fn mel_test() {
    let input_file = PathBuf::from("./assets/mel_rpu.bin");
    let mut f = File::open(input_file).unwrap();
    let metadata = f.metadata().unwrap();

    let mut original_data = vec![0; metadata.len() as usize];
    f.read(&mut original_data).unwrap();

    let parsed_data = parse_dovi_rpu(&original_data, 0);

    assert_eq!(&original_data[2..], parsed_data.as_slice());
}
