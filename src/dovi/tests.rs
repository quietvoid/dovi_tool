use anyhow::Result;
use std::fs::File;
use std::{io::Read, path::PathBuf};

use dolby_vision::rpu::dovi_rpu::DoviRpu;

pub fn _parse_file(input: PathBuf) -> Result<(Vec<u8>, DoviRpu)> {
    let mut f = File::open(input).unwrap();
    let metadata = f.metadata().unwrap();

    let mut original_data = vec![0; metadata.len() as usize];
    f.read_exact(&mut original_data).unwrap();

    let dovi_rpu = DoviRpu::parse(&original_data)?;

    Ok((original_data, dovi_rpu))
}

#[test]
fn profile4() -> Result<()> {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/profile4.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 4);
    let parsed_data = dovi_rpu.write_rpu_data()?;

    assert_eq!(&original_data, &parsed_data);

    Ok(())
}

#[test]
fn profile5() -> Result<()> {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/profile5.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 5);
    let parsed_data = dovi_rpu.write_rpu_data()?;

    assert_eq!(&original_data, &parsed_data);

    Ok(())
}

#[test]
fn profile8() -> Result<()> {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/profile8.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 8);
    let parsed_data = dovi_rpu.write_rpu_data()?;

    assert_eq!(&original_data, &parsed_data);

    Ok(())
}

#[test]
fn fel() -> Result<()> {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_rpu.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data()?;

    assert_eq!(&original_data, &parsed_data);

    Ok(())
}

#[test]
fn mel() -> Result<()> {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/mel_rpu.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data()?;

    assert_eq!(&original_data, &parsed_data);

    Ok(())
}

