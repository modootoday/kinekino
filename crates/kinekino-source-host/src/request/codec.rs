use super::*;

pub(super) fn encode(request: &HttpBootstrap) -> Result<Vec<u8>, BootstrapError> {
    let target = request.target.as_str().as_bytes();
    let referer = request
        .referer
        .as_ref()
        .map_or(&[][..], |url| url.as_str().as_bytes());
    let size = lengths(target.len(), referer.len())?;
    let mut out = Vec::new();
    out.try_reserve_exact(size)
        .map_err(|_| BootstrapError::Limit)?;
    out.extend_from_slice(&REQUEST_MAGIC);
    out.extend_from_slice(&REQUEST_VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&request.binding);
    out.extend_from_slice(&request.maximum_bytes.to_le_bytes());
    out.extend_from_slice(&(target.len() as u16).to_le_bytes());
    out.extend_from_slice(&(referer.len() as u16).to_le_bytes());
    out.extend_from_slice(target);
    out.extend_from_slice(referer);
    Ok(out)
}
pub(super) fn declared_size(bytes: &[u8]) -> Result<usize, BootstrapError> {
    if bytes.len() < REQUEST_HEADER_BYTES {
        return Err(BootstrapError::Truncated);
    }
    if bytes[..4] != REQUEST_MAGIC || bytes[6..8] != [0, 0] {
        return Err(BootstrapError::Framing);
    }
    if u16::from_le_bytes([bytes[4], bytes[5]]) != REQUEST_VERSION {
        return Err(BootstrapError::Version);
    }
    let target_len = usize::from(u16::from_le_bytes([bytes[48], bytes[49]]));
    let referer_len = usize::from(u16::from_le_bytes([bytes[50], bytes[51]]));
    lengths(target_len, referer_len)
}
pub(super) fn decode_exact(bytes: &[u8]) -> Result<HttpBootstrap, BootstrapError> {
    if bytes.len() > MAX_REQUEST_BYTES {
        return Err(BootstrapError::Limit);
    }
    // Validate declared lengths before reading/allocating URL payloads.
    let expected = declared_size(bytes)?;
    if bytes.len() < expected {
        return Err(BootstrapError::Truncated);
    }
    if bytes.len() != expected {
        return Err(BootstrapError::Framing);
    }
    let target_end = REQUEST_HEADER_BYTES + usize::from(u16::from_le_bytes([bytes[48], bytes[49]]));
    let referer_len = usize::from(u16::from_le_bytes([bytes[50], bytes[51]]));
    let target = std::str::from_utf8(&bytes[REQUEST_HEADER_BYTES..target_end])
        .map_err(|_| BootstrapError::InvalidUrl)?;
    let referer = if referer_len == 0 {
        None
    } else {
        Some(std::str::from_utf8(&bytes[target_end..]).map_err(|_| BootstrapError::InvalidUrl)?)
    };
    HttpBootstrap::new(
        bytes[8..40]
            .try_into()
            .map_err(|_| BootstrapError::Framing)?,
        u64::from_le_bytes(
            bytes[40..48]
                .try_into()
                .map_err(|_| BootstrapError::Framing)?,
        ),
        target,
        referer,
    )
}
