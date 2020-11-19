use super::{vdr_dm_data, vdr_rpu_data, prelude, BitVecReader, BitVecWriter};
use vdr_dm_data::VdrDmData;
use vdr_rpu_data::{NlqData, VdrRpuData};

use crc32fast::Hasher;
use prelude::*;

const MEL_PRED_PIVOT_VALUE: &[u64] = &[0, 1023];

#[derive(Default, Debug)]
pub struct RpuNal {
    pub rpu_nal_prefix: u8,
    pub rpu_type: u8,
    pub rpu_format: u16,
    pub vdr_rpu_profile: u8,
    pub vdr_rpu_level: u8,
    pub vdr_seq_info_present_flag: bool,
    pub chroma_resampling_explicit_filter_flag: bool,
    pub coefficient_data_type: u8,
    pub coefficient_log2_denom: u64,
    pub vdr_rpu_normalized_idc: u8,
    pub bl_video_full_range_flag: bool,
    pub bl_bit_depth_minus8: u64,
    pub el_bit_depth_minus8: u64,
    pub vdr_bit_depth_minus_8: u64,
    pub spatial_resampling_filter_flag: bool,
    pub reserved_zero_3bits: u8,
    pub el_spatial_resampling_filter_flag: bool,
    pub disable_residual_flag: bool,
    pub vdr_dm_metadata_present_flag: bool,
    pub use_prev_vdr_rpu_flag: bool,
    pub prev_vdr_rpu_id: u64,
    pub vdr_rpu_id: u64,
    pub mapping_color_space: u64,
    pub mapping_chroma_format_idc: u64,
    pub num_pivots_minus_2: [u64; 3],
    pub pred_pivot_value: Vec<Vec<u64>>,
    pub nlq_method_idc: u8,
    pub nlq_num_pivots_minus2: u8,
    pub num_x_partitions_minus1: u64,
    pub num_y_partitions_minus1: u64,
    pub vdr_rpu_data: Option<VdrRpuData>,
    pub nlq_data: Option<NlqData>,
    pub vdr_dm_data: Option<VdrDmData>,
    pub remaining: BitVec<Msb0, u8>,
    pub crc32_offset: usize,
    pub rpu_data_crc32: u32,
}

impl RpuNal {
    pub fn read_rpu_data(reader: &mut BitVecReader) -> RpuNal {
        let mut rpu_nal = RpuNal::rpu_data_header(reader);
    
        if rpu_nal.rpu_type == 2 {
            if !rpu_nal.use_prev_vdr_rpu_flag {
                VdrRpuData::vdr_rpu_data_payload(reader, &mut rpu_nal);
            }
    
            if rpu_nal.vdr_dm_metadata_present_flag {
                rpu_nal.vdr_dm_data = Some(VdrDmData::vdr_dm_data_payload(reader));
            }
        }
    
        while !reader.is_aligned() {
            rpu_nal.remaining.push(reader.get());
        }
    
        rpu_nal.crc32_offset = reader.pos();
        rpu_nal.rpu_data_crc32 = reader.get_n(32);
    
        rpu_nal
    }

