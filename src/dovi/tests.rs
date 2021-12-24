use anyhow::Result;
use std::fs::File;
use std::{io::Read, path::PathBuf};

use dolby_vision::rpu::dovi_rpu::DoviRpu;
use dolby_vision::rpu::generate::GenerateConfig;

pub fn _parse_file(input: PathBuf) -> Result<(Vec<u8>, DoviRpu)> {
    let mut f = File::open(input)?;
    let metadata = f.metadata()?;

    let mut original_data = vec![0; metadata.len() as usize];
    f.read_exact(&mut original_data)?;

    let dovi_rpu = DoviRpu::parse_unspec62_nalu(&original_data)?;

    Ok((original_data, dovi_rpu))
}

fn _debug(data: &[u8]) -> Result<()> {
    use crate::dovi::OUT_NAL_HEADER;
    use std::fs::OpenOptions;
    use std::io::Write;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("test.bin")?;

    file.write_all(OUT_NAL_HEADER)?;
    file.write_all(&data[2..])?;

    file.flush()?;

    Ok(())
}

fn _debug_generate(config: &GenerateConfig) -> Result<()> {
    let path = PathBuf::from("test.bin");
    config.write_rpus(&path)?;

    Ok(())
}

#[test]
fn profile4() -> Result<()> {
    let (original_data, dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/profile4.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 4);
    let parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    Ok(())
}

#[test]
fn profile5() -> Result<()> {
    let (original_data, dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/profile5.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 5);
    let parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    Ok(())
}

#[test]
fn profile8() -> Result<()> {
    let (original_data, dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/profile8.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 8);
    let parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    Ok(())
}

