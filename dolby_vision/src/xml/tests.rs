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
    let level6 = config.level6.as_ref().unwrap();
    assert_eq!(level6.max_display_mastering_luminance, 1000);
    assert_eq!(level6.min_display_mastering_luminance, 1);
    assert_eq!(level6.max_content_light_level, 756);
    assert_eq!(level6.max_frame_average_light_level, 97);

    // No L254
    assert!(config.level254.is_none());

    let shot1 = &config.shots[0];
    let shot1_blocks = &shot1.metadata_blocks;
    assert_eq!(shot1.duration, 12);
    assert_eq!(shot1_blocks.len(), 4);

    assert_num_blocks_for_level(shot1_blocks, 1, 1);
    assert_num_blocks_for_level(shot1_blocks, 2, 3);

    let shot2 = &config.shots[1];
    let shot2_blocks = &shot2.metadata_blocks;
    assert_eq!(shot2.duration, 96);
    assert_eq!(shot2_blocks.len(), 4);

    assert_num_blocks_for_level(shot2_blocks, 1, 1);
    assert_num_blocks_for_level(shot2_blocks, 2, 3);

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
    let level6 = config.level6.as_ref().unwrap();
    assert_eq!(level6.max_display_mastering_luminance, 1000);
    assert_eq!(level6.min_display_mastering_luminance, 1);
    assert_eq!(level6.max_content_light_level, 3948);
    assert_eq!(level6.max_frame_average_light_level, 120);

    // XML L254
    assert!(config.level254.is_some());

    let level254 = config.level254.as_ref().unwrap();
    assert_eq!(level254.dm_mode, 0);
    assert_eq!(level254.dm_version_index, 2);

    let shot1 = &config.shots[0];
    let shot1_blocks = &shot1.metadata_blocks;
    assert_eq!(shot1.duration, 120);
    assert_eq!(shot1_blocks.len(), 4);

    assert_num_blocks_for_level(shot1_blocks, 1, 1);
    assert_num_blocks_for_level(shot1_blocks, 3, 1);
    assert_num_blocks_for_level(shot1_blocks, 5, 1);
    assert_num_blocks_for_level(shot1_blocks, 9, 1);

    let shot2 = &config.shots[1];
    let shot2_blocks = &shot2.metadata_blocks;
    assert_eq!(shot2.duration, 99);
    assert_eq!(shot2_blocks.len(), 8);

    assert_num_blocks_for_level(shot2_blocks, 1, 1);
    assert_num_blocks_for_level(shot2_blocks, 2, 3);
    assert_num_blocks_for_level(shot2_blocks, 3, 1);
    assert_num_blocks_for_level(shot2_blocks, 8, 2);
    assert_num_blocks_for_level(shot2_blocks, 9, 1);

    let shot3 = &config.shots[2];
    let shot3_blocks = &shot3.metadata_blocks;
    assert_eq!(shot3.duration, 40);
    assert_eq!(shot3_blocks.len(), 3);

    let rpus = config.generate_rpu_list()?;
    let rpu = &rpus[0];
    let vdr_dm_data = rpu.vdr_dm_data.as_ref().unwrap();
    let level254 = vdr_dm_data.get_block(254).unwrap();

    if let ExtMetadataBlock::Level254(block) = &level254 {
        assert_eq!(block.dm_mode, 0);
        assert_eq!(block.dm_version_index, 2);
    }

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

