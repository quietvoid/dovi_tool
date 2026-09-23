use std::path::Path;

use anyhow::Result;
use assert_cmd::cargo;
use assert_fs::prelude::*;
use predicates::prelude::*;

const SUBCOMMAND: &str = "remove";

fn remove(input_file: &str, expected: &str) -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

    let output_bl = temp.child("BL");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--output")
        .arg(output_bl.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_bl
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(Path::new(expected)));

    Ok(())
}

#[test]
fn remove_raw() -> Result<()> {
    remove(
        "assets/av1_tests/regular.av1",
        "assets/av1_tests/regular_bl.av1",
    )
}

#[test]
fn remove_ivf() -> Result<()> {
    remove(
        "assets/av1_tests/regular.ivf",
        "assets/av1_tests/regular_bl.ivf",
    )
}

/// Matroska input is written out as a raw AV1 stream
#[test]
fn remove_mkv() -> Result<()> {
    remove(
        "assets/av1_tests/regular.mkv",
        "assets/av1_tests/regular_bl.av1",
    )
}

#[test]
fn remove_stdin() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

    let output_bl = temp.child("BL.av1");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("-")
        .arg("--output")
        .arg(output_bl.as_ref())
        .pipe_stdin("assets/av1_tests/regular.av1")?
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_bl
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(Path::new(
            "assets/av1_tests/regular_bl.av1",
        )));

    Ok(())
}