#[test]
fn fel() -> Result<()> {
    let (original_data, dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_rpu.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    Ok(())
}

#[test]
fn mel() -> Result<()> {
    let (original_data, dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/mel_rpu.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    Ok(())
}

#[test]
fn fel_conversions() -> Result<()> {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_orig.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let mut parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    // FEL to MEL
    let (mel_data, mel_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_to_mel.bin"))?;
    assert_eq!(mel_rpu.dovi_profile, 7);

    dovi_rpu.convert_with_mode(1)?;
    parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;
    assert_eq!(&mel_data[4..], &parsed_data[2..]);

    // FEL to 8.1
    let (p81_data, p81_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_to_81.bin"))?;
    assert_eq!(p81_rpu.dovi_profile, 8);

    dovi_rpu.convert_with_mode(2)?;
    parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;
    assert_eq!(&p81_data[4..], &parsed_data[2..]);

    Ok(())
}

#[test]
fn fel_to_mel() -> Result<()> {
    let (original_data, dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_to_mel.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    Ok(())
}

#[test]
fn fel_to_profile8() -> Result<()> {
    let (original_data, dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_to_81.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 8);
    let parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    Ok(())
}

#[test]
fn mel_conversions() -> Result<()> {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/mel_orig.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let mut parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    // MEL to MEL
    let (mel_data, mel_rpu) = _parse_file(PathBuf::from("./assets/tests/mel_to_mel.bin"))?;
    assert_eq!(mel_rpu.dovi_profile, 7);

    dovi_rpu.convert_with_mode(1)?;
    parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;
    assert_eq!(&mel_data[4..], &parsed_data[2..]);

    // MEL to 8.1
    let (p81_data, p81_rpu) = _parse_file(PathBuf::from("./assets/tests/mel_to_81.bin"))?;
    assert_eq!(p81_rpu.dovi_profile, 8);

    dovi_rpu.convert_with_mode(2)?;
    parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;
    assert_eq!(&p81_data[4..], &parsed_data[2..]);

    Ok(())
}

#[test]
fn data_before_crc32() -> Result<()> {
    let (original_data, dovi_rpu) =
        _parse_file(PathBuf::from("./assets/tests/data_before_crc32.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    Ok(())
}

#[test]
fn fix_se_write() -> Result<()> {
    let (original_data, dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/fix_se_write.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    Ok(())
}

#[test]
fn eof_rpu() -> Result<()> {
    let (original_data, dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/eof_rpu.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    Ok(())
}

#[test]
fn poly_coef_int_logic_rpu() -> Result<()> {
    let (original_data, dovi_rpu) =
        _parse_file(PathBuf::from("./assets/tests/poly_coef_int_logic.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    Ok(())
}

#[test]
fn sets_offsets_to_zero() -> Result<()> {
    use dolby_vision::rpu::extension_metadata::blocks::ExtMetadataBlock;

    let (_original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/fel_orig.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);

    dovi_rpu.crop()?;
    let parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    let dovi_rpu = DoviRpu::parse_unspec62_nalu(&parsed_data)?;
    if let Some(vdr_dm_data) = dovi_rpu.vdr_dm_data {
        let block = vdr_dm_data.get_block(5);

        assert!(block.is_some());

        if let Some(ExtMetadataBlock::Level5(b)) = block {
            assert_eq!(vec![0, 0, 0, 0], b.get_offsets_vec());
        }
    } else {
        panic!("No DM metadata");
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
    use dolby_vision::rpu::extension_metadata::blocks::*;
    use dolby_vision::rpu::generate::GenerateConfig;

    let config = GenerateConfig {
        length: 1000,
        target_nits: Some(600),
        source_min_pq: None,
        source_max_pq: None,
        level5: ExtMetadataBlockLevel5::from_offsets(0, 0, 280, 280),
        level6: ExtMetadataBlockLevel6 {
            max_display_mastering_luminance: 1000,
            min_display_mastering_luminance: 1,
            max_content_light_level: 1000,
            max_frame_average_light_level: 400,
        },
        ..Default::default()
    };

    let rpu = DoviRpu::profile81_config(&config)?;

    let encoded_rpu = rpu.write_rpu()?;

    let vdr_dm_data = rpu.vdr_dm_data.unwrap();
    assert_eq!(vdr_dm_data.source_min_pq, 7);
    assert_eq!(vdr_dm_data.source_max_pq, 3079);

    let l2_meta = vdr_dm_data.get_block(2).unwrap();
    if let ExtMetadataBlock::Level2(b) = l2_meta {
        assert_eq!(b.target_max_pq, 2851);
    }

    let reparsed_rpu = DoviRpu::parse_rpu(&encoded_rpu);
    assert!(reparsed_rpu.is_ok());

    Ok(())
}

#[test]
fn p8_to_mel() -> Result<()> {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/mel_orig.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 7);
    let mut parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    // MEL to 8.1
    let (p81_data, p81_rpu) = _parse_file(PathBuf::from("./assets/tests/mel_to_81.bin"))?;
    assert_eq!(p81_rpu.dovi_profile, 8);

    dovi_rpu.convert_with_mode(2)?;
    parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;
    assert_eq!(&p81_data[4..], &parsed_data[2..]);

    assert_eq!(dovi_rpu.dovi_profile, 8);

    // 8.1 to MEL
    dovi_rpu.convert_with_mode(1)?;
    parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;
    assert_eq!(&original_data[4..], &parsed_data[2..]);

    assert_eq!(dovi_rpu.dovi_profile, 7);

    Ok(())
}

#[test]
fn profile5_to_p81() -> Result<()> {
    let (original_data, mut dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/profile5.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 5);
    let mut parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    // Profile 5 to 8.1
    let (p81_data, p81_rpu) = _parse_file(PathBuf::from("./assets/tests/profile8.bin"))?;
    assert_eq!(p81_rpu.dovi_profile, 8);

    dovi_rpu.convert_with_mode(3)?;
    parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;
    assert_eq!(&p81_data[4..], &parsed_data[2..]);

    assert_eq!(dovi_rpu.dovi_profile, 8);

    Ok(())
}

#[test]
fn profile5_to_p81_2() -> Result<()> {
    let (original_data, mut dovi_rpu) =
        _parse_file(PathBuf::from("./assets/tests/profile5-02.bin"))?;
    assert_eq!(dovi_rpu.dovi_profile, 5);
    let mut parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    // Profile 5 to 8.1
    let (p81_data, p81_rpu) = _parse_file(PathBuf::from(
        "./assets/tests/profile8_from_profile5-02.bin",
    ))?;
    assert_eq!(p81_rpu.dovi_profile, 8);

    dovi_rpu.convert_with_mode(3)?;
    parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;
    assert_eq!(&p81_data[4..], &parsed_data[2..]);

    assert_eq!(dovi_rpu.dovi_profile, 8);

    Ok(())
}

#[test]
fn st2094_10_level3_invalid() -> Result<()> {
    let res = _parse_file(PathBuf::from("./assets/tests/st2094_10_level3.bin"));
    assert!(res.is_err());

    Ok(())
}

#[test]
fn cmv40_full_rpu() -> Result<()> {
    use dolby_vision::rpu::extension_metadata::blocks::*;
    use dolby_vision::rpu::generate::GenerateConfig;
    use dolby_vision::rpu::generate::VideoShot;

    let mut config = GenerateConfig {
        length: 10,
        target_nits: Some(600),
        source_min_pq: None,
        source_max_pq: None,
        level5: ExtMetadataBlockLevel5::from_offsets(0, 0, 280, 280),
        level6: ExtMetadataBlockLevel6 {
            max_display_mastering_luminance: 1000,
            min_display_mastering_luminance: 1,
            max_content_light_level: 1000,
            max_frame_average_light_level: 400,
        },
        ..Default::default()
    };

    // Single shot with L3, L4, L8, L9 and L11 metadata
    config.shots.push(VideoShot {
        start: 0,
        duration: 10,
        metadata_blocks: vec![
            ExtMetadataBlock::Level1(ExtMetadataBlockLevel1 {
                min_pq: 0,
                max_pq: 2081,
                avg_pq: 819,
            }),
            ExtMetadataBlock::Level3(ExtMetadataBlockLevel3 {
                min_pq_offset: 2048,
                max_pq_offset: 2048,
                avg_pq_offset: 2048,
            }),
            ExtMetadataBlock::Level4(ExtMetadataBlockLevel4 {
                anchor_pq: 0,
                anchor_power: 0,
            }),
            ExtMetadataBlock::Level8(ExtMetadataBlockLevel8 {
                target_display_index: 255,
                ..Default::default()
            }),
            ExtMetadataBlock::Level9(ExtMetadataBlockLevel9 {
                source_primary_index: 0,
            }),
            ExtMetadataBlock::Level10(ExtMetadataBlockLevel10 {
                target_display_index: 255,
                target_max_pq: 3000,
                target_min_pq: 0,
                target_primary_index: 2,
            }),
            ExtMetadataBlock::Level11(ExtMetadataBlockLevel11::default_reference_cinema()),
        ],
        ..Default::default()
    });

    let mut rpus = config.generate_rpu_list()?;
    assert_eq!(rpus.len(), config.length);

    let encoded_rpus = GenerateConfig::encode_rpus(&mut rpus);
    assert_eq!(encoded_rpus.len(), config.length);

    let vdr_dm_data = rpus[0].vdr_dm_data.as_ref().unwrap();
    assert_eq!(vdr_dm_data.source_min_pq, 7);
    assert_eq!(vdr_dm_data.source_max_pq, 3079);

    let l2_meta = vdr_dm_data.get_block(2).unwrap();
    if let ExtMetadataBlock::Level2(b) = l2_meta {
        assert_eq!(b.target_max_pq, 2851);
    }

    let reparsed_rpus = DoviRpu::parse_list_of_unspec62_nalus(&encoded_rpus);
    assert_eq!(reparsed_rpus.len(), config.length);

    Ok(())
}

#[test]
fn profile8_unordered_l8_blocks() -> Result<()> {
    let (original_data, dovi_rpu) = _parse_file(PathBuf::from("./assets/tests/unordered_l8_blocks.bin"))?;
    assert!(!dovi_rpu.modified);
    assert_eq!(dovi_rpu.dovi_profile, 8);

    let parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&original_data[4..], &parsed_data[2..]);

    let reparsed_rpu = DoviRpu::parse_unspec62_nalu(&parsed_data)?;
    assert!(!reparsed_rpu.modified);
    assert_eq!(reparsed_rpu.dovi_profile, 8);

    assert_eq!(dovi_rpu.rpu_data_crc32, reparsed_rpu.rpu_data_crc32);

    Ok(())
}
