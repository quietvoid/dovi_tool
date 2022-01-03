use anyhow::{bail, ensure, Result};
use bitvec::prelude::*;
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use super::compute_crc32;
use super::extension_metadata::blocks::{ExtMetadataBlock, ExtMetadataBlockLevel5};
use super::generate::GenerateConfig;
use super::rpu_data_header::{rpu_data_header, RpuDataHeader};
use super::rpu_data_mapping::RpuDataMapping;
use super::rpu_data_nlq::RpuDataNlq;
use super::vdr_dm_data::VdrDmData;

use crate::rpu::rpu_data_mapping::vdr_rpu_data_payload;
use crate::rpu::vdr_dm_data::vdr_dm_data_payload;

use crate::utils::{
    add_start_code_emulation_prevention_3_byte, clear_start_code_emulation_prevention_3_byte,
};

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct DoviRpu {
    pub dovi_profile: u8,
    pub header: RpuDataHeader,

    #[cfg_attr(
        feature = "serde_feature",
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub rpu_data_mapping: Option<RpuDataMapping>,

    #[cfg_attr(
        feature = "serde_feature",
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub rpu_data_nlq: Option<RpuDataNlq>,

    #[cfg_attr(
        feature = "serde_feature",
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub vdr_dm_data: Option<VdrDmData>,

    #[cfg_attr(
        feature = "serde_feature",
        serde(
            serialize_with = "crate::utils::bitvec_ser_bits",
            skip_serializing_if = "BitVec::is_empty"
        )
    )]
    pub remaining: BitVec<Msb0, u8>,
    pub rpu_data_crc32: u32,

    #[cfg_attr(feature = "serde_feature", serde(skip_serializing))]
    pub last_byte: u8,

    #[cfg_attr(feature = "serde_feature", serde(skip_serializing))]
    pub modified: bool,
}

impl DoviRpu {
    pub fn validated_trimmed_data(data: &[u8]) -> Result<&[u8]> {
        if data.len() < 25 {
            bail!("Invalid RPU length: {}", &data.len());
        }

        // Including 0x7C01 prepended
        let trimmed_data = match &data[..5] {
            [0, 0, 0, 1, 25] => &data[4..],
            [0, 0, 1, 25, 8] => &data[3..],
            [0, 1, 25, 8, 9] | [124, 1, 25, 8, 9] => &data[2..],
            [1, 25, 8, 9, _] => &data[1..],
            [25, 8, 9, _, _] => data,
            _ => bail!("Invalid RPU data start bytes\n{:?}", &data[..5]),
        };

        Ok(trimmed_data)
    }

    pub fn parse_unspec62_nalu(data: &[u8]) -> Result<DoviRpu> {
        let trimmed_data = DoviRpu::validated_trimmed_data(data)?;

        // Clear start code emulation prevention 3 byte
        let bytes: Vec<u8> = clear_start_code_emulation_prevention_3_byte(trimmed_data);

        DoviRpu::parse(&bytes)
    }

    pub fn parse_rpu(data: &[u8]) -> Result<DoviRpu> {
        let trimmed_data = DoviRpu::validated_trimmed_data(data)?;

        DoviRpu::parse(trimmed_data)
    }