#[test]
fn parse_cmv4_0_2_custom_displays() -> Result<()> {
    let lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let assets_path = lib_path.parent().unwrap();

    let opts = XmlParserOpts::default();
    let parser = CmXmlParser::parse_file(
        &assets_path.join("assets/tests/cmv4_0_2_custom_displays.xml"),
        opts,
    )?;

    let config = parser.config;

    assert_eq!(config.cm_version, CmVersion::V40);
    let rpus = config.generate_rpu_list()?;

    let rpu = &rpus[0];
    let vdr_dm_data = rpu.vdr_dm_data.as_ref().unwrap();

    // L1, L5, L6 in DMv1
    assert_eq!(vdr_dm_data.metadata_blocks(1).unwrap().len(), 3);

    // L3, L8, L9, L10, L11, L254
    assert_eq!(vdr_dm_data.metadata_blocks(3).unwrap().len(), 6);

    let level8 = vdr_dm_data.get_block(8).unwrap();

    if let ExtMetadataBlock::Level8(block) = level8 {
        assert_eq!(block.length, 25);
        assert_eq!(block.target_display_index, 255);
        assert_eq!(block.trim_slope, 2068);
        assert_eq!(block.trim_offset, 2069);
        assert_eq!(block.trim_power, 1987);
        assert_eq!(block.trim_chroma_weight, 2130);
        assert_eq!(block.trim_saturation_gain, 2150);
        assert_eq!(block.ms_weight, 2171);
        assert_eq!(block.target_mid_contrast, 2089);
        assert_eq!(block.clip_trim, 2011);

        assert_eq!(block.saturation_vector_field0, 128);
        assert_eq!(block.saturation_vector_field1, 128);
        assert_eq!(block.saturation_vector_field2, 128);
        assert_eq!(block.saturation_vector_field3, 128);
        assert_eq!(block.saturation_vector_field4, 150);
        assert_eq!(block.saturation_vector_field5, 128);

        assert_eq!(block.hue_vector_field0, 128);
        assert_eq!(block.hue_vector_field1, 160);
        assert_eq!(block.hue_vector_field2, 128);
        assert_eq!(block.hue_vector_field3, 128);
        assert_eq!(block.hue_vector_field4, 128);
        assert_eq!(block.hue_vector_field5, 128);
    } else {
        panic!("No L8 block");
    }

    let level9 = vdr_dm_data.get_block(9).unwrap();
    if let ExtMetadataBlock::Level9(block) = level9 {
        assert_eq!(block.length, 17);
        assert_eq!(block.source_primary_index, 255);

        assert_eq!(block.source_primary_red_x, 22314);
        assert_eq!(block.source_primary_red_y, 10551);
        assert_eq!(block.source_primary_green_x, 8693);
        assert_eq!(block.source_primary_green_y, 22740);
        assert_eq!(block.source_primary_blue_x, 5079);
        assert_eq!(block.source_primary_blue_y, 2163);
        assert_eq!(block.source_primary_white_x, 10249);
        assert_eq!(block.source_primary_white_y, 10807);
    } else {
        panic!("No L9 block");
    }

    let level10 = vdr_dm_data.get_block(10).unwrap();
    if let ExtMetadataBlock::Level10(block) = level10 {
        assert_eq!(block.length, 21);
        assert_eq!(block.target_display_index, 255);

        assert_eq!(block.target_max_pq, 2081);
        assert_eq!(block.target_min_pq, 62);
        assert_eq!(block.target_primary_index, 255);
        assert_eq!(block.target_primary_red_x, 21004);
        assert_eq!(block.target_primary_red_y, 10879);
        assert_eq!(block.target_primary_green_x, 10813);
        assert_eq!(block.target_primary_green_y, 20971);
        assert_eq!(block.target_primary_blue_x, 5079);
        assert_eq!(block.target_primary_blue_y, 2163);
        assert_eq!(block.target_primary_white_x, 10249);
        assert_eq!(block.target_primary_white_y, 10807);
    } else {
        panic!("No L10 block");
    }

    Ok(())
}

#[test]
fn parse_cmv4_2_xml_510() -> Result<()> {
    let lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let assets_path = lib_path.parent().unwrap();

    let opts = XmlParserOpts::default();
    let parser =
        CmXmlParser::parse_file(&assets_path.join("assets/tests/cmv4_2_xml_510.xml"), opts)?;

    // Only HOME targets
    assert_eq!(parser.target_displays.len(), 3);

    let config = parser.config;

    assert_eq!(config.cm_version, CmVersion::V40);
    let rpus = config.generate_rpu_list()?;

    let rpu = &rpus[0];
    let vdr_dm_data = rpu.vdr_dm_data.as_ref().unwrap();

    // L1, L5, L6 in DMv1
    assert_eq!(vdr_dm_data.metadata_blocks(1).unwrap().len(), 3);

    // L3, L9, L11, L254 in DMv2
    assert_eq!(vdr_dm_data.metadata_blocks(3).unwrap().len(), 4);

    // Level 9 block recognized as preset
    let level9 = vdr_dm_data.get_block(9).unwrap();
    if let ExtMetadataBlock::Level9(block) = level9 {
        assert_eq!(block.length, 1);
        assert_eq!(block.source_primary_index, 0);
    } else {
        panic!("No L9 block");
    }

    let level11 = vdr_dm_data.get_block(11).unwrap();
    if let ExtMetadataBlock::Level11(block) = level11 {
        assert_eq!(block.content_type, 2);
        assert_eq!(block.whitepoint, 0);
        assert!(!block.reference_mode_flag);
    } else {
        panic!("No L11 block");
    }

    Ok(())
}