#[test]
fn fel_conversions() -> Result<()> {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_orig.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let mut parsed_data = dovi_rpu.write_rpu_data()?;

    assert_eq!(&original_data, &parsed_data);

    // FEL to MEL
    let (mel_data, mel_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_to_mel.bin"))?;
    assert_eq!(mel_rpu.dovi_profile, 7);

    dovi_rpu.convert_with_mode(1)?;
    parsed_data = dovi_rpu.write_rpu_data()?;
    assert_eq!(&mel_data, &parsed_data);

    // FEL to 8.1
    let (p81_data, p81_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_to_81.bin"))?;
    assert_eq!(p81_rpu.dovi_profile, 8);

    dovi_rpu.convert_with_mode(2)?;
    parsed_data = dovi_rpu.write_rpu_data()?;
    assert_eq!(&p81_data, &parsed_data);

    Ok(())
}

#[test]
fn fel_to_mel() -> Result<()> {
    let (original_data, mut dovi_rpu) =
        _parse_file(PathBuf::from("./assets/tests/fel_to_mel.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data()?;

    assert_eq!(&original_data, &parsed_data);

    Ok(())
}

#[test]
fn fel_to_profile8() -> Result<()> {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_to_81.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 8);
    let parsed_data = dovi_rpu.write_rpu_data()?;

    assert_eq!(&original_data, &parsed_data);

    Ok(())
}

#[test]
fn mel_conversions() -> Result<()> {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/mel_orig.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let mut parsed_data = dovi_rpu.write_rpu_data()?;

    assert_eq!(&original_data, &parsed_data);

    // MEL to MEL
    let (mel_data, mel_rpu) = _parse_file(PathBuf::from("./assets/tests/mel_to_mel.bin"))?;
    assert_eq!(mel_rpu.dovi_profile, 7);

    dovi_rpu.convert_with_mode(1)?;
    parsed_data = dovi_rpu.write_rpu_data()?;
    assert_eq!(&mel_data, &parsed_data);

    // MEL to 8.1
    let (p81_data, p81_rpu) = _parse_file(PathBuf::from("./assets/tests/mel_to_81.bin"))?;
    assert_eq!(p81_rpu.dovi_profile, 8);

    dovi_rpu.convert_with_mode(2)?;
    parsed_data = dovi_rpu.write_rpu_data()?;
    assert_eq!(&p81_data, &parsed_data);

    Ok(())
}

#[test]
fn data_before_crc32() -> Result<()> {
    let (original_data, mut dovi_rpu) =
        _parse_file(PathBuf::from("./assets/tests/data_before_crc32.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data()?;

    assert_eq!(&original_data, &parsed_data);

    Ok(())
}

#[test]
fn fix_se_write() -> Result<()> {
    let (original_data, mut dovi_rpu) =
        _parse_file(PathBuf::from("./assets/tests/fix_se_write.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data()?;

    assert_eq!(&original_data, &parsed_data);

    Ok(())
}

#[test]
fn eof_rpu() -> Result<()> {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/eof_rpu.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data()?;

    assert_eq!(&original_data, &parsed_data);

    Ok(())
}

#[test]
fn poly_coef_int_logic_rpu() -> Result<()> {
    let (original_data, mut dovi_rpu) =
        _parse_file(PathBuf::from("./assets/tests/poly_coef_int_logic.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_rpu_data()?;

    assert_eq!(&original_data, &parsed_data);

    Ok(())
}

#[test]
fn sets_offsets_to_zero() -> Result<()> {
    let (_original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_orig.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);

    dovi_rpu.crop();
    let parsed_data = dovi_rpu.write_rpu_data()?;

    let mut dovi_rpu = DoviRpu::parse(&parsed_data).unwrap();
    if let Some(block) = dovi_rpu.get_level5_block_mut() {
        assert_eq!(vec![0, 0, 0, 0], block.get_offsets_vec());
    }

    Ok(())
}

#[test]
fn profile8_001_end_crc32() -> Result<()> {
    use crate::dovi::parse_rpu_file;

    let rpus = parse_rpu_file(&PathBuf::from("./assets/tests/p8_001_end_crc32.bin"))?;
    assert!(rpus.is_some());

    let rpus = rpus.unwrap();
    assert_eq!(rpus.len(), 3);

    let dovi_rpu = &rpus[0];
    assert_eq!(8, dovi_rpu.dovi_profile);
    assert_eq!([216, 0, 0, 1], dovi_rpu.rpu_data_crc32.to_be_bytes());

    Ok(())
}

#[test]
fn generated_rpu() -> Result<()> {
    use dolby_vision::rpu::rpu_data_header::RpuDataHeader;
    use dolby_vision::rpu::rpu_data_mapping::VdrRpuData;
    use dolby_vision::rpu::vdr_dm_data::VdrDmData;
    use dolby_vision::st2094_10::generate::{GenerateConfig, Level6Metadata};
    use dolby_vision::st2094_10::ExtMetadataBlock;

    let config = GenerateConfig {
        length: 1000,
        target_nits: Some(600),
        source_min_pq: None,
        source_max_pq: None,
        level5: None,
        level6: Some(Level6Metadata {
            max_display_mastering_luminance: 1000,
            min_display_mastering_luminance: 1,
            max_content_light_level: 1000,
            max_frame_average_light_level: 400,
        }),
        ..Default::default()
    };

    let vdr_dm_data = VdrDmData::from_config(&config);
    assert_eq!(vdr_dm_data.source_min_pq, 7);
    assert_eq!(vdr_dm_data.source_max_pq, 3079);

    let level2_index = vdr_dm_data
        .st2094_10_metadata
        .ext_metadata_blocks
        .iter()
        .position(|e| match e {
            ExtMetadataBlock::Level2(_) => true,
            _ => false,
        });

    assert!(level2_index.is_some());
    let l2_meta = &vdr_dm_data.st2094_10_metadata.ext_metadata_blocks[level2_index.unwrap()];

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
        last_byte: 0x80,
        ..Default::default()
    };

    let encoded_rpu = rpu.write_rpu_data()?;

    let reparsed_rpu = DoviRpu::parse(&encoded_rpu[2..]);
    assert!(reparsed_rpu.is_ok());

    Ok(())
}
