use anyhow::{bail, ensure, Result};
use bitvec::prelude::*;
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};
use crc::{Crc, CRC_32_MPEG_2};
use hevc_parser::utils::{
    add_start_code_emulation_prevention_3_byte, clear_start_code_emulation_prevention_3_byte,
};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use super::rpu_data_header::RpuDataHeader;
use super::rpu_data_mapping::VdrRpuData;
use super::rpu_data_nlq::NlqData;
use super::vdr_dm_data::VdrDmData;

use crate::st2094_10::generate::GenerateConfig;
use crate::st2094_10::{ExtMetadataBlock, ExtMetadataBlockLevel5};

#[derive(Default, Debug)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct DoviRpu {
    pub dovi_profile: u8,

    #[cfg_attr(feature = "serde_feature", serde(skip_serializing))]
    pub reader: BitVecReader,

    pub header: RpuDataHeader,
    pub vdr_rpu_data: Option<VdrRpuData>,
    pub nlq_data: Option<NlqData>,
    pub vdr_dm_data: Option<VdrDmData>,

    #[cfg_attr(
        feature = "serde_feature",
        serde(serialize_with = "crate::utils::bitvec_ser_bits")
    )]
    pub remaining: BitVec<Msb0, u8>,
    pub rpu_data_crc32: u32,

    #[cfg_attr(feature = "serde_feature", serde(skip_serializing))]
    pub last_byte: u8,

    #[cfg_attr(feature = "serde_feature", serde(skip_serializing))]
    pub modified: bool,
}

impl DoviRpu {
    pub fn new(bytes: Vec<u8>) -> DoviRpu {
        DoviRpu {
            reader: BitVecReader::new(bytes),
            ..Default::default()
        }
    }

    #[inline(always)]
    pub fn parse(data: &[u8]) -> Result<DoviRpu> {
        if data.len() < 25 {
            bail!("Invalid RPU length\n{:?}", &data);
        }

        // Including 0x7C01 prepended
        let trimmed_data = match &data[..5] {
            [0, 0, 0, 1, 25] => &data[4..],
            [0, 0, 1, 25, 8] => &data[3..],
            [0, 1, 25, 8, 9] | [124, 1, 25, 8, 9] => &data[2..],
            [1, 25, 8, 9, _] => &data[1..],
            [25, 8, 9, _, _] => data,
            _ => bail!("Invalid RPU data start bytes\n{:?}", &data),
        };

        // Clear start code emulation prevention 3 byte
        let bytes: Vec<u8> = clear_start_code_emulation_prevention_3_byte(trimmed_data);

        let len = bytes.len();

        let mut received_crc32 = DoviRpu::compute_crc32(&bytes[1..len - 5]);
        let last_byte = bytes[len - 1];

        // Final RPU exception
        if last_byte == 0 && bytes[len - 2] == 0x80 {
            received_crc32 = DoviRpu::compute_crc32(&bytes[1..len - 6]);
        } else if last_byte != 0x80 {
            bail!("Invalid RPU \n{:?}", &bytes);
        }

        let mut dovi_rpu = DoviRpu::read_rpu_data(bytes, last_byte)?;

        if received_crc32 != dovi_rpu.rpu_data_crc32 {
            bail!(
                "RPU CRC32 does not match the data. Received: {}, expected {}",
                received_crc32,
                dovi_rpu.rpu_data_crc32
            );
        }

        dovi_rpu.dovi_profile = dovi_rpu.header.get_dovi_profile();

        Ok(dovi_rpu)
    }

    #[inline(always)]
    pub fn read_rpu_data(bytes: Vec<u8>, end_byte: u8) -> Result<DoviRpu> {
        let mut dovi_rpu = DoviRpu::new(bytes);
        dovi_rpu.last_byte = end_byte;

        dovi_rpu.header = RpuDataHeader::parse(&mut dovi_rpu.reader);

        // Preliminary header validation
        dovi_rpu.dovi_profile = dovi_rpu.header.get_dovi_profile();

        dovi_rpu.header.validate(dovi_rpu.dovi_profile)?;

        if dovi_rpu.header.rpu_type == 2 {
            if !dovi_rpu.header.use_prev_vdr_rpu_flag {
                VdrRpuData::parse(&mut dovi_rpu)?;
            }

            let mut reader = &mut dovi_rpu.reader;

            if dovi_rpu.header.vdr_dm_metadata_present_flag {
                dovi_rpu.vdr_dm_data = Some(VdrDmData::parse(&mut reader)?);
            }

            while !reader.is_aligned() {
                dovi_rpu.remaining.push(reader.get());
            }

            // EOF case
            let final_len = if end_byte == 0 { 48 } else { 40 };

            // CRC32 is at the end, apparently sometimes there is more unknown data
            if reader.available() != final_len {
                while reader.available() != final_len {
                    dovi_rpu.remaining.push(reader.get());
                }
            }

            dovi_rpu.rpu_data_crc32 = reader.get_n(32);

            let last_byte: u8 = reader.get_n(8);
            ensure!(last_byte == 0x80, "last byte should be 0x80");
        }

        dovi_rpu.validate()?;

        Ok(dovi_rpu)
    }

    fn convert_to_mel(&mut self) -> Result<()> {
        if let Some(ref mut nlq_data) = self.nlq_data {
            nlq_data.convert_to_mel();
        } else {
            bail!("Not profile 7, cannot convert to MEL!");
        }

        Ok(())
    }