    pub fn rpu_data_header(reader: &mut BitVecReader) -> RpuNal {
        let mut rpu_nal = RpuNal::default();
    
        rpu_nal.rpu_nal_prefix = reader.get_n(8);
    
        if rpu_nal.rpu_nal_prefix == 25 {
            rpu_nal.rpu_type = reader.get_n(6);
            rpu_nal.rpu_format = reader.get_n(11);
    
            if rpu_nal.rpu_type == 2 {
                rpu_nal.vdr_rpu_profile = reader.get_n(4);
                rpu_nal.vdr_rpu_level = reader.get_n(4);
                rpu_nal.vdr_seq_info_present_flag = reader.get();
    
                if rpu_nal.vdr_seq_info_present_flag {
                    rpu_nal.chroma_resampling_explicit_filter_flag = reader.get();
                    rpu_nal.coefficient_data_type = reader.get_n(2);
    
                    if rpu_nal.coefficient_data_type == 0 {
                        rpu_nal.coefficient_log2_denom = reader.get_ue();
                    }
    
                    rpu_nal.vdr_rpu_normalized_idc = reader.get_n(2);
                    rpu_nal.bl_video_full_range_flag = reader.get();
    
                    if rpu_nal.rpu_format & 0x700 == 0 {
                        rpu_nal.bl_bit_depth_minus8 = reader.get_ue();
                        rpu_nal.el_bit_depth_minus8 = reader.get_ue();
                        rpu_nal.vdr_bit_depth_minus_8 = reader.get_ue();
                        rpu_nal.spatial_resampling_filter_flag = reader.get();
                        rpu_nal.reserved_zero_3bits = reader.get_n(3);
                        rpu_nal.el_spatial_resampling_filter_flag = reader.get();
                        rpu_nal.disable_residual_flag = reader.get();
                    }
                }
    
                rpu_nal.vdr_dm_metadata_present_flag = reader.get();
                rpu_nal.use_prev_vdr_rpu_flag = reader.get();
    
                if rpu_nal.use_prev_vdr_rpu_flag {
                    rpu_nal.prev_vdr_rpu_id = reader.get_ue();
                } else {
                    rpu_nal.vdr_rpu_id = reader.get_ue();
                    rpu_nal.mapping_color_space = reader.get_ue();
                    rpu_nal.mapping_chroma_format_idc = reader.get_ue();
    
                    for cmp in 0..3 {
                        rpu_nal.num_pivots_minus_2[cmp] = reader.get_ue();
    
                        let pivot_idx_count = (rpu_nal.num_pivots_minus_2[cmp] + 2) as usize;
    
                        rpu_nal.pred_pivot_value.push(vec![0; pivot_idx_count]);
                        for pivot_idx in 0..pivot_idx_count {
                            rpu_nal.pred_pivot_value[cmp][pivot_idx] =
                                reader.get_n((rpu_nal.bl_bit_depth_minus8 + 8) as usize);
                        }
                    }
    
                    if rpu_nal.rpu_format & 0x700 == 0 && !rpu_nal.disable_residual_flag {
                        rpu_nal.nlq_method_idc = reader.get_n(3);
                        rpu_nal.nlq_num_pivots_minus2 = 0;
                    }
    
                    rpu_nal.num_x_partitions_minus1 = reader.get_ue();
                    rpu_nal.num_y_partitions_minus1 = reader.get_ue();
                }
            }
        }
    
        rpu_nal.validate_header();
    
        rpu_nal
    }

    pub fn validate_header(&self) {
        assert_eq!(self.rpu_nal_prefix, 25);
        assert_eq!(self.vdr_rpu_profile, 1);
        assert_eq!(self.vdr_rpu_level, 0);
        assert_eq!(self.bl_bit_depth_minus8, 2);
        assert_eq!(self.el_bit_depth_minus8, 2);
        assert!(self.vdr_bit_depth_minus_8 <= 6);
        assert_eq!(self.mapping_color_space, 0);
        assert_eq!(self.mapping_chroma_format_idc, 0);
        assert!(self.coefficient_log2_denom <= 23);

        assert_eq!(self.nlq_method_idc, 0);
        assert_eq!(self.nlq_num_pivots_minus2, 0);
    }

    pub fn validate_crc32(&self, reader: &mut BitVecReader) {
        let whole_data = reader.get_inner()[..self.crc32_offset].as_slice();
        let mut hasher = Hasher::new();
        hasher.update(whole_data);

        let calculated_crc32 = hasher.finalize();

        assert_eq!(calculated_crc32, self.rpu_data_crc32);
    }

    pub fn convert_to_mel(&mut self) {
        // Set pivots to 0
        self.num_pivots_minus_2.iter_mut().for_each(|v| *v = 0);

        // Set pivot values to [0, 1023]
        self.pred_pivot_value.iter_mut().for_each(|v| {
            v.clear();
            v.extend_from_slice(&MEL_PRED_PIVOT_VALUE);
        });

        if let Some(ref mut vdr_rpu_data) = self.vdr_rpu_data {
            vdr_rpu_data.convert_to_mel();
        }

        if let Some(ref mut nlq_data) = self.nlq_data {
            nlq_data.convert_to_mel();
        }
    }

    pub fn convert_to_81(&mut self) {
        // Change to RPU only (8.1)
        self.el_spatial_resampling_filter_flag = false;
        self.disable_residual_flag = true;
    }

