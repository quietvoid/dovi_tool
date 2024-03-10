use anyhow::{anyhow, bail, ensure, Result};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

use crate::{
    av1::emdf::{parse_emdf_container, write_emdf_container_with_dovi_rpu_payload},
    rpu::dovi_rpu::DoviRpu,
};

mod emdf;

pub const ITU_T35_DOVI_RPU_PAYLOAD_HEADER: &[u8] =
    &[0x00, 0x3B, 0x00, 0x00, 0x08, 0x00, 0x37, 0xCD, 0x08];
const ITU_T35_DOVI_RPU_PAYLOAD_HEADER_LEN: usize = ITU_T35_DOVI_RPU_PAYLOAD_HEADER.len();

/// Parse AV1 ITU-T T.35 metadata OBU into a `DoviRpu`
/// The payload is extracted out of the EMDF wrapper
pub fn parse_itu_t35_dovi_metadata_obu(data: &[u8]) -> Result<DoviRpu> {
    let data = validated_trimmed_data(data)?;
    let converted_buf = convert_av1_rpu_payload_to_regular(data)?;

    DoviRpu::parse_rpu(&converted_buf)
}

fn validated_trimmed_data(data: &[u8]) -> Result<&[u8]> {
    if data.len() < 34 {
        bail!("Invalid RPU length: {}", data.len());
    }

    let data = if data[0] == 0xB5 {
        // itu_t_t35_country_code - United States
        // Remove from buffer
        &data[1..]
    } else {
        data
    };

    let trimmed_data = match &data[..ITU_T35_DOVI_RPU_PAYLOAD_HEADER_LEN] {
        ITU_T35_DOVI_RPU_PAYLOAD_HEADER => data,
        _ => bail!(
            "Invalid AV1 RPU payload header: {:?}",
            &data[..ITU_T35_DOVI_RPU_PAYLOAD_HEADER_LEN]
        ),
    };

    Ok(trimmed_data)
}

/// Internal function, use `parse_itu_t35_dovi_metadata_obu`
///
/// Returns the EMDF payload bytes representing the RPU buffer
fn convert_av1_rpu_payload_to_regular(data: &[u8]) -> Result<Vec<u8>> {
    let mut reader = BsIoSliceReader::from_slice(data);

    let itu_t_t35_terminal_provider_code = reader.get_n::<u16>(16)?;
    ensure!(itu_t_t35_terminal_provider_code == 0x3B);

    let itu_t_t35_terminal_provider_oriented_code = reader.get_n::<u32>(32)?;
    ensure!(itu_t_t35_terminal_provider_oriented_code == 0x800);

    let emdf_payload_size = parse_emdf_container(&mut reader)?;
    let mut converted_buf = Vec::with_capacity(emdf_payload_size + 1);
    converted_buf.push(0x19);

    for _ in 0..emdf_payload_size {
        converted_buf.push(reader.get_n(8)?);
    }

    Ok(converted_buf)
}

/// Wraps a regular RPU into EMDF container with ITU-T T.35 header
/// Buffer must start with 0x19 prefix.
///
/// Returns payload for AV1 ITU T-T.35 metadata OBU
pub fn convert_regular_rpu_to_av1_payload(data: &[u8]) -> Result<Vec<u8>> {
    ensure!(data[0] == 0x19);

    // Exclude 0x19 prefix
    let data = &data[1..];
    let rpu_size = data.len();
    let capacity = 16 + rpu_size;

    let mut writer = BitstreamIoWriter::with_capacity(capacity * 8);

    writer.write_n(&0x3B, 16)?; // itu_t_t35_terminal_provider_code
    writer.write_n(&0x800, 32)?; // itu_t_t35_terminal_provider_oriented_code

    write_emdf_container_with_dovi_rpu_payload(&mut writer, data)?;

    while !writer.is_aligned() {
        writer.write(true)?;
    }

    Ok(writer
        .as_slice()
        .ok_or_else(|| anyhow!("Unaligned bytes"))?
        .to_owned())
}
