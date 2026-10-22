use anyhow::{Context, Result, ensure};
use bitvec_helpers::{
    bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
};

/// Parse the expected EMDF container with fixed values according to spec
/// Returns `emdf_payload_size`
pub(crate) fn parse_emdf_container(reader: &mut BsIoSliceReader) -> Result<u16> {
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

    let emdf_payload_size = parse_variable_bits::<8>(reader)?;
    Ok(emdf_payload_size)
}

/// Write the DOVI RPU EMDF container with payload
pub(crate) fn write_emdf_container_with_dovi_rpu_payload(
    writer: &mut BitstreamIoWriter,
    payload: &[u8],
) -> Result<()> {
    let emdf_payload_size = u16::try_from(payload.len())?;

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

fn parse_variable_bits<const BITS: u32>(reader: &mut BsIoSliceReader) -> Result<u16> {
    let mut value: u16 = 0;

    loop {
        let tmp = reader.read::<BITS, u16>()?;
        value = value
            .checked_add(tmp)
            .context("parse_variable_bits add overflow")?;

        // read_more flag
        if !reader.read_bit()? {
            break;
        }

        value = value
            .checked_shl(BITS)
            .and_then(|v| v.checked_add(1 << BITS))
            .context("variable_bits shift overflow")?;
    }

    Ok(value)
}

fn write_variable_bits<const BITS: u32>(writer: &mut BitstreamIoWriter, value: u16) -> Result<()> {
    let max = 1 << BITS;

    if value >= max {
        let mut remaining = value;

        loop {
            let tmp = remaining >> BITS;
            let clipped = tmp << BITS;
            remaining -= clipped;

            let byte = (clipped - max) >> BITS;
            writer.write::<BITS, u16>(byte)?;
            writer.write_bit(true)?; // read_more

            // Stop once the remaining can be written in N bits
            if remaining <= max {
                break;
            }
        }

        writer.write::<BITS, u16>(remaining)?;
    } else {
        writer.write::<BITS, u16>(value)?;
    }

    writer.write_bit(false)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bitvec_helpers::{
        bitstream_io_reader::BsIoSliceReader, bitstream_io_writer::BitstreamIoWriter,
    };

    use super::{parse_variable_bits, write_variable_bits};

    #[test]
    fn variable_bits_one_group_roundtrip() -> Result<()> {
        let value = 148;

        let mut writer = BitstreamIoWriter::with_capacity(2);
        write_variable_bits::<8>(&mut writer, value)?;

        writer.byte_align()?;
        let out = writer.as_slice().unwrap();
        assert_eq!(out, &[148, 0]);

        let mut reader = BsIoSliceReader::from_slice(out);
        assert_eq!(parse_variable_bits::<8>(&mut reader)?, value);

        Ok(())
    }

    #[test]
    fn write_variable_bits_u8_max() -> Result<()> {
        let value = 256;

        let mut writer = BitstreamIoWriter::with_capacity(2);
        write_variable_bits::<8>(&mut writer, value)?;

        writer.byte_align()?;
        let out = writer.as_slice().unwrap();
        assert_eq!(out, &[0, 128, 0]);

        let mut reader = BsIoSliceReader::from_slice(out);
        assert_eq!(parse_variable_bits::<8>(&mut reader)?, value);

        Ok(())
    }

    #[test]
    fn write_variable_bits_two_groups_roundtrip() -> Result<()> {
        let value = 58472;

        let mut writer = BitstreamIoWriter::with_capacity(2);
        write_variable_bits::<8>(&mut writer, value)?;

        writer.byte_align()?;
        let out = writer.as_slice().unwrap();
        assert_eq!(out, &[227, 180, 0]);

        let mut reader = BsIoSliceReader::from_slice(out);
        assert_eq!(parse_variable_bits::<8>(&mut reader)?, value);

        Ok(())
    }

    #[test]
    fn write_variable_bits_max_supported_roundtrip() -> Result<()> {
        let value = 65535;

        let mut writer = BitstreamIoWriter::with_capacity(2);
        write_variable_bits::<8>(&mut writer, value)?;

        writer.byte_align()?;
        let out = writer.as_slice().unwrap();
        assert_eq!(out, &[254, 255, 128]);

        let mut reader = BsIoSliceReader::from_slice(out);
        assert_eq!(parse_variable_bits::<8>(&mut reader)?, value);

        Ok(())
    }

    #[test]
    fn parse_variable_bits_checked_acc() -> Result<()> {
        let mut reader = BsIoSliceReader::from_slice(&[255, 255, 0]);
        assert!(parse_variable_bits::<8>(&mut reader).is_err());

        Ok(())
    }
}
