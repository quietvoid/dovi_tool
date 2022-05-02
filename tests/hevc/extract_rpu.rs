use std::path::Path;

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

use dolby_vision::rpu::extension_metadata::blocks::ExtMetadataBlock;

const SUBCOMMAND: &str = "extract-rpu";

#[test]
fn help() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg(SUBCOMMAND).arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "dovi_tool extract-rpu [OPTIONS] [input_pos]",
        ));
    Ok(())
}

#[test]
fn extract_rpu() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular.hevc");
    let expected_rpu = Path::new("assets/hevc_tests/regular_rpu.bin");

    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
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
fn mode_mel() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular.hevc");
    let expected_rpu = Path::new("assets/hevc_tests/regular_rpu_mel.bin");

    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg("--mode")
        .arg("1")
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_rpu));

    Ok(())
}

/// Edit config with specific active area
#[test]
fn edit_config() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular.hevc");
    let edit_config = Path::new("assets/editor_examples/active_area_all.json");

    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg("--edit-config")
        .arg(edit_config)
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu.assert(predicate::path::is_file());

    let rpus = utilities_dovi::parse_rpu_file(output_rpu.as_ref())?.unwrap();
    assert_eq!(rpus.len(), 259);

    rpus.iter().for_each(|rpu| {
        let block = rpu.vdr_dm_data.as_ref().unwrap().get_block(5).unwrap();
        if let ExtMetadataBlock::Level5(b) = block {
            assert_eq!(vec![0, 0, 210, 210], b.get_offsets_vec());
        }
    });

    Ok(())
}
