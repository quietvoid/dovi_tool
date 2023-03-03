use std::path::Path;

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

use dolby_vision::rpu::extension_metadata::blocks::ExtMetadataBlock;

const SUBCOMMAND: &str = "generate";

#[test]
fn help() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg(SUBCOMMAND).arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains("dovi_tool generate [OPTIONS]"));
    Ok(())
}

#[test]
fn generate_default_cmv29() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let generate_config = Path::new("assets/generator_examples/default_cmv29.json");
    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--json")
        .arg(generate_config)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu.assert(predicate::path::is_file());

    let rpus = dolby_vision::rpu::utils::parse_rpu_file(output_rpu.as_ref())?;

    let first_rpu = &rpus[0];
    let vdr_dm_data = first_rpu.vdr_dm_data.as_ref().unwrap();

    assert_eq!(vdr_dm_data.scene_refresh_flag, 1);

    // Only L5 and L6
    assert_eq!(vdr_dm_data.metadata_blocks(1).unwrap().len(), 2);
    // No CM v4.0

    assert!(vdr_dm_data.metadata_blocks(3).is_none());

    if let ExtMetadataBlock::Level5(level5) = vdr_dm_data.get_block(5).unwrap() {
        assert_eq!(level5.get_offsets(), (0, 0, 0, 0));
    }

    if let ExtMetadataBlock::Level6(level6) = vdr_dm_data.get_block(6).unwrap() {
        assert_eq!(level6.min_display_mastering_luminance, 1);
        assert_eq!(level6.max_display_mastering_luminance, 1000);
        assert_eq!(level6.max_content_light_level, 1000);
        assert_eq!(level6.max_frame_average_light_level, 400);
    }

    Ok(())
}

#[test]
fn generate_default_cmv40() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let generate_config = Path::new("assets/generator_examples/default_cmv40.json");
    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--json")
        .arg(generate_config)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu.assert(predicate::path::is_file());

    let rpus = dolby_vision::rpu::utils::parse_rpu_file(output_rpu.as_ref())?;
    assert_eq!(rpus.len(), 10);

    let first_rpu = &rpus[0];
    let vdr_dm_data = first_rpu.vdr_dm_data.as_ref().unwrap();

    assert_eq!(vdr_dm_data.scene_refresh_flag, 1);

    // Only L5 and L6
    assert_eq!(vdr_dm_data.metadata_blocks(1).unwrap().len(), 2);
    // Only L9, L11 and L254

    assert_eq!(vdr_dm_data.metadata_blocks(3).unwrap().len(), 3);

    if let ExtMetadataBlock::Level5(level5) = vdr_dm_data.get_block(5).unwrap() {
        assert_eq!(level5.get_offsets(), (0, 0, 0, 0));
    }

    if let ExtMetadataBlock::Level6(level6) = vdr_dm_data.get_block(6).unwrap() {
        assert_eq!(level6.min_display_mastering_luminance, 1);
        assert_eq!(level6.max_display_mastering_luminance, 1000);
        assert_eq!(level6.max_content_light_level, 1000);
        assert_eq!(level6.max_frame_average_light_level, 400);
    }

    if let ExtMetadataBlock::Level9(level9) = vdr_dm_data.get_block(9).unwrap() {
        assert_eq!(level9.length, 1);
        assert_eq!(level9.source_primary_index, 0);
    }

    if let ExtMetadataBlock::Level11(level11) = vdr_dm_data.get_block(11).unwrap() {
        assert_eq!(level11.content_type, 1);
        assert_eq!(level11.whitepoint, 0);
        assert!(level11.reference_mode_flag);
    }

    Ok(())
}

