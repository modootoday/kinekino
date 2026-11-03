//! Resume key (#9): stable, local-file-only identity for a playback source.
//! Remote URLs never get a key: they may carry signed tokens and change per request.
use std::fmt;
use std::path::{Component, Path, PathBuf};

const FILE_URL_PREFIX: &str = "file://";

/// `"file:" + normalized absolute path` for local sources; never built for remote URLs.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ResumeKey(String);

impl ResumeKey {
    /// Absolute path or local `file://` URL -> key; relative, empty, remote-host `file://`
    /// or any other scheme -> `None`. Reads the file system (canonicalize) to resolve symlinks.
    pub fn for_source(source: &str) -> Option<ResumeKey> {
        let path = local_path(source)?;
        let normalized = std::fs::canonicalize(&path).unwrap_or_else(|_| lexical_normalize(&path));
        // Non-UTF-8 paths get no key rather than a lossy one that could collide.
        let text = normalized.to_str()?;
        Some(ResumeKey(format!("file:{text}")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ResumeKey {
    /// Never prints the path: keys name private media.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ResumeKey(<redacted>)")
    }
}

fn local_path(source: &str) -> Option<PathBuf> {
    let is_file_url = source
        .get(..FILE_URL_PREFIX.len())
        .is_some_and(|p| p.eq_ignore_ascii_case(FILE_URL_PREFIX));
    let path = if is_file_url {
        let url = url::Url::parse(source).ok()?;
        // A host other than localhost names a network share, which is not a local file.
        // Checked here because `to_file_path` turns such URLs into UNC paths on Windows.
        if url
            .host_str()
            .is_some_and(|h| !h.eq_ignore_ascii_case("localhost"))
        {
            return None;
        }
        url.to_file_path().ok()?
    } else {
        PathBuf::from(source)
    };
    path.is_absolute().then_some(path)
}

/// Resolves `.` and `..` without touching the file system; `..` never climbs above the root.
fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(out.components().next_back(), Some(Component::Normal(_))) {
                    out.pop();
                }
            }
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::MAIN_SEPARATOR as SEP;

    // Symlink resolution is OS-specific; its test lives in the unix adapter crate.

    fn key(s: &str) -> Option<String> {
        ResumeKey::for_source(s).map(|k| k.as_str().to_owned())
    }

    /// OS-independent absolute base that never exists, so keys stay lexical.
    fn missing_base(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("kk-reskey-{}-{tag}-missing", std::process::id()))
    }

    fn expected(path: &Path) -> String {
        format!("file:{}", path.to_str().unwrap())
    }

    fn file_url(path: &Path) -> String {
        url::Url::from_file_path(path).unwrap().to_string()
    }

    #[test]
    fn plain_paths_are_normalized_lexically_when_missing() {
        let base = missing_base("lex");
        let b = base.to_str().unwrap();
        let dotted = format!("{b}{SEP}a{SEP}.{SEP}b{SEP}..{SEP}c.mkv");
        assert_eq!(key(&dotted), Some(expected(&base.join("a").join("c.mkv"))));
        let doubled = format!("{b}{SEP}{SEP}m{SEP}{SEP}x.mkv");
        assert_eq!(key(&doubled), Some(expected(&base.join("m").join("x.mkv"))));
    }

    #[test]
    fn file_url_is_percent_decoded() {
        let path = missing_base("pct").join("a b.mkv");
        let url = file_url(&path);
        assert!(url.contains("a%20b.mkv"), "{url}");
        assert_eq!(key(&url), Some(expected(&path)));
    }

    #[test]
    fn file_url_and_plain_path_share_a_key() {
        let base = missing_base("same");
        let url = format!("{}/x/../y.mkv", file_url(&base));
        let a = key(&url);
        assert!(a.is_some());
        assert_eq!(a, key(base.join("y.mkv").to_str().unwrap()));
    }

    #[test]
    fn remote_relative_and_empty_sources_have_no_key() {
        for s in [
            "",
            "relative/x.mkv",
            "./x.mkv",
            "x.mkv",
            "http://example.invalid/a.mkv",
            "https://example.invalid/a.mkv",
            "rtsp://example.invalid/stream",
            "data:video/mp4;base64,AAAA",
        ] {
            assert_eq!(key(s), None, "{s:?}");
        }
    }

    #[test]
    fn file_url_with_remote_host_has_no_key() {
        for s in [
            "file://server/share/x.mkv",
            "file://example.invalid/m/x.mkv",
            "FILE://server/share/x.mkv",
        ] {
            assert_eq!(key(s), None, "{s:?}");
        }
    }

    #[test]
    fn file_url_with_localhost_host_is_local() {
        let path = missing_base("localhost").join("x.mkv");
        let url = file_url(&path).replacen("file://", "file://localhost", 1);
        assert_eq!(key(&url), Some(expected(&path)), "{url}");
    }

    #[test]
    fn debug_does_not_reveal_path() {
        let src = missing_base("dbg").join("secret").join("movie.mkv");
        let k =
            ResumeKey::for_source(src.to_str().unwrap()).expect("absolute path must yield a key");
        let dbg = format!("{k:?}");
        assert!(!dbg.contains("secret"), "{dbg}");
        assert!(!dbg.contains("movie"), "{dbg}");
    }
}
