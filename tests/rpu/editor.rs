use std::path::Path;

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

use dolby_vision::rpu::extension_metadata::blocks::ExtMetadataBlock;

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

    let rpus = utilities_dovi::parse_rpu_file(output_rpu.as_ref())?.unwrap();
    assert_eq!(rpus.len(), 1);

    let rpu = &rpus[0];
    assert_eq!(rpu.dovi_profile, 8);

    Ok(())
}

#[test]
fn convert_to_cmv4() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_rpu = Path::new("assets/tests/fel_orig.bin");
    let edit_config = Path::new("assets/editor_examples/convert_to_cmv4.json");

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

    let rpus = utilities_dovi::parse_rpu_file(output_rpu.as_ref())?.unwrap();
    assert_eq!(rpus.len(), 1);

    let rpu = &rpus[0];
    assert_eq!(rpu.dovi_profile, 7);

    let vdr_dm_data = rpu.vdr_dm_data.as_ref().unwrap();

    // Only L9, L11 and L254
    assert_eq!(vdr_dm_data.metadata_blocks(3).unwrap().len(), 3);

    if let ExtMetadataBlock::Level9(level9) = vdr_dm_data.get_block(9).unwrap() {
        assert_eq!(level9.length, 1);
        assert_eq!(level9.source_primary_index, 0);
    }

    if let ExtMetadataBlock::Level11(level11) = vdr_dm_data.get_block(11).unwrap() {
        assert_eq!(level11.content_type, 1);
        assert_eq!(level11.whitepoint, 0);
        assert!(level11.reference_mode_flag);
    }

    if let ExtMetadataBlock::Level254(level254) = vdr_dm_data.get_block(11).unwrap() {
        assert_eq!(level254.dm_mode, 0);
        assert_eq!(level254.dm_version_index, 2);
    }

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

    let rpus = utilities_dovi::parse_rpu_file(output_rpu.as_ref())?.unwrap();
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
