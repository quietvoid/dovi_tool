use std::path::Path;

use anyhow::Result;
use assert_cmd::cargo;
use assert_fs::prelude::*;
use predicates::prelude::*;

const SUBCOMMAND: &str = "extract-rpu";

fn extract(input_file: &Path) -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

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
fn extract_rpu_raw() -> Result<()> {
    extract(Path::new("assets/av1_tests/regular.av1"))
}

#[test]
fn extract_rpu_ivf() -> Result<()> {
    extract(Path::new("assets/av1_tests/regular.ivf"))
}

#[test]
fn extract_rpu_mkv() -> Result<()> {
    extract(Path::new("assets/av1_tests/regular.mkv"))
}

#[test]
fn extract_rpu_stdin() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

    let expected_rpu = Path::new("assets/hevc_tests/regular_rpu.bin");
    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("-")
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .pipe_stdin("assets/av1_tests/regular.av1")?
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_rpu));

    Ok(())
}

#[test]
fn mode_mel() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/av1_tests/regular.av1");
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

#[test]
fn limit() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/av1_tests/regular.mkv");
    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .arg("--limit")
        .arg("10")
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    let rpus = dolby_vision::rpu::utils::parse_rpu_file(output_rpu.path())?;
    assert_eq!(rpus.len(), 10);

    Ok(())
}

#[test]
fn no_rpu() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/av1_tests/regular_bl.av1");
    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert
        .failure()
        .stderr(predicate::str::contains("No RPU was found in input file"));

    output_rpu.assert(predicate::path::missing());

    Ok(())
}
