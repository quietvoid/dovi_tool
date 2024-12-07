use std::path::Path;

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

use dolby_vision::rpu::extension_metadata::{blocks::ExtMetadataBlock, MasteringDisplayPrimaries};

const SUBCOMMAND: &str = "editor";

#[test]
fn help() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg(SUBCOMMAND).arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "dovi_tool editor [OPTIONS] --json <json> [input_pos]",
        ));
    Ok(())
}

#[test]
fn mode() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_rpu = Path::new("assets/tests/fel_orig.bin");
    let edit_config = Path::new("assets/editor_examples/mode.json");

    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_rpu)
        .arg("--json")
        .arg(edit_config)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu.assert(predicate::path::is_file());

    let rpus = dolby_vision::rpu::utils::parse_rpu_file(output_rpu.as_ref())?;
    assert_eq!(rpus.len(), 1);

    let rpu = &rpus[0];
    assert_eq!(rpu.dovi_profile, 8);

    Ok(())
}

#[test]
fn remove_cmv4() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_rpu = Path::new("assets/tests/mel_variable_l8_length13.bin");
    let edit_config = Path::new("assets/editor_examples/remove_cmv4.json");

    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_rpu)
        .arg("--json")
        .arg(edit_config)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu.assert(predicate::path::is_file());

    // Original
    let rpus = dolby_vision::rpu::utils::parse_rpu_file(input_rpu)?;
    assert_eq!(rpus.len(), 1);

    let rpu = &rpus[0];
    assert_eq!(rpu.dovi_profile, 7);

    let vdr_dm_data = rpu.vdr_dm_data.as_ref().unwrap();
    assert!(vdr_dm_data.cmv40_metadata.is_some());

    // Removed RPU
    let rpus = dolby_vision::rpu::utils::parse_rpu_file(output_rpu.as_ref())?;
    assert_eq!(rpus.len(), 1);

    let rpu = &rpus[0];
    assert_eq!(rpu.dovi_profile, 7);

    let vdr_dm_data = rpu.vdr_dm_data.as_ref().unwrap();
    assert!(vdr_dm_data.cmv40_metadata.is_none());

    Ok(())
}

#[test]
fn active_area_specific() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_rpu = Path::new("assets/hevc_tests/regular_rpu.bin");
    let edit_config = Path::new("assets/editor_examples/active_area.json");

    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_rpu)
        .arg("--json")
        .arg(edit_config)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu.assert(predicate::path::is_file());

    let rpus = dolby_vision::rpu::utils::parse_rpu_file(output_rpu.as_ref())?;
    assert_eq!(rpus.len(), 259);

    let start_rpu = &rpus[0];
    let end_rpu = &rpus[40];
    let last_rpu = &rpus.last().unwrap();

    let block = start_rpu
        .vdr_dm_data
        .as_ref()
        .unwrap()
        .get_block(5)
        .unwrap();
    if let ExtMetadataBlock::Level5(b) = block {
        assert_eq!(vec![0, 0, 210, 210], b.get_offsets_vec());
    }

    let block = end_rpu.vdr_dm_data.as_ref().unwrap().get_block(5).unwrap();
    if let ExtMetadataBlock::Level5(b) = block {
        assert_eq!(vec![0, 0, 210, 210], b.get_offsets_vec());
    }

    let block = last_rpu.vdr_dm_data.as_ref().unwrap().get_block(5).unwrap();
    if let ExtMetadataBlock::Level5(b) = block {
        assert_eq!(vec![0, 0, 0, 0], b.get_offsets_vec());
    }

    Ok(())
}

