use anyhow::{bail, ensure, Result};
use bitvec_helpers::bitvec_reader::BitVecReader;

#[cfg(feature = "serde_feature")]
use serde::{Deserialize, Serialize};

use super::WithExtMetadataBlocks;
use crate::rpu::extension_metadata::blocks::*;

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Deserialize, Serialize))]
pub struct CmV29DmData {
    num_ext_blocks: u64,
    ext_metadata_blocks: Vec<ExtMetadataBlock>,
}

impl WithExtMetadataBlocks for CmV29DmData {
    const VERSION: &'static str = "CM v2.9";
    const ALLOWED_BLOCK_LEVELS: &'static [u8] = &[1, 2, 4, 5, 6, 255];

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

    fn parse_block(&mut self, reader: &mut BitVecReader) -> Result<()> {
        let ext_block_length = reader.get_ue()?;
        let ext_block_level = reader.get_n(8);

        let ext_metadata_block = match ext_block_level {
            1 => level1::ExtMetadataBlockLevel1::parse(reader),
            2 => level2::ExtMetadataBlockLevel2::parse(reader),
            4 => level4::ExtMetadataBlockLevel4::parse(reader),
            5 => level5::ExtMetadataBlockLevel5::parse(reader),
            6 => level6::ExtMetadataBlockLevel6::parse(reader),
            255 => level255::ExtMetadataBlockLevel255::parse(reader),
            3 | 8 | 10 | 11 | 254 => bail!(
                "Invalid block level {} for {} RPU",
                ext_block_level,
                Self::VERSION,
            ),
            _ => {
                ensure!(
                    false,
                    format!("{} - Unknown metadata block found: Level {}, length {}, please open an issue.", Self::VERSION, ext_block_level, ext_block_length)
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

impl CmV29DmData {
    pub fn replace_level2_block(&mut self, block: &ExtMetadataBlockLevel2) {
        let blocks = self.blocks_mut();

        let existing_idx = blocks.iter().position(|b| match b {
            ExtMetadataBlock::Level2(b) => b.target_max_pq == block.target_max_pq,
            _ => false,
        });

        // Replace or add level 2 block
        if let Some(i) = existing_idx {
            blocks[i] = ExtMetadataBlock::Level2(block.clone());
        } else {
            blocks.push(ExtMetadataBlock::Level2(block.clone()));
        }

        self.update_extension_block_info();
    }

    /// Validates different level block counts.
    /// The specification requires one block of L1, L4, L5, L6 and L255.
    /// However they are not really required, so YMMV.
    pub fn validate(&self) -> Result<()> {
        let blocks = self.blocks_ref();

        let invalid_blocks_count = blocks
            .iter()
            .filter(|b| !Self::ALLOWED_BLOCK_LEVELS.contains(&b.level()))
            .count();

        let level1_count = blocks.iter().filter(|b| b.level() == 1).count();

        let level2_count = blocks.iter().filter(|b| b.level() == 2).count();

        let level255_count = blocks.iter().filter(|b| b.level() == 255).count();

        let level4_count = blocks.iter().filter(|b| b.level() == 4).count();

        let level5_count = blocks.iter().filter(|b| b.level() == 5).count();

        let level6_count = blocks.iter().filter(|b| b.level() == 6).count();

        ensure!(
            invalid_blocks_count == 0,
            format!(
                "{}: Only allowed blocks level 1, 2, 4, 5, 6, and 255",
                Self::VERSION
            )
        );

        ensure!(
            level1_count <= 1,
            format!(
                "{}: There must be at most one L1 metadata block",
                Self::VERSION
            )
        );
        ensure!(
            level2_count <= 8,
            format!(
                "{}: There must be at most 8 L2 metadata blocks",
                Self::VERSION
            )
        );
        ensure!(
            level255_count <= 1,
            format!(
                "{}: There must be at most one L255 metadata block",
                Self::VERSION
            )
        );
        ensure!(
            level4_count <= 1,
            format!(
                "{}: There must be at most one L4 metadata block",
                Self::VERSION
            )
        );
        ensure!(
            level5_count <= 1,
            format!(
                "{}: There must be at most one L5 metadata block",
                Self::VERSION
            )
        );
        ensure!(
            level6_count <= 1,
            format!(
                "{}: There must be at most one L6 metadata block",
                Self::VERSION
            )
        );

        Ok(())
    }
}
