// Re-export everything the rest of the codebase uses from the av1_parser crate
#[allow(unused_imports)]
pub use av1_parser::{
    IvfFrameHeader, IvfWriter, Obu, ObuReader, ObuWriter,
    OBU_TEMPORAL_DELIMITER, OBU_METADATA,
    encode_leb128, decode_leb128,
    try_read_ivf_file_header, read_ivf_frame_header, write_ivf_frame_header,
    read_obus_from_ivf_frame,
};

use anyhow::Result;
use dolby_vision::rpu::dovi_rpu::DoviRpu;

use dolby_vision::av1::ITU_T35_DOVI_RPU_PAYLOAD_HEADER;

/// Metadata type for ITU-T T.35
const METADATA_TYPE_ITUT_T35: u64 = 4;

/// Dolby Vision T.35 country code (United States)
const DOVI_COUNTRY_CODE: u8 = 0xB5;

/// Returns the T.35 payload bytes (starting at `0xB5` country code) if this
/// `OBU_METADATA` payload contains a Dolby Vision RPU.
///
/// Layout after `metadata_type = 4` (LEB128):
/// ```text
/// country_code          (u8)      = 0xB5
/// terminal_provider_code (u16 BE) = 0x003B
/// terminal_provider_oriented_code (u32 BE) = 0x00000800
/// <EMDF container with RPU>
/// ```
pub fn extract_dovi_t35_payload(obu_payload: &[u8]) -> Option<&[u8]> {
    if obu_payload.is_empty() {
        return None;
    }

    // metadata_type (LEB128) must be 4
    let (mt, mt_len) = decode_leb128(obu_payload);
    if mt != METADATA_TYPE_ITUT_T35 {
        return None;
    }

    let t35 = &obu_payload[mt_len..];

    // Must start with Dolby Vision country code
    if t35.is_empty() || t35[0] != DOVI_COUNTRY_CODE {
        return None;
    }

    // After country code, the next bytes must match the Dolby Vision header
    let after_cc = &t35[1..];
    let hdr_len = ITU_T35_DOVI_RPU_PAYLOAD_HEADER.len();
    if after_cc.len() < hdr_len {
        return None;
    }

    if &after_cc[..hdr_len] == ITU_T35_DOVI_RPU_PAYLOAD_HEADER {
        Some(t35) // return slice starting at 0xB5
    } else {
        None
    }
}

/// Returns `true` if this OBU is an `OBU_METADATA` carrying a Dolby Vision RPU.
pub fn is_dovi_rpu_obu(obu: &Obu) -> bool {
    obu.obu_type == OBU_METADATA && extract_dovi_t35_payload(&obu.payload).is_some()
}

/// Build a complete `OBU_METADATA` unit containing the Dolby Vision RPU.
///
/// Structure:
/// ```text
/// OBU header byte  = 0x2A  (type=5, has_size_field=1)
/// OBU size         (LEB128)
/// metadata_type    (LEB128) = 4
/// 0xB5             country_code
/// <EMDF-wrapped RPU payload>
/// ```
pub fn build_dovi_obu(rpu: &DoviRpu) -> Result<Vec<u8>> {
    // write_av1_rpu_metadata_obu_t35_complete returns: 0xB5 + EMDF payload
    let t35_complete = rpu.write_av1_rpu_metadata_obu_t35_complete()?;

    // OBU_METADATA payload: metadata_type(LEB128=4) + T.35 complete payload
    let mut obu_payload = encode_leb128(METADATA_TYPE_ITUT_T35);
    obu_payload.extend_from_slice(&t35_complete);

    // OBU header byte:
    //   bit 7:   forbidden = 0
    //   bits 6-3: obu_type = 5 (OBU_METADATA)
    //   bit 2:   obu_extension_flag = 0
    //   bit 1:   obu_has_size_field = 1
    //   bit 0:   reserved = 0
    // => (5 << 3) | 0x02 = 0x2A
    let header_byte = (OBU_METADATA << 3) | 0x02u8;
    let size_bytes = encode_leb128(obu_payload.len() as u64);

    let mut result = Vec::with_capacity(1 + size_bytes.len() + obu_payload.len());
    result.push(header_byte);
    result.extend_from_slice(&size_bytes);
    result.extend_from_slice(&obu_payload);

    Ok(result)
}
