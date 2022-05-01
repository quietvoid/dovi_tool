use std::path::Path;

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use dolby_vision::rpu::extension_metadata::blocks::ExtMetadataBlock;
use predicates::prelude::*;

const SUBCOMMAND: &str = "demux";

#[test]
fn help() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg(SUBCOMMAND).arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "dovi_tool demux [OPTIONS] [input_pos]",
        ));
    Ok(())
}

#[test]
fn demux() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular_start_code_4_muxed_el.hevc");
    let expected_bl = Path::new("assets/hevc_tests/regular_bl_start_code_4.hevc");
    let expected_el = Path::new("assets/hevc_tests/regular_start_code_4.hevc");

    let output_bl = temp.child("BL.hevc");
    let output_el = temp.child("EL.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--bl-out")
        .arg(output_bl.as_ref())
        .arg("--el-out")
        .arg(output_el.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_bl
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_bl));

    output_el
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_el));

    Ok(())
}

#[test]
fn el_only() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular_start_code_4_muxed_el.hevc");
    let expected_el = Path::new("assets/hevc_tests/regular_start_code_4.hevc");

    let output_bl = temp.child("BL.hevc");
    let output_el = temp.child("EL.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--bl-out")
        .arg(output_bl.as_ref())
        .arg("--el-out")
        .arg(output_el.as_ref())
        .arg("--el-only")
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_bl.assert(predicate::path::missing());

    output_el
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_el));

    Ok(())
}

#[test]
fn mode_lossless_el_only() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular_start_code_4_muxed_el.hevc");
    let expected_el = Path::new("assets/hevc_tests/regular_start_code_4.hevc");

    let output_bl = temp.child("BL.hevc");
    let output_el = temp.child("EL.hevc");

    let assert = cmd
        .arg("--mode")
        .arg("0")
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--bl-out")
        .arg(output_bl.as_ref())
        .arg("--el-out")
        .arg(output_el.as_ref())
        .arg("--el-only")
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_bl.assert(predicate::path::missing());

    output_el
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_el));

    Ok(())
}

/// Edit config with specific active area
#[test]
fn edit_config() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular_start_code_4_muxed_el.hevc");
    let edit_config = Path::new("assets/editor_examples/active_area_all.json");

    let output_bl = temp.child("BL.hevc");
    let output_el = temp.child("EL.hevc");

    let assert = cmd
        .arg("--edit-config")
        .arg(edit_config)
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--bl-out")
        .arg(output_bl.as_ref())
        .arg("--el-out")
        .arg(output_el.as_ref())
        .arg("--el-only")
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_bl.assert(predicate::path::missing());
    output_el.assert(predicate::path::is_file());

    // Extract result
    let output_rpu = temp.child("RPU.bin");
    let assert = Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .arg("extract-rpu")
        .arg(output_el.as_ref())
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
