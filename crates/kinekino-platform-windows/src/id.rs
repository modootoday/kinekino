//! OS-random identifiers.
use std::io;

/// `CoCreateGuid` bytes in display order (`data1` BE, `data2` BE, `data3` BE, `data4`).
pub fn fresh_guid() -> io::Result<[u8; 16]> {
    // SAFETY: CoCreateGuid has no preconditions and returns the GUID by value.
    let guid = unsafe { windows::Win32::System::Com::CoCreateGuid() }.map_err(io::Error::other)?;
    let mut bytes = [0; 16];
    bytes[..4].copy_from_slice(&guid.data1.to_be_bytes());
    bytes[4..6].copy_from_slice(&guid.data2.to_be_bytes());
    bytes[6..8].copy_from_slice(&guid.data3.to_be_bytes());
    bytes[8..].copy_from_slice(&guid.data4);
    Ok(bytes)
}