#[test]
fn add_l9_l11_no_effect() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_rpu = Path::new("assets/tests/fel_orig.bin");
    let edit_config = Path::new("assets/editor_examples/l9_and_l11.json");

    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_rpu)
        .arg("--json")
        .arg(edit_config)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu.assert(predicate::path::is_file());

    // Original
    let rpus = dolby_vision::rpu::utils::parse_rpu_file(input_rpu)?;
    assert_eq!(rpus.len(), 1);

    let rpu = &rpus[0];
    assert_eq!(rpu.dovi_profile, 7);

    let vdr_dm_data = rpu.vdr_dm_data.as_ref().unwrap();
    assert!(vdr_dm_data.cmv40_metadata.is_none());

    // No change
    let rpus = dolby_vision::rpu::utils::parse_rpu_file(output_rpu.as_ref())?;
    assert_eq!(rpus.len(), 1);

    let rpu = &rpus[0];
    assert_eq!(rpu.dovi_profile, 7);

    let vdr_dm_data = rpu.vdr_dm_data.as_ref().unwrap();
    assert!(vdr_dm_data.cmv40_metadata.is_none());

    Ok(())
}

#[test]
fn add_l9_l11() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_rpu = Path::new("assets/tests/mel_variable_l8_length13.bin");
    let edit_config = Path::new("assets/editor_examples/l9_and_l11.json");

    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_rpu)
        .arg("--json")
        .arg(edit_config)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu.assert(predicate::path::is_file());

    // Original
    let rpus = dolby_vision::rpu::utils::parse_rpu_file(input_rpu)?;
    assert_eq!(rpus.len(), 1);

    let rpu = &rpus[0];
    assert_eq!(rpu.dovi_profile, 7);

    let vdr_dm_data = rpu.vdr_dm_data.as_ref().unwrap();
    assert!(vdr_dm_data.cmv40_metadata.is_some());

    let orig_l9 = vdr_dm_data.get_block(9).unwrap();
    let orig_l11 = vdr_dm_data.get_block(11);

    if let ExtMetadataBlock::Level9(block) = orig_l9 {
        assert_eq!(
            block.source_primary_index,
            MasteringDisplayPrimaries::DCIP3D65 as u8
        )
    }
    assert!(orig_l11.is_none());

    // Modifies the blocks
    let rpus = dolby_vision::rpu::utils::parse_rpu_file(output_rpu.as_ref())?;
    assert_eq!(rpus.len(), 1);

    let rpu = &rpus[0];
    assert_eq!(rpu.dovi_profile, 7);

    let vdr_dm_data = rpu.vdr_dm_data.as_ref().unwrap();
    assert!(vdr_dm_data.cmv40_metadata.is_some());

    let orig_l9 = vdr_dm_data.get_block(9).unwrap();
    let orig_l11 = vdr_dm_data.get_block(11).unwrap();

    if let ExtMetadataBlock::Level9(block) = orig_l9 {
        assert_eq!(
            block.source_primary_index,
            MasteringDisplayPrimaries::BT2020 as u8
        );
    }
    if let ExtMetadataBlock::Level11(block) = orig_l11 {
        assert_eq!(block.content_type, 1);
        assert!(block.reference_mode_flag);
    }

    Ok(())
}

#[test]
fn source_rpu() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_rpu = Path::new("assets/tests/fel_orig.bin");
    let edit_config = Path::new("assets/editor_examples/source_rpu.json");

    let output_rpu = temp.child("RPU.bin");
    let expected_rpu = Path::new("assets/tests/source_rpu_replaced_fel_orig.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_rpu)
        .arg("--json")
        .arg(edit_config)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_rpu));

    Ok(())
}

#[test]
fn duplicate() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_rpu = Path::new("assets/tests/cmv4_2_510_xml_rpu.bin");

    let edit_config = temp.child("duplicate.json");
    let cfg_file = std::fs::File::create(&edit_config)?;
    serde_json::to_writer(
        cfg_file,
        &serde_json::json!({
            "duplicate": [
                {
                    "source": 0,
                    "offset": 24,
                    "length": 1
                }
            ]
        }),
    )?;

    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_rpu)
        .arg("--json")
        .arg(edit_config.as_ref())
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());
    output_rpu.assert(predicate::path::is_file());

    let rpus = dolby_vision::rpu::utils::parse_rpu_file(output_rpu)?;
    assert_eq!(rpus.len(), 25);

    // Duplicated and appended
    assert_eq!(rpus[0].rpu_data_crc32, rpus[24].rpu_data_crc32);

    Ok(())
}
