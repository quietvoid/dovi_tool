use anyhow::{ensure, Result};
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use crate::st2094_10::{level254, level3, level8, level9, reserved, ExtMetadataBlock};

#[derive(Debug, Default)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct CmV4DmData {
    pub num_ext_blocks: u64,
    pub ext_metadata_blocks: Vec<ExtMetadataBlock>,
}

impl CmV4DmData {
    pub fn parse(reader: &mut BitVecReader) -> Result<CmV4DmData> {
        let mut meta = CmV4DmData {
            num_ext_blocks: reader.get_ue()?,
            ..Default::default()
        };

        if meta.num_ext_blocks > 0 {
            while !reader.is_aligned() {
                ensure!(!reader.get()?, "CMv4: dm_alignment_zero_bit != 0");
            }

            for _ in 0..meta.num_ext_blocks {
                let ext_block_length = reader.get_ue()?;
                let ext_block_level: u8 = reader.get_n(8);
                
                let ext_metadata_block = match ext_block_level {
                    3 => level3::ExtMetadataBlockLevel3::parse(reader),
                    8 => level8::ExtMetadataBlockLevel8::parse(reader),
                    9 => level9::ExtMetadataBlockLevel9::parse(reader),
                    254 => level254::ExtMetadataBlockLevel254::parse(reader),
                    _ => {
                        ensure!(
                            false,
                            format!("CMv4.0 - Unknown metadata block found: Level {}, length {}, please open an issue.", ext_block_level, ext_block_length)
                        );

                        reserved::ReservedExtMetadataBlock::parse(
                            ext_block_length,
                            ext_block_level,
                            reader,
                        )?
                    }
                };

                let ext_block_use_bits = ext_metadata_block.length_bits() - ext_metadata_block.required_bits();

                for _ in 0..ext_block_use_bits {
                    ensure!(!reader.get()?, "CMv4: ext_dm_alignment_zero_bit != 0");
                }

                meta.ext_metadata_blocks.push(ext_metadata_block);
            }
        }

        Ok(meta)
    }

    pub fn write(&self, writer: &mut BitVecWriter) {
        writer.write_ue(self.num_ext_blocks);

        if self.num_ext_blocks > 0 {
            // dm_alignment_zero_bit
            while !writer.is_aligned() {
                writer.write(false);
            }

            for ext_metadata_block in &self.ext_metadata_blocks {
                let remaining_bits =
                    ext_metadata_block.length_bits() - ext_metadata_block.required_bits();

                writer.write_ue(ext_metadata_block.length_bytes());
                writer.write_n(&ext_metadata_block.level().to_be_bytes(), 8);

                ext_metadata_block.write(writer);

                // ext_dm_alignment_zero_bit
                (0..remaining_bits).for_each(|_| writer.write(false));
            }
        }
    }
}
