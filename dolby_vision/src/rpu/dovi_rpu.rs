use anyhow::{bail, ensure, Result};
use bitvec::prelude::{BitVec, Msb0};
use bitvec_helpers::{
    bitstream_io_reader::BitstreamIoReader, bitstream_io_writer::BitstreamIoWriter,
};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::extension_metadata::blocks::{ExtMetadataBlock, ExtMetadataBlockLevel5};
use super::generate::GenerateConfig;
use super::profiles::profile81::Profile81;
use super::profiles::profile84::Profile84;
use super::rpu_data_header::RpuDataHeader;
use super::rpu_data_mapping::{DoviNlqMethod, RpuDataMapping};
use super::rpu_data_nlq::{DoviELType, RpuDataNlq};
use super::vdr_dm_data::VdrDmData;
use super::{compute_crc32, ConversionMode};

use crate::rpu::vdr_dm_data::vdr_dm_data_payload;

use crate::utils::{
    add_start_code_emulation_prevention_3_byte, clear_start_code_emulation_prevention_3_byte,
};

const FINAL_BYTE: u8 = 0x80;

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct DoviRpu {
    pub dovi_profile: u8,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub el_type: Option<DoviELType>,

    pub header: RpuDataHeader,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub rpu_data_mapping: Option<RpuDataMapping>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub vdr_dm_data: Option<VdrDmData>,

    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "crate::utils::opt_bitvec_ser_bits",
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub remaining: Option<BitVec<u8, Msb0>>,
    pub rpu_data_crc32: u32,

    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    pub modified: bool,

    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    trailing_zeroes: usize,

    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    original_payload_size: usize,
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
        let trailing_zeroes = data.iter().rev().take_while(|b| **b == 0).count();

        // Ignore trailing bytes
        let rpu_end = data.len() - trailing_zeroes;
        let last_byte = data[rpu_end - 1];

        // Minus 4 bytes for the CRC32, 1 for the 0x80 ending byte
        let crc32_start = rpu_end - 5;

        let received_crc32 = compute_crc32(&data[1..crc32_start]);

        if last_byte != FINAL_BYTE {
            bail!("Invalid RPU last byte: {}", last_byte);
        }

        let dovi_rpu = DoviRpu::read_rpu_data(data, trailing_zeroes)?;

        if received_crc32 != dovi_rpu.rpu_data_crc32 {
            bail!(
                "RPU CRC32 does not match the data. Received: {}, expected {}",
                received_crc32,
                dovi_rpu.rpu_data_crc32
            );
        }

        Ok(dovi_rpu)
    }

    #[inline(always)]
    fn read_rpu_data(bytes: &[u8], trailing_zeroes: usize) -> Result<DoviRpu> {
        let cursor = std::io::Cursor::new(bytes);
        let mut reader = BitstreamIoReader::new(cursor, bytes.len() as u64);

        // CRC32 + 0x80 + trailing
        let final_length = (32 + 8 + (trailing_zeroes * 8)) as u64;

        let header = RpuDataHeader::parse(&mut reader)?;

        // Preliminary header validation
        let dovi_profile = header.get_dovi_profile();
        header.validate(dovi_profile)?;

        let rpu_data_mapping = if !header.use_prev_vdr_rpu_flag {
            Some(RpuDataMapping::parse(&mut reader, &header)?)
        } else {
            None
        };

        let el_type = rpu_data_mapping
            .as_ref()
            .map(|e| e.get_enhancement_layer_type())
            .unwrap_or(None);

        let vdr_dm_data = if header.vdr_dm_metadata_present_flag {
            Some(vdr_dm_data_payload(&mut reader, &header, final_length)?)
        } else {
            None
        };

        // rpu_alignment_zero_bit
        while !reader.is_aligned() {
            ensure!(!reader.get()?, "rpu_alignment_zero_bit != 0");
        }

        // CRC32 is at the end, there can be more data in between

        let remaining = if reader.available()? != final_length {
            let mut remaining: BitVec<u8, Msb0> = BitVec::new();

            while reader.available()? != final_length {
                remaining.push(reader.get()?);
            }

            Some(remaining)
        } else {
            None
        };

        let rpu_data_crc32 = reader.get_n(32)?;
        let last_byte: u8 = reader.get_n(8)?;
        ensure!(last_byte == FINAL_BYTE, "last byte should be 0x80");

        let dovi_rpu = DoviRpu {
            dovi_profile: header.get_dovi_profile(),
            el_type,
            header,
            rpu_data_mapping,
            vdr_dm_data,
            remaining,
            rpu_data_crc32,
            modified: false,
            trailing_zeroes,
            original_payload_size: bytes.len(),
        };

        // Validate
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
        // Capacity is in bits
        let mut writer = BitstreamIoWriter::with_capacity(self.original_payload_size * 8);

        self.validate()?;

        let header = &self.header;
        header.write_header(&mut writer)?;

        if header.rpu_type == 2 {
            if !header.use_prev_vdr_rpu_flag {
                if let Some(mapping) = &self.rpu_data_mapping {
                    mapping.write(&mut writer, &self.header)?;
                }
            }

            if header.vdr_dm_metadata_present_flag {
                if let Some(vdr_dm_data) = &self.vdr_dm_data {
                    vdr_dm_data.write(&mut writer)?;
                }
            }
        }

        if let Some(remaining) = &self.remaining {
            for b in remaining {
                writer.write(*b)?;
            }
        }

        writer.byte_align()?;

        let computed_crc32 = compute_crc32(&writer.as_slice().unwrap()[1..]);

        if !self.modified {
            // Validate the parsed crc32 is the same
            ensure!(
                self.rpu_data_crc32 == computed_crc32,
                "RPU CRC32 does not match computed value"
            );
        }

        // Write crc32
        writer.write_n(&computed_crc32, 32)?;
        writer.write_n(&0x80_u8, 8)?;

        // Trailing bytes
        if self.trailing_zeroes > 0 {
            for _ in 0..self.trailing_zeroes {
                writer.write_n(&0_u8, 8)?;
            }
        }

        Ok(writer.as_slice().unwrap().to_owned())
    }

    fn validate(&self) -> Result<()> {
        self.header.validate(self.dovi_profile)?;

        if let Some(mapping) = self.rpu_data_mapping.as_ref() {
            mapping.validate(self.dovi_profile)?;
        }

        if let Some(vdr_dm_data) = &self.vdr_dm_data {
            vdr_dm_data.validate()?;
        }

        Ok(())
    }

    pub fn get_enhancement_layer_type(&self) -> Option<DoviELType> {
        self.rpu_data_mapping
            .as_ref()
            .map(|e| e.get_enhancement_layer_type())
            .unwrap_or(None)
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
        self.el_type = self.get_enhancement_layer_type();

        Ok(())
    }

    fn convert_to_mel(&mut self) -> Result<()> {
        let header = &mut self.header;

        header.el_spatial_resampling_filter_flag = true;
        header.disable_residual_flag = false;

        if let Some(mapping) = self.rpu_data_mapping.as_mut() {
            mapping.nlq_method_idc = Some(DoviNlqMethod::LinearDeadzone);
            mapping.nlq_num_pivots_minus2 = Some(0);

            // BL is always 10 bit in current spec
            mapping.nlq_pred_pivot_value = Some([0, 1023]);

            if let Some(ref mut nlq) = mapping.nlq.as_mut() {
                nlq.convert_to_mel();
            } else if self.dovi_profile == 8 {
                mapping.nlq = Some(RpuDataNlq::mel_default());
            } else {
                bail!("Not profile 7 or 8, cannot convert to MEL!");
            }
        }

        Ok(())
    }

    fn convert_to_p81(&mut self) {
        self.modified = true;

        let header = &mut self.header;

        // Change to 8.1
        header.el_spatial_resampling_filter_flag = false;
        header.disable_residual_flag = true;

        if let Some(mapping) = self.rpu_data_mapping.as_mut() {
            mapping.nlq_method_idc = None;
            mapping.nlq_num_pivots_minus2 = None;
            mapping.nlq_pred_pivot_value = None;

            mapping.num_x_partitions_minus1 = 0;
            mapping.num_y_partitions_minus1 = 0;

            mapping.nlq = None;
        }

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

    pub fn set_active_area_offsets(
        &mut self,
        left: u16,
        right: u16,
        top: u16,
        bottom: u16,
    ) -> Result<()> {
        self.modified = true;

        if let Some(ref mut vdr_dm_data) = self.vdr_dm_data {
            vdr_dm_data.replace_metadata_block(ExtMetadataBlock::Level5(
                ExtMetadataBlockLevel5::from_offsets(left, right, top, bottom),
            ))?;
        }

        Ok(())
    }

    pub fn remove_mapping(&mut self) {
        self.modified = true;

        if let Some(mapping) = self.rpu_data_mapping.as_mut() {
            mapping.curves.iter_mut().for_each(|e| {
                e.num_pivots_minus2 = 0;

                e.pivots.clear();
                e.pivots.extend([0, 1023]);
            });
        }

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

    pub fn profile84_config(config: &GenerateConfig) -> Result<Self> {
        Ok(DoviRpu {
            dovi_profile: 8,
            modified: true,
            header: RpuDataHeader::p8_default(),
            rpu_data_mapping: Some(Profile84::rpu_data_mapping()),
            vdr_dm_data: Some(VdrDmData::from_generate_config(config)?),
            ..Default::default()
        })
    }

    fn convert_to_p84(&mut self) {
        self.convert_to_p81();

        self.header = RpuDataHeader::p8_default();
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

    pub fn replace_levels_from_rpu(&mut self, src_rpu: &Self, levels: &Vec<u8>) -> Result<()> {
        ensure!(!levels.is_empty(), "Must have levels to replace");

        if let (Some(dst_vdr_dm_data), Some(src_vdr_dm_data)) =
            (self.vdr_dm_data.as_mut(), src_rpu.vdr_dm_data.as_ref())
        {
            self.modified = true;

            for level in levels {
                dst_vdr_dm_data
                    .replace_metadata_blocks(src_vdr_dm_data.level_blocks_iter(*level))?;
            }
        }

        Ok(())
    }
}
