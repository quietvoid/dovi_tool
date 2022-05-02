use std::path::Path;

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

const SUBCOMMAND: &str = "mux";

#[test]
fn help() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg(SUBCOMMAND).arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "dovi_tool mux [OPTIONS] --bl <bl> --el <el>",
        ));
    Ok(())
}

#[test]
fn mux() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_bl = Path::new("assets/hevc_tests/regular_bl_start_code_4.hevc");
    let input_el = Path::new("assets/hevc_tests/regular.hevc");

    let output_file = temp.child("BL_EL_RPU.hevc");
    let expected_bl_el_rpu = Path::new("assets/hevc_tests/regular_start_code_4_muxed_el.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--bl")
        .arg(input_bl)
        .arg("--el")
        .arg(input_el)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_bl_el_rpu));

    Ok(())
}

#[test]
fn discard() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_bl = Path::new("assets/hevc_tests/regular_bl_start_code_4.hevc");
    let input_el = Path::new("assets/hevc_tests/regular.hevc");

    let output_file = temp.child("BL_EL_RPU.hevc");
    let expected_bl_el_rpu = Path::new("assets/hevc_tests/regular_start_code_4.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--bl")
        .arg(input_bl)
        .arg("--el")
        .arg(input_el)
        .arg("--output")
        .arg(output_file.as_ref())
        .arg("--discard")
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_bl_el_rpu));

    Ok(())
}

#[test]
fn eos_before_el() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_bl = Path::new("assets/hevc_tests/regular_bl_start_code_4.hevc");
    let input_el = Path::new("assets/hevc_tests/regular.hevc");

    let output_file = temp.child("BL_EL_RPU.hevc");
    let expected_bl_el_rpu = Path::new("assets/hevc_tests/yusesope_regular_muxed.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--bl")
        .arg(input_bl)
        .arg("--el")
        .arg(input_el)
        .arg("--output")
        .arg(output_file.as_ref())
        .arg("--eos-before-el")
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_bl_el_rpu));

    Ok(())
}

#[test]
fn no_add_aud() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_bl = Path::new("assets/hevc_tests/no_aud_bl.hevc");
    let input_el = Path::new("assets/hevc_tests/no_aud_injected.hevc");

    let output_file = temp.child("BL_EL_RPU.hevc");
    let expected_bl_el_rpu = Path::new("assets/hevc_tests/no_aud_muxed.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--bl")
        .arg(input_bl)
        .arg("--el")
        .arg(input_el)
        .arg("--output")
        .arg(output_file.as_ref())
        .arg("--no-add-aud")
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_bl_el_rpu));

    Ok(())
}
