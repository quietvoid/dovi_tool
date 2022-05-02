use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg("export").arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "dovi_tool export [OPTIONS] [input_pos]",
        ));
    Ok(())
}
