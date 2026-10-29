//! r-src: tracked files and paths must not name a concrete media source site.
//!
//! The forbidden words are stored only as FNV-1a 64-bit digests (with their
//! lengths) so the rule itself does not reintroduce the words it forbids.
use std::{path::Path, process::Command};

/// (word length, FNV-1a 64 digest of the lowercase word).
const FORBIDDEN: &[(usize, u64)] = &[
    (7, 0xf010_5fe1_ab87_95e5),
    (8, 0x8e90_5d1a_8572_abd1),
    (6, 0x8e26_9ec9_c492_bed1),
];

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// True when any window of `text` (ASCII-lowercased) matches a forbidden digest.
pub fn contains_forbidden(text: &[u8]) -> bool {
    let lower: Vec<u8> = text.iter().map(u8::to_ascii_lowercase).collect();
    FORBIDDEN.iter().any(|(len, digest)| {
        lower.len() >= *len && lower.windows(*len).any(|w| fnv1a(w) == *digest)
    })
}

/// One `r-src <path>` violation per tracked file whose path or content names a source site.
pub fn check(root: &Path) -> Vec<String> {
    let Ok(out) = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z"])
        .output()
    else {
        return vec!["r-src git ls-files failed".to_owned()];
    };
    out.stdout
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .filter_map(|p| {
            let rel = String::from_utf8_lossy(p).into_owned();
            let hit = contains_forbidden(p)
                || std::fs::read(root.join(&rel)).is_ok_and(|body| contains_forbidden(&body));
            hit.then(|| format!("r-src {rel}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::contains_forbidden;

    #[test]
    fn neutral_text_passes() {
        assert!(!contains_forbidden(
            b"source.example.test media-cdn frame-host"
        ));
    }

    #[test]
    fn digest_matches_reconstructed_word() {
        // Build the word at runtime so the literal never appears in the tree.
        let word: String = ['a', 'n', 'i', 'w', 'e', 'e', 'k'].iter().collect();
        assert!(contains_forbidden(
            format!("https://{}.com/x", word.to_uppercase()).as_bytes()
        ));
    }
}