    #[inline(always)]
    fn parse(data: &[u8]) -> Result<DoviRpu> {
        let len = data.len();

        let mut received_crc32 = compute_crc32(&data[1..len - 5]);
        let last_byte = data[len - 1];

        // Final RPU exception
        if last_byte == 0 && data[len - 2] == 0x80 {
            received_crc32 = compute_crc32(&data[1..len - 6]);
        } else if last_byte != 0x80 {
            bail!("Invalid RPU last byte: {}", last_byte);
        }

        let mut dovi_rpu = DoviRpu::read_rpu_data(data.to_owned(), last_byte)?;

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
    fn read_rpu_data(bytes: Vec<u8>, end_byte: u8) -> Result<DoviRpu> {
        let mut reader = BitVecReader::new(bytes);
        let mut dovi_rpu = DoviRpu {
            last_byte: end_byte,
            ..Default::default()
        };

        // EOF case
        let final_length = if end_byte == 0 { 48 } else { 40 };

        rpu_data_header(&mut dovi_rpu, &mut reader)?;

        // Preliminary header validation
        dovi_rpu.dovi_profile = dovi_rpu.header.get_dovi_profile();

        dovi_rpu.header.validate(dovi_rpu.dovi_profile)?;

        if dovi_rpu.header.rpu_type == 2 {
            if !dovi_rpu.header.use_prev_vdr_rpu_flag {
                vdr_rpu_data_payload(&mut dovi_rpu, &mut reader)?;
            }

            if dovi_rpu.header.vdr_dm_metadata_present_flag {
                vdr_dm_data_payload(&mut dovi_rpu, &mut reader)?;
            }

            // rpu_alignment_zero_bit
            while !reader.is_aligned() {
                ensure!(!reader.get()?, "rpu_alignment_zero_bit != 0");
            }

            // CRC32 is at the end, there can be more data in between
            if reader.available() != final_length {
                while reader.available() != final_length {
                    dovi_rpu.remaining.push(reader.get()?);
                }
            }

            dovi_rpu.rpu_data_crc32 = reader.get_n(32);

            let last_byte: u8 = reader.get_n(8);
            ensure!(last_byte == 0x80, "last byte should be 0x80");
        }

        // Update the profile and validate
        dovi_rpu.dovi_profile = dovi_rpu.header.get_dovi_profile();
        dovi_rpu.validate()?;

        Ok(dovi_rpu)
    }

    pub fn write_hevc_unspec62_nalu(&self) -> Result<Vec<u8>> {
        let mut out = self.write_rpu_data()?;
        add_start_code_emulation_prevention_3_byte(&mut out);

        // Put back NAL unit type
        out.insert(0, 0x01);
        out.insert(0, 0x7C);

        Ok(out)
    }

    pub fn write_rpu(&self) -> Result<Vec<u8>> {
        self.write_rpu_data()
    }

    #[inline(always)]
    fn write_rpu_data(&self) -> Result<Vec<u8>> {
        let mut writer = BitVecWriter::new();

        self.validate()?;

        let header = &self.header;
        header.write_header(&mut writer);

        if header.rpu_type == 2 {
            if !header.use_prev_vdr_rpu_flag {
                self.write_vdr_rpu_data_payload(&mut writer)?;
            }

            if header.vdr_dm_metadata_present_flag {
                self.write_vdr_dm_data_payload(&mut writer)?;
            }
        }

        if !self.remaining.is_empty() {
            self.remaining.iter().for_each(|b| writer.write(*b));
        }

        while !writer.is_aligned() {
            writer.write(false);
        }

        let computed_crc32 = compute_crc32(&writer.as_slice()[1..]);

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

        Ok(writer.as_slice().to_owned())
    }

    fn write_vdr_rpu_data_payload(&self, writer: &mut BitVecWriter) -> Result<()> {
        if let Some(ref rpu_data_mapping) = self.rpu_data_mapping {
            rpu_data_mapping.write(writer, &self.header)?;
        }

        if let Some(ref rpu_data_nlq) = self.rpu_data_nlq {
            rpu_data_nlq.write(writer, &self.header)?;
        }

        Ok(())
    }

    fn write_vdr_dm_data_payload(&self, writer: &mut BitVecWriter) -> Result<()> {
        if let Some(ref vdr_dm_data) = self.vdr_dm_data {
            vdr_dm_data.write(writer)?;
        }

        Ok(())
    }

    fn validate(&self) -> Result<()> {
        self.header.validate(self.dovi_profile)?;

        if let Some(vdr_dm_data) = &self.vdr_dm_data {
            vdr_dm_data.validate()?;
        }

        Ok(())
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
        } else if self.dovi_profile == 8 && mode == 1 {
            self.convert_to_mel()?;
        } else if mode != 0 {
            bail!("Invalid profile for mode {} conversion!", mode);
        }

        // Update profile value
        self.dovi_profile = self.header.get_dovi_profile();

        Ok(())
    }

    fn convert_to_mel(&mut self) -> Result<()> {
        let header = &mut self.header;

        header.el_spatial_resampling_filter_flag = true;
        header.disable_residual_flag = false;

        header.nlq_method_idc = Some(0);
        header.nlq_num_pivots_minus2 = Some(0);
        header.num_x_partitions_minus1 = 2046;

        if let Some(ref mut rpu_data_nlq) = self.rpu_data_nlq {
            rpu_data_nlq.convert_to_mel();
        } else if self.dovi_profile == 8 {
            self.rpu_data_nlq = Some(RpuDataNlq::mel_default());
        } else {
            bail!("Not profile 7 or 8, cannot convert to MEL!");
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

        self.rpu_data_nlq = None;

        if let Some(ref mut vdr_dm_data) = self.vdr_dm_data {
            vdr_dm_data.set_p81_coeffs();
        }
    }

    fn p5_to_p81(&mut self) -> Result<()> {
        self.modified = true;

        if self.dovi_profile == 5 {
            self.convert_to_81();

            self.dovi_profile = 8;

            self.header.vdr_rpu_profile = 1;
            self.header.bl_video_full_range_flag = false;

            self.remove_mapping();

            if let Some(ref mut vdr_dm_data) = self.vdr_dm_data {
                vdr_dm_data.set_p81_coeffs();
            }
        } else {
            bail!("Attempt to convert profile 5: RPU is not profile 5!");
        }

        Ok(())
    }

    pub fn profile81_config(config: &GenerateConfig) -> Result<Self> {
        Ok(DoviRpu {
            dovi_profile: 8,
            modified: true,
            header: RpuDataHeader::p8_default(),
            rpu_data_mapping: Some(RpuDataMapping::p8_default()),
            rpu_data_nlq: None,
            vdr_dm_data: Some(VdrDmData::from_generate_config(config)?),
            last_byte: 0x80,
            ..Default::default()
        })
    }

    /// Set existing L5 metadata to zero offsets
    /// If there is no L5 metadata, creates it with zero offsets
    pub fn crop(&mut self) -> Result<()> {
        self.modified = true;

        if let Some(ref mut vdr_dm_data) = self.vdr_dm_data {
            vdr_dm_data.replace_metadata_block(ExtMetadataBlock::Level5(
                ExtMetadataBlockLevel5::default(),
            ))?;
        }

        Ok(())
    }

    pub fn remove_mapping(&mut self) {
        self.modified = true;

        self.header.num_pivots_minus_2 = [0, 0, 0];
        self.header.pred_pivot_value.iter_mut().for_each(|v| {
            v.clear();
            v.extend(&[0, 1023]);
        });

        if let Some(ref mut rpu_data_mapping) = self.rpu_data_mapping {
            rpu_data_mapping.set_empty_p81_mapping();
        }
    }

    pub fn parse_list_of_unspec62_nalus(data: &[Vec<u8>]) -> Vec<DoviRpu> {
        data.iter()
            .map(|rpu| DoviRpu::parse_unspec62_nalu(rpu))
            .filter_map(Result::ok)
            .collect()
    }
}
