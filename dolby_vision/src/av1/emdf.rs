use anyhow::{ensure, Result};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

/// Parse the expected EMDF container with fixed values according to spec
/// Returns `emdf_payload_size`
pub(crate) fn parse_emdf_container(reader: &mut BsIoSliceReader) -> Result<usize> {
    let emdf_version = reader.get_n::<u8>(2)?;
    ensure!(emdf_version == 0);

    let key_id = reader.get_n::<u8>(3)?;
    ensure!(key_id == 6);

    let emdf_payload_id = reader.get_n::<u8>(5)?;
    ensure!(emdf_payload_id == 31);

    let emdf_payload_id_ext = parse_variable_bits(reader, 5)?;
    ensure!(emdf_payload_id_ext == 225);

    ensure!(!reader.get()?); // smploffste = 0
    ensure!(!reader.get()?); // duratione = 0
    ensure!(!reader.get()?); // groupide = 0
    ensure!(!reader.get()?); // codecdatae = 0
    ensure!(reader.get()?); // discard_unknown_payload = 1

    let emdf_payload_size = parse_variable_bits(reader, 8)? as usize;
    Ok(emdf_payload_size)
}

/// Write the DOVI RPU EMDF container with payload
pub(crate) fn write_emdf_container_with_dovi_rpu_payload(
    writer: &mut BitstreamIoWriter,
    payload: &[u8],
) -> Result<()> {
    let emdf_payload_size = payload.len() as u32;

    write_dovi_rpu_emdf_header(writer)?;
    write_variable_bits(writer, emdf_payload_size, 8)?;

    for b in payload {
        writer.write_n(b, 8)?;
    }

    // emdf_payload_id and emdf_protection
    writer.write_n(&0, 5)?;
    writer.write_n(&1, 2)?;
    writer.write_n(&0, 2)?;
    writer.write_n(&0, 8)?;

    Ok(())
}

fn write_dovi_rpu_emdf_header(writer: &mut BitstreamIoWriter) -> Result<()> {
    writer.write_n(&0, 2)?; // emdf_version
    writer.write_n(&6, 3)?; // key_id
    writer.write_n(&31, 5)?; // emdf_payload_id
    write_variable_bits(writer, 225, 5)?; // emdf_payload_id_ext

    writer.write_n(&0, 4)?; // smploffste, duratione, groupide, codecdatae
    writer.write(true)?; // discard_unknown_payload

    Ok(())
}

fn parse_variable_bits(reader: &mut BsIoSliceReader, n: u32) -> Result<u32> {
    let mut value: u32 = 0;

    loop {
        let tmp: u32 = reader.get_n(n)?;
        value += tmp;

        // read_more flag
        if !reader.get()? {
            break;
        }

        value <<= n;
        value += 1 << n;
    }

    Ok(value)
}

fn write_variable_bits(writer: &mut BitstreamIoWriter, value: u32, n: u32) -> Result<()> {
    let max = 1 << n;

    if value > max {
        let mut remaining = value;

        loop {
            let tmp = remaining >> n;
            let clipped = tmp << n;
            remaining -= clipped;

            let byte = (clipped - max) >> n;
            writer.write_n(&byte, n)?;
            writer.write(true)?; // read_more

            // Stop once the remaining can be written in N bits
            if remaining <= max {
                break;
            }
        }

        writer.write_n(&remaining, n)?;
    } else {
        writer.write_n(&value, n)?;
    }

    writer.write(false)?;

    Ok(())
}
