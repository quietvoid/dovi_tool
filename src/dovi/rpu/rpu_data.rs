use super::{
    add_start_code_emulation_prevention_3_byte, rpu_data_header, vdr_dm_data, vdr_rpu_data,
    BitVecReader, BitVecWriter,
};

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
    pub crc32_offset: usize,
    pub rpu_data_crc32: u32,
}

impl DoviRpu {
    pub fn new(bytes: Vec<u8>) -> DoviRpu {
        DoviRpu {
            reader: BitVecReader::new(bytes),
            ..Default::default()
        }
    }

    pub fn read_rpu_data(bytes: Vec<u8>) -> DoviRpu {
        let mut dovi_rpu = DoviRpu::new(bytes);
        let reader = &mut dovi_rpu.reader;
        dovi_rpu.header = RpuDataHeader::rpu_data_header(reader);

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
                assert_eq!(reader.get(), false);
            }

            dovi_rpu.crc32_offset = reader.pos();
            dovi_rpu.rpu_data_crc32 = reader.get_n(32);
        }

        dovi_rpu
    }

    pub fn convert_to_mel(&mut self) {
        if let Some(ref mut nlq_data) = self.nlq_data {
            nlq_data.convert_to_mel();
        } else {
            panic!("Not profile 7, cannot convert to MEL!");
        }
    }

    pub fn convert_to_81(&mut self) {
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

    pub fn write_rpu_data(&mut self, mode: u8) -> Vec<u8> {
        if self.dovi_profile == 7 {
            match mode {
                1 => self.convert_to_mel(),
                2 => self.convert_to_81(),
                _ => (),
            }
        } else if mode != 0 {
            panic!("Can only change profile 7 RPU!");
        }

        let reader = &self.reader;
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

        while !writer.is_aligned() {
            writer.write(false);
        }

        let computed_crc32 = DoviRpu::compute_crc32(&writer.as_slice()[1..]);

        // Write crc32
        writer.write_n(&computed_crc32.to_be_bytes(), 32);

        // Write whatever is left
        let rest = &reader.get_inner()[reader.pos()..];
        let inner_w = writer.inner_mut();
        inner_w.extend_from_bitslice(&rest);

        // Back to a u8 slice
        let mut data_to_write = inner_w.as_slice().to_vec();
        add_start_code_emulation_prevention_3_byte(&mut data_to_write);

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

    pub fn compute_crc32(data: &[u8]) -> u32 {
        let crc = Crc::<u32>::new(&CRC_32_MPEG_2);
        let mut digest = crc.digest();
        digest.update(&data);

        digest.finalize()
    }
}
