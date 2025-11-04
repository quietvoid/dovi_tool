use anyhow::Result;
use assert_cmd::cargo;
use predicates::prelude::*;

mod hevc;
mod rpu;

#[test]
fn help() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let assert = cmd.arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains("dovi_tool [OPTIONS] <COMMAND>"));
    Ok(())
}

#[test]
fn version() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let assert = cmd.arg("--version").assert();

    assert.success().stderr(predicate::str::is_empty());
    Ok(())
}
