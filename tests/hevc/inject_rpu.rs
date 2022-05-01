use std::path::Path;

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

const SUBCOMMAND: &str = "inject-rpu";

#[test]
fn help() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg(SUBCOMMAND).arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "dovi_tool inject-rpu [OPTIONS] --rpu-in <RPU_IN> [input_pos]",
        ));
    Ok(())
}

#[test]
fn inject() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular_bl_start_code_4.hevc");
    let input_rpu = Path::new("assets/hevc_tests/regular_rpu.bin");

    let output_file = temp.child("injected_output.hevc");
    let expected_bl_rpu = Path::new("assets/hevc_tests/regular_start_code_4.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--rpu-in")
        .arg(input_rpu)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_bl_rpu));

    Ok(())
}

#[test]
fn inject_aud() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/no_aud_bl.hevc");
    let input_rpu = Path::new("assets/hevc_tests/regular_rpu.bin");

    let output_file = temp.child("injected_output.hevc");
    let expected_bl_rpu = Path::new("assets/hevc_tests/regular_start_code_4.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--rpu-in")
        .arg(input_rpu)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_bl_rpu));

    Ok(())
}

#[test]
fn inject_no_add_aud() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/no_aud_bl.hevc");
    let input_rpu = Path::new("assets/hevc_tests/regular_rpu.bin");

    let output_file = temp.child("injected_output.hevc");
    let expected_bl_rpu = Path::new("assets/hevc_tests/no_aud_injected.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--rpu-in")
        .arg(input_rpu)
        .arg("--output")
        .arg(output_file.as_ref())
        .arg("--no-add-aud")
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_bl_rpu));

    Ok(())
}
