use anyhow::{Result, bail, ensure};
use bitvec_helpers::bitstream_io_reader::BsIoSliceReader;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::WithExtMetadataBlocks;
use crate::rpu::extension_metadata::blocks::*;

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct CmV40DmData {
    num_ext_blocks: u64,
    ext_metadata_blocks: Vec<ExtMetadataBlock>,
}

impl WithExtMetadataBlocks for CmV40DmData {
    const VERSION: &'static str = "CM v4.0";
    const ALLOWED_BLOCK_LEVELS: &'static [u8] = &[3, 8, 9, 10, 11, 15, 254];

    fn with_blocks_allocation(num_ext_blocks: u64) -> Self {
        Self {
            ext_metadata_blocks: Vec::with_capacity(num_ext_blocks as usize),
            ..Default::default()
        }
    }

    fn set_num_ext_blocks(&mut self, num_ext_blocks: u64) {
        self.num_ext_blocks = num_ext_blocks;
    }

    fn num_ext_blocks(&self) -> u64 {
        self.num_ext_blocks
    }

    fn blocks_ref(&self) -> &Vec<ExtMetadataBlock> {
        self.ext_metadata_blocks.as_ref()
    }

    fn blocks_mut(&mut self) -> &mut Vec<ExtMetadataBlock> {
        self.ext_metadata_blocks.as_mut()
    }

    fn parse_block(&mut self, reader: &mut BsIoSliceReader) -> Result<()> {
        let ext_block_length = reader.read_ue()?;
        let ext_block_level: u8 = reader.read::<8, u8>()?;

        let ext_metadata_block = match ext_block_level {
            3 => level3::ExtMetadataBlockLevel3::parse(reader)?,
            8 => level8::ExtMetadataBlockLevel8::parse(reader, ext_block_length)?,
            9 => level9::ExtMetadataBlockLevel9::parse(reader, ext_block_length)?,
            10 => level10::ExtMetadataBlockLevel10::parse(reader, ext_block_length)?,
            11 => level11::ExtMetadataBlockLevel11::parse(reader)?,
            15 => level15::ExtMetadataBlockLevel15::parse(reader)?,
            254 => level254::ExtMetadataBlockLevel254::parse(reader)?,
            1 | 2 | 4 | 5 | 6 | 255 => bail!(
                "Invalid block level {} for {} RPU",
                ext_block_level,
                Self::VERSION,
            ),
            _ => {
                ensure!(
                    false,
                    format!(
                        "{} - Unknown metadata block found: Level {}, length {}, please open an issue.",
                        Self::VERSION,
                        ext_block_level,
                        ext_block_length
                    )
                );

                reserved::ReservedExtMetadataBlock::parse(
                    ext_block_length,
                    ext_block_level,
                    reader,
                )?
            }
        };

        ext_metadata_block.validate_and_read_remaining::<Self>(reader, ext_block_length)?;

        self.ext_metadata_blocks.push(ext_metadata_block);

        Ok(())
    }
}

impl CmV40DmData {
    pub fn replace_level8_block(&mut self, block: &ExtMetadataBlockLevel8) {
        let blocks = self.blocks_mut();

        let existing_idx = blocks.iter().position(|b| match b {
            ExtMetadataBlock::Level8(b) => b.target_display_index == block.target_display_index,
            _ => false,
        });

        // Replace or add level 8 block
        if let Some(i) = existing_idx {
            blocks[i] = ExtMetadataBlock::Level8(block.clone());
        } else {
            blocks.push(ExtMetadataBlock::Level8(block.clone()));
        }

        self.update_extension_block_info();
    }

    pub fn replace_level10_block(&mut self, block: &ExtMetadataBlockLevel10) {
        let blocks = self.blocks_mut();

        let existing_idx = blocks.iter().position(|b| match b {
            ExtMetadataBlock::Level10(b) => b.target_display_index == block.target_display_index,
            _ => false,
        });

        // Replace or add level 10 block
        if let Some(i) = existing_idx {
            blocks[i] = ExtMetadataBlock::Level10(block.clone());
        } else {
            blocks.push(ExtMetadataBlock::Level10(block.clone()));
        }

        self.update_extension_block_info();
    }

    /// Validates different level block counts.
    /// The specification requires one block of L254 metadata
    pub fn validate(&self) -> Result<()> {
        let blocks = self.blocks_ref();

        let invalid_blocks_count = blocks
            .iter()
            .filter(|b| !Self::ALLOWED_BLOCK_LEVELS.contains(&b.level()))
            .count();

        let level254_count = blocks.iter().filter(|b| b.level() == 254).count();

        let level3_count = blocks.iter().filter(|b| b.level() == 3).count();

        let level8_count = blocks.iter().filter(|b| b.level() == 8).count();

        let level9_count = blocks.iter().filter(|b| b.level() == 9).count();

        let level10_count = blocks.iter().filter(|b| b.level() == 10).count();

        let level11_count = blocks.iter().filter(|b| b.level() == 11).count();

        ensure!(
            invalid_blocks_count == 0,
            format!(
                "{}: Only allowed blocks level 3, 8, 9, 10, 11 and 254",
                Self::VERSION
            )
        );

        ensure!(
            level254_count == 1,
            format!("{}: There must be one L254 metadata block", Self::VERSION)
        );

        ensure!(
            level3_count <= 1,
            format!(
                "{}: There must be at most one L3 metadata block",
                Self::VERSION
            )
        );
        ensure!(
            level8_count <= 5,
            format!(
                "{}: There must be at most 5 L8 metadata blocks",
                Self::VERSION
            )
        );
        ensure!(
            level9_count <= 1,
            format!(
                "{}: There must be at most one L9 metadata block",
                Self::VERSION
            )
        );
        ensure!(
            level10_count <= 4,
            format!(
                "{}: There must be at most 4 L10 metadata blocks",
                Self::VERSION
            )
        );
        ensure!(
            level11_count <= 1,
            format!(
                "{}: There must be at most one L11 metadata block",
                Self::VERSION
            )
        );

        Ok(())
    }

    pub fn new_with_l254_402() -> Self {
        Self {
            num_ext_blocks: 1,
            ext_metadata_blocks: vec![ExtMetadataBlock::Level254(
                ExtMetadataBlockLevel254::cmv402_default(),
            )],
        }
    }

    pub fn new_with_custom_l254(level254: &ExtMetadataBlockLevel254) -> Self {
        Self {
            num_ext_blocks: 1,
            ext_metadata_blocks: vec![ExtMetadataBlock::Level254(level254.clone())],
        }
    }
}