#[test]
fn generate_full() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let generate_config = Path::new("assets/generator_examples/full_example.json");
    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--json")
        .arg(generate_config)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu.assert(predicate::path::is_file());

    let rpus = dolby_vision::rpu::utils::parse_rpu_file(output_rpu.as_ref())?;
    assert_eq!(rpus.len(), 10);

    let first_rpu = &rpus[0];
    let vdr_dm_data = first_rpu.vdr_dm_data.as_ref().unwrap();

    assert_eq!(vdr_dm_data.scene_refresh_flag, 1);

    // L1, L2 * 2, L5, L6
    assert_eq!(vdr_dm_data.metadata_blocks(1).unwrap().len(), 5);
    // Only L9, L11 and L254
    assert_eq!(vdr_dm_data.metadata_blocks(3).unwrap().len(), 3);

    if let ExtMetadataBlock::Level5(level5) = vdr_dm_data.get_block(5).unwrap() {
        assert_eq!(level5.get_offsets(), (0, 0, 40, 40));
    }

    if let ExtMetadataBlock::Level6(level6) = vdr_dm_data.get_block(6).unwrap() {
        assert_eq!(level6.min_display_mastering_luminance, 1);
        assert_eq!(level6.max_display_mastering_luminance, 1000);
        assert_eq!(level6.max_content_light_level, 1000);
        assert_eq!(level6.max_frame_average_light_level, 400);
    }

    // From default blocks
    assert_eq!(vdr_dm_data.level_blocks_iter(2).count(), 2);
    let mut shot_level2_iter = vdr_dm_data.level_blocks_iter(2);

    if let ExtMetadataBlock::Level2(level2) = shot_level2_iter.next().unwrap() {
        assert_eq!(level2.target_max_pq, 2851);
        assert_eq!(level2.trim_slope, 2048);
        assert_eq!(level2.trim_offset, 2048);
        assert_eq!(level2.trim_power, 1800);
        assert_eq!(level2.trim_chroma_weight, 2048);
        assert_eq!(level2.trim_saturation_gain, 2048);
        assert_eq!(level2.ms_weight, 2048);
    }

    if let ExtMetadataBlock::Level2(level2) = shot_level2_iter.next().unwrap() {
        assert_eq!(level2.target_max_pq, 3079);
        assert_eq!(level2.trim_slope, 2048);
        assert_eq!(level2.trim_offset, 2048);
        assert_eq!(level2.trim_power, 2048);
        assert_eq!(level2.trim_chroma_weight, 2048);
        assert_eq!(level2.trim_saturation_gain, 2048);
        assert_eq!(level2.ms_weight, 2048);
    }

    // From default blocks
    if let ExtMetadataBlock::Level9(level9) = vdr_dm_data.get_block(9).unwrap() {
        assert_eq!(level9.source_primary_index, 0);
    }

    // Default block L11 overrides
    if let ExtMetadataBlock::Level11(level11) = vdr_dm_data.get_block(11).unwrap() {
        assert_eq!(level11.content_type, 4);
        assert_eq!(level11.whitepoint, 0);
        assert!(level11.reference_mode_flag);
    }

    Ok(())
}

