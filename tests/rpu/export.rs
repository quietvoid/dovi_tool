use std::path::Path;

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

const SUBCOMMAND: &str = "export";

#[test]
fn help() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg(SUBCOMMAND).arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "dovi_tool export [OPTIONS] [input_pos]",
        ));
    Ok(())
}

#[test]
fn exports_json() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_rpu = Path::new("assets/tests/fel_orig.bin");
    let output_json = temp.child("RPU_export.json");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_rpu)
        .arg("--output")
        .arg(output_json.as_ref())
        .assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains("Exporting serialized RPU list..."));

    output_json.assert(predicate::path::is_file());

    Ok(())
}

#[test]
fn export_all_and_scenes() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let root_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    let input_rpu = root_path.join("assets/tests/fel_orig.bin");
    let all_json = temp.child("RPU_export.json");
    let scenes_file = temp.child("RPU_scenes_test.txt");

    let assert = cmd
        .current_dir(temp.canonicalize().unwrap())
        .arg(SUBCOMMAND)
        .arg(input_rpu)
        .arg("--data")
        .arg(format!("all,scenes={}", scenes_file.to_str().unwrap()))
        .assert();

    assert.success().stderr(predicate::str::is_empty()).stdout(
        predicate::str::contains("Exporting serialized RPU list...")
            .and(predicate::str::contains("Exporting scenes list...")),
    );

    all_json.assert(predicate::path::is_file());
    scenes_file.assert(predicate::path::is_file());

    Ok(())
}
