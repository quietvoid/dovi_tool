use super::{
    add_start_code_emulation_prevention_3_byte, rpu_data_header,
    vdr_dm_data::{self, ExtMetadataBlockLevel5},
    vdr_rpu_data, BitVecReader, BitVecWriter,
};

use super::prelude::*;
use crc::{Crc, CRC_32_MPEG_2};
use rpu_data_header::RpuDataHeader;
use vdr_dm_data::VdrDmData;
use vdr_rpu_data::{NlqData, VdrRpuData};

#[derive(Default, Debug)]
pub struct DoviRpu {
    pub dovi_profile: u8,
    pub reader: BitVecReader,
    pub header: RpuDataHeader,
    pub vdr_rpu_data: Option<VdrRpuData>,
    pub nlq_data: Option<NlqData>,
    pub vdr_dm_data: Option<VdrDmData>,
    pub remaining: BitVec<Msb0, u8>,
    pub rpu_data_crc32: u32,
    pub last_byte: u8,

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
    pub fn read_rpu_data(bytes: Vec<u8>, end_byte: u8) -> DoviRpu {
        let mut dovi_rpu = DoviRpu::new(bytes);
        dovi_rpu.last_byte = end_byte;

        let reader = &mut dovi_rpu.reader;
        dovi_rpu.header = RpuDataHeader::rpu_data_header(reader);

        // Preliminary header validation
        dovi_rpu.dovi_profile = dovi_rpu.header.get_dovi_profile();

        dovi_rpu.header.validate(dovi_rpu.dovi_profile);

        if dovi_rpu.header.rpu_type == 2 {
            if !dovi_rpu.header.use_prev_vdr_rpu_flag {
                let (vdr_rpu_data, nlq_data) =
                    VdrRpuData::vdr_rpu_data_payload(reader, &mut dovi_rpu.header);

                dovi_rpu.vdr_rpu_data = vdr_rpu_data;
                dovi_rpu.nlq_data = nlq_data;
            }

            if dovi_rpu.header.vdr_dm_metadata_present_flag {
                dovi_rpu.vdr_dm_data = Some(VdrDmData::vdr_dm_data_payload(reader));
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
            assert_eq!(last_byte, 0x80);
        }

        dovi_rpu.validate();

        dovi_rpu
    }

    fn convert_to_mel(&mut self) {
        if let Some(ref mut nlq_data) = self.nlq_data {
            nlq_data.convert_to_mel();
        } else {
            panic!("Not profile 7, cannot convert to MEL!");
        }
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
    pub fn write_rpu_data(&mut self) -> Vec<u8> {
        let mut writer = BitVecWriter::new();

        let header = &self.header;
        header.write_header(&mut writer);

        if header.rpu_type == 2 {
            if !header.use_prev_vdr_rpu_flag {
                self.write_vdr_rpu_data(&mut writer);
            }

            if header.vdr_dm_metadata_present_flag {
                self.write_vdr_dm_data(&mut writer);
            }
        }

        self.remaining.iter().for_each(|b| writer.write(*b));

        let computed_crc32 = DoviRpu::compute_crc32(&writer.as_slice()[1..]);

        if !self.modified {
            // Validate the parsed crc32 is the same
            assert_eq!(self.rpu_data_crc32, computed_crc32);
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

        data_to_write
    }

    pub fn write_vdr_rpu_data(&self, writer: &mut BitVecWriter) {
        if let Some(ref vdr_rpu_data) = self.vdr_rpu_data {
            vdr_rpu_data.write(writer, &self.header);
        }

        if let Some(ref nlq_data) = self.nlq_data {
            nlq_data.write(writer, &self.header);
        }
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
        digest.update(&data);

        digest.finalize()
    }

    pub fn convert_with_mode(&mut self, mode: u8) {
        if mode != 0 {
            self.modified = true;
        }

        if self.dovi_profile == 7 {
            match mode {
                1 => self.convert_to_mel(),
                2 => self.convert_to_81(),
                _ => (),
            }
        } else if self.dovi_profile == 5 && mode == 3 {
            self.p5_to_p81();
        } else if mode != 0 {
            panic!("Invalid profile for mode {} conversion!", mode);
        }
    }

    pub fn crop(&mut self) {
        self.modified = true;

        if let Some(block) = ExtMetadataBlockLevel5::get_mut(self) {
            block.crop();
        }
    }

    fn p5_to_p81(&mut self) {
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
            panic!("Attempt to convert profile 5: RPU is not profile 5!");
        }
    }

    pub fn validate(&mut self) {
        self.dovi_profile = self.header.get_dovi_profile();
        self.header.validate(self.dovi_profile);

        if let Some(ref mut vdr_dm_data) = self.vdr_dm_data {
            vdr_dm_data.validate(self.dovi_profile);
        }
    }
}
