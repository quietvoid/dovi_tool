use std::fs::File;
use std::{io::Read, path::PathBuf};

use anyhow::Result;

use dolby_vision::rpu::dovi_rpu::DoviRpu;

pub fn _parse_file(input: PathBuf, hevc: bool) -> Result<(Vec<u8>, DoviRpu)> {
    let mut f = File::open(input)?;
    let metadata = f.metadata()?;

    let mut original_data = vec![0; metadata.len() as usize];
    f.read_exact(&mut original_data)?;

    let mut cloned_data = original_data.clone();
    let dovi_rpu = if hevc {
        DoviRpu::parse_unspec62_nalu(&original_data)?
    } else {
        DoviRpu::parse_itu_t35_dovi_metadata_obu(cloned_data.as_mut_slice())?
    };

    Ok((original_data, dovi_rpu))
}

#[test]
fn profile5_dolby_sample() -> Result<()> {
    let mut f = File::open("./assets/av1-rpu/p5-01-ref.bin")?;
    let metadata = f.metadata()?;

    let mut ref_data = vec![0; metadata.len() as usize];
    f.read_exact(&mut ref_data)?;

    let (orig_payload, dovi_rpu) = _parse_file(PathBuf::from("./assets/av1-rpu/p5-01.bin"), false)?;

    let rewritten_payload = dovi_rpu.write_av1_rpu_metadata_obu_t35_payload()?;
    assert_eq!(rewritten_payload, orig_payload);

    assert_eq!(dovi_rpu.dovi_profile, 5);
    let parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&ref_data[4..], &parsed_data[2..]);

    Ok(())
}

#[test]
fn profile_84_dolby_sample() -> Result<()> {
    let mut f = File::open("./assets/av1-rpu/p84-01-ref.bin")?;
    let metadata = f.metadata()?;

    let mut ref_data = vec![0; metadata.len() as usize];
    f.read_exact(&mut ref_data)?;

    let (orig_payload, dovi_rpu) =
        _parse_file(PathBuf::from("./assets/av1-rpu/p84-01.bin"), false)?;

    let rewritten_payload = dovi_rpu.write_av1_rpu_metadata_obu_t35_payload()?;
    assert_eq!(rewritten_payload, orig_payload);

    assert_eq!(dovi_rpu.dovi_profile, 8);
    let parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&ref_data[4..], &parsed_data[2..]);

    Ok(())
}

#[test]
fn av1_fel_orig() -> Result<()> {
    let mut f = File::open("./assets/tests/fel_orig.bin")?;
    let metadata = f.metadata()?;

    let mut ref_data = vec![0; metadata.len() as usize];
    f.read_exact(&mut ref_data)?;

    let (orig_payload, dovi_rpu) =
        _parse_file(PathBuf::from("./assets/av1-rpu/fel_orig.bin"), false)?;

    let rewritten_payload = dovi_rpu.write_av1_rpu_metadata_obu_t35_payload()?;
    assert_eq!(rewritten_payload, orig_payload);

    assert_eq!(dovi_rpu.dovi_profile, 7);
    let parsed_data = dovi_rpu.write_hevc_unspec62_nalu()?;

    assert_eq!(&ref_data[4..], &parsed_data[2..]);

    Ok(())
}

#[test]
fn trailing_bytes_rpu() -> Result<()> {
    let (original_data, dovi_rpu) =
        _parse_file(PathBuf::from("./assets/tests/trailing_bytes_rpu.bin"), true)?;
    let original_without_trailing = &original_data[..original_data.len() - 2];
    assert_eq!(original_without_trailing.last().copied().unwrap(), 0x80);

    let av1_payload = dovi_rpu.write_av1_rpu_metadata_obu_t35_payload()?;
    let parsed_rpu = DoviRpu::parse_itu_t35_dovi_metadata_obu(&av1_payload)?;

    let rewritten_data = parsed_rpu.write_hevc_unspec62_nalu()?;
    assert_eq!(&original_without_trailing[4..], &rewritten_data[2..]);

    Ok(())
}