    fn convert_to_81(&mut self) {
        let header = &mut self.header;

        // Change to 8.1
        header.el_spatial_resampling_filter_flag = false;
        header.disable_residual_flag = true;

        header.nlq_method_idc = None;
        header.nlq_num_pivots_minus2 = None;

        header.num_x_partitions_minus1 = 0;
        header.num_y_partitions_minus1 = 0;

        self.nlq_data = None;
    }

    #[inline(always)]
    pub fn write_rpu_data(&mut self) -> Result<Vec<u8>> {
        let mut writer = BitVecWriter::new();

        let header = &self.header;
        header.write_header(&mut writer);

        if header.rpu_type == 2 {
            if !header.use_prev_vdr_rpu_flag {
                self.write_vdr_rpu_data(&mut writer)?;
            }

            if header.vdr_dm_metadata_present_flag {
                self.write_vdr_dm_data(&mut writer);
            }
        }

        if !self.remaining.is_empty() {
            self.remaining.iter().for_each(|b| writer.write(*b));
        }

        // Since we edited, remaining is not accurate
        if self.modified {
            while !writer.is_aligned() {
                writer.write(false);
            }
        }

        let computed_crc32 = DoviRpu::compute_crc32(&writer.as_slice()[1..]);

        if !self.modified {
            // Validate the parsed crc32 is the same
            ensure!(
                self.rpu_data_crc32 == computed_crc32,
                "RPU CRC32 does not match computed value"
            );
        }

        // Write crc32
        writer.write_n(&computed_crc32.to_be_bytes(), 32);
        writer.write_n(&[0x80], 8);

        if self.last_byte != 0x80 {
            writer.write_n(&[self.last_byte], 8);
        }

        // Back to a u8 slice
        let mut data_to_write = writer.as_slice().to_vec();
        add_start_code_emulation_prevention_3_byte(&mut data_to_write);

        // Put back NAL unit type
        data_to_write.insert(0, 0x01);
        data_to_write.insert(0, 0x7C);

        Ok(data_to_write)
    }

    pub fn write_vdr_rpu_data(&self, writer: &mut BitVecWriter) -> Result<()> {
        if let Some(ref vdr_rpu_data) = self.vdr_rpu_data {
            vdr_rpu_data.write(writer, &self.header)?;
        }

        if let Some(ref nlq_data) = self.nlq_data {
            nlq_data.write(writer, &self.header)?;
        }

        Ok(())
    }

    pub fn write_vdr_dm_data(&self, writer: &mut BitVecWriter) {
        if let Some(ref vdr_dm_data) = self.vdr_dm_data {
            vdr_dm_data.write(writer);
        }
    }

    #[inline(always)]
    pub fn compute_crc32(data: &[u8]) -> u32 {
        let crc = Crc::<u32>::new(&CRC_32_MPEG_2);
        let mut digest = crc.digest();
        digest.update(data);

        digest.finalize()
    }

    pub fn convert_with_mode(&mut self, mode: u8) -> Result<()> {
        if mode != 0 {
            self.modified = true;
        }

        if self.dovi_profile == 7 {
            match mode {
                1 => self.convert_to_mel()?,
                2 => self.convert_to_81(),
                _ => (),
            };
        } else if self.dovi_profile == 5 && mode == 3 {
            self.p5_to_p81()?;
        } else if mode != 0 {
            bail!("Invalid profile for mode {} conversion!", mode);
        }

        Ok(())
    }

    pub fn crop(&mut self) {
        self.modified = true;

        if let Some(block) = self.get_level5_block_mut() {
            block.crop();
        }
    }

    fn p5_to_p81(&mut self) -> Result<()> {
        self.modified = true;

        if self.dovi_profile == 5 {
            self.convert_to_81();

            self.dovi_profile = 8;

            self.header.vdr_rpu_profile = 1;
            self.header.bl_video_full_range_flag = false;

            self.header.num_pivots_minus_2 = [0, 0, 0];
            self.header.pred_pivot_value.iter_mut().for_each(|v2| {
                v2.truncate(2);
                v2[0] = 0;
                v2[1] = 1023;
            });

            if let Some(ref mut vdr_rpu_data) = self.vdr_rpu_data {
                vdr_rpu_data.p5_to_p81();
            }

            if let Some(ref mut vdr_dm_data) = self.vdr_dm_data {
                vdr_dm_data.p5_to_p81();
            }
        } else {
            bail!("Attempt to convert profile 5: RPU is not profile 5!");
        }

        Ok(())
    }

    pub fn validate(&mut self) -> Result<()> {
        self.dovi_profile = self.header.get_dovi_profile();
        self.header.validate(self.dovi_profile)?;

        if let Some(ref mut vdr_dm_data) = self.vdr_dm_data {
            vdr_dm_data.validate()?;
        }

        Ok(())
    }

    pub fn profile8_config(config: &GenerateConfig) -> Self {
        DoviRpu {
            dovi_profile: 8,
            modified: true,
            header: RpuDataHeader::p8_default(),
            vdr_rpu_data: Some(VdrRpuData::p8_default()),
            nlq_data: None,
            vdr_dm_data: Some(VdrDmData::from_config(config)),
            last_byte: 0x80,
            ..Default::default()
        }
    }

    pub fn get_level5_block_mut(&mut self) -> Option<&mut ExtMetadataBlockLevel5> {
        if let Some(ref mut vdr_dm_data) = self.vdr_dm_data {
            let level5_block = vdr_dm_data
                .st2094_10_metadata
                .ext_metadata_blocks
                .iter_mut()
                .find(|e| matches!(e, ExtMetadataBlock::Level5(_)));

            if let Some(ExtMetadataBlock::Level5(ref mut block)) = level5_block {
                return Some(block);
            }
        }

        None
    }
}