#[test]
fn generate_full_hdr10plus() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let generate_config = Path::new("assets/generator_examples/no_duration.json");
    let hdr10plus_json = Path::new("./assets/tests/hdr10plus_metadata.json");

    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--json")
        .arg(generate_config)
        .arg("--hdr10plus-json")
        .arg(hdr10plus_json)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu.assert(predicate::path::is_file());

    let rpus = dolby_vision::rpu::utils::parse_rpu_file(output_rpu.as_ref())?;
    assert_eq!(rpus.len(), 9);

    let shot1_rpu = &rpus[0];
    let shot1_vdr_dm_data = shot1_rpu.vdr_dm_data.as_ref().unwrap();

    assert_eq!(shot1_vdr_dm_data.scene_refresh_flag, 1);

    // Only L1, L2 and L5 and L6
    assert_eq!(shot1_vdr_dm_data.metadata_blocks(1).unwrap().len(), 4);
    // Only L9, L11 and L254
    assert_eq!(shot1_vdr_dm_data.metadata_blocks(3).unwrap().len(), 3);

    // Shot L1 is ignored, HDR10+ is used
    if let ExtMetadataBlock::Level1(level1) = shot1_vdr_dm_data.get_block(1).unwrap() {
        assert_eq!(level1.min_pq, 0);
        assert_eq!(level1.max_pq, 3243);
        assert_eq!(level1.avg_pq, 2097);
    }

    // From shot blocks
    assert_eq!(shot1_vdr_dm_data.level_blocks_iter(2).count(), 1);
    let mut shot1_level2_iter = shot1_vdr_dm_data.level_blocks_iter(2);

    if let ExtMetadataBlock::Level2(level2) = shot1_level2_iter.next().unwrap() {
        assert_eq!(level2.target_max_pq, 2851);
        assert_eq!(level2.trim_slope, 2048);
        assert_eq!(level2.trim_offset, 2048);
        assert_eq!(level2.trim_power, 1800);
        assert_eq!(level2.trim_chroma_weight, 2048);
        assert_eq!(level2.trim_saturation_gain, 2048);
        assert_eq!(level2.ms_weight, 2048);
    }

    if let ExtMetadataBlock::Level5(level5) = shot1_vdr_dm_data.get_block(5).unwrap() {
        assert_eq!(level5.get_offsets(), (0, 0, 0, 0));
    }

    let shot2_rpu = &rpus[3];
    let shot2_vdr_dm_data = shot2_rpu.vdr_dm_data.as_ref().unwrap();

    assert_eq!(shot2_vdr_dm_data.scene_refresh_flag, 1);

    // Only L1, L5 and L6
    assert_eq!(shot2_vdr_dm_data.metadata_blocks(1).unwrap().len(), 4);
    // Only L9, L11 and L254
    assert_eq!(shot2_vdr_dm_data.metadata_blocks(3).unwrap().len(), 3);

    if let ExtMetadataBlock::Level1(level1) = shot2_vdr_dm_data.get_block(1).unwrap() {
        assert_eq!(level1.min_pq, 0);
        assert_eq!(level1.max_pq, 2568);
        assert_eq!(level1.avg_pq, 1609);
    }

    // From shot blocks
    assert_eq!(shot2_vdr_dm_data.level_blocks_iter(2).count(), 1);
    let mut shot2_level2_iter = shot2_vdr_dm_data.level_blocks_iter(2);

    if let ExtMetadataBlock::Level2(level2) = shot2_level2_iter.next().unwrap() {
        assert_eq!(level2.target_max_pq, 2851);
        assert_eq!(level2.trim_slope, 1400);
        assert_eq!(level2.trim_offset, 1234);
        assert_eq!(level2.trim_power, 1800);
        assert_eq!(level2.trim_chroma_weight, 2048);
        assert_eq!(level2.trim_saturation_gain, 2048);
        assert_eq!(level2.ms_weight, 2048);
    }

    if let ExtMetadataBlock::Level5(level5) = shot2_vdr_dm_data.get_block(5).unwrap() {
        assert_eq!(level5.get_offsets(), (0, 0, 276, 276));
    }

    if let ExtMetadataBlock::Level6(level6) = shot2_vdr_dm_data.get_block(6).unwrap() {
        assert_eq!(level6.min_display_mastering_luminance, 1);
        assert_eq!(level6.max_display_mastering_luminance, 1000);
        assert_eq!(level6.max_content_light_level, 1000);
        assert_eq!(level6.max_frame_average_light_level, 400);
    }

    let frame_edit_rpu = &rpus[5];
    let edit_vdr_dm_data = frame_edit_rpu.vdr_dm_data.as_ref().unwrap();

    assert_eq!(edit_vdr_dm_data.scene_refresh_flag, 0);

    // Only L1, L2 * 2, L5 and L6
    assert_eq!(edit_vdr_dm_data.metadata_blocks(1).unwrap().len(), 5);
    // Only L9, L11 and L254
    assert_eq!(edit_vdr_dm_data.metadata_blocks(3).unwrap().len(), 3);

    // Also ignored L1 from edit
    if let ExtMetadataBlock::Level1(level1) = edit_vdr_dm_data.get_block(1).unwrap() {
        assert_eq!(level1.min_pq, 0);
        assert_eq!(level1.max_pq, 2568);
        assert_eq!(level1.avg_pq, 1609);
    }

    // From edit blocks
    assert_eq!(edit_vdr_dm_data.level_blocks_iter(2).count(), 2);
    let mut edit_level2_iter = edit_vdr_dm_data.level_blocks_iter(2);

    // Replaced same target display trim
    if let ExtMetadataBlock::Level2(level2) = edit_level2_iter.next().unwrap() {
        assert_eq!(level2.target_max_pq, 2851);
        assert_eq!(level2.trim_slope, 1999);
        assert_eq!(level2.trim_offset, 1999);
        assert_eq!(level2.trim_power, 1999);
        assert_eq!(level2.trim_chroma_weight, 2048);
        assert_eq!(level2.trim_saturation_gain, 2048);
        assert_eq!(level2.ms_weight, 2048);
    }

    if let ExtMetadataBlock::Level2(level2) = edit_level2_iter.next().unwrap() {
        assert_eq!(level2.target_max_pq, 3079);
        assert_eq!(level2.trim_slope, 2048);
        assert_eq!(level2.trim_offset, 2048);
        assert_eq!(level2.trim_power, 2048);
        assert_eq!(level2.trim_chroma_weight, 2048);
        assert_eq!(level2.trim_saturation_gain, 2048);
        assert_eq!(level2.ms_weight, 2048);
    }

    Ok(())
}

