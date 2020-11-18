mod vdr_dm_data;
mod vdr_rpu_data;

use vdr_dm_data::VdrDmData;
use vdr_rpu_data::{NlqData, VdrRpuData};
use bitvec::prelude;

use prelude::*;

use super::{
    add_start_code_emulation_prevention_3_byte, clear_start_code_emulation_prevention_3_byte,
    BitVecReader, BitVecWriter,
};

#[derive(Default, Debug)]
pub struct RpuNal {
    header_end: usize,
    rpu_nal_prefix: u8,
    rpu_type: u8,
    rpu_format: u16,
    vdr_rpu_profile: u8,
    vdr_rpu_level: u8,
    vdr_seq_info_present_flag: bool,
    chroma_resampling_explicit_filter_flag: bool,
    coefficient_data_type: u8,
    coefficient_log2_denom: u64,
    vdr_rpu_normalized_idc: u8,
    bl_video_full_range_flag: bool,
    bl_bit_depth_minus8: u64,
    el_bit_depth_minus8: u64,
    vdr_bit_depth_minus_8: u64,
    spatial_resampling_filter_flag: bool,
    reserved_zero_3bits: u8,
    el_spatial_resampling_filter_flag: bool,
    disable_residual_flag: bool,
    vdr_dm_metadata_present_flag: bool,
    use_prev_vdr_rpu_flag: bool,
    prev_vdr_rpu_id: u64,
    vdr_rpu_id: u64,
    mapping_color_space: u64,
    mapping_chroma_format_idc: u64,
    num_pivots_minus_2: [u64; 3],
    pred_pivot_value: Vec<Vec<u64>>,
    nlq_method_idc: u8,
    nlq_num_pivots_minus2: u8,
    num_x_partitions_minus1: u64,
    num_y_partitions_minus1: u64,
    vdr_rpu_data: Option<VdrRpuData>,
    nlq_data: Option<NlqData>,
    vdr_dm_data: Option<VdrDmData>,
    remaining: BitVec<Msb0, u8>,
    rpu_data_crc32: u32,
}

#[inline(always)]
pub fn parse_dovi_rpu(data: &[u8]) -> Vec<u8> {
    // Clear start code emulation prevention 3 byte
    let bytes: Vec<u8> = clear_start_code_emulation_prevention_3_byte(&data);
    println!("{:?}", &bytes);

    let mut reader = BitVecReader::new(bytes);
    let mut rpu_nal = read_rpu_data(&mut reader, false);
    //rpu_nal.convert_to_81();

    //println!("{:#?}", rpu_nal);

    //println!("{:#?}", rpu_nal);
    //println!("{} {} {}", &reader.pos(), &reader.len(), &reader.remaining());

    let mut writer = BitVecWriter::new();
    let rest = &reader.get_inner()[rpu_nal.header_end..];

    write_rpu_data(rpu_nal, &mut writer);
    let inner_w = writer.inner_mut();
    //inner_w.extend_from_bitslice(&rest);

    let mut data_to_write = inner_w.as_slice().to_vec();
    //add_start_code_emulation_prevention_3_byte(&mut data_to_write);
    println!("{:?}", data_to_write);

    data_to_write
}

pub fn read_rpu_data(reader: &mut BitVecReader, header_only: bool) -> RpuNal {
    let mut rpu_nal = rpu_data_header(reader);
    rpu_nal.header_end = reader.pos();

    if !header_only {
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

        rpu_nal.rpu_data_crc32 = reader.get_n(32);
    }

    rpu_nal
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

    return;

    rpu_nal.remaining.iter().for_each(|b| writer.write(*b));

    writer.write_n(&rpu_nal.rpu_data_crc32.to_be_bytes(), 32);
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

    rpu_nal.validate();

    rpu_nal
}

impl RpuNal {
    pub fn validate(&self) {
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

    pub fn convert_to_81(&mut self) {
        // Change to RPU only (8.1)
        self.el_spatial_resampling_filter_flag = false;
        self.disable_residual_flag = true;
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