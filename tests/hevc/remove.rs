use std::path::Path;

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

const SUBCOMMAND: &str = "remove";

#[test]
fn help() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg(SUBCOMMAND).arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "dovi_tool remove [OPTIONS] [input_pos]",
        ));
    Ok(())
}

#[test]
fn remove() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular_start_code_4_muxed_el.hevc");
    let expected_bl = Path::new("assets/hevc_tests/regular_bl_start_code_4.hevc");

    let output_bl = temp.child("BL.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--output")
        .arg(output_bl.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_bl
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_bl));

    Ok(())
}

#[test]
fn annexb() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular_start_code_4_muxed_el.hevc");
    let expected_bl = Path::new("assets/hevc_tests/regular_demux_bl_annexb.hevc");

    let output_bl = temp.child("BL.hevc");

    let assert = cmd
        .arg("--start-code")
        .arg("annex-b")
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--output")
        .arg(output_bl.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_bl
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_bl));

    Ok(())
}