#[test]
fn xml_cmv2_9_with_l5() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let xml = Path::new("assets/tests/cmv2_9.xml");
    let output_rpu = temp.child("RPU.bin");

    let expected_rpu = Path::new("assets/tests/cmv2_9_xml_with_l5_rpu.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--xml")
        .arg(xml)
        .arg("--canvas-width")
        .arg("3840")
        .arg("--canvas-height")
        .arg("2160")
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
fn xml_cmv4_0_2() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let xml = Path::new("assets/tests/cmv4_0_2.xml");
    let output_rpu = temp.child("RPU.bin");

    let expected_rpu = Path::new("assets/tests/cmv4_0_2_xml_rpu.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--xml")
        .arg(xml)
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
fn xml_cmv4_0_2_with_l5() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let xml = Path::new("assets/tests/cmv4_0_2.xml");
    let output_rpu = temp.child("RPU.bin");

    let expected_rpu = Path::new("assets/tests/cmv4_0_2_xml_with_l5_rpu.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--xml")
        .arg(xml)
        .arg("--canvas-width")
        .arg("3840")
        .arg("--canvas-height")
        .arg("2160")
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
fn xml_cmv4_0_2_custom_displays() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let xml = Path::new("assets/tests/cmv4_0_2_custom_displays.xml");
    let output_rpu = temp.child("RPU.bin");

    let expected_rpu = Path::new("assets/tests/cmv4_0_2_custom_displays_xml_rpu.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--xml")
        .arg(xml)
        .arg("--canvas-width")
        .arg("3840")
        .arg("--canvas-height")
        .arg("2160")
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
fn xml_cmv4_2_510() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let xml = Path::new("assets/tests/cmv4_2_xml_510.xml");
    let output_rpu = temp.child("RPU.bin");

    let expected_rpu = Path::new("assets/tests/cmv4_2_510_xml_rpu.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--xml")
        .arg(xml)
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
fn generate_l1_cmv29() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let generate_config = Path::new("assets/generator_examples/l1_cmv29.json");
    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--json")
        .arg(generate_config)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu.assert(predicate::path::is_file());

    let rpus = dolby_vision::rpu::utils::parse_rpu_file(output_rpu.as_ref())?;
    assert_eq!(rpus.len(), 2);

    let shot1_rpu = &rpus[0];
    let shot1_vdr_dm_data = shot1_rpu.vdr_dm_data.as_ref().unwrap();

    assert_eq!(shot1_vdr_dm_data.scene_refresh_flag, 1);

    // Only L1, L5 and L6
    assert_eq!(shot1_vdr_dm_data.metadata_blocks(1).unwrap().len(), 3);
    // No CM v4.0
    assert!(shot1_vdr_dm_data.metadata_blocks(3).is_none());

    if let ExtMetadataBlock::Level1(level1) = shot1_vdr_dm_data.get_block(1).unwrap() {
        assert_eq!(level1.min_pq, 0);
        // Clamped to 2081
        assert_eq!(level1.max_pq, 2081);
        // Clamped to 819
        assert_eq!(level1.avg_pq, 819);
    }

    let shot2_rpu = &rpus[1];
    let shot2_vdr_dm_data = shot2_rpu.vdr_dm_data.as_ref().unwrap();

    assert_eq!(shot2_vdr_dm_data.scene_refresh_flag, 1);

    // Only L1, L5 and L6
    assert_eq!(shot2_vdr_dm_data.metadata_blocks(1).unwrap().len(), 3);
    // No CM v4.0
    assert!(shot2_vdr_dm_data.metadata_blocks(3).is_none());

    if let ExtMetadataBlock::Level1(level1) = shot2_vdr_dm_data.get_block(1).unwrap() {
        assert_eq!(level1.min_pq, 0);
        assert_eq!(level1.max_pq, 2604);
        assert_eq!(level1.avg_pq, 1340);
    }

    Ok(())
}

