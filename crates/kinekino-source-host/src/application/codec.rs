use crate::domain::*;
use std::io::{self, Cursor, Read};

fn read_exact(reader: &mut impl Read, bytes: &mut [u8]) -> Result<(), ProtocolError> {
    reader.read_exact(bytes).map_err(|error| {
        if error.kind() == io::ErrorKind::UnexpectedEof {
            ProtocolError::Truncated
        } else {
            ProtocolError::Io
        }
    })
}
fn u64_at(bytes: &[u8]) -> Result<u64, ProtocolError> {
    Ok(u64::from_le_bytes(
        bytes.try_into().map_err(|_| ProtocolError::Framing)?,
    ))
}
pub(crate) fn validate(frame: &Frame) -> Result<(), ProtocolError> {
    frame.kind.check_length(frame.payload.len())?;
    if frame.kind == Kind::Failure && std::str::from_utf8(&frame.payload).is_err() {
        return Err(ProtocolError::Framing);
    }
    if frame.kind == Kind::Open {
        Cutoffs::new(u64_at(&frame.payload[..8])?, u64_at(&frame.payload[8..])?)?;
    }
    Ok(())
}

/// None is transport EOF at a frame boundary. It is never BodyEnd or physical ACK.
pub fn read_frame(reader: &mut impl Read) -> Result<Option<Frame>, ProtocolError> {
    let mut header = [0u8; HEADER_BYTES];
    loop {
        match reader.read(&mut header[..1]) {
            Ok(0) => return Ok(None),
            Ok(_) => break,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return Err(ProtocolError::Io),
        }
    }
    read_exact(reader, &mut header[1..])?;
    let (kind, length) = header_fields(&header)?;
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(length)
        .map_err(|_| ProtocolError::Limit)?;
    payload.resize(length, 0);
    read_exact(reader, &mut payload)?;
    let frame = Frame {
        binding: header[8..40]
            .try_into()
            .map_err(|_| ProtocolError::Framing)?,
        sequence: u64_at(&header[40..48])?,
        offset: u64_at(&header[48..56])?,
        kind,
        payload,
    };
    validate(&frame)?;
    Ok(Some(frame))
}

pub(crate) fn header_fields(header: &[u8]) -> Result<(Kind, usize), ProtocolError> {
    if header.len() != HEADER_BYTES {
        return Err(ProtocolError::Truncated);
    }
    if header[..4] != MAGIC
        || u16::from_le_bytes([header[4], header[5]]) != VERSION
        || header[7] != 0
    {
        return Err(ProtocolError::Framing);
    }
    let kind = Kind::from_byte(header[6])?;
    let length = u32::from_le_bytes(
        header[56..60]
            .try_into()
            .map_err(|_| ProtocolError::Framing)?,
    ) as usize;
    // Validate the declared kind/length before any payload allocation or body read.
    kind.check_length(length)?;
    Ok((kind, length))
}

pub fn decode_exact(bytes: &[u8]) -> Result<Frame, ProtocolError> {
    let mut reader = Cursor::new(bytes);
    let frame = read_frame(&mut reader)?.ok_or(ProtocolError::Truncated)?;
    if reader.position() != bytes.len() as u64 {
        return Err(ProtocolError::Framing);
    }
    Ok(frame)
}

pub fn encode(frame: &Frame) -> Result<Vec<u8>, ProtocolError> {
    validate(frame)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(HEADER_BYTES + frame.payload.len())
        .map_err(|_| ProtocolError::Limit)?;
    bytes.extend_from_slice(&MAGIC);
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.extend_from_slice(&[frame.kind as u8, 0]);
    bytes.extend_from_slice(&frame.binding);
    bytes.extend_from_slice(&frame.sequence.to_le_bytes());
    bytes.extend_from_slice(&frame.offset.to_le_bytes());
    bytes.extend_from_slice(&(frame.payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&frame.payload);
    Ok(bytes)
}
