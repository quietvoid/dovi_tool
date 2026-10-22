use anyhow::{Result, bail, ensure};
use bitvec_helpers::bitstream_io_reader::BsIoSliceReader;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{WithExtMetadataBlocks, blocks::*};

pub(crate) const CM_V29_SUPPORTED_LEVELS: [u8; 6] = [1, 2, 4, 5, 6, 255];
pub(crate) const CM_V29_MAX_ALLOWED_BLOCKS: usize = {
    let mut i = 0;
    let mut sum = 0;

    while i < CM_V29_SUPPORTED_LEVELS.len() {
        let level = CM_V29_SUPPORTED_LEVELS[i];
        sum += cm_v29_max_num_blocks_for_level(level).unwrap();
        i += 1;
    }

    sum
};

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct CmV29DmData {
    num_ext_blocks: u64,
    ext_metadata_blocks: Vec<ExtMetadataBlock>,
}

impl WithExtMetadataBlocks for CmV29DmData {
    const VERSION: &'static str = "CM v2.9";
    const ALLOWED_BLOCK_LEVELS: &'static [u8] = &CM_V29_SUPPORTED_LEVELS;
    const MAX_ALLOWED_BLOCKS: usize = CM_V29_MAX_ALLOWED_BLOCKS;

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
        let ext_block_level = reader.read::<8, u8>()?;

        let ext_metadata_block = match ext_block_level {
            1 => level1::ExtMetadataBlockLevel1::parse(reader)?,
            2 => level2::ExtMetadataBlockLevel2::parse(reader)?,
            4 => level4::ExtMetadataBlockLevel4::parse(reader)?,
            5 => level5::ExtMetadataBlockLevel5::parse(reader)?,
            6 => level6::ExtMetadataBlockLevel6::parse(reader)?,
            255 => level255::ExtMetadataBlockLevel255::parse(reader)?,
            3 | 8 | 9 | 10 | 11 | 254 => bail!("Disallowed block level {}", ext_block_level),
            _ => {
                // FIXME: This returns early so the parsing doesn't actually take place
                ensure!(
                    false,
                    format!(
                        "Unknown metadata block found: Level {}, length {}, please open an issue.",
                        ext_block_level, ext_block_length
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

        ensure!(
            blocks.len() <= CM_V29_MAX_ALLOWED_BLOCKS,
            "cannot have more than {CM_V29_MAX_ALLOWED_BLOCKS} extension metadata blocks"
        );

        let invalid_blocks_count = blocks
            .iter()
            .filter(|b| !CM_V29_SUPPORTED_LEVELS.contains(&b.level()))
            .count();

        ensure!(
            invalid_blocks_count == 0,
            "Only allowed blocks level {:?}",
            CM_V29_SUPPORTED_LEVELS
        );

        let counts = CM_V29_SUPPORTED_LEVELS.iter().filter_map(|&level| {
            let count = blocks.iter().filter(|b| b.level() == level).count();
            (count > 0).then_some((level, count))
        });

        for (level, count) in counts {
            let Some(max) = cm_v29_max_num_blocks_for_level(level) else {
                bail!("invalid level {level}");
            };

            ensure!(
                count <= max,
                "there must be at most {max} level {level} blocks, found {count}"
            );
        }

        Ok(())
    }
}

const fn cm_v29_max_num_blocks_for_level(level: u8) -> Option<usize> {
    let count = match level {
        1 => 1,
        2 => 8,
        4 => 1,
        5 => 1,
        6 => 1,
        255 => 1,
        _ => return None,
    };

    Some(count)
}

#[cfg(test)]
mod tests {
    use super::{
        CM_V29_MAX_ALLOWED_BLOCKS, CM_V29_SUPPORTED_LEVELS, cm_v29_max_num_blocks_for_level,
    };

    #[test]
    fn validate_cm_v29_constants() {
        assert_eq!(CM_V29_SUPPORTED_LEVELS, [1, 2, 4, 5, 6, 255]);
        assert_eq!(CM_V29_MAX_ALLOWED_BLOCKS, 13);

        assert_eq!(cm_v29_max_num_blocks_for_level(1).unwrap(), 1);
        assert_eq!(cm_v29_max_num_blocks_for_level(2).unwrap(), 8);
        assert_eq!(cm_v29_max_num_blocks_for_level(4).unwrap(), 1);
        assert_eq!(cm_v29_max_num_blocks_for_level(5).unwrap(), 1);
        assert_eq!(cm_v29_max_num_blocks_for_level(6).unwrap(), 1);
        assert_eq!(cm_v29_max_num_blocks_for_level(255).unwrap(), 1);
        assert_eq!(cm_v29_max_num_blocks_for_level(0), None);
    }
}
