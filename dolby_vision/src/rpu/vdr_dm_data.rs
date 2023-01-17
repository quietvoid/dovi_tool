use std::io;

use anyhow::{bail, ensure, Result};
use bitvec_helpers::{
    bitstream_io_reader::BitstreamIoReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::extension_metadata::blocks::{
    ExtMetadataBlock, ExtMetadataBlockLevel11, ExtMetadataBlockLevel9,
};
use super::extension_metadata::*;
use super::generate::{GenerateConfig, GenerateProfile};
use super::profiles::profile5::Profile5;
use super::profiles::profile81::Profile81;
use super::profiles::profile84::Profile84;
use super::profiles::DoviProfile;

use super::extension_metadata::WithExtMetadataBlocks;
use super::rpu_data_header::RpuDataHeader;

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VdrDmData {
    pub compressed: bool,

    pub affected_dm_metadata_id: u64,
    pub current_dm_metadata_id: u64,
    pub scene_refresh_flag: u64,

    pub ycc_to_rgb_coef0: i16,
    pub ycc_to_rgb_coef1: i16,
    pub ycc_to_rgb_coef2: i16,
    pub ycc_to_rgb_coef3: i16,
    pub ycc_to_rgb_coef4: i16,
    pub ycc_to_rgb_coef5: i16,
    pub ycc_to_rgb_coef6: i16,
    pub ycc_to_rgb_coef7: i16,
    pub ycc_to_rgb_coef8: i16,
    pub ycc_to_rgb_offset0: u32,
    pub ycc_to_rgb_offset1: u32,
    pub ycc_to_rgb_offset2: u32,
    pub rgb_to_lms_coef0: i16,
    pub rgb_to_lms_coef1: i16,
    pub rgb_to_lms_coef2: i16,
    pub rgb_to_lms_coef3: i16,
    pub rgb_to_lms_coef4: i16,
    pub rgb_to_lms_coef5: i16,
    pub rgb_to_lms_coef6: i16,
    pub rgb_to_lms_coef7: i16,
    pub rgb_to_lms_coef8: i16,
    pub signal_eotf: u16,
    pub signal_eotf_param0: u16,
    pub signal_eotf_param1: u16,
    pub signal_eotf_param2: u32,
    pub signal_bit_depth: u8,
    pub signal_color_space: u8,
    pub signal_chroma_format: u8,
    pub signal_full_range_flag: u8,
    pub source_min_pq: u16,
    pub source_max_pq: u16,
    pub source_diagonal: u16,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub cmv29_metadata: Option<DmData>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub cmv40_metadata: Option<DmData>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum CmVersion {
    V29,
    V40,
}

pub(crate) fn vdr_dm_data_payload<R: io::Read + io::Seek>(
    reader: &mut BitstreamIoReader<R>,
    header: &RpuDataHeader,
    final_length: u64,
) -> Result<VdrDmData> {
    let compressed_dm_data = header.reserved_zero_3bits == 1;

    let mut vdr_dm_data = if compressed_dm_data {
        VdrDmData {
            compressed: true,

            affected_dm_metadata_id: reader.get_ue()?,
            current_dm_metadata_id: reader.get_ue()?,
            scene_refresh_flag: reader.get_ue()?,
            ..Default::default()
        }
    } else {
        VdrDmData::parse(reader)?
    };

    if let Some(cmv29_dm_data) = DmData::parse::<CmV29DmData, R>(reader)? {
        vdr_dm_data.cmv29_metadata = Some(DmData::V29(cmv29_dm_data));
    }

    // 16 bits min for required level 254
    if reader.available()? >= final_length + 16 {
        if let Some(cmv40_dm_data) = DmData::parse::<CmV40DmData, R>(reader)? {
            vdr_dm_data.cmv40_metadata = Some(DmData::V40(cmv40_dm_data));
        }
    }

    Ok(vdr_dm_data)
}

impl VdrDmData {
    pub(crate) fn parse<R: io::Read + io::Seek>(
        reader: &mut BitstreamIoReader<R>,
    ) -> Result<VdrDmData> {
        let data = VdrDmData {
            affected_dm_metadata_id: reader.get_ue()?,
            current_dm_metadata_id: reader.get_ue()?,
            scene_refresh_flag: reader.get_ue()?,

            ycc_to_rgb_coef0: reader.get_n::<u16>(16)? as i16,
            ycc_to_rgb_coef1: reader.get_n::<u16>(16)? as i16,
            ycc_to_rgb_coef2: reader.get_n::<u16>(16)? as i16,
            ycc_to_rgb_coef3: reader.get_n::<u16>(16)? as i16,
            ycc_to_rgb_coef4: reader.get_n::<u16>(16)? as i16,
            ycc_to_rgb_coef5: reader.get_n::<u16>(16)? as i16,
            ycc_to_rgb_coef6: reader.get_n::<u16>(16)? as i16,
            ycc_to_rgb_coef7: reader.get_n::<u16>(16)? as i16,
            ycc_to_rgb_coef8: reader.get_n::<u16>(16)? as i16,
            ycc_to_rgb_offset0: reader.get_n(32)?,
            ycc_to_rgb_offset1: reader.get_n(32)?,
            ycc_to_rgb_offset2: reader.get_n(32)?,

            rgb_to_lms_coef0: reader.get_n::<u16>(16)? as i16,
            rgb_to_lms_coef1: reader.get_n::<u16>(16)? as i16,
            rgb_to_lms_coef2: reader.get_n::<u16>(16)? as i16,
            rgb_to_lms_coef3: reader.get_n::<u16>(16)? as i16,
            rgb_to_lms_coef4: reader.get_n::<u16>(16)? as i16,
            rgb_to_lms_coef5: reader.get_n::<u16>(16)? as i16,
            rgb_to_lms_coef6: reader.get_n::<u16>(16)? as i16,
            rgb_to_lms_coef7: reader.get_n::<u16>(16)? as i16,
            rgb_to_lms_coef8: reader.get_n::<u16>(16)? as i16,

            signal_eotf: reader.get_n(16)?,
            signal_eotf_param0: reader.get_n(16)?,
            signal_eotf_param1: reader.get_n(16)?,
            signal_eotf_param2: reader.get_n(32)?,
            signal_bit_depth: reader.get_n(5)?,
            signal_color_space: reader.get_n(2)?,
            signal_chroma_format: reader.get_n(2)?,
            signal_full_range_flag: reader.get_n(2)?,
            source_min_pq: reader.get_n(12)?,
            source_max_pq: reader.get_n(12)?,
            source_diagonal: reader.get_n(10)?,
            ..Default::default()
        };

        Ok(data)
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.affected_dm_metadata_id <= 15,
            "affected_dm_metadata_id should be <= 15"
        );

        // FIXME: Compressed DM metadata, should be set from a state somehow
        if !self.compressed {
            ensure!(
                self.signal_bit_depth >= 8 && self.signal_bit_depth <= 16,
                "signal_bit_depth should be between 8 and 16"
            );

            if self.signal_eotf_param0 == 0
                && self.signal_eotf_param1 == 0
                && self.signal_eotf_param2 == 0
            {
                ensure!(self.signal_eotf == 65535, "signal_eotf should be 65535");
            }
        }

        if let Some(cmv29) = &self.cmv29_metadata {
            cmv29.validate()?;
        }

        if let Some(cmv40) = &self.cmv40_metadata {
            cmv40.validate()?;
        }

        Ok(())
    }

    pub fn write(&self, writer: &mut BitstreamIoWriter) -> Result<()> {
        writer.write_ue(&self.affected_dm_metadata_id)?;
        writer.write_ue(&self.current_dm_metadata_id)?;
        writer.write_ue(&self.scene_refresh_flag)?;

        if !self.compressed {
            writer.write_signed_n(&self.ycc_to_rgb_coef0, 16)?;
            writer.write_signed_n(&self.ycc_to_rgb_coef1, 16)?;
            writer.write_signed_n(&self.ycc_to_rgb_coef2, 16)?;
            writer.write_signed_n(&self.ycc_to_rgb_coef3, 16)?;
            writer.write_signed_n(&self.ycc_to_rgb_coef4, 16)?;
            writer.write_signed_n(&self.ycc_to_rgb_coef5, 16)?;
            writer.write_signed_n(&self.ycc_to_rgb_coef6, 16)?;
            writer.write_signed_n(&self.ycc_to_rgb_coef7, 16)?;
            writer.write_signed_n(&self.ycc_to_rgb_coef8, 16)?;

            writer.write_n(&self.ycc_to_rgb_offset0, 32)?;
            writer.write_n(&self.ycc_to_rgb_offset1, 32)?;
            writer.write_n(&self.ycc_to_rgb_offset2, 32)?;

            writer.write_signed_n(&self.rgb_to_lms_coef0, 16)?;
            writer.write_signed_n(&self.rgb_to_lms_coef1, 16)?;
            writer.write_signed_n(&self.rgb_to_lms_coef2, 16)?;
            writer.write_signed_n(&self.rgb_to_lms_coef3, 16)?;
            writer.write_signed_n(&self.rgb_to_lms_coef4, 16)?;
            writer.write_signed_n(&self.rgb_to_lms_coef5, 16)?;
            writer.write_signed_n(&self.rgb_to_lms_coef6, 16)?;
            writer.write_signed_n(&self.rgb_to_lms_coef7, 16)?;
            writer.write_signed_n(&self.rgb_to_lms_coef8, 16)?;

            writer.write_n(&self.signal_eotf, 16)?;
            writer.write_n(&self.signal_eotf_param0, 16)?;
            writer.write_n(&self.signal_eotf_param1, 16)?;
            writer.write_n(&self.signal_eotf_param2, 32)?;

            writer.write_n(&self.signal_bit_depth, 5)?;
            writer.write_n(&self.signal_color_space, 2)?;
            writer.write_n(&self.signal_chroma_format, 2)?;
            writer.write_n(&self.signal_full_range_flag, 2)?;

            writer.write_n(&self.source_min_pq, 12)?;
            writer.write_n(&self.source_max_pq, 12)?;
            writer.write_n(&self.source_diagonal, 10)?;
        }

        if let Some(cmv29) = &self.cmv29_metadata {
            cmv29.write(writer)?;
        }

        if let Some(cmv40) = &self.cmv40_metadata {
            cmv40.write(writer)?;
        }

        Ok(())
    }

    pub fn extension_metadata_for_level(&self, level: u8) -> Option<&DmData> {
        if CmV29DmData::ALLOWED_BLOCK_LEVELS.contains(&level) {
            return self.cmv29_metadata.as_ref();
        } else if CmV40DmData::ALLOWED_BLOCK_LEVELS.contains(&level) {
            return self.cmv40_metadata.as_ref();
        }

        None
    }

    pub fn extension_metadata_for_level_mut(&mut self, level: u8) -> Option<&mut DmData> {
        if CmV29DmData::ALLOWED_BLOCK_LEVELS.contains(&level) {
            return self.cmv29_metadata.as_mut();
        } else if CmV40DmData::ALLOWED_BLOCK_LEVELS.contains(&level) {
            return self.cmv40_metadata.as_mut();
        }

        None
    }

    pub fn metadata_blocks(&self, level: u8) -> Option<&Vec<ExtMetadataBlock>> {
        self.extension_metadata_for_level(level)
            .map(|dm_data| match dm_data {
                DmData::V29(meta) => meta.blocks_ref(),
                DmData::V40(meta) => meta.blocks_ref(),
            })
    }

    pub fn metadata_blocks_mut(&mut self, level: u8) -> Option<&mut Vec<ExtMetadataBlock>> {
        self.extension_metadata_for_level_mut(level)
            .map(|dm_data| match dm_data {
                DmData::V29(meta) => meta.blocks_mut(),
                DmData::V40(meta) => meta.blocks_mut(),
            })
    }

    pub fn level_blocks_iter(&self, level: u8) -> impl Iterator<Item = &ExtMetadataBlock> {
        self.metadata_blocks(level)
            .into_iter()
            .flat_map(|e| e.iter())
            .filter(move |e| e.level() == level)
    }

    pub fn level_blocks_iter_mut(
        &mut self,
        level: u8,
    ) -> impl Iterator<Item = &mut ExtMetadataBlock> {
        self.metadata_blocks_mut(level)
            .into_iter()
            .flat_map(|e| e.iter_mut())
            .filter(move |e| e.level() == level)
    }

    pub fn get_block(&self, level: u8) -> Option<&ExtMetadataBlock> {
        self.level_blocks_iter(level).next()
    }

    pub fn get_block_mut(&mut self, level: u8) -> Option<&mut ExtMetadataBlock> {
        self.level_blocks_iter_mut(level).next()
    }

    pub fn add_metadata_block(&mut self, block: ExtMetadataBlock) -> Result<()> {
        let level = block.level();

        if let Some(dm_data) = self.extension_metadata_for_level_mut(level) {
            match dm_data {
                DmData::V29(meta) => meta.add_block(block)?,
                DmData::V40(meta) => meta.add_block(block)?,
            }
        }

        Ok(())
    }

    pub fn remove_metadata_level(&mut self, level: u8) {
        if let Some(dm_data) = self.extension_metadata_for_level_mut(level) {
            match dm_data {
                DmData::V29(meta) => meta.remove_level(level),
                DmData::V40(meta) => meta.remove_level(level),
            }
        }
    }

    pub fn replace_metadata_level(&mut self, block: ExtMetadataBlock) -> Result<()> {
        let level = block.level();

        self.remove_metadata_level(level);
        self.add_metadata_block(block)?;

        Ok(())
    }

    pub fn replace_metadata_block(&mut self, block: ExtMetadataBlock) -> Result<()> {
        let level = block.level();

        match &block {
            ExtMetadataBlock::Level1(_) => self.replace_metadata_level(block),
            ExtMetadataBlock::Level2(level2) => {
                if let Some(dm_data) = self.extension_metadata_for_level_mut(level) {
                    match dm_data {
                        DmData::V29(cmv29) => cmv29.replace_level2_block(level2),
                        _ => unreachable!(),
                    };

                    Ok(())
                } else {
                    bail!("Cannot replace L2 metadata, no CM v4.0 DM data")
                }
            }
            ExtMetadataBlock::Level3(_) => self.replace_metadata_level(block),
            ExtMetadataBlock::Level4(_) => self.replace_metadata_level(block),
            ExtMetadataBlock::Level5(_) => self.replace_metadata_level(block),
            ExtMetadataBlock::Level6(_) => self.replace_metadata_level(block),
            ExtMetadataBlock::Level8(level8) => {
                if let Some(dm_data) = self.extension_metadata_for_level_mut(level) {
                    match dm_data {
                        DmData::V40(cmv40) => cmv40.replace_level8_block(level8),
                        _ => unreachable!(),
                    };

                    Ok(())
                } else {
                    bail!("Cannot replace L8 metadata, no CM v4.0 DM data")
                }
            }
            ExtMetadataBlock::Level9(_) => self.replace_metadata_level(block),
            ExtMetadataBlock::Level10(level10) => {
                if let Some(dm_data) = self.extension_metadata_for_level_mut(level) {
                    match dm_data {
                        DmData::V40(cmv40) => cmv40.replace_level10_block(level10),
                        _ => unreachable!(),
                    };

                    Ok(())
                } else {
                    bail!("Cannot replace L10 metadata, no CM v4.0 DM data")
                }
            }
            ExtMetadataBlock::Level11(_) => self.replace_metadata_level(block),
            ExtMetadataBlock::Level254(_) => self.replace_metadata_level(block),
            ExtMetadataBlock::Level255(_) => self.replace_metadata_level(block),
            ExtMetadataBlock::Reserved(_) => bail!("Cannot replace specific reserved block"),
        }
    }

    /// Clones every block to replace
    pub fn replace_metadata_blocks<'a, I>(&mut self, blocks: I) -> Result<()>
    where
        I: Iterator<Item = &'a ExtMetadataBlock>,
    {
        for block in blocks {
            self.replace_metadata_block(block.clone())?;
        }

        Ok(())
    }

    pub fn set_p81_coeffs(&mut self) {
        self.ycc_to_rgb_coef0 = 9574;
        self.ycc_to_rgb_coef1 = 0;
        self.ycc_to_rgb_coef2 = 13802;
        self.ycc_to_rgb_coef3 = 9574;
        self.ycc_to_rgb_coef4 = -1540;
        self.ycc_to_rgb_coef5 = -5348;
        self.ycc_to_rgb_coef6 = 9574;
        self.ycc_to_rgb_coef7 = 17610;
        self.ycc_to_rgb_coef8 = 0;
        self.ycc_to_rgb_offset0 = 16777216;
        self.ycc_to_rgb_offset1 = 134217728;
        self.ycc_to_rgb_offset2 = 134217728;

        self.rgb_to_lms_coef0 = 7222;
        self.rgb_to_lms_coef1 = 8771;
        self.rgb_to_lms_coef2 = 390;
        self.rgb_to_lms_coef3 = 2654;
        self.rgb_to_lms_coef4 = 12430;
        self.rgb_to_lms_coef5 = 1300;
        self.rgb_to_lms_coef6 = 0;
        self.rgb_to_lms_coef7 = 422;
        self.rgb_to_lms_coef8 = 15962;

        self.signal_color_space = 0;
    }

    // Source PQ means the mastering display
    // MDL 1000,1-10 = 7,3079
    // MDL 4000,50   = 62,3696
    pub fn change_source_levels(&mut self, min_pq: Option<u16>, max_pq: Option<u16>) {
        if let Some(v) = min_pq {
            self.source_min_pq = v;
        }

        if let Some(v) = max_pq {
            self.source_max_pq = v;
        }

        if let Some(ExtMetadataBlock::Level6(level6_block)) = self.get_block(6) {
            let (derived_min_pq, derived_max_pq) = level6_block.source_meta_from_l6();

            if self.source_min_pq == 0 {
                self.source_min_pq = derived_min_pq;
            }

            if self.source_max_pq == 0 {
                self.source_max_pq = derived_max_pq;
            }
        }
    }

    pub fn set_scene_cut(&mut self, is_scene_cut: bool) {
        self.scene_refresh_flag = is_scene_cut as u64;
    }

    pub fn default_pq() -> VdrDmData {
        VdrDmData {
            signal_eotf: 65535,
            signal_bit_depth: 12,
            signal_full_range_flag: 1,
            source_diagonal: 42,
            ..Default::default()
        }
    }

    /// Sets static metadata (L5/L6/L11) and source levels
    pub fn from_generate_config(config: &GenerateConfig) -> Result<VdrDmData> {
        let mut vdr_dm_data = match config.profile {
            GenerateProfile::Profile5 => Profile5::dm_data(),
            GenerateProfile::Profile81 => Profile81::dm_data(),
            GenerateProfile::Profile84 => Profile84::dm_data(),
        };

        match config.cm_version {
            CmVersion::V29 => {
                vdr_dm_data.cmv29_metadata = Some(DmData::V29(CmV29DmData::default()))
            }
            CmVersion::V40 => {
                vdr_dm_data.cmv29_metadata = Some(DmData::V29(CmV29DmData::default()));

                vdr_dm_data.cmv40_metadata = if let Some(level254) = &config.level254 {
                    Some(DmData::V40(CmV40DmData::new_with_custom_l254(level254)))
                } else {
                    Some(DmData::V40(CmV40DmData::new_with_l254_402()))
                }
            }
        }

        vdr_dm_data.set_static_metadata(config)?;
        vdr_dm_data.change_source_levels(config.source_min_pq, config.source_max_pq);

        Ok(vdr_dm_data)
    }

    pub fn set_static_metadata(&mut self, config: &GenerateConfig) -> Result<()> {
        self.replace_metadata_block(ExtMetadataBlock::Level5(config.level5.clone()))?;

        if let Some(level6) = &config.level6 {
            self.replace_metadata_block(ExtMetadataBlock::Level6(level6.clone()))?;
        }

        // Default to inserting both L9 (required) and L11 metadata
        self.replace_metadata_block(ExtMetadataBlock::Level9(
            ExtMetadataBlockLevel9::default_dci_p3(),
        ))?;
        self.replace_metadata_block(ExtMetadataBlock::Level11(
            ExtMetadataBlockLevel11::default_reference_cinema(),
        ))?;

        if !config.default_metadata_blocks.is_empty() {
            const LEVEL_BLOCK_LIST: &[u8] = &[5, 6];

            let allowed_default_blocks = config
                .default_metadata_blocks
                .iter()
                .filter(|block| !LEVEL_BLOCK_LIST.contains(&block.level()));

            for block in allowed_default_blocks {
                self.replace_metadata_block(block.clone())?;
            }
        }

        Ok(())
    }
}

impl CmVersion {
    pub fn v29() -> Self {
        CmVersion::V29
    }

    pub fn v40() -> Self {
        CmVersion::V40
    }
}
