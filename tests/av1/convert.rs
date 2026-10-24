use std::path::Path;

use anyhow::Result;
use assert_cmd::cargo;
use assert_fs::prelude::*;
use predicates::prelude::*;

const SUBCOMMAND: &str = "convert";

/// Without a mode, the RPUs are left as they are
#[test]
fn convert_untouched() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/av1_tests/regular.av1");
    let output_file = temp.child("BL_RPU.av1");

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

/// The RPUs of the converted stream are those extract-rpu gives in the same mode
#[test]
fn convert_mel() -> Result<()> {
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/av1_tests/regular.av1");
    let output_file = temp.child("BL_RPU.av1");
    let output_rpu = temp.child("RPU.bin");

    let mut cmd = cargo::cargo_bin_cmd!();
    cmd.arg("--mode")
        .arg("1")
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert()
        .success()
        .stderr(predicate::str::is_empty());

    let mut cmd = cargo::cargo_bin_cmd!();
    cmd.arg("extract-rpu")
        .arg(output_file.as_ref())
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert()
        .success();

    output_rpu.assert(predicate::path::eq_file(Path::new(
        "assets/hevc_tests/regular_rpu_mel.bin",
    )));

    Ok(())
}
