use anyhow::{ensure, Result};
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

pub mod level1;
pub mod level2;
pub mod level3;
pub mod level4;
pub mod level5;
pub mod level6;
pub mod reserved;

#[derive(Debug)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub enum ExtMetadataBlock {
    Level1(level1::ExtMetadataBlockLevel1),
    Level2(level2::ExtMetadataBlockLevel2),
    Level3(level3::ExtMetadataBlockLevel3),
    Level4(level4::ExtMetadataBlockLevel4),
    Level5(level5::ExtMetadataBlockLevel5),
    Level6(level6::ExtMetadataBlockLevel6),
    Reserved(reserved::ReservedExtMetadataBlock),
}

pub fn ext_metadata_block(reader: &mut BitVecReader) -> Result<ExtMetadataBlock> {
    let ext_block_length = reader.get_ue();
    let ext_block_level = reader.get_n(8);

    let ext_metadata_block = match ext_block_level {
        1 => {
            ensure!(ext_block_length == 5, "level 1 block should have length 5");

            level1::ExtMetadataBlockLevel1::parse(reader)
        }
        2 => {
            ensure!(
                ext_block_length == 11,
                "level 2 block should have length 11"
            );

            level2::ExtMetadataBlockLevel2::parse(reader)
        }
        3 => {
            ensure!(ext_block_length == 2, "level 3 block should have length 2");

            level3::ExtMetadataBlockLevel3::parse(reader)
        }
        4 => {
            ensure!(ext_block_length == 3, "level 4 block should have length 4");

            level4::ExtMetadataBlockLevel4::parse(reader)
        }
        5 => {
            ensure!(ext_block_length == 7, "level 5 block should have length 7");

            level5::ExtMetadataBlockLevel5::parse(reader)
        }
        6 => {
            ensure!(ext_block_length == 8, "level 6 block should have length 8");

            level6::ExtMetadataBlockLevel6::parse(reader)
        }
        _ => {
            ensure!(
                false,
                "Reserved metadata block found, please open an issue."
            );

            reserved::ReservedExtMetadataBlock::parse(ext_block_length, ext_block_level, reader)
        }
    };

    let ext_block_use_bits = (8 * ext_block_length) - ext_metadata_block.bits();

    for _ in 0..ext_block_use_bits {
        ensure!(!reader.get(), "ext_dm_alignment_zero_bit != 0");
    }

    Ok(ext_metadata_block)
}

impl ExtMetadataBlock {
    pub fn length(&self) -> u64 {
        match self {
            ExtMetadataBlock::Level1(_) => 5,
            ExtMetadataBlock::Level2(_) => 11,
            ExtMetadataBlock::Level3(_) => 2,
            ExtMetadataBlock::Level4(_) => 3,
            ExtMetadataBlock::Level5(_) => 7,
            ExtMetadataBlock::Level6(_) => 8,
            ExtMetadataBlock::Reserved(b) => b.ext_block_length,
        }
    }

    pub fn bits(&self) -> u64 {
        match self {
            ExtMetadataBlock::Level1(_) => 36,
            ExtMetadataBlock::Level2(_) => 85,
            ExtMetadataBlock::Level3(_) => 36,
            ExtMetadataBlock::Level4(_) => 24,
            ExtMetadataBlock::Level5(_) => 52,
            ExtMetadataBlock::Level6(_) => 64,
            ExtMetadataBlock::Reserved(b) => b.data.len() as u64,
        }
    }

    pub fn level(&self) -> u8 {
        match self {
            ExtMetadataBlock::Level1(_) => 1,
            ExtMetadataBlock::Level2(_) => 2,
            ExtMetadataBlock::Level3(_) => 3,
            ExtMetadataBlock::Level4(_) => 4,
            ExtMetadataBlock::Level5(_) => 5,
            ExtMetadataBlock::Level6(_) => 6,
            ExtMetadataBlock::Reserved(_) => 255,
        }
    }

    pub fn sort_key(&self) -> (u8, u16) {
        match self {
            ExtMetadataBlock::Level1(_) => (1, 0),
            ExtMetadataBlock::Level2(b) => (2, b.target_max_pq),
            ExtMetadataBlock::Level3(_) => (3, 0),
            ExtMetadataBlock::Level4(_) => (4, 0),
            ExtMetadataBlock::Level5(_) => (5, 0),
            ExtMetadataBlock::Level6(_) => (6, 0),
            ExtMetadataBlock::Reserved(_) => (255, 0),
        }
    }

    pub fn write(&self, writer: &mut BitVecWriter) {
        match self {
            ExtMetadataBlock::Level1(b) => b.write(writer),
            ExtMetadataBlock::Level2(b) => b.write(writer),
            ExtMetadataBlock::Level3(b) => b.write(writer),
            ExtMetadataBlock::Level4(b) => b.write(writer),
            ExtMetadataBlock::Level5(b) => b.write(writer),
            ExtMetadataBlock::Level6(b) => b.write(writer),
            ExtMetadataBlock::Reserved(b) => b.write(writer),
        }
    }
}
