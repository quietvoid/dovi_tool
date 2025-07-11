use anyhow::{Result, ensure};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

/// Parse the expected EMDF container with fixed values according to spec
/// Returns `emdf_payload_size`
pub(crate) fn parse_emdf_container(reader: &mut BsIoSliceReader) -> Result<usize> {
    let emdf_version = reader.read::<2, u8>()?;
    ensure!(emdf_version == 0);

    let key_id = reader.read::<3, u8>()?;
    ensure!(key_id == 6);

    let emdf_payload_id = reader.read::<5, u8>()?;
    ensure!(emdf_payload_id == 31);

    let emdf_payload_id_ext = parse_variable_bits::<5>(reader)?;
    ensure!(emdf_payload_id_ext == 225);

    ensure!(!reader.read_bit()?); // smploffste = 0
    ensure!(!reader.read_bit()?); // duratione = 0
    ensure!(!reader.read_bit()?); // groupide = 0
    ensure!(!reader.read_bit()?); // codecdatae = 0
    ensure!(reader.read_bit()?); // discard_unknown_payload = 1

    let emdf_payload_size = parse_variable_bits::<8>(reader)? as usize;
    Ok(emdf_payload_size)
}

/// Write the DOVI RPU EMDF container with payload
pub(crate) fn write_emdf_container_with_dovi_rpu_payload(
    writer: &mut BitstreamIoWriter,
    payload: &[u8],
) -> Result<()> {
    let emdf_payload_size = payload.len() as u32;

    write_dovi_rpu_emdf_header(writer)?;
    write_variable_bits::<8>(writer, emdf_payload_size)?;

    writer.write_bytes(payload)?;

    // emdf_payload_id and emdf_protection
    writer.write_const::<5, 0>()?;
    writer.write_const::<2, 1>()?;
    writer.write_const::<2, 0>()?;
    writer.write_const::<8, 0>()?;

    Ok(())
}

fn write_dovi_rpu_emdf_header(writer: &mut BitstreamIoWriter) -> Result<()> {
    writer.write_const::<2, 0>()?; // emdf_version
    writer.write_const::<3, 6>()?; // key_id
    writer.write_const::<5, 31>()?; // emdf_payload_id
    write_variable_bits::<5>(writer, 225)?; // emdf_payload_id_ext

    writer.write_const::<4, 0>()?; // smploffste, duratione, groupide, codecdatae
    writer.write_bit(true)?; // discard_unknown_payload

    Ok(())
}

fn parse_variable_bits<const BITS: u32>(reader: &mut BsIoSliceReader) -> Result<u32> {
    let mut value: u32 = 0;

    loop {
        let tmp = reader.read::<BITS, u32>()?;
        value += tmp;

        // read_more flag
        if !reader.read_bit()? {
            break;
        }

        value <<= BITS;
        value += 1 << BITS;
    }

    Ok(value)
}

fn write_variable_bits<const BITS: u32>(writer: &mut BitstreamIoWriter, value: u32) -> Result<()> {
    let max = 1 << BITS;

    if value > max {
        let mut remaining = value;

        loop {
            let tmp = remaining >> BITS;
            let clipped = tmp << BITS;
            remaining -= clipped;

            let byte = (clipped - max) >> BITS;
            writer.write::<BITS, u32>(byte)?;
            writer.write_bit(true)?; // read_more

            // Stop once the remaining can be written in N bits
            if remaining <= max {
                break;
            }
        }

        writer.write::<BITS, u32>(remaining)?;
    } else {
        writer.write::<BITS, u32>(value)?;
    }

    writer.write_bit(false)?;

    Ok(())
}
