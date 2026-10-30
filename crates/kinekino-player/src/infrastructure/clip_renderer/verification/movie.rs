use crate::application::api::{ApiError, ErrorCode};
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::NeedsInput,
        "Finalized ordinary MP4 movie header is invalid",
    )
}
type Atom<'a> = (&'a [u8], &'a [u8]);
fn atoms(bytes: &[u8]) -> Result<Vec<Atom<'_>>, ApiError> {
    let mut found = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let header = bytes.get(offset..offset + 8).ok_or_else(invalid)?;
        let short = u32::from_be_bytes(header[..4].try_into().map_err(|_| invalid())?);
        let (size, head) = if short == 1 {
            let size = u64::from_be_bytes(
                bytes
                    .get(offset + 8..offset + 16)
                    .ok_or_else(invalid)?
                    .try_into()
                    .map_err(|_| invalid())?,
            );
            (usize::try_from(size).map_err(|_| invalid())?, 16)
        } else if short == 0 {
            (bytes.len() - offset, 8)
        } else {
            (short as usize, 8)
        };
        if size < head || found.len() >= 4096 {
            return Err(invalid());
        }
        let end = offset
            .checked_add(size)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(invalid)?;
        found.push((&header[4..8], &bytes[offset + head..end]));
        offset = end;
    }
    Ok(found)
}
/// Parse original bytes instead of treating a reported duration as a finalized movie proof.
pub(super) fn header(bytes: &[u8]) -> Result<(u32, u64), ApiError> {
    let root = atoms(bytes)?;
    if root.first().is_none_or(|(kind, _)| *kind != b"ftyp")
        || root.iter().any(|(kind, _)| *kind == b"moof")
    {
        return Err(invalid());
    }
    let movies: Vec<_> = root.iter().filter(|(kind, _)| *kind == b"moov").collect();
    if movies.len() != 1 {
        return Err(invalid());
    }
    let children = atoms(movies[0].1)?;
    let headers: Vec<_> = children
        .iter()
        .filter(|(kind, _)| *kind == b"mvhd")
        .collect();
    if headers.len() != 1 {
        return Err(invalid());
    }
    let data = headers[0].1;
    let (scale_at, duration_at, size) = match data.first() {
        Some(0) => (12, 16, 4),
        Some(1) => (20, 24, 8),
        _ => return Err(invalid()),
    };
    let scale = u32::from_be_bytes(
        data.get(scale_at..scale_at + 4)
            .ok_or_else(invalid)?
            .try_into()
            .map_err(|_| invalid())?,
    );
    let duration = if size == 4 {
        u64::from(u32::from_be_bytes(
            data.get(duration_at..duration_at + 4)
                .ok_or_else(invalid)?
                .try_into()
                .map_err(|_| invalid())?,
        ))
    } else {
        u64::from_be_bytes(
            data.get(duration_at..duration_at + 8)
                .ok_or_else(invalid)?
                .try_into()
                .map_err(|_| invalid())?,
        )
    };
    if scale == 0 || duration == 0 {
        return Err(invalid());
    }
    Ok((scale, duration))
}
