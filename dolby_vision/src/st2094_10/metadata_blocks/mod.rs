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

pub trait ExtMetadataBlockInfo {
    fn level(&self) -> u8;
    fn bytes_size(&self) -> u64;
    fn required_bits(&self) -> u64;

    fn bits_size(&self) -> u64 {
        self.bytes_size() * 8
    }

    fn sort_key(&self) -> (u8, u16) {
        (self.level(), 0)
    }
}

pub fn ext_metadata_block(reader: &mut BitVecReader) -> Result<ExtMetadataBlock> {
    let ext_block_length = reader.get_ue()?;
    let ext_block_level = reader.get_n(8);

    let ext_metadata_block = match ext_block_level {
        1 => level1::ExtMetadataBlockLevel1::parse(reader),
        2 => level2::ExtMetadataBlockLevel2::parse(reader),
        3 => level3::ExtMetadataBlockLevel3::parse(reader),
        4 => level4::ExtMetadataBlockLevel4::parse(reader),
        5 => level5::ExtMetadataBlockLevel5::parse(reader),
        6 => level6::ExtMetadataBlockLevel6::parse(reader),
        _ => {
            ensure!(
                false,
                "Reserved metadata block found, please open an issue."
            );

            reserved::ReservedExtMetadataBlock::parse(ext_block_length, ext_block_level, reader)?
        }
    };

    ensure!(
        ext_block_length == ext_metadata_block.length_bytes(),
        format!(
            "level {} block should have length {}",
            ext_block_level,
            ext_metadata_block.length_bytes()
        )
    );

    let ext_block_use_bits = ext_metadata_block.length_bits() - ext_metadata_block.required_bits();

    for _ in 0..ext_block_use_bits {
        ensure!(!reader.get()?, "ext_dm_alignment_zero_bit != 0");
    }

    Ok(ext_metadata_block)
}

impl ExtMetadataBlock {
    pub fn length_bytes(&self) -> u64 {
        match self {
            ExtMetadataBlock::Level1(b) => b.bytes_size(),
            ExtMetadataBlock::Level2(b) => b.bytes_size(),
            ExtMetadataBlock::Level3(b) => b.bytes_size(),
            ExtMetadataBlock::Level4(b) => b.bytes_size(),
            ExtMetadataBlock::Level5(b) => b.bytes_size(),
            ExtMetadataBlock::Level6(b) => b.bytes_size(),
            ExtMetadataBlock::Reserved(b) => b.bytes_size(),
        }
    }

    pub fn length_bits(&self) -> u64 {
        match self {
            ExtMetadataBlock::Level1(b) => b.bits_size(),
            ExtMetadataBlock::Level2(b) => b.bits_size(),
            ExtMetadataBlock::Level3(b) => b.bits_size(),
            ExtMetadataBlock::Level4(b) => b.bits_size(),
            ExtMetadataBlock::Level5(b) => b.bits_size(),
            ExtMetadataBlock::Level6(b) => b.bits_size(),
            ExtMetadataBlock::Reserved(b) => b.bits_size(),
        }
    }

    pub fn required_bits(&self) -> u64 {
        match self {
            ExtMetadataBlock::Level1(b) => b.required_bits(),
            ExtMetadataBlock::Level2(b) => b.required_bits(),
            ExtMetadataBlock::Level3(b) => b.required_bits(),
            ExtMetadataBlock::Level4(b) => b.required_bits(),
            ExtMetadataBlock::Level5(b) => b.required_bits(),
            ExtMetadataBlock::Level6(b) => b.required_bits(),
            ExtMetadataBlock::Reserved(b) => b.required_bits(),
        }
    }

    pub fn level(&self) -> u8 {
        match self {
            ExtMetadataBlock::Level1(b) => b.level(),
            ExtMetadataBlock::Level2(b) => b.level(),
            ExtMetadataBlock::Level3(b) => b.level(),
            ExtMetadataBlock::Level4(b) => b.level(),
            ExtMetadataBlock::Level5(b) => b.level(),
            ExtMetadataBlock::Level6(b) => b.level(),
            ExtMetadataBlock::Reserved(b) => b.level(),
        }
    }

    pub fn sort_key(&self) -> (u8, u16) {
        match self {
            ExtMetadataBlock::Level1(b) => b.sort_key(),
            ExtMetadataBlock::Level2(b) => b.sort_key(),
            ExtMetadataBlock::Level3(b) => b.sort_key(),
            ExtMetadataBlock::Level4(b) => b.sort_key(),
            ExtMetadataBlock::Level5(b) => b.sort_key(),
            ExtMetadataBlock::Level6(b) => b.sort_key(),
            ExtMetadataBlock::Reserved(b) => b.sort_key(),
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
