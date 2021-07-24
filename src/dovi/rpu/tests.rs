use std::fs::File;
use std::{io::Read, path::PathBuf};

use super::parse_dovi_rpu;
use super::DoviRpu;

pub fn _parse_file(input: PathBuf) -> (Vec<u8>, DoviRpu) {
    let mut f = File::open(input).unwrap();
    let metadata = f.metadata().unwrap();

    let mut original_data = vec![0; metadata.len() as usize];
    f.read_exact(&mut original_data).unwrap();

    let dovi_rpu = parse_dovi_rpu(&original_data).unwrap();

    (original_data, dovi_rpu)
}

#[test]
fn profile4() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/profile4.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 4);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn profile5() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/profile5.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 5);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn profile8() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/profile8.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 8);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn fel() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/fel_rpu.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn mel() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/mel_rpu.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn fel_conversions() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/fel_orig.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let mut parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);

    // FEL to MEL
    let (mel_data, mel_rpu) = _parse_file(PathBuf::from("./assets/fel_to_mel.bin"));
    assert_eq!(mel_rpu.dovi_profile, 7);

    dovi_rpu.convert_with_mode(1);
    parsed_data = dovi_rpu.write_rpu_data();
    assert_eq!(&mel_data, &parsed_data);

    // FEL to 8.1
    let (p81_data, p81_rpu) = _parse_file(PathBuf::from("./assets/fel_to_81.bin"));
    assert_eq!(p81_rpu.dovi_profile, 8);

    dovi_rpu.convert_with_mode(2);
    parsed_data = dovi_rpu.write_rpu_data();
    assert_eq!(&p81_data, &parsed_data);
}

#[test]
fn fel_to_mel() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/fel_to_mel.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn fel_to_profile8() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/fel_to_81.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 8);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn mel_conversions() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/mel_orig.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let mut parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);

    // MEL to MEL
    let (mel_data, mel_rpu) = _parse_file(PathBuf::from("./assets/mel_to_mel.bin"));
    assert_eq!(mel_rpu.dovi_profile, 7);

    dovi_rpu.convert_with_mode(1);
    parsed_data = dovi_rpu.write_rpu_data();
    assert_eq!(&mel_data, &parsed_data);

    // MEL to 8.1
    let (p81_data, p81_rpu) = _parse_file(PathBuf::from("./assets/mel_to_81.bin"));
    assert_eq!(p81_rpu.dovi_profile, 8);

    dovi_rpu.convert_with_mode(2);
    parsed_data = dovi_rpu.write_rpu_data();
    assert_eq!(&p81_data, &parsed_data);
}

#[test]
fn data_before_crc32() {
    let (original_data, mut dovi_rpu) =
        _parse_file(PathBuf::from("./assets/data_before_crc32.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn fix_se_write() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/fix_se_write.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn eof_rpu() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/eof_rpu.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn poly_coef_int_logic_rpu() {
    let (original_data, mut dovi_rpu) =
        _parse_file(PathBuf::from("./assets/poly_coef_int_logic.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn sets_offsets_to_zero() {
    let (_original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/fel_orig.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);

    dovi_rpu.crop();
    let parsed_data = dovi_rpu.write_rpu_data();

    let mut dovi_rpu = parse_dovi_rpu(&parsed_data).unwrap();
    if let Some(block) = super::vdr_dm_data::ExtMetadataBlockLevel5::get_mut(&mut dovi_rpu) {
        assert_eq!(vec![0, 0, 0, 0], block._get_offsets());
    }
}

#[test]
fn profile8_001_end_crc32() {
    use crate::dovi::parse_rpu_file;

    let rpus = parse_rpu_file(&PathBuf::from("./assets/p8_001_end_crc32.bin"));
    assert!(rpus.is_some());

    let rpus = rpus.unwrap();
    assert_eq!(rpus.len(), 3);

    let dovi_rpu = &rpus[0];
    assert_eq!(8, dovi_rpu.dovi_profile);
    assert_eq!([216, 0, 0, 1], dovi_rpu.rpu_data_crc32.to_be_bytes());
}
