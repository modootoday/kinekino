//! Lists the paths inside a release archive: `.zip` is read here, `.tar.gz` through `tar -tzf`.
use std::path::Path;
use std::process::Command;

pub fn list(path: &Path) -> Result<Vec<String>, String> {
    let name = path.to_string_lossy();
    if name.ends_with(".zip") {
        let bytes = std::fs::read(path).map_err(|e| format!("read {name}: {e}"))?;
        zip_names(&bytes)
    } else if name.ends_with(".tar.gz") {
        let out = Command::new("tar")
            .arg("-tzf")
            .arg(path)
            .output()
            .map_err(|e| format!("run tar: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "tar -tzf {name} failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::to_owned)
            .collect())
    } else {
        Err(format!("{name}: expected a .zip or .tar.gz archive"))
    }
}

fn u16_at(b: &[u8], i: usize) -> Option<usize> {
    Some(u16::from_le_bytes(b.get(i..i + 2)?.try_into().ok()?) as usize)
}

fn u32_at(b: &[u8], i: usize) -> Option<usize> {
    Some(u32::from_le_bytes(b.get(i..i + 4)?.try_into().ok()?) as usize)
}

/// Entry names from the zip central directory (no Zip64; release archives are small).
pub fn zip_names(b: &[u8]) -> Result<Vec<String>, String> {
    const EOCD: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
    const CENTRAL: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
    let bad = || "not a readable zip archive".to_owned();
    let end = b.windows(4).rposition(|w| w == EOCD).ok_or_else(bad)?;
    let count = u16_at(b, end + 10).ok_or_else(bad)?;
    let mut at = u32_at(b, end + 16).ok_or_else(bad)?;
    if count == 0xffff || at == 0xffff_ffff {
        return Err("Zip64 archives are not supported".into());
    }
    let mut names = Vec::with_capacity(count);
    for _ in 0..count {
        if b.get(at..at + 4) != Some(&CENTRAL[..]) {
            return Err(bad());
        }
        let name_len = u16_at(b, at + 28).ok_or_else(bad)?;
        let extra = u16_at(b, at + 30).ok_or_else(bad)?;
        let comment = u16_at(b, at + 32).ok_or_else(bad)?;
        let name = b.get(at + 46..at + 46 + name_len).ok_or_else(bad)?;
        names.push(String::from_utf8_lossy(name).replace('\\', "/"));
        at += 46 + name_len + extra + comment;
    }
    Ok(names)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// A zip holding only a central directory and end record, enough for listing.
    pub fn central_only_zip(names: &[&str]) -> Vec<u8> {
        let mut b = Vec::new();
        for n in names {
            b.extend_from_slice(&[0x50, 0x4b, 0x01, 0x02]);
            b.extend_from_slice(&[0; 24]);
            b.extend_from_slice(&(n.len() as u16).to_le_bytes());
            b.extend_from_slice(&[0; 16]);
            b.extend_from_slice(n.as_bytes());
        }
        let size = b.len() as u32;
        b.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0]);
        let count = (names.len() as u16).to_le_bytes();
        b.extend_from_slice(&count);
        b.extend_from_slice(&count);
        b.extend_from_slice(&size.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&[0, 0]);
        b
    }

    #[test]
    fn reads_zip_central_directory_names() {
        let z = central_only_zip(&["kinekino-x/", "kinekino-x/player.exe"]);
        assert_eq!(
            zip_names(&z).unwrap(),
            ["kinekino-x/", "kinekino-x/player.exe"]
        );
    }

    #[test]
    fn rejects_non_zip_and_truncated_bytes() {
        assert!(zip_names(b"hello").is_err());
        let z = central_only_zip(&["a"]);
        assert!(zip_names(&z[10..]).is_err());
    }
}
