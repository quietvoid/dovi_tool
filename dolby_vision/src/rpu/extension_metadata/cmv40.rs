use anyhow::{Result, bail, ensure};
use bitvec_helpers::bitstream_io_reader::BsIoSliceReader;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{WithExtMetadataBlocks, blocks::*};

pub(crate) const CM_V40_SUPPORTED_LEVELS: [u8; 7] = [3, 8, 9, 10, 11, 253, 254];
pub(crate) const CM_V40_MAX_ALLOWED_BLOCKS: usize = {
    let mut i = 0;
    let mut sum = 0;

    while i < CM_V40_SUPPORTED_LEVELS.len() {
        let level = CM_V40_SUPPORTED_LEVELS[i];
        sum += cm_v40_max_num_blocks_for_level(level).unwrap();
        i += 1;
    }

    sum
};

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct CmV40DmData {
    num_ext_blocks: u64,
    ext_metadata_blocks: Vec<ExtMetadataBlock>,
}

impl WithExtMetadataBlocks for CmV40DmData {
    const VERSION: &'static str = "CM v4.0";
    const ALLOWED_BLOCK_LEVELS: &'static [u8] = &CM_V40_SUPPORTED_LEVELS;
    const MAX_ALLOWED_BLOCKS: usize = CM_V40_MAX_ALLOWED_BLOCKS;

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
            253 => level253::ExtMetadataBlockLevel253::parse(reader, ext_block_length)?,
            254 => level254::ExtMetadataBlockLevel254::parse(reader)?,
            1 | 2 | 4 | 5 | 6 | 255 => bail!("Disallowed block level {}", ext_block_level),
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

        ensure!(
            blocks.len() <= CM_V40_MAX_ALLOWED_BLOCKS,
            "cannot have more than {CM_V40_MAX_ALLOWED_BLOCKS} extension metadata blocks"
        );

        let invalid_blocks_count = blocks
            .iter()
            .filter(|b| !CM_V40_SUPPORTED_LEVELS.contains(&b.level()))
            .count();

        ensure!(
            invalid_blocks_count == 0,
            "Only allowed blocks level {:?}",
            CM_V40_SUPPORTED_LEVELS
        );

        let counts = CM_V40_SUPPORTED_LEVELS.into_iter().filter_map(|level| {
            let count = blocks.iter().filter(|b| b.level() == level).count();
            (count > 0).then_some((level, count))
        });

        for (level, count) in counts {
            let Some(max) = cm_v40_max_num_blocks_for_level(level) else {
                bail!("invalid level {level}");
            };

            ensure!(
                count <= max,
                "there must be at most {max} level {level} blocks, found {count}"
            );
        }

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

    /// Creates CMv4.0 DM data with default static blocks: L3, L9, L11, L254.
    pub fn default_safe() -> Self {
        let ext_metadata_blocks = vec![
            ExtMetadataBlock::Level3(ExtMetadataBlockLevel3::default()),
            ExtMetadataBlock::Level9(ExtMetadataBlockLevel9::default_dci_p3()),
            ExtMetadataBlock::Level254(ExtMetadataBlockLevel254::cmv402_default()),
            ExtMetadataBlock::Level11(ExtMetadataBlockLevel11::default_cinema()),
        ];

        Self {
            num_ext_blocks: ext_metadata_blocks.len() as u64,
            ext_metadata_blocks,
        }
    }

    pub fn new_with_custom_l254(level254: &ExtMetadataBlockLevel254) -> Self {
        Self {
            num_ext_blocks: 1,
            ext_metadata_blocks: vec![ExtMetadataBlock::Level254(level254.clone())],
        }
    }
}

const fn cm_v40_max_num_blocks_for_level(level: u8) -> Option<usize> {
    let count = match level {
        3 => 1,
        8 => 5,
        9 => 1,
        10 => 4,
        11 => 1,
        253 => 1,
        254 => 1,
        _ => return None,
    };

    Some(count)
}

#[cfg(test)]
mod tests {
    use super::{
        CM_V40_MAX_ALLOWED_BLOCKS, CM_V40_SUPPORTED_LEVELS, cm_v40_max_num_blocks_for_level,
    };

    #[test]
    fn validate_cm_v40_constants() {
        assert_eq!(CM_V40_SUPPORTED_LEVELS, [3, 8, 9, 10, 11, 253, 254]);
        assert_eq!(CM_V40_MAX_ALLOWED_BLOCKS, 14);

        assert_eq!(cm_v40_max_num_blocks_for_level(3).unwrap(), 1);
        assert_eq!(cm_v40_max_num_blocks_for_level(8).unwrap(), 5);
        assert_eq!(cm_v40_max_num_blocks_for_level(9).unwrap(), 1);
        assert_eq!(cm_v40_max_num_blocks_for_level(10).unwrap(), 4);
        assert_eq!(cm_v40_max_num_blocks_for_level(11).unwrap(), 1);
        assert_eq!(cm_v40_max_num_blocks_for_level(253).unwrap(), 1);
        assert_eq!(cm_v40_max_num_blocks_for_level(254).unwrap(), 1);
        assert_eq!(cm_v40_max_num_blocks_for_level(0), None);
    }
}