#[test]
fn generate_l1_cmv40() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let generate_config = Path::new("assets/generator_examples/l1_cmv40.json");
    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--json")
        .arg(generate_config)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_rpu.assert(predicate::path::is_file());

    let rpus = dolby_vision::rpu::utils::parse_rpu_file(output_rpu.as_ref())?;
    assert_eq!(rpus.len(), 2);

    let shot1_rpu = &rpus[0];
    let shot1_vdr_dm_data = shot1_rpu.vdr_dm_data.as_ref().unwrap();

    assert_eq!(shot1_vdr_dm_data.scene_refresh_flag, 1);

    // Only L1, L5 and L6
    assert_eq!(shot1_vdr_dm_data.metadata_blocks(1).unwrap().len(), 3);

    if let ExtMetadataBlock::Level1(level1) = shot1_vdr_dm_data.get_block(1).unwrap() {
        assert_eq!(level1.min_pq, 0);
        // Clamped to 2081
        assert_eq!(level1.max_pq, 2081);
        // Clamped to 1229
        assert_eq!(level1.avg_pq, 1229);
    }

    let shot2_rpu = &rpus[1];
    let shot2_vdr_dm_data = shot2_rpu.vdr_dm_data.as_ref().unwrap();

    assert_eq!(shot2_vdr_dm_data.scene_refresh_flag, 1);

    // Only L1, L5 and L6
    assert_eq!(shot2_vdr_dm_data.metadata_blocks(1).unwrap().len(), 3);

    if let ExtMetadataBlock::Level1(level1) = shot2_vdr_dm_data.get_block(1).unwrap() {
        assert_eq!(level1.min_pq, 0);
        assert_eq!(level1.max_pq, 3074);
        assert_eq!(level1.avg_pq, 1450);
    }

    Ok(())
}

#[test]
fn l1_cmv29_override_avg_cmv40() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let generate_config = Path::new("assets/generator_examples/l1_cmv29_override_avg_cmv40.json");
    let output_rpu = temp.child("RPU.bin");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg("--json")
        .arg(generate_config)
        .arg("--rpu-out")
        .arg(output_rpu.as_ref())
        .assert();

    println!(
        "{:?}",
        std::str::from_utf8(assert.get_output().stdout.as_ref())
    );
    assert.success().stderr(predicate::str::is_empty());

    output_rpu.assert(predicate::path::is_file());

    let rpus = dolby_vision::rpu::utils::parse_rpu_file(output_rpu.as_ref())?;
    assert_eq!(rpus.len(), 2);

    let shot1_rpu = &rpus[0];
    let shot1_vdr_dm_data = shot1_rpu.vdr_dm_data.as_ref().unwrap();

    assert_eq!(shot1_vdr_dm_data.scene_refresh_flag, 1);

    // Only L1, L5 and L6
    assert_eq!(shot1_vdr_dm_data.metadata_blocks(1).unwrap().len(), 3);

    // No CM v4.0
    assert!(shot1_vdr_dm_data.metadata_blocks(3).is_none());

    if let ExtMetadataBlock::Level1(level1) = shot1_vdr_dm_data.get_block(1).unwrap() {
        assert_eq!(level1.min_pq, 0);
        // Clamped to 2081
        assert_eq!(level1.max_pq, 2081);
        // Clamped to 1229
        assert_eq!(level1.avg_pq, 1229);
    }

    let shot2_rpu = &rpus[1];
    let shot2_vdr_dm_data = shot2_rpu.vdr_dm_data.as_ref().unwrap();

    assert_eq!(shot2_vdr_dm_data.scene_refresh_flag, 1);

    // Only L1, L5 and L6
    assert_eq!(shot2_vdr_dm_data.metadata_blocks(1).unwrap().len(), 3);

    // No CM v4.0
    assert!(shot2_vdr_dm_data.metadata_blocks(3).is_none());

    if let ExtMetadataBlock::Level1(level1) = shot2_vdr_dm_data.get_block(1).unwrap() {
        assert_eq!(level1.min_pq, 0);
        assert_eq!(level1.max_pq, 3074);
        assert_eq!(level1.avg_pq, 1450);
    }

    Ok(())
}
