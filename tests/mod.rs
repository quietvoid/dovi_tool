use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;

mod hevc;
mod rpu;

#[test]
fn help() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains("dovi_tool [OPTIONS] <COMMAND>"));
    Ok(())
}

#[test]
fn version() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg("--version").assert();

    assert.success().stderr(predicate::str::is_empty());
    Ok(())
}
