//! Lowercase hex encoding for digest bytes (e.g. SHA-256 output).

/// Encodes `bytes` as lowercase hex, two characters per byte, no separators.
pub fn hex_lower(bytes: impl AsRef<[u8]>) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let bytes = bytes.as_ref();
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(DIGITS[usize::from(b >> 4)] as char);
        out.push(DIGITS[usize::from(b & 0x0f)] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::hex_lower;

    #[test]
    fn encodes_empty_input() {
        assert_eq!(hex_lower([]), "");
    }

    #[test]
    fn encodes_every_nibble_lowercase_with_leading_zeros() {
        assert_eq!(hex_lower([0x00, 0x0f, 0xa0, 0xff, 0x12]), "000fa0ff12");
    }

    #[test]
    fn matches_known_sha256_of_abc() {
        let digest: [u8; 32] = [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ];
        assert_eq!(
            hex_lower(digest),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
