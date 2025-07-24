use anyhow::{Result, ensure};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod level1;
pub mod level10;
pub mod level11;
pub mod level15;
pub mod level16;
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
pub use level2::ExtMetadataBlockLevel2;
pub use level3::ExtMetadataBlockLevel3;
pub use level4::ExtMetadataBlockLevel4;
pub use level5::ExtMetadataBlockLevel5;
pub use level6::ExtMetadataBlockLevel6;
pub use level8::ExtMetadataBlockLevel8;
pub use level9::ExtMetadataBlockLevel9;
pub use level10::ExtMetadataBlockLevel10;
pub use level11::ExtMetadataBlockLevel11;
pub use level15::ExtMetadataBlockLevel15;
pub use level16::ExtMetadataBlockLevel16;
pub use level254::ExtMetadataBlockLevel254;
pub use level255::ExtMetadataBlockLevel255;
pub use reserved::ReservedExtMetadataBlock;

use super::{ColorPrimaries, WithExtMetadataBlocks};

/// cbindgen:ignore
pub const MAX_12_BIT_VALUE: u16 = 4095;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
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
    Level15(ExtMetadataBlockLevel15),
    Level16(ExtMetadataBlockLevel16),
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
            Self::Level1(b) => b.bytes_size(),
            Self::Level2(b) => b.bytes_size(),
            Self::Level3(b) => b.bytes_size(),
            Self::Level4(b) => b.bytes_size(),
            Self::Level5(b) => b.bytes_size(),
            Self::Level6(b) => b.bytes_size(),
            Self::Level8(b) => b.bytes_size(),
            Self::Level9(b) => b.bytes_size(),
            Self::Level10(b) => b.bytes_size(),
            Self::Level11(b) => b.bytes_size(),
            Self::Level15(b) => b.bytes_size(),
            Self::Level16(b) => b.bytes_size(),
            Self::Level254(b) => b.bytes_size(),
            Self::Level255(b) => b.bytes_size(),
            Self::Reserved(b) => b.bytes_size(),
        }
    }

    pub fn length_bits(&self) -> u64 {
        match self {
            Self::Level1(b) => b.bits_size(),
            Self::Level2(b) => b.bits_size(),
            Self::Level3(b) => b.bits_size(),
            Self::Level4(b) => b.bits_size(),
            Self::Level5(b) => b.bits_size(),
            Self::Level6(b) => b.bits_size(),
            Self::Level8(b) => b.bits_size(),
            Self::Level9(b) => b.bits_size(),
            Self::Level10(b) => b.bits_size(),
            Self::Level11(b) => b.bits_size(),
            Self::Level15(b) => b.bits_size(),
            Self::Level16(b) => b.bits_size(),
            Self::Level254(b) => b.bits_size(),
            Self::Level255(b) => b.bits_size(),
            Self::Reserved(b) => b.bits_size(),
        }
    }

    pub fn required_bits(&self) -> u64 {
        match self {
            Self::Level1(b) => b.required_bits(),
            Self::Level2(b) => b.required_bits(),
            Self::Level3(b) => b.required_bits(),
            Self::Level4(b) => b.required_bits(),
            Self::Level5(b) => b.required_bits(),
            Self::Level6(b) => b.required_bits(),
            Self::Level8(b) => b.required_bits(),
            Self::Level9(b) => b.required_bits(),
            Self::Level10(b) => b.required_bits(),
            Self::Level11(b) => b.required_bits(),
            Self::Level15(b) => b.required_bits(),
            Self::Level16(b) => b.required_bits(),
            Self::Level254(b) => b.required_bits(),
            Self::Level255(b) => b.required_bits(),
            Self::Reserved(b) => b.required_bits(),
        }
    }

    pub fn level(&self) -> u8 {
        match self {
            Self::Level1(b) => b.level(),
            Self::Level2(b) => b.level(),
            Self::Level3(b) => b.level(),
            Self::Level4(b) => b.level(),
            Self::Level5(b) => b.level(),
            Self::Level6(b) => b.level(),
            Self::Level8(b) => b.level(),
            Self::Level9(b) => b.level(),
            Self::Level10(b) => b.level(),
            Self::Level11(b) => b.level(),
            Self::Level15(b) => b.level(),
            Self::Level16(b) => b.level(),
            Self::Level254(b) => b.level(),
            Self::Level255(b) => b.level(),
            Self::Reserved(b) => b.level(),
        }
    }

    pub fn sort_key(&self) -> (u8, u16) {
        match self {
            Self::Level1(b) => b.sort_key(),
            Self::Level2(b) => b.sort_key(),
            Self::Level3(b) => b.sort_key(),
            Self::Level4(b) => b.sort_key(),
            Self::Level5(b) => b.sort_key(),
            Self::Level6(b) => b.sort_key(),
            Self::Level8(b) => b.sort_key(),
            Self::Level9(b) => b.sort_key(),
            Self::Level10(b) => b.sort_key(),
            Self::Level11(b) => b.sort_key(),
            Self::Level15(b) => b.sort_key(),
            Self::Level16(b) => b.sort_key(),
            Self::Level254(b) => b.sort_key(),
            Self::Level255(b) => b.sort_key(),
            Self::Reserved(b) => b.sort_key(),
        }
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        match self {
            Self::Level1(b) => b.write(writer),
            Self::Level2(b) => b.write(writer),
            Self::Level3(b) => b.write(writer),
            Self::Level4(b) => b.write(writer),
            Self::Level5(b) => b.write(writer),
            Self::Level6(b) => b.write(writer),
            Self::Level8(b) => b.write(writer),
            Self::Level9(b) => b.write(writer),
            Self::Level10(b) => b.write(writer),
            Self::Level11(b) => b.write(writer),
            Self::Level15(b) => b.write(writer),
            Self::Level16(b) => b.write(writer),
            Self::Level254(b) => b.write(writer),
            Self::Level255(b) => b.write(writer),
            Self::Reserved(b) => b.write(writer),
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

    pub(crate) fn validate_and_read_remaining<T: WithExtMetadataBlocks>(
        &self,
        reader: &mut BsIoSliceReader,
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
                !reader.read_bit()?,
                format!("{}: ext_dm_alignment_zero_bit != 0", T::VERSION)
            );
        }

        Ok(())
    }
}
