use std::path::Path;

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

const SUBCOMMAND: &str = "plot";

#[test]
fn help() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg(SUBCOMMAND).arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "dovi_tool plot [OPTIONS] [input_pos]",
        ));
    Ok(())
}

#[test]
fn plot_p7() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_rpu = Path::new("assets/hevc_tests/regular_rpu.bin");
    let output_file = temp.child("L1_plot.png");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_rpu)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file.assert(predicate::path::is_file());

    Ok(())
}
