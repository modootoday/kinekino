use super::*;
pub(super) fn encode(metadata: &HttpMetadata) -> Result<Vec<u8>, MetadataError> {
    let url = metadata.url.as_str().as_bytes();
    let content_type = metadata.content_type.as_deref().unwrap_or("").as_bytes();
    let mut output = Vec::new();
    output
        .try_reserve_exact(METADATA_HEADER_BYTES + url.len() + content_type.len())
        .map_err(|_| MetadataError::Limit)?;
    output.extend_from_slice(b"MHM1");
    output.extend_from_slice(&1u16.to_le_bytes());
    output.push(
        u8::from(metadata.length.is_some()) | (u8::from(metadata.content_type.is_some()) << 1),
    );
    output.push(0);
    output.extend_from_slice(&metadata.length.unwrap_or(0).to_le_bytes());
    output.extend_from_slice(&(url.len() as u16).to_le_bytes());
    output.extend_from_slice(&(content_type.len() as u16).to_le_bytes());
    output.extend_from_slice(url);
    output.extend_from_slice(content_type);
    Ok(output)
}
pub(super) fn decode_exact(bytes: &[u8]) -> Result<HttpMetadata, MetadataError> {
    if bytes.len() < METADATA_HEADER_BYTES {
        return Err(MetadataError::Framing);
    }
    if bytes.len() > MAX_HTTP_METADATA_BYTES {
        return Err(MetadataError::Limit);
    }
    if &bytes[..4] != b"MHM1" || bytes[7] != 0 || bytes[6] & !3 != 0 {
        return Err(MetadataError::Framing);
    }
    if u16::from_le_bytes([bytes[4], bytes[5]]) != 1 {
        return Err(MetadataError::Version);
    }
    let url_len = usize::from(u16::from_le_bytes([bytes[16], bytes[17]]));
    let type_len = usize::from(u16::from_le_bytes([bytes[18], bytes[19]]));
    if url_len == 0 || url_len > MAX_URL_BYTES || type_len > MAX_TYPE_BYTES {
        return Err(MetadataError::Limit);
    }
    if METADATA_HEADER_BYTES + url_len + type_len != bytes.len() {
        return Err(MetadataError::Framing);
    }
    if (bytes[6] & 2 != 0) != (type_len != 0) {
        return Err(MetadataError::Framing);
    }
    let length = u64::from_le_bytes(
        bytes[8..16]
            .try_into()
            .map_err(|_| MetadataError::Framing)?,
    );
    if bytes[6] & 1 == 0 && length != 0 {
        return Err(MetadataError::Framing);
    }
    let url_end = METADATA_HEADER_BYTES + url_len;
    let url = std::str::from_utf8(&bytes[METADATA_HEADER_BYTES..url_end])
        .map_err(|_| MetadataError::Url)?;
    let content_type = if type_len == 0 {
        None
    } else {
        Some(std::str::from_utf8(&bytes[url_end..]).map_err(|_| MetadataError::ContentType)?)
    };
    HttpMetadata::new(url, (bytes[6] & 1 != 0).then_some(length), content_type)
}
