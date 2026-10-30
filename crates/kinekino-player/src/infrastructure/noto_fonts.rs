//! Pinned Noto faces (SIL OFL-1.1) read from the verified download cache. No font bytes are
//! compiled into the binary: `kinekino fonts fetch` downloads Meari's pinned revision
//! (`meari::domain::design::fonts`) into `runtime/fonts/noto/<rev>/`, and every load here
//! re-verifies length and SHA-256.
use meari::domain::{
    build::ContentHash,
    design::fonts::{MANIFEST, Pin, REVISION},
};
use std::path::{Path, PathBuf};

/// Environment variable naming the cache root; shared with Meari's `FontCache`.
pub const CACHE_ENV: &str = "MEARI_FONT_CACHE";
/// Asset path prefix of a delivered face (`fonts/noto/<file>`), as in Meari's font catalog.
pub const SOURCE_PREFIX: &str = "fonts/noto/";

/// Product faces: Latin, Hangul (+KS X 1001 hanja) and Japanese, upright sans.
pub const LATIN: Pin = pin("Noto Sans");
pub const KOREAN: Pin = pin("Noto Sans KR");
pub const JAPANESE: Pin = pin("Noto Sans JP");
pub const PRODUCT_FACES: [Pin; 3] = [LATIN, KOREAN, JAPANESE];
/// CSS family stack of the product faces (no host face is named).
pub const FAMILY_STACK: &str = "\"Noto Sans\", \"Noto Sans KR\", \"Noto Sans JP\", sans-serif";

const fn same(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// Upright pin of `family` in Meari's manifest; a missing family fails the build.
pub const fn pin(family: &str) -> Pin {
    let mut i = 0;
    while i < MANIFEST.len() {
        let candidate = MANIFEST[i];
        if same(candidate.family, family) && same(candidate.style, "normal") {
            return candidate;
        }
        i += 1;
    }
    panic!("family absent from the pinned Noto manifest")
}

/// Short revision used in cache paths and provenance.
pub fn revision() -> &'static str {
    &REVISION[..12]
}

/// `$MEARI_FONT_CACHE`, else the nearest `runtime/fonts` above the current directory or the
/// executable, else `runtime/fonts` beside the executable (same rule as Meari's cache).
pub fn cache_root() -> PathBuf {
    if let Some(root) = std::env::var_os(CACHE_ENV).filter(|v| !v.is_empty()) {
        return PathBuf::from(root);
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf));
    let starts = [Some(cwd.clone()), exe.clone()];
    starts
        .iter()
        .flatten()
        .flat_map(|start| start.ancestors().map(|dir| dir.join("runtime/fonts")))
        .find(|dir| dir.is_dir())
        .unwrap_or_else(|| exe.unwrap_or(cwd).join("runtime/fonts"))
}

pub fn path(root: &Path, pin: &Pin) -> PathBuf {
    root.join("noto").join(revision()).join(pin.file)
}

pub fn notice_path(root: &Path, pin: &Pin) -> PathBuf {
    root.join("noto")
        .join(revision())
        .join(format!("{}-OFL.txt", pin.dir))
}

/// Asset path of `pin` in product manifests.
pub fn source_path(pin: &Pin) -> String {
    format!("{SOURCE_PREFIX}{}", pin.file)
}

/// Verified bytes of one cached face; never downloads.
pub fn load(root: &Path, pin: &Pin) -> Result<Vec<u8>, String> {
    read_verified(&path(root, pin), pin.length, pin.sha256)
}

/// Verified OFL notice of one cached face.
pub fn notice(root: &Path, pin: &Pin) -> Result<String, String> {
    let bytes = read_verified(&notice_path(root, pin), pin.ofl_length, pin.ofl_sha256)?;
    String::from_utf8(bytes).map_err(|e| format!("{} OFL notice: {e}", pin.family))
}

fn read_verified(path: &Path, length: u64, sha256: &str) -> Result<Vec<u8>, String> {
    let bytes = std::fs::read(path).map_err(|e| {
        format!(
            "{}: {e}; run `kinekino fonts fetch` (pinned Noto revision {})",
            path.display(),
            revision()
        )
    })?;
    if bytes.len() as u64 != length || ContentHash::of(&bytes).as_str() != sha256 {
        return Err(format!(
            "{}: length or sha256 differs from the pinned Noto revision {}",
            path.display(),
            revision()
        ));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_faces_are_upright_pins_of_distinct_files() {
        let files: std::collections::BTreeSet<_> = PRODUCT_FACES.iter().map(|p| p.file).collect();
        assert_eq!(files.len(), 3);
        assert!(PRODUCT_FACES.iter().all(|p| p.style == "normal"));
        assert_eq!(source_path(&KOREAN), "fonts/noto/NotoSansKR[wght].ttf");
    }

    #[test]
    fn a_corrupt_cache_file_is_rejected() {
        let root = std::env::temp_dir().join(format!("kk-noto-{}", std::process::id()));
        let file = path(&root, &LATIN);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, b"not a font").unwrap();
        assert!(load(&root, &LATIN).unwrap_err().contains("sha256"));
        assert!(
            load(&root, &KOREAN)
                .unwrap_err()
                .contains("kinekino fonts fetch")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
