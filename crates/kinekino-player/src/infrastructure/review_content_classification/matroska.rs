//! Bounded EBML header classification (RFC8794/RFC9559), never codec/decode proof.
use super::*;
pub struct MatroskaReviewClassifier;
fn rejected() -> ApiError {
    ApiError::new(
        ErrorCode::Unsupported,
        "Raw source does not match the supported Matroska EBML header contract",
    )
}
/// ID keeps its marker; size removes it. Unknown-size values are forbidden in headers.
fn vint(bytes: &[u8], at: &mut usize, size: bool, maximum: usize) -> Result<u64, ApiError> {
    let first = *bytes.get(*at).ok_or_else(rejected)?;
    let width = first.leading_zeros() as usize + 1;
    if width > maximum {
        return Err(rejected());
    }
    let end = at
        .checked_add(width)
        .filter(|end| *end <= bytes.len())
        .ok_or_else(rejected)?;
    let mut value = if size {
        u64::from(first) & (0xffu64 >> width)
    } else {
        u64::from(first)
    };
    for byte in &bytes[*at + 1..end] {
        value = (value << 8) | u64::from(*byte);
    }
    *at = end;
    if size && value == (1u64 << (7 * width)) - 1 {
        return Err(rejected());
    }
    Ok(value)
}
fn uint(bytes: &[u8]) -> Result<u64, ApiError> {
    if bytes.is_empty() || bytes.len() > 8 {
        return Err(rejected());
    }
    Ok(bytes
        .iter()
        .fold(0, |value, byte| (value << 8) | u64::from(*byte)))
}
impl ReviewContentClassifier for MatroskaReviewClassifier {
    fn classify(&self, prefix: &[u8]) -> Result<ReviewContentClass, ApiError> {
        let prefix = &prefix[..prefix.len().min(65536)];
        if !prefix.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]) {
            return Err(rejected());
        }
        let mut at = 4;
        let length = usize::try_from(vint(prefix, &mut at, true, 8)?).map_err(|_| rejected())?;
        let end = at
            .checked_add(length)
            .filter(|end| *end <= prefix.len())
            .ok_or_else(rejected)?;
        let header = &prefix[..end];
        let mut fields = std::collections::BTreeMap::new();
        let mut count = 0;
        while at < end {
            count += 1;
            if count > 32 {
                return Err(rejected());
            }
            let id = vint(header, &mut at, false, 4)?;
            // Header child size VINTs are limited to four octets by RFC8794 8.1.
            let size = usize::try_from(vint(header, &mut at, true, 4)?).map_err(|_| rejected())?;
            let next = at
                .checked_add(size)
                .filter(|next| *next <= end)
                .ok_or_else(rejected)?;
            let value = &header[at..next];
            at = next;
            match id {
                0xec => continue, // Finite Void padding; not an authority-bearing field.
                0xbf if size == 4 => continue, // CRC presence is not checksum validation.
                0x4286 | 0x42f7 | 0x42f2 | 0x42f3 | 0x4282 | 0x4287 | 0x4285 => {}
                _ => return Err(rejected()),
            }
            if fields.insert(id, value).is_some() {
                return Err(rejected());
            }
        }
        let number = |id, default| fields.get(&id).map_or(Ok(default), |bytes| uint(bytes));
        let version = number(0x4286, 1)?;
        let read_version = number(0x42f7, 1)?;
        let document_version = number(0x4287, 1)?;
        let document_read_version = number(0x4285, 1)?;
        if version < read_version
            || read_version != 1
            || version != 1
            || number(0x42f2, 4)? != 4
            || !(1..=8).contains(&number(0x42f3, 8)?)
            || fields.get(&0x4282).copied() != Some(b"matroska".as_slice())
            || document_version < document_read_version
            || !(1..=4).contains(&document_read_version)
        {
            return Err(rejected());
        }
        Ok(ReviewContentClass::MatroskaHeader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn header(payload: &[u8]) -> Vec<u8> {
        assert!(payload.len() < 127);
        [
            vec![0x1a, 0x45, 0xdf, 0xa3, 0x80 | payload.len() as u8],
            payload.to_vec(),
        ]
        .concat()
    }
    const DOC: &[u8] = b"\x42\x82\x88matroska";
    #[test]
    fn bounded_matroska_header_and_legacy_mp4_are_independent_classes() {
        assert_eq!(
            MatroskaReviewClassifier.classify(&header(DOC)).unwrap(),
            ReviewContentClass::MatroskaHeader
        );
        let mut payload = DOC.to_vec();
        payload
            .extend_from_slice(b"\x42\xf2\x81\x04\x42\xf3\x81\x08\x42\x87\x81\x04\x42\x85\x81\x02");
        assert!(MatroskaReviewClassifier.classify(&header(&payload)).is_ok());
        assert!(IsoBmffReviewClassifier.classify(&header(DOC)).is_err());
    }
    #[test]
    fn malformed_versions_duplicates_doctypes_sizes_and_truncations_fail_closed() {
        let valid = header(DOC);
        for end in 0..valid.len() {
            assert!(MatroskaReviewClassifier.classify(&valid[..end]).is_err());
        }
        for extra in [
            b"\x42\xf2\x81\x05".as_slice(),
            b"\x42\xf3\x81\x09",
            b"\x42\xf7\x81\x02",
            b"\x42\x85\x81\x05",
            b"\x42\x87\x81\x00",
            DOC,
            b"\x42\xf2\xff",
            b"\x42\xf2\x80",
            b"\x42\xf2\x00",
        ] {
            assert!(
                MatroskaReviewClassifier
                    .classify(&header(&[DOC, extra].concat()))
                    .is_err()
            );
        }
        assert!(
            MatroskaReviewClassifier
                .classify(&header(b"\x42\x82\x84webm"))
                .is_err()
        );
        assert!(
            MatroskaReviewClassifier
                .classify(b"\x1a\x45\xdf\xa3\xff")
                .is_err()
        );
        assert!(
            MatroskaReviewClassifier
                .classify(&header(&[DOC, &b"\xec\x80".repeat(32)].concat()))
                .is_err()
        );
    }
}
