use anyhow::{bail, ensure, Result};

use crate::rpu::dovi_rpu::DoviRpu;

pub const ITU_T35_DOVI_RPU_PAYLOAD_HEADER: &[u8] =
    &[0x00, 0x3B, 0x00, 0x00, 0x08, 0x00, 0x37, 0xCD, 0x08];
const ITU_T35_DOVI_RPU_PAYLOAD_HEADER_LEN: usize = ITU_T35_DOVI_RPU_PAYLOAD_HEADER.len();

fn validated_trimmed_data(data: &mut [u8]) -> Result<&mut [u8]> {
    if data.len() < 34 {
        bail!("Invalid RPU length: {}", data.len());
    }

    let data = if data[0] == 0xB5 {
        // itu_t_t35_country_code - United States
        // Remove from buffer
        &mut data[1..]
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
/// Expects the payload to have `ITU_T35_DOVI_RPU_PAYLOAD_HEADER` discarded
/// The payload is converted in-place in input slice
///
/// Returns the converted slice truncated to final RPU size
pub fn convert_av1_rpu_payload_to_regular(data: &mut [u8]) -> Result<&[u8]> {
    let mut rpu_size;

    // 256+ bytes size
    if data[1] & 0x10 > 0 {
        if data[2] & 0x08 > 0 {
            bail!("RPU exceeds 512 bytes");
        }

        rpu_size = 0x100;
        rpu_size |= (data[1] as usize & 0x0F) << 4;
        rpu_size |= (data[2] as usize >> 4) & 0x0F;

        ensure!(rpu_size + 2 < data.len());

        for i in 0..rpu_size {
            let mut converted_byte = (data[2 + i] & 0x07) << 5;
            converted_byte |= (data[3 + i] >> 3) & 0x1F;

            data[1 + i] = converted_byte;
        }
    } else {
        rpu_size = (data[0] as usize & 0x1F) << 3;
        rpu_size |= (data[1] as usize >> 5) & 0x07;

        ensure!(rpu_size + 1 < data.len());

        for i in 0..rpu_size {
            let mut converted_byte = (data[1 + i] & 0x0F) << 4;
            converted_byte |= (data[2 + i] >> 4) & 0x0F;

            data[1 + i] = converted_byte;
        }
    }

    // Set prefix
    data[0] = 0x19;

    Ok(&data[..rpu_size + 1])
}

/// Buffer must start with 0x19 prefix, the payload is converted in-place
///
/// Returns payload for AV1 ITU T-T.35 metadata OBU
pub fn convert_regular_rpu_to_av1_payload(data: &mut Vec<u8>) -> Result<()> {
    ensure!(data[0] == 0x19);

    // Exclude 0x19 prefix
    let rpu_size = data.len() - 1;

    // Header + size bytes
    data.reserve(16);

    // 256+ bytes size
    if rpu_size > 0xFF {
        // Unknown first byte
        let size_byte1 = 32;

        data.splice(
            0..1,
            [
                size_byte1,
                (rpu_size >> 4) as u8,
                ((rpu_size & 0x0F) as u8) << 4,
            ],
        );
        let start_idx = 3;
        let end_idx = rpu_size + 2;

        for i in start_idx..end_idx {
            let mut byte = (data[i] & 0x1F) << 3;
            byte |= (data[1 + i] >> 5) & 0x07;

            data[i] = byte;
        }

        // Last byte
        data[end_idx] = (data[end_idx] & 0x1F) << 3;

        // Unknown necessary bytes
        data.extend(&[16, 0]);
    } else {
        // Unknown additional diff for first size byte
        let size_byte1_diff = 32; // 2^5

        data.splice(
            0..1,
            [
                (rpu_size >> 3) as u8 + size_byte1_diff,
                ((rpu_size & 0x07) as u8) << 5,
            ],
        );
        let start_idx = 2;
        let end_idx = rpu_size + 1;

        for i in start_idx..end_idx {
            let mut byte = (data[i] & 0x0F) << 4;
            byte |= (data[1 + i] >> 4) & 0x0F;

            data[i] = byte;
        }

        // Last byte
        data[end_idx] = (data[end_idx] & 0x0F) << 4;

        // Unknown necessary bytes
        data.extend(&[size_byte1_diff, 0]);
    }

    // Prefix header
    data.splice(0..0, ITU_T35_DOVI_RPU_PAYLOAD_HEADER.iter().copied());

    Ok(())
}

/// Parse AV1 RPU metadata payload starting with `ITU_T35_DOVI_RPU_PAYLOAD_HEADER`
///
/// The payload is converted in-place in input slice, then parsed into a `DoviRpu` struct.
pub fn parse_itu_t35_dovi_metadata_obu(data: &mut [u8]) -> Result<DoviRpu> {
    let data = validated_trimmed_data(data)?;
    let converted_buf =
        convert_av1_rpu_payload_to_regular(&mut data[ITU_T35_DOVI_RPU_PAYLOAD_HEADER_LEN..])?;

    DoviRpu::parse_rpu(converted_buf)
}
