use std::fs::File;
use std::{io::Read, path::PathBuf};

use super::{parse_dovi_rpu, DoviRpu};

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
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/profile4.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 4);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn profile5() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/profile5.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 5);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn profile8() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/profile8.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 8);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn fel() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_rpu.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn mel() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/mel_rpu.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn fel_conversions() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_orig.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let mut parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);

    // FEL to MEL
    let (mel_data, mel_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_to_mel.bin"));
    assert_eq!(mel_rpu.dovi_profile, 7);

    dovi_rpu.convert_with_mode(1);
    parsed_data = dovi_rpu.write_rpu_data();
    assert_eq!(&mel_data, &parsed_data);

    // FEL to 8.1
    let (p81_data, p81_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_to_81.bin"));
    assert_eq!(p81_rpu.dovi_profile, 8);

    dovi_rpu.convert_with_mode(2);
    parsed_data = dovi_rpu.write_rpu_data();
    assert_eq!(&p81_data, &parsed_data);
}

#[test]
fn fel_to_mel() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_to_mel.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn fel_to_profile8() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_to_81.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 8);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn mel_conversions() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/mel_orig.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let mut parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);

    // MEL to MEL
    let (mel_data, mel_rpu) = _parse_file(PathBuf::from("./assets/tests/mel_to_mel.bin"));
    assert_eq!(mel_rpu.dovi_profile, 7);

    dovi_rpu.convert_with_mode(1);
    parsed_data = dovi_rpu.write_rpu_data();
    assert_eq!(&mel_data, &parsed_data);

    // MEL to 8.1
    let (p81_data, p81_rpu) = _parse_file(PathBuf::from("./assets/tests/mel_to_81.bin"));
    assert_eq!(p81_rpu.dovi_profile, 8);

    dovi_rpu.convert_with_mode(2);
    parsed_data = dovi_rpu.write_rpu_data();
    assert_eq!(&p81_data, &parsed_data);
}

#[test]
fn data_before_crc32() {
    let (original_data, mut dovi_rpu) =
        _parse_file(PathBuf::from("./assets/tests/data_before_crc32.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn fix_se_write() {
    let (original_data, mut dovi_rpu) =
        _parse_file(PathBuf::from("./assets/tests/fix_se_write.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn eof_rpu() {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/eof_rpu.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn poly_coef_int_logic_rpu() {
    let (original_data, mut dovi_rpu) =
        _parse_file(PathBuf::from("./assets/tests/poly_coef_int_logic.bin"));
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data();

    assert_eq!(&original_data, &parsed_data);
}

#[test]
fn sets_offsets_to_zero() {
    let (_original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_orig.bin"));
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

    let rpus = parse_rpu_file(&PathBuf::from("./assets/tests/p8_001_end_crc32.bin"));
    assert!(rpus.is_some());

    let rpus = rpus.unwrap();
    assert_eq!(rpus.len(), 3);

    let dovi_rpu = &rpus[0];
    assert_eq!(8, dovi_rpu.dovi_profile);
    assert_eq!([216, 0, 0, 1], dovi_rpu.rpu_data_crc32.to_be_bytes());
}

#[test]
fn generated_rpu() {
    use crate::dovi::generator::{GenerateConfig, Level6Metadata};
    use crate::dovi::rpu::rpu_data_header::RpuDataHeader;
    use crate::dovi::rpu::vdr_dm_data::{ExtMetadataBlock, VdrDmData};
    use crate::dovi::rpu::vdr_rpu_data::VdrRpuData;

    let config = GenerateConfig {
        length: 1000,
        target_nits: 600,
        source_min_pq: None,
        source_max_pq: None,
        level5: None,
        level6: Some(Level6Metadata {
            max_display_mastering_luminance: 1000,
            min_display_mastering_luminance: 1,
            max_content_light_level: 1000,
            max_frame_average_light_level: 400,
        }),
    };

    let vdr_dm_data = VdrDmData::from_config(&config);
    assert_eq!(vdr_dm_data.source_min_pq, 7);
    assert_eq!(vdr_dm_data.source_max_pq, 3079);

    let level2_index = vdr_dm_data
        .ext_metadata_blocks
        .iter()
        .position(|e| match e {
            ExtMetadataBlock::Level2(_) => true,
            _ => false,
        });

    assert!(level2_index.is_some());
    let l2_meta = &vdr_dm_data.ext_metadata_blocks[level2_index.unwrap()];

    if let ExtMetadataBlock::Level2(b) = l2_meta {
        assert_eq!(b.target_max_pq, 2851);
    }

    let mut rpu = DoviRpu {
        dovi_profile: 8,
        modified: true,
        header: RpuDataHeader::p8_default(),
        vdr_rpu_data: Some(VdrRpuData::p8_default()),
        nlq_data: None,
        vdr_dm_data: Some(vdr_dm_data),
        last_byte: 80,
        ..Default::default()
    };

    let encoded_rpu = rpu.write_rpu_data();

    let reparsed_rpu = parse_dovi_rpu(&encoded_rpu[2..&encoded_rpu.len() - 1]);
    assert!(reparsed_rpu.is_ok());
}
