use anyhow::{bail, ensure, Result};
use bitvec::prelude::*;
use bitvec_helpers::{bitvec_reader::BitVecReader, bitvec_writer::BitVecWriter};

#[cfg(feature = "serde_feature")]
use serde::Serialize;

use super::extension_metadata::blocks::{
    ExtMetadataBlock, ExtMetadataBlockLevel11, ExtMetadataBlockLevel5, ExtMetadataBlockLevel9,
};
use super::extension_metadata::{CmV40DmData, DmData};
use super::generate::GenerateConfig;
use super::profiles::profile81::Profile81;
use super::profiles::profile84::Profile84;
use super::rpu_data_header::{rpu_data_header, RpuDataHeader};
use super::rpu_data_mapping::RpuDataMapping;
use super::rpu_data_nlq::RpuDataNlq;
use super::vdr_dm_data::VdrDmData;
use super::{compute_crc32, ConversionMode, FEL_STR, MEL_STR};

use crate::rpu::rpu_data_mapping::vdr_rpu_data_payload;
use crate::rpu::vdr_dm_data::vdr_dm_data_payload;

use crate::utils::{
    add_start_code_emulation_prevention_3_byte, clear_start_code_emulation_prevention_3_byte,
};

const FINAL_BYTE: u8 = 0x80;

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde_feature", derive(Serialize))]
pub struct DoviRpu {
    pub dovi_profile: u8,

    #[cfg_attr(
        feature = "serde_feature",
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub subprofile: Option<String>,

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
    pub remaining: BitVec<u8, Msb0>,
    pub rpu_data_crc32: u32,

    #[cfg_attr(feature = "serde_feature", serde(skip_serializing))]
    pub trailing_bytes: Vec<u8>,

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
        let trailing_bytes: Vec<u8> = data
            .iter()
            .rev()
            .take_while(|b| **b == 0)
            .cloned()
            .collect();

        // Ignore trailing bytes
        let rpu_end = data.len() - trailing_bytes.len();
        let last_byte = data[rpu_end - 1];

        // Minus 4 bytes for the CRC32, 1 for the 0x80 ending byte
        let crc32_start = rpu_end - 5;

        let received_crc32 = compute_crc32(&data[1..crc32_start]);

        if last_byte != FINAL_BYTE {
            bail!("Invalid RPU last byte: {}", last_byte);
        }

        let mut dovi_rpu = DoviRpu::read_rpu_data(data.to_owned(), trailing_bytes)?;

        if received_crc32 != dovi_rpu.rpu_data_crc32 {
            bail!(
                "RPU CRC32 does not match the data. Received: {}, expected {}",
                received_crc32,
                dovi_rpu.rpu_data_crc32
            );
        }

        dovi_rpu.dovi_profile = dovi_rpu.header.get_dovi_profile();
        dovi_rpu.subprofile = dovi_rpu.get_dovi_subprofile();

