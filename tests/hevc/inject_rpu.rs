use std::path::Path;

use anyhow::Result;
use assert_cmd::cargo;
use assert_fs::prelude::*;
use predicates::prelude::*;

const SUBCOMMAND: &str = "inject-rpu";

#[test]
fn help() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
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
    let mut cmd = cargo::cargo_bin_cmd!();
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
    let mut cmd = cargo::cargo_bin_cmd!();
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
    let mut cmd = cargo::cargo_bin_cmd!();
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

#[test]
fn annexb() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular_bl_start_code_4.hevc");
    let input_rpu = Path::new("assets/hevc_tests/regular_rpu.bin");

    let output_file = temp.child("injected_output.hevc");
    let expected_bl_rpu = Path::new("assets/hevc_tests/regular_inject_annexb.hevc");

    let assert = cmd
        .arg("--start-code")
        .arg("annex-b")
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
fn duplicated_end() -> Result<()> {
    // Generate shorter RPU
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

    let generate_config = Path::new("assets/generator_examples/default_cmv29.json");
    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg("generate")
        .arg("--json")
        .arg(generate_config)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu.assert(predicate::path::is_file());

    // Inject and expect to duplicate
    let mut cmd = cargo::cargo_bin_cmd!();

    let input_bl = Path::new("assets/hevc_tests/regular_bl_start_code_4.hevc");
    let output_file = temp.child("injected_output.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_bl)
        .arg("--rpu-in")
        .arg(output_rpu.as_ref())
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "Metadata will be duplicated at the end to match video length",
        ));

    output_file.assert(predicate::path::is_file());

    // Extract result and verify count
    let mut cmd = cargo::cargo_bin_cmd!();

    let output_rpu = temp.child("RPU_duplicated.bin");

    let assert = cmd
        .arg("extract-rpu")
        .arg(output_file.as_ref())
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu.assert(predicate::path::is_file());

    let rpus = dolby_vision::rpu::utils::parse_rpu_file(output_rpu.as_ref())?;
    assert_eq!(rpus.len(), 259);

    Ok(())
}

// Makes sure the injected RPU NALU is placed after SEI_SUFFIX NALUs
#[test]
fn sei_suffix_before_rpu() -> Result<()> {
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/sei-suffix-muxed-rpu.hevc");

    // Demux
    let output_bl = temp.child("BL.hevc");
    let output_el = temp.child("EL.hevc");

    let mut cmd = cargo::cargo_bin_cmd!();
    let assert = cmd
        .arg("demux")
        .arg(input_file)
        .arg("--bl-out")
        .arg(output_bl.as_ref())
        .arg("--el-out")
        .arg(output_el.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_bl.assert(predicate::path::is_file());
    output_el.assert(predicate::path::is_file());

    // Extract RPU
    let output_rpu = temp.child("RPU.bin");

    let mut cmd = cargo::cargo_bin_cmd!();
    let assert = cmd
        .arg("extract-rpu")
        .arg(input_file)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());
    output_rpu.assert(predicate::path::is_file());

    // Reinject
    let output_file = temp.child("injected_output.hevc");

    let mut cmd = cargo::cargo_bin_cmd!();
    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(output_bl.as_ref())
        .arg("--rpu-in")
        .arg(output_rpu.as_ref())
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(input_file));

    Ok(())
}
