#![allow(dead_code)]

use std::io::{BufRead, ErrorKind, Read, Write};

use anyhow::{Result, bail};

// ---------------------------------------------------------------------------
// OBU type constants (AV1 spec Table 5)
// ---------------------------------------------------------------------------
pub const OBU_SEQUENCE_HEADER: u8 = 1;
pub const OBU_TEMPORAL_DELIMITER: u8 = 2;
pub const OBU_FRAME_HEADER: u8 = 3;
pub const OBU_METADATA: u8 = 5;
pub const OBU_FRAME: u8 = 6;
pub const OBU_REDUNDANT_FRAME_HEADER: u8 = 7;

// ---------------------------------------------------------------------------
// Obu — a single parsed OBU with its complete raw bytes
// ---------------------------------------------------------------------------

/// A single parsed AV1 Open Bitstream Unit.
pub struct Obu {
    pub obu_type: u8,
    pub temporal_id: u8,
    pub spatial_id: u8,
    /// Decoded payload bytes (after header + LEB128 size).
    pub payload: Vec<u8>,
    /// Complete raw bytes of this OBU as it appeared on disk.
    /// Used for pass-through writing.
    pub raw_bytes: Vec<u8>,
}

impl Obu {
    /// Read one OBU from `reader`.  Returns `None` on clean EOF.
    ///
    /// Only supports the *Low Overhead Bitstream Format* where every OBU
    /// carries a size field (`obu_has_size_field == 1`).
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Option<Self>> {
        // ---- header byte ----
        let mut header_byte = [0u8; 1];
        match reader.read_exact(&mut header_byte) {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        }

        let byte = header_byte[0];
        if byte >> 7 != 0 {
            bail!("AV1 OBU forbidden bit is set (byte = 0x{byte:02X})");
        }

        let obu_type = (byte >> 3) & 0x0F;
        let has_extension = (byte >> 2) & 1 != 0;
        let has_size_field = (byte >> 1) & 1 != 0;

        let mut raw = vec![byte];
        let mut temporal_id = 0u8;
        let mut spatial_id = 0u8;

        // ---- optional extension header ----
        if has_extension {
            let mut ext = [0u8; 1];
            reader.read_exact(&mut ext)?;
            temporal_id = (ext[0] >> 5) & 0x07;
            spatial_id = (ext[0] >> 3) & 0x03;
            raw.push(ext[0]);
        }

        if !has_size_field {
            bail!(
                "OBU (type {obu_type}) has no size field; \
                 only Low Overhead Bitstream Format is supported"
            );
        }

        // ---- LEB128 payload size ----
        let payload_size = {
            let mut size: u64 = 0;
            let mut shift = 0u32;
            loop {
                let mut b = [0u8; 1];
                reader.read_exact(&mut b)?;
                raw.push(b[0]);
                size |= ((b[0] & 0x7F) as u64) << shift;
                shift += 7;
                if b[0] & 0x80 == 0 {
                    break;
                }
                if shift >= 56 {
                    bail!("LEB128 overflow while reading OBU size");
                }
            }
            size as usize
        };

        // ---- payload ----
        let payload_start = raw.len();
        raw.resize(payload_start + payload_size, 0);
        reader.read_exact(&mut raw[payload_start..])?;
        let payload = raw[payload_start..].to_vec();

        Ok(Some(Obu {
            obu_type,
            temporal_id,
            spatial_id,
            payload,
            raw_bytes: raw,
        }))
    }
}

// ---------------------------------------------------------------------------
// LEB128 encoding / decoding
// ---------------------------------------------------------------------------

/// Encode a `u64` value as LEB128 (unsigned).
pub fn encode_leb128(mut value: u64) -> Vec<u8> {
    let mut result = Vec::new();
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        result.push(byte);
        if value == 0 {
            break;
        }
    }
    result
}

/// Decode a LEB128-encoded value from `data`.
/// Returns `(value, bytes_consumed)`.
pub fn decode_leb128(data: &[u8]) -> (u64, usize) {
    let mut value = 0u64;
    let mut bytes_read = 0usize;
    for (i, &byte) in data.iter().enumerate() {
        if i >= 8 {
            break;
        }
        value |= ((byte & 0x7F) as u64) << (7 * i);
        bytes_read += 1;
        if byte & 0x80 == 0 {
            break;
        }
    }
    (value, bytes_read)
}

// ---------------------------------------------------------------------------
// IVF container support
// ---------------------------------------------------------------------------

/// IVF file signature ("DKIF").
pub const IVF_SIGNATURE: [u8; 4] = *b"DKIF";

