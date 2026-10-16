//! Malformed RPUs must be rejected with an error.
//! The C API is built with `panic=abort`, so a panic or a failed allocation
//! while parsing takes down the host process instead of failing one frame.

use anyhow::Result;
use bitvec_helpers::bitstream_io_writer::BitstreamIoWriter;

use super::compute_crc32;
use super::dovi_rpu::{DoviRpu, FINAL_BYTE};
use super::rpu_data_header::RpuDataHeader;

/// Profile 8.1 RPU: the header is written by the crate, everything after it by `body`.
/// The RPU is closed with alignment, the correct CRC32 and the final byte.
fn craft(
    vdr_dm_metadata_present: bool,
    body: impl FnOnce(&mut BitstreamIoWriter) -> Result<()>,
) -> Result<Vec<u8>> {
    let header = RpuDataHeader {
        vdr_dm_metadata_present_flag: vdr_dm_metadata_present,
        ..RpuDataHeader::p8_default()
    };

    let mut writer = BitstreamIoWriter::with_capacity(512);
    writer.write_const::<8, 0x19>()?;
    header.write_header(&mut writer)?;
    body(&mut writer)?;
    writer.byte_align()?;

    let crc32 = compute_crc32(&writer.as_slice().unwrap()[1..]);
    writer.write::<32, u32>(crc32)?;
    writer.write::<8, u8>(FINAL_BYTE)?;

    Ok(writer.into_inner())
}

fn write_mapping_prefix(writer: &mut BitstreamIoWriter) -> Result<()> {
    writer.write_ue(0)?; // vdr_rpu_id
    writer.write_ue(0)?; // mapping_color_space
    writer.write_ue(0)?; // mapping_chroma_format_idc

    Ok(())
}

/// `rpu_data_mapping` with one polynomial piece per pivot interval.
/// Luma has `luma_num_pivots_minus2 + 2` pivots, chroma two each.
fn write_mapping(
    writer: &mut BitstreamIoWriter,
    luma_num_pivots_minus2: u64,
    linear_interp: bool,
) -> Result<()> {
    write_mapping_prefix(writer)?;

    let num_pivots_minus2 = [luma_num_pivots_minus2, 0, 0];

    for n in num_pivots_minus2 {
        writer.write_ue(n)?;

        for i in 0..n + 2 {
            writer.write::<10, u16>((i * 1023 / (n + 1)) as u16)?;
        }
    }

    writer.write_ue(0)?; // num_x_partitions_minus1
    writer.write_ue(0)?; // num_y_partitions_minus1

    for n in num_pivots_minus2 {
        for _ in 0..n + 1 {
            writer.write_ue(0)?; // mapping_idc: polynomial
            writer.write_ue(0)?; // poly_order_minus1
            writer.write_bit(linear_interp)?;

            if !linear_interp {
                for coef_int in [0, 1] {
                    writer.write_se(coef_int)?;
                    writer.write::<23, u32>(0)?;
                }
            }
        }
    }

    Ok(())
}

/// `DoviRpu::parse` is what every entry point ends up in.
/// Unlike them, it accepts RPUs shorter than 25 bytes.
fn error_message(data: &[u8]) -> String {
    match DoviRpu::parse(data) {
        Ok(_) => panic!("malformed RPU was parsed"),
        Err(e) => format!("{e:#}"),
    }
}

#[test]
fn crafted_rpus_parse() -> Result<()> {
    let no_dm = craft(false, |w| write_mapping(w, 0, false))?;
    let rpu = DoviRpu::parse(&no_dm)?;
    assert_eq!(rpu.dovi_profile, 8);

    let nine_pivots = craft(false, |w| write_mapping(w, 7, false))?;
    let rpu = DoviRpu::parse(&nine_pivots)?;
    assert_eq!(rpu.rpu_data_mapping.unwrap().curves[0].pivots.len(), 9);

    Ok(())
}

#[test]
fn bad_crc32_is_rejected_before_parsing() -> Result<()> {
    // The payload would fail to parse, the CRC32 must be what rejects it
    let mut data = craft(false, |w| write_mapping(w, 0, true))?;
    let crc32_start = data.len() - 5;
    data[crc32_start] ^= 1;

    assert!(error_message(&data).contains("CRC32"));

    Ok(())
}

#[test]
fn linear_interpolation_is_an_error() -> Result<()> {
    let data = craft(false, |w| write_mapping(w, 0, true))?;
    assert!(error_message(&data).contains("interpolation"));

    Ok(())
}

#[test]
fn num_pivots_above_nine_is_an_error() -> Result<()> {
    for num_pivots_minus2 in [u64::MAX - 1, 1 << 40, 8] {
        let data = craft(false, |w| {
            write_mapping_prefix(w)?;
            Ok(w.write_ue(num_pivots_minus2)?)
        })?;
        assert!(error_message(&data).contains("num_pivots_minus2"));
    }

    Ok(())
}
