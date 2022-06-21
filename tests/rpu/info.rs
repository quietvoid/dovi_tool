use std::path::Path;

use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;

const SUBCOMMAND: &str = "info";

#[test]
fn help() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg(SUBCOMMAND).arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "dovi_tool info [OPTIONS] [input_pos]",
        ));
    Ok(())
}

#[test]
fn summary_p7_mel() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    let input_rpu = Path::new("assets/hevc_tests/regular_rpu_mel.bin");

    let assert = cmd.arg(SUBCOMMAND).arg(input_rpu).arg("--summary").assert();

    assert.success().stderr(predicate::str::is_empty()).stdout(
        predicate::str::contains("Summary:")
            .and(predicate::str::contains("  Frames: 259"))
            .and(predicate::str::contains("  Profile: 7 (MEL)"))
            .and(predicate::str::contains("  DM version: 2 (CM v4.x)"))
            .and(predicate::str::contains("  Scene/shot count: 3")),
    );

    Ok(())
}

#[test]
fn summary_p7_fel() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    let input_rpu = Path::new("assets/tests/fel_orig.bin");

    let assert = cmd.arg(SUBCOMMAND).arg(input_rpu).arg("--summary").assert();

    assert.success().stderr(predicate::str::is_empty()).stdout(
        predicate::str::contains("Summary:")
            .and(predicate::str::contains("  Frames: 1"))
            .and(predicate::str::contains("  Profile: 7 (FEL)"))
            .and(predicate::str::contains("  DM version: 1 (CM v2.9)"))
            .and(predicate::str::contains("  Scene/shot count: 0")),
    );

    Ok(())
}

#[test]
fn summary_p8() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    let input_rpu = Path::new("assets/hevc_tests/regular_rpu.bin");

    let assert = cmd.arg(SUBCOMMAND).arg(input_rpu).arg("--summary").assert();

    assert.success().stderr(predicate::str::is_empty()).stdout(
        predicate::str::contains("Summary:")
            .and(predicate::str::contains("  Frames: 259"))
            .and(predicate::str::contains("  Profile: 8"))
            .and(predicate::str::contains("  DM version: 2 (CM v4.x)"))
            .and(predicate::str::contains("  Scene/shot count: 3")),
    );

    Ok(())
}

#[test]
fn invalid_l3_error() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    let input_rpu = Path::new("assets/tests/st2094_10_level3.bin");

    let assert = cmd.arg(SUBCOMMAND).arg(input_rpu).arg("-s").assert();

    assert.failure().stderr(predicate::str::contains(
        "Error: Found invalid RPU: Index 0, error: Invalid block level 3 for CM v2.9 RPU",
    ));

    Ok(())
}
