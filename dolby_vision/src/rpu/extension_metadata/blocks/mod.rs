use anyhow::{ensure, Result};
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

pub mod level1;
pub mod level10;
pub mod level11;
pub mod level2;
pub mod level254;
pub mod level255;
pub mod level3;
pub mod level4;
pub mod level5;
pub mod level6;
pub mod level8;
pub mod level9;
pub mod reserved;

pub use level1::ExtMetadataBlockLevel1;
pub use level10::ExtMetadataBlockLevel10;
pub use level11::ExtMetadataBlockLevel11;
pub use level2::ExtMetadataBlockLevel2;
pub use level254::ExtMetadataBlockLevel254;
pub use level255::ExtMetadataBlockLevel255;
pub use level3::ExtMetadataBlockLevel3;
pub use level4::ExtMetadataBlockLevel4;
pub use level5::ExtMetadataBlockLevel5;
pub use level6::ExtMetadataBlockLevel6;
pub use level8::ExtMetadataBlockLevel8;
pub use level9::ExtMetadataBlockLevel9;
pub use reserved::ReservedExtMetadataBlock;

use crate::utils::f64_to_integer_primaries;

use super::WithExtMetadataBlocks;

/// cbindgen:ignore
pub const MAX_12_BIT_VALUE: u16 = 4095;

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct ColorPrimaries {
    pub red_x: u16,
    pub red_y: u16,
    pub green_x: u16,
    pub green_y: u16,
    pub blue_x: u16,
    pub blue_y: u16,
    pub white_x: u16,
    pub white_y: u16,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub enum ExtMetadataBlock {
    Level1(ExtMetadataBlockLevel1),
    Level2(ExtMetadataBlockLevel2),
    Level3(ExtMetadataBlockLevel3),
    Level4(ExtMetadataBlockLevel4),
    Level5(ExtMetadataBlockLevel5),
    Level6(ExtMetadataBlockLevel6),
    Level8(ExtMetadataBlockLevel8),
    Level9(ExtMetadataBlockLevel9),
    Level10(ExtMetadataBlockLevel10),
    Level11(ExtMetadataBlockLevel11),
    Level254(ExtMetadataBlockLevel254),
    Level255(ExtMetadataBlockLevel255),
    Reserved(ReservedExtMetadataBlock),
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
            ExtMetadataBlock::Level10(b) => b.bytes_size(),
            ExtMetadataBlock::Level11(b) => b.bytes_size(),
            ExtMetadataBlock::Level254(b) => b.bytes_size(),
            ExtMetadataBlock::Level255(b) => b.bytes_size(),
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
            ExtMetadataBlock::Level10(b) => b.bits_size(),
            ExtMetadataBlock::Level11(b) => b.bits_size(),
            ExtMetadataBlock::Level254(b) => b.bits_size(),
            ExtMetadataBlock::Level255(b) => b.bits_size(),
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
            ExtMetadataBlock::Level10(b) => b.required_bits(),
            ExtMetadataBlock::Level11(b) => b.required_bits(),
            ExtMetadataBlock::Level254(b) => b.required_bits(),
            ExtMetadataBlock::Level255(b) => b.required_bits(),
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
            ExtMetadataBlock::Level10(b) => b.level(),
            ExtMetadataBlock::Level11(b) => b.level(),
            ExtMetadataBlock::Level254(b) => b.level(),
            ExtMetadataBlock::Level255(b) => b.level(),
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
            ExtMetadataBlock::Level10(b) => b.sort_key(),
            ExtMetadataBlock::Level11(b) => b.sort_key(),
            ExtMetadataBlock::Level254(b) => b.sort_key(),
            ExtMetadataBlock::Level255(b) => b.sort_key(),
            ExtMetadataBlock::Reserved(b) => b.sort_key(),
        }
    }

    pub fn write(&self, writer: &mut BitVecWriter) -> Result<()> {
        match self {
            ExtMetadataBlock::Level1(b) => b.write(writer),
            ExtMetadataBlock::Level2(b) => b.write(writer),
            ExtMetadataBlock::Level3(b) => b.write(writer),
            ExtMetadataBlock::Level4(b) => b.write(writer),
            ExtMetadataBlock::Level5(b) => b.write(writer),
            ExtMetadataBlock::Level6(b) => b.write(writer),
            ExtMetadataBlock::Level8(b) => b.write(writer),
            ExtMetadataBlock::Level9(b) => b.write(writer),
            ExtMetadataBlock::Level10(b) => b.write(writer),
            ExtMetadataBlock::Level11(b) => b.write(writer),
            ExtMetadataBlock::Level254(b) => b.write(writer),
            ExtMetadataBlock::Level255(b) => b.write(writer),
            ExtMetadataBlock::Reserved(b) => b.write(writer),
        }
    }

    pub fn validate_correct_dm_data<T: WithExtMetadataBlocks>(&self) -> Result<()> {
        let level = self.level();

        ensure!(
            T::ALLOWED_BLOCK_LEVELS.contains(&level),
            "Metadata block level {} is invalid for {}",
            &level,
            T::VERSION
        );

        Ok(())
    }

    pub fn validate_and_read_remaining<T: WithExtMetadataBlocks>(
        &self,
        reader: &mut BitVecReader,
        block_length: u64,
    ) -> Result<()> {
        let level = self.level();

        ensure!(
            block_length == self.length_bytes(),
            format!(
                "{}: Invalid metadata block. Block level {} should have length {}",
                T::VERSION,
                level,
                self.length_bytes()
            )
        );

        self.validate_correct_dm_data::<T>()?;

        let ext_block_use_bits = self.length_bits() - self.required_bits();

        for _ in 0..ext_block_use_bits {
            ensure!(
                !reader.get()?,
                format!("{}: ext_dm_alignment_zero_bit != 0", T::VERSION)
            );
        }

        Ok(())
    }
}

impl ColorPrimaries {
    pub fn from_array_int(primaries: &[u16; 8]) -> ColorPrimaries {
        Self {
            red_x: primaries[0],
            red_y: primaries[1],
            green_x: primaries[2],
            green_y: primaries[3],
            blue_x: primaries[4],
            blue_y: primaries[5],
            white_x: primaries[6],
            white_y: primaries[7],
        }
    }

    pub fn from_array_float(primaries: &[f64; 8]) -> ColorPrimaries {
        // Float to integer primaries
        let primaries_int = f64_to_integer_primaries(primaries);

        Self::from_array_int(&primaries_int)
    }
}
