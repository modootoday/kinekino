//! OS-random identifiers.
use std::io::{self, Read};

/// 16 bytes from the kernel CSPRNG (`/dev/urandom`, present on Linux and macOS).
pub fn fresh_guid() -> io::Result<[u8; 16]> {
    let mut bytes = [0; 16];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    #[test]
    fn guids_are_fresh() {
        assert_ne!(super::fresh_guid().unwrap(), super::fresh_guid().unwrap());
    }
}