        Ok(dovi_rpu)
    }

    #[inline(always)]
    fn read_rpu_data(bytes: Vec<u8>, trailing_bytes: Vec<u8>) -> Result<DoviRpu> {
        let mut reader = BitVecReader::new(bytes);
        let mut dovi_rpu = DoviRpu {
            trailing_bytes,
            ..Default::default()
        };

        // CRC32 + 0x80 + trailing
        let final_length = (8 * 4) + 8 + (dovi_rpu.trailing_bytes.len() * 8);

        rpu_data_header(&mut dovi_rpu, &mut reader)?;

        // Preliminary header validation
        dovi_rpu.dovi_profile = dovi_rpu.header.get_dovi_profile();

        dovi_rpu.header.validate(dovi_rpu.dovi_profile)?;

        if dovi_rpu.header.rpu_type == 2 {
            if !dovi_rpu.header.use_prev_vdr_rpu_flag {
                vdr_rpu_data_payload(&mut dovi_rpu, &mut reader)?;
            }

            if dovi_rpu.header.vdr_dm_metadata_present_flag {
                vdr_dm_data_payload(&mut dovi_rpu, &mut reader, final_length)?;
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
            ensure!(last_byte == FINAL_BYTE, "last byte should be 0x80");
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

        // Trailing bytes
        if !self.trailing_bytes.is_empty() {
            self.trailing_bytes
                .iter()
                .for_each(|b| writer.write_n(&[*b], 8));
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

    fn get_dovi_subprofile(&self) -> Option<String> {
        if self.dovi_profile == 7 {
            if let Some(nlq) = &self.rpu_data_nlq {
                let subprofile = if nlq.is_mel() {
                    String::from(MEL_STR)
                } else {
                    String::from(FEL_STR)
                };

                return Some(subprofile);
            }
        }

        None
    }

    /// Modes:
    ///     0: Don't modify the RPU
    ///     1: Converts the RPU to be MEL compatible
    ///     2: Converts the RPU to be profile 8.1 compatible
    ///     3: Converts profile 5 to 8.1
    ///     4: Converts to static profile 8.4
    ///
    /// noop when profile 8 and mode 2 is used
    pub fn convert_with_mode<T: Into<ConversionMode>>(&mut self, mode: T) -> Result<()> {
        let mode: ConversionMode = mode.into();

        if mode != ConversionMode::Lossless {
            self.modified = true;
        }

        let valid_conversion = match mode {
            ConversionMode::Lossless => true,
            ConversionMode::ToMel => {
                if matches!(self.dovi_profile, 7 | 8) {
                    self.convert_to_mel()?;
                    true
                } else {
                    false
                }
            }
            ConversionMode::To81 => match self.dovi_profile {
                7 | 8 => {
                    self.convert_to_p81();
                    true
                }
                5 => {
                    self.p5_to_p81()?;
                    true
                }
                _ => false,
            },
            ConversionMode::To84 => {
                self.convert_to_p84();
                true
            }
        };

        if !valid_conversion {
            bail!("Invalid profile for mode {} conversion!", mode);
        }

        // Update profile value
        self.dovi_profile = self.header.get_dovi_profile();
        self.subprofile = self.get_dovi_subprofile();

        Ok(())
    }

    fn convert_to_mel(&mut self) -> Result<()> {
        let header = &mut self.header;

        header.el_spatial_resampling_filter_flag = true;
        header.disable_residual_flag = false;

        header.nlq_method_idc = Some(0);
        header.nlq_num_pivots_minus2 = Some(0);

        // BL is always 10 bit in current spec
        header.nlq_pred_pivot_value = Some([0, 1023]);

        if let Some(ref mut rpu_data_nlq) = self.rpu_data_nlq {
            rpu_data_nlq.convert_to_mel();
        } else if self.dovi_profile == 8 {
            self.rpu_data_nlq = Some(RpuDataNlq::mel_default());
        } else {
            bail!("Not profile 7 or 8, cannot convert to MEL!");
        }

        Ok(())
    }

    fn convert_to_p81(&mut self) {
        self.modified = true;

        let header = &mut self.header;

        // Change to 8.1
        header.el_spatial_resampling_filter_flag = false;
        header.disable_residual_flag = true;

        header.nlq_method_idc = None;
        header.nlq_num_pivots_minus2 = None;
        header.nlq_pred_pivot_value = None;

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
            self.convert_to_p81();

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

    pub fn profile5_config(config: &GenerateConfig) -> Result<Self> {
        Ok(DoviRpu {
            dovi_profile: 5,
            modified: true,
            header: RpuDataHeader::p5_default(),
            rpu_data_mapping: Some(Profile81::rpu_data_mapping()),
            rpu_data_nlq: None,
            vdr_dm_data: Some(VdrDmData::from_generate_config(config)?),
            ..Default::default()
        })
    }

    pub fn profile81_config(config: &GenerateConfig) -> Result<Self> {
        Ok(DoviRpu {
            dovi_profile: 8,
            modified: true,
            header: RpuDataHeader::p8_default(),
            rpu_data_mapping: Some(Profile81::rpu_data_mapping()),
            rpu_data_nlq: None,
            vdr_dm_data: Some(VdrDmData::from_generate_config(config)?),
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

    #[deprecated(
        since = "1.6.6",
        note = "Causes issues in playback when L8 metadata is not present. Will be removed"
    )]
    pub fn convert_to_cmv40(&mut self) -> Result<()> {
        if let Some(ref mut vdr_dm_data) = self.vdr_dm_data {
            if vdr_dm_data.cmv40_metadata.is_none() {
                self.modified = true;

                vdr_dm_data.cmv40_metadata = Some(DmData::V40(CmV40DmData::new_with_l254_402()));

                // Defaults
                vdr_dm_data.add_metadata_block(ExtMetadataBlock::Level9(
                    ExtMetadataBlockLevel9::default_dci_p3(),
                ))?;
                vdr_dm_data.add_metadata_block(ExtMetadataBlock::Level11(
                    ExtMetadataBlockLevel11::default_reference_cinema(),
                ))?;
            }
        }

        Ok(())
    }

    pub fn profile84_config(config: &GenerateConfig) -> Result<Self> {
        Ok(DoviRpu {
            dovi_profile: 8,
            modified: true,
            header: Profile84::rpu_data_header(),
            rpu_data_mapping: Some(Profile84::rpu_data_mapping()),
            rpu_data_nlq: None,
            vdr_dm_data: Some(VdrDmData::from_generate_config(config)?),
            ..Default::default()
        })
    }

    fn convert_to_p84(&mut self) {
        self.convert_to_p81();

        self.header = Profile84::rpu_data_header();
        self.rpu_data_mapping = Some(Profile84::rpu_data_mapping());
    }

    pub fn remove_cmv40_extension_metadata(&mut self) -> Result<()> {
        if let Some(ref mut vdr_dm_data) = self.vdr_dm_data {
            if vdr_dm_data.cmv40_metadata.is_some() {
                self.modified = true;

                vdr_dm_data.cmv40_metadata = None;
            }
        }

        Ok(())
    }
}