    pub fn write_rpu_data(mut rpu_nal: RpuNal, mut writer: &mut BitVecWriter) {
        rpu_nal.write_header(&mut writer);
    
        if rpu_nal.rpu_type == 2 {
            if !rpu_nal.use_prev_vdr_rpu_flag {
                rpu_nal.write_vdr_rpu_data(&mut writer);
            }
    
            if rpu_nal.vdr_dm_metadata_present_flag {
                rpu_nal.write_vdr_dm_data(&mut writer);
            }
        }
    
        rpu_nal.remaining.iter().for_each(|b| writer.write(*b));
    
        writer.write_n(&rpu_nal.rpu_data_crc32.to_be_bytes(), 32);
    
    }

    pub fn write_header(&mut self, writer: &mut BitVecWriter) {
        writer.write_n(&self.rpu_nal_prefix.to_be_bytes(), 8);

        if self.rpu_nal_prefix == 25 {
            writer.write_n(&self.rpu_type.to_be_bytes(), 6);
            writer.write_n(&self.rpu_format.to_be_bytes(), 11);

            if self.rpu_type == 2 {
                writer.write_n(&self.vdr_rpu_profile.to_be_bytes(), 4);
                writer.write_n(&self.vdr_rpu_level.to_be_bytes(), 4);
                writer.write(self.vdr_seq_info_present_flag);

                if self.vdr_seq_info_present_flag {
                    writer.write(self.chroma_resampling_explicit_filter_flag);
                    writer.write_n(&self.coefficient_data_type.to_be_bytes(), 2);

                    if self.coefficient_data_type == 0 {
                        writer.write_ue(self.coefficient_log2_denom);
                    }

                    writer.write_n(&self.vdr_rpu_normalized_idc.to_be_bytes(), 2);
                    writer.write(self.bl_video_full_range_flag);

                    if self.rpu_format & 0x700 == 0 {
                        writer.write_ue(self.bl_bit_depth_minus8);
                        writer.write_ue(self.el_bit_depth_minus8);
                        writer.write_ue(self.vdr_bit_depth_minus_8);
                        writer.write(self.spatial_resampling_filter_flag);
                        writer.write_n(&self.reserved_zero_3bits.to_be_bytes(), 3);
                        writer.write(self.el_spatial_resampling_filter_flag);
                        writer.write(self.disable_residual_flag);
                    }
                }

                writer.write(self.vdr_dm_metadata_present_flag);
                writer.write(self.use_prev_vdr_rpu_flag);

                if self.use_prev_vdr_rpu_flag {
                    writer.write_ue(self.prev_vdr_rpu_id);
                } else {
                    writer.write_ue(self.vdr_rpu_id);
                    writer.write_ue(self.mapping_color_space);
                    writer.write_ue(self.mapping_chroma_format_idc);

                    for cmp in 0..3 {
                        writer.write_ue(self.num_pivots_minus_2[cmp]);

                        let pivot_idx_count = (self.num_pivots_minus_2[cmp] + 2) as usize;

                        for pivot_idx in 0..pivot_idx_count {
                            writer.write_n(
                                &self.pred_pivot_value[cmp][pivot_idx].to_be_bytes(),
                                (self.bl_bit_depth_minus8 + 8) as usize,
                            );
                        }
                    }

                    if self.rpu_format & 0x700 == 0 && !self.disable_residual_flag {
                        writer.write_n(&self.nlq_method_idc.to_be_bytes(), 3);
                    }

                    writer.write_ue(self.num_x_partitions_minus1);
                    writer.write_ue(self.num_y_partitions_minus1);
                }
            }
        }
    }

    pub fn write_vdr_rpu_data(&self, writer: &mut BitVecWriter) {
        if let Some(ref vdr_rpu_data) = self.vdr_rpu_data {
            vdr_rpu_data.write(writer, self);
        }

        if let Some(ref nlq_data) = self.nlq_data {
            nlq_data.write(writer, self);
        }
    }

    pub fn write_vdr_dm_data(&self, writer: &mut BitVecWriter) {
        if let Some(ref vdr_dm_data) = self.vdr_dm_data {
            vdr_dm_data.write(writer);
        }
    }
}