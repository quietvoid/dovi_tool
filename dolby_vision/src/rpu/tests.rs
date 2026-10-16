//! Malformed RPUs must be rejected with an error.
//! The C API is built with `panic=abort`, so a panic or a failed allocation
//! while parsing takes down the host process instead of failing one frame.

use anyhow::Result;
use bitvec_helpers::bitstream_io_writer::BitstreamIoWriter;

use super::compute_crc32;
use super::dovi_rpu::{DoviRpu, FINAL_BYTE};
use super::rpu_data_header::RpuDataHeader;
use super::vdr_dm_data::VdrDmData;

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

/// `vdr_dm_data_payload` with no CM v2.9 blocks and the given CM v4.0 blocks.
/// Each block is (level, ext_block_length, payload), the payload zero padded up to the length.
fn write_dm_data(writer: &mut BitstreamIoWriter, cmv40_blocks: &[(u8, u64, &[u8])]) -> Result<()> {
    VdrDmData::default_pq().write(writer)?;

    writer.write_ue(0)?; // CM v2.9 num_ext_blocks
    writer.byte_align()?;

    writer.write_ue(cmv40_blocks.len() as u64)?;
    writer.byte_align()?;

    for &(level, length, payload) in cmv40_blocks {
        writer.write_ue(length)?;
        writer.write::<8, u8>(level)?;
        writer.write_bytes(payload)?;

        for _ in payload.len() as u64..length {
            writer.write::<8, u8>(0)?;
        }
    }

    Ok(())
}

/// L254 is mandatory in CM v4.0 metadata.
const LEVEL254: (u8, u64, &[u8]) = (254, 2, &[0, 2]);

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

    let with_dm = craft(true, |w| {
        write_mapping(w, 0, false)?;
        write_dm_data(w, &[(8, 10, &[1]), LEVEL254])
    })?;
    let rpu = DoviRpu::parse(&with_dm)?;
    assert!(rpu.vdr_dm_data.as_ref().unwrap().get_block(8).is_some());

    // The crafter writes what the crate's own writer writes
    assert_eq!(rpu.write_rpu()?, with_dm);

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

#[test]
fn undefined_block_lengths_are_errors() -> Result<()> {
    // Enough payload for the longest variant, so only the length is wrong
    let payload = [0; 25];

    for (level, length) in [(8, 11), (8, 26), (9, 2), (10, 6)] {
        let data = craft(true, |w| {
            write_mapping(w, 0, false)?;
            write_dm_data(w, &[(level, length, &payload), LEVEL254])
        })?;
        let message = error_message(&data);
        assert!(
            message.contains(&format!("Level {level} block length")),
            "L{level} length {length}: {message}"
        );
    }

    Ok(())
}
