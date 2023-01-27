use std::path::Path;

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

use dolby_vision::rpu::extension_metadata::blocks::ExtMetadataBlock;

const SUBCOMMAND: &str = "convert";

#[test]
fn help() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg(SUBCOMMAND).arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "dovi_tool convert [OPTIONS] [input_pos]",
        ));
    Ok(())
}

#[test]
fn copy() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular_start_code_4.hevc");
    let output_file = temp.child("BL_EL_RPU.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(input_file));

    Ok(())
}

#[test]
fn copy_discard() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular_start_code_4_muxed_el.hevc");
    let non_el_file = Path::new("assets/hevc_tests/regular_start_code_4.hevc");

    let output_file = temp.child("BL_EL_RPU.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--discard")
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(non_el_file));

    Ok(())
}

#[test]
fn mode_lossless_discard() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular_start_code_4_muxed_el.hevc");
    let non_el_file = Path::new("assets/hevc_tests/regular_start_code_4.hevc");

    let output_file = temp.child("BL_EL_RPU.hevc");

    let assert = cmd
        .arg("--mode")
        .arg("0")
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--discard")
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(non_el_file));

    Ok(())
}

/// Edit config with specific active area
#[test]
fn edit_config() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular_start_code_4.hevc");
    let edit_config = Path::new("assets/editor_examples/active_area_all.json");

    let output_file = temp.child("BL_EL_RPU.hevc");

    let assert = cmd
        .arg("--edit-config")
        .arg(edit_config)
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file.assert(predicate::path::is_file());

    // Extract result
    let output_rpu = temp.child("RPU.bin");
    let assert = Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("extract-rpu")
        .arg(output_file.as_ref())
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());
    output_rpu.assert(predicate::path::is_file());

    let rpus = dolby_vision::rpu::utils::parse_rpu_file(output_rpu.as_ref())?;
    assert_eq!(rpus.len(), 259);

    rpus.iter().for_each(|rpu| {
        let block = rpu.vdr_dm_data.as_ref().unwrap().get_block(5).unwrap();
        if let ExtMetadataBlock::Level5(b) = block {
            assert_eq!(vec![0, 0, 210, 210], b.get_offsets_vec());
        }
    });

    Ok(())
}

#[test]
fn annexb() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular.hevc");
    let output_file = temp.child("BL_EL_RPU.hevc");

    let expected_output = Path::new("assets/hevc_tests/regular_convert_annexb.hevc");

    let assert = cmd
        .arg("--start-code")
        .arg("annex-b")
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_output));

    Ok(())
}

#[test]
fn drop_hdr10plus_case() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/sei-double-3byte-case.hevc");

    let output_file = temp.child("converted.hevc");
    let expected_removed = Path::new("assets/hevc_tests/sei-double-3byte-start-code-4.hevc");

    let assert = cmd
        .arg("--drop-hdr10plus")
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_removed));

    Ok(())
}
