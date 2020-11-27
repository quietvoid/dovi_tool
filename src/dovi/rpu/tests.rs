use std::fs::File;
use std::{io::Read, path::PathBuf};

use super::parse_dovi_rpu;
use super::DoviRpu;

pub fn parse_file(input: PathBuf) -> (Vec<u8>, DoviRpu) {
    let mut f = File::open(input).unwrap();
    let metadata = f.metadata().unwrap();

    let mut original_data = vec![0; metadata.len() as usize];
    f.read_exact(&mut original_data).unwrap();

    let dovi_rpu = parse_dovi_rpu(&original_data).unwrap();

    (original_data, dovi_rpu)
}

#[test]
fn profile5() {
    let mode = 0;
    let (original_data, mut dovi_rpu) = parse_file(PathBuf::from("./assets/profile5.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 5);
    let parsed_data = dovi_rpu.write_rpu_data(mode);

    assert_eq!(&original_data[2..], &parsed_data[2..]);
}

#[test]
fn profile8() {
    let mode = 0;
    let (original_data, mut dovi_rpu) = parse_file(PathBuf::from("./assets/profile8.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 8);
    let parsed_data = dovi_rpu.write_rpu_data(mode);

    assert_eq!(&original_data[2..], &parsed_data[2..]);
}

#[test]
fn fel() {
    let mode = 0;
    let (original_data, mut dovi_rpu) = parse_file(PathBuf::from("./assets/fel_rpu.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data(mode);

    assert_eq!(&original_data[2..], &parsed_data[2..]);
}

#[test]
fn mel() {
    let mode = 0;
    let (original_data, mut dovi_rpu) = parse_file(PathBuf::from("./assets/mel_rpu.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data(mode);

    assert_eq!(&original_data[2..], &parsed_data[2..]);
}

#[test]
fn fel_conversions() {
    let mode = 0;
    let (original_data, mut dovi_rpu) = parse_file(PathBuf::from("./assets/fel_orig.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let mut parsed_data = dovi_rpu.write_rpu_data(mode);

    assert_eq!(&original_data[2..], &parsed_data[2..]);

    // FEL to MEL
    let (mel_data, mel_rpu) = parse_file(PathBuf::from("./assets/fel_to_mel.bin"));
    assert_eq!(mel_rpu.dovi_profile, 7);

    parsed_data = dovi_rpu.write_rpu_data(1);
    assert_eq!(&mel_data[2..], &parsed_data[2..]);

    // FEL to 8.1
    let (p81_data, p81_rpu) = parse_file(PathBuf::from("./assets/fel_to_81.bin"));
    assert_eq!(p81_rpu.dovi_profile, 8);

    parsed_data = dovi_rpu.write_rpu_data(2);
    assert_eq!(&p81_data[2..], &parsed_data[2..]);
}

#[test]
fn fel_to_mel() {
    let mode = 0;
    let (original_data, mut dovi_rpu) = parse_file(PathBuf::from("./assets/fel_to_mel.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data(mode);

    assert_eq!(&original_data[2..], &parsed_data[2..]);
}

#[test]
fn fel_to_profile8() {
    let mode = 0;
    let (original_data, mut dovi_rpu) = parse_file(PathBuf::from("./assets/fel_to_81.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 8);
    let parsed_data = dovi_rpu.write_rpu_data(mode);

    assert_eq!(&original_data[2..], &parsed_data[2..]);
}

#[test]
fn mel_conversions() {
    let mode = 0;
    let (original_data, mut dovi_rpu) = parse_file(PathBuf::from("./assets/mel_orig.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let mut parsed_data = dovi_rpu.write_rpu_data(mode);

    assert_eq!(&original_data[2..], &parsed_data[2..]);

    // MEL to MEL
    let (mel_data, mel_rpu) = parse_file(PathBuf::from("./assets/mel_to_mel.bin"));
    assert_eq!(mel_rpu.dovi_profile, 7);

    parsed_data = dovi_rpu.write_rpu_data(1);
    assert_eq!(&mel_data[2..], &parsed_data[2..]);

    // MEL to 8.1
    let (p81_data, p81_rpu) = parse_file(PathBuf::from("./assets/mel_to_81.bin"));
    assert_eq!(p81_rpu.dovi_profile, 8);

    parsed_data = dovi_rpu.write_rpu_data(2);
    assert_eq!(&p81_data[2..], &parsed_data[2..]);
}

#[test]
fn data_before_crc32() {
    let mode = 0;
    let (original_data, mut dovi_rpu) = parse_file(PathBuf::from("./assets/data_before_crc32.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data(mode);

    assert_eq!(&original_data[2..], &parsed_data[2..]);
}

#[test]
fn fix_se_write() {
    let mode = 0;
    let (original_data, mut dovi_rpu) = parse_file(PathBuf::from("./assets/fix_se_write.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data(mode);

    assert_eq!(&original_data[2..], &parsed_data[2..]);
}