use anyhow::{bail, ensure, Result};
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

pub mod level1;
pub mod level2;
pub mod level254;
pub mod level3;
pub mod level4;
pub mod level5;
pub mod level6;
pub mod level8;
pub mod level9;
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
    Level8(level8::ExtMetadataBlockLevel8),
    Level9(level9::ExtMetadataBlockLevel9),
    Level254(level254::ExtMetadataBlockLevel254),
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
        4 => level4::ExtMetadataBlockLevel4::parse(reader),
        5 => level5::ExtMetadataBlockLevel5::parse(reader),
        6 => level6::ExtMetadataBlockLevel6::parse(reader),
        3 | 8 | 10 | 11 | 254 => bail!("Invalid block level {} for CMv2.9 RPU", ext_block_level),
        _ => {
            ensure!(
                false,
                format!("CMv2.9 - Reserved metadata block found: Level {}, length {}, please open an issue.", ext_block_level, ext_block_length)
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
        ensure!(!reader.get()?, "CMv2.9: ext_dm_alignment_zero_bit != 0");
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
            ExtMetadataBlock::Level8(b) => b.bytes_size(),
            ExtMetadataBlock::Level9(b) => b.bytes_size(),
            ExtMetadataBlock::Level254(b) => b.bytes_size(),
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
            ExtMetadataBlock::Level8(b) => b.bits_size(),
            ExtMetadataBlock::Level9(b) => b.bits_size(),
            ExtMetadataBlock::Level254(b) => b.bits_size(),
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
            ExtMetadataBlock::Level8(b) => b.required_bits(),
            ExtMetadataBlock::Level9(b) => b.required_bits(),
            ExtMetadataBlock::Level254(b) => b.required_bits(),
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
            ExtMetadataBlock::Level8(b) => b.level(),
            ExtMetadataBlock::Level9(b) => b.level(),
            ExtMetadataBlock::Level254(b) => b.level(),
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
            ExtMetadataBlock::Level8(b) => b.sort_key(),
            ExtMetadataBlock::Level9(b) => b.sort_key(),
            ExtMetadataBlock::Level254(b) => b.sort_key(),
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
            ExtMetadataBlock::Level8(b) => b.write(writer),
            ExtMetadataBlock::Level9(b) => b.write(writer),
            ExtMetadataBlock::Level254(b) => b.write(writer),
            ExtMetadataBlock::Reserved(b) => b.write(writer),
        }
    }
}