/// Size of the IVF file header in bytes.
pub const IVF_FILE_HEADER_LEN: usize = 32;

/// Size of an IVF frame header in bytes.
pub const IVF_FRAME_HEADER_LEN: usize = 12;

/// Header of a single IVF frame.
pub struct IvfFrameHeader {
    /// Number of bytes in the frame data that follows.
    pub frame_size: u32,
    /// Presentation timestamp (in stream timebase).
    pub timestamp: u64,
}

/// Probe the first bytes of `reader` to decide whether the stream is an IVF
/// container. If the IVF signature is detected the 32-byte file header is
/// consumed from `reader` and returned; otherwise `None` is returned and
/// **no bytes are consumed**.
pub fn try_read_ivf_file_header<R: BufRead>(
    reader: &mut R,
) -> Result<Option<[u8; IVF_FILE_HEADER_LEN]>> {
    {
        let buf = reader.fill_buf()?;
        if buf.len() < 4 || buf[..4] != IVF_SIGNATURE {
            return Ok(None);
        }
    }
    let mut header = [0u8; IVF_FILE_HEADER_LEN];
    reader.read_exact(&mut header)?;
    Ok(Some(header))
}

/// Read one IVF frame header from `reader`. Returns `None` on clean EOF.
pub fn read_ivf_frame_header<R: Read>(reader: &mut R) -> Result<Option<IvfFrameHeader>> {
    let mut buf = [0u8; IVF_FRAME_HEADER_LEN];
    match reader.read_exact(&mut buf) {
        Ok(()) => {}
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    }
    Ok(Some(IvfFrameHeader {
        frame_size: u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]),
        timestamp: u64::from_le_bytes(buf[4..12].try_into().unwrap()),
    }))
}

/// Write an IVF frame header (frame_size + timestamp) to `writer`.
pub fn write_ivf_frame_header<W: Write>(
    writer: &mut W,
    frame_size: u32,
    timestamp: u64,
) -> Result<()> {
    writer.write_all(&frame_size.to_le_bytes())?;
    writer.write_all(&timestamp.to_le_bytes())?;
    Ok(())
}

/// Read all OBUs from a single IVF frame's data bytes.
pub fn read_obus_from_ivf_frame(frame_data: Vec<u8>) -> Result<Vec<Obu>> {
    let mut cursor = std::io::Cursor::new(frame_data);
    let mut obus = Vec::new();
    while let Some(obu) = Obu::read_from(&mut cursor)? {
        obus.push(obu);
    }
    Ok(obus)
}

// ---------------------------------------------------------------------------
// I/O structs
// ---------------------------------------------------------------------------

/// Iterates OBUs from a raw AV1 byte stream.
pub struct ObuReader<R: Read> {
    reader: R,
}

impl<R: Read> ObuReader<R> {
    pub fn new(reader: R) -> Self {
        ObuReader { reader }
    }
    pub fn next_obu(&mut self) -> Result<Option<Obu>> {
        Obu::read_from(&mut self.reader)
    }
    pub fn into_inner(self) -> R {
        self.reader
    }
}

impl<R: Read> Iterator for ObuReader<R> {
    type Item = Result<Obu>;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_obu().transpose()
    }
}

/// Writes IVF frames. Writes the file header in `new()`.
pub struct IvfWriter<W: Write> {
    writer: W,
}

impl<W: Write> IvfWriter<W> {
    /// Writes the 32-byte IVF file header immediately.
    pub fn new(mut writer: W, file_header: &[u8; 32]) -> Result<Self> {
        writer.write_all(file_header)?;
        Ok(IvfWriter { writer })
    }

    /// Writes one IVF frame (12-byte frame header + frame data).
    pub fn write_frame(&mut self, timestamp: u64, frame_data: &[u8]) -> Result<()> {
        write_ivf_frame_header(&mut self.writer, frame_data.len() as u32, timestamp)?;
        self.writer.write_all(frame_data)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush().map_err(Into::into)
    }

    pub fn into_inner(self) -> W {
        self.writer
    }
}

/// Writes raw AV1 OBUs directly.
pub struct ObuWriter<W: Write> {
    writer: W,
}

impl<W: Write> ObuWriter<W> {
    pub fn new(writer: W) -> Self {
        ObuWriter { writer }
    }
    pub fn write_raw(&mut self, bytes: &[u8]) -> Result<()> {
        self.writer.write_all(bytes).map_err(Into::into)
    }
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush().map_err(Into::into)
    }
    pub fn into_inner(self) -> W {
        self.writer
    }
}
