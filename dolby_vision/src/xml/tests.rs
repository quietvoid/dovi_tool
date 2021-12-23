use std::path::PathBuf;

use crate::rpu::{extension_metadata::blocks::ExtMetadataBlock, vdr_dm_data::CmVersion};

use super::{CmXmlParser, XmlParserOpts};
use anyhow::Result;

fn assert_num_blocks_for_level(blocks: &[ExtMetadataBlock], level: u8, count: usize) {
    let filtered = blocks.iter().filter(|b| b.level() == level).count();

    assert_eq!(filtered, count);
}

#[test]
fn parse_cmv2_9() -> Result<()> {
    let lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let assets_path = lib_path.parent().unwrap();

    let opts = XmlParserOpts {
        canvas_width: Some(3840),
        canvas_height: Some(2160),
    };
    let parser = CmXmlParser::parse_file(&assets_path.join("assets/tests/cmv2_9.xml"), opts)?;

    let config = parser.config;

    assert_eq!(config.cm_version, CmVersion::V29);
    assert_eq!(config.length, 108);
    assert_eq!(config.shots.len(), 2);

    // L5
    assert_eq!(config.level5.get_offsets(), (0, 0, 276, 276));

    // L6
    assert_eq!(config.level6.max_display_mastering_luminance, 1000);
    assert_eq!(config.level6.min_display_mastering_luminance, 1);
    assert_eq!(config.level6.max_content_light_level, 756);
    assert_eq!(config.level6.max_frame_average_light_level, 97);

    let shot1 = &config.shots[0];
    let shot1_blocks = &shot1.metadata_blocks;
    assert_eq!(shot1.duration, 12);
    assert_eq!(shot1_blocks.len(), 4);

    assert_num_blocks_for_level(&shot1_blocks, 1, 1);
    assert_num_blocks_for_level(&shot1_blocks, 2, 3);

    let shot2 = &config.shots[1];
    let shot2_blocks = &shot2.metadata_blocks;
    assert_eq!(shot2.duration, 96);
    assert_eq!(shot2_blocks.len(), 4);

    assert_num_blocks_for_level(&shot2_blocks, 1, 1);
    assert_num_blocks_for_level(&shot2_blocks, 2, 3);

    Ok(())
}

#[test]
fn parse_cmv4_0_2() -> Result<()> {
    let lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let assets_path = lib_path.parent().unwrap();

    let opts = XmlParserOpts::default();
    let parser = CmXmlParser::parse_file(&assets_path.join("assets/tests/cmv4_0_2.xml"), opts)?;

    let config = parser.config;

    assert_eq!(config.cm_version, CmVersion::V40);
    assert_eq!(config.length, 259);
    assert_eq!(config.shots.len(), 3);

    // L5
    assert_eq!(config.level5.get_offsets(), (0, 0, 0, 0));

    // L6
    assert_eq!(config.level6.max_display_mastering_luminance, 1000);
    assert_eq!(config.level6.min_display_mastering_luminance, 1);
    assert_eq!(config.level6.max_content_light_level, 3948);
    assert_eq!(config.level6.max_frame_average_light_level, 120);

    let shot1 = &config.shots[0];
    let shot1_blocks = &shot1.metadata_blocks;
    assert_eq!(shot1.duration, 120);
    assert_eq!(shot1_blocks.len(), 4);

    assert_num_blocks_for_level(&shot1_blocks, 1, 1);
    assert_num_blocks_for_level(&shot1_blocks, 3, 1);
    assert_num_blocks_for_level(&shot1_blocks, 5, 1);
    assert_num_blocks_for_level(&shot1_blocks, 9, 1);

    let shot2 = &config.shots[1];
    let shot2_blocks = &shot2.metadata_blocks;
    assert_eq!(shot2.duration, 99);
    assert_eq!(shot2_blocks.len(), 8);

    assert_num_blocks_for_level(&shot2_blocks, 1, 1);
    assert_num_blocks_for_level(&shot2_blocks, 2, 3);
    assert_num_blocks_for_level(&shot2_blocks, 3, 1);
    assert_num_blocks_for_level(&shot2_blocks, 8, 2);
    assert_num_blocks_for_level(&shot2_blocks, 9, 1);

    let shot3 = &config.shots[2];
    let shot3_blocks = &shot3.metadata_blocks;
    assert_eq!(shot3.duration, 40);
    assert_eq!(shot3_blocks.len(), 3);

    Ok(())
}

#[test]
fn parse_cmv4_0_2_with_l5() -> Result<()> {
    let lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let assets_path = lib_path.parent().unwrap();

    let opts = XmlParserOpts {
        canvas_width: Some(3840),
        canvas_height: Some(2160),
    };

    let parser = CmXmlParser::parse_file(&assets_path.join("assets/tests/cmv4_0_2.xml"), opts)?;

    let config = parser.config;

    assert_eq!(config.cm_version, CmVersion::V40);

    // L5
    assert_eq!(config.level5.get_offsets(), (480, 480, 0, 0));

    Ok(())
}
