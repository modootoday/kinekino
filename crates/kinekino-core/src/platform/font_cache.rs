//! Font cache location from injected lookups (never reads `std::env` or the disk here).
//! ADR 20261109101726 item 4: the program directory is never written, so the last fallback
//! is the per-user cache, not `runtime/fonts` beside the executable.
use std::path::{Path, PathBuf};

/// Environment variable naming the cache root; shared with Meari's `FontCache`.
pub const CACHE_ENV: &str = "MEARI_FONT_CACHE";

/// Where the process looks for the font cache.
#[derive(Debug, Clone, Copy)]
pub struct FontCacheLookup<'a> {
    pub cwd: &'a Path,
    /// Directory of the running executable, when known.
    pub exe_dir: Option<&'a Path>,
    /// `true` on macOS (per-user cache under `Library/Caches`).
    pub macos: bool,
}

/// `$MEARI_FONT_CACHE`, else the nearest existing `runtime/fonts` above the working
/// directory or the executable (development layout), else the per-user cache:
/// `XDG_CACHE_HOME/kinekino/fonts`, `LOCALAPPDATA/kinekino/fonts`,
/// `HOME/Library/Caches/kinekino/fonts` (macOS) or `HOME/.cache/kinekino/fonts`.
/// With none of those set, `runtime/fonts` under the working directory. Never a new
/// directory beside the executable. Empty values count as unset.
pub fn font_cache_root(
    get: impl Fn(&str) -> Option<String>,
    is_dir: impl Fn(&Path) -> bool,
    at: &FontCacheLookup<'_>,
) -> PathBuf {
    let var = |name: &str| get(name).filter(|v| !v.is_empty()).map(PathBuf::from);
    if let Some(root) = var(CACHE_ENV) {
        return root;
    }
    let found = [Some(at.cwd), at.exe_dir]
        .into_iter()
        .flatten()
        .flat_map(Path::ancestors)
        .map(|dir| dir.join("runtime").join("fonts"))
        .find(|dir| is_dir(dir));
    if let Some(root) = found {
        return root;
    }
    let home = var("HOME");
    let user_cache = var("XDG_CACHE_HOME")
        .or_else(|| var("LOCALAPPDATA"))
        .or_else(|| {
            home.map(|h| match at.macos {
                true => h.join("Library").join("Caches"),
                false => h.join(".cache"),
            })
        });
    match user_cache {
        Some(base) => base.join("kinekino").join("fonts"),
        None => at.cwd.join("runtime").join("fonts"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |k| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| (*v).to_owned())
        }
    }
    fn dirs<'a>(existing: &'a [&'a str]) -> impl Fn(&Path) -> bool + 'a {
        move |p| existing.iter().any(|e| Path::new(e) == p)
    }
    fn installed() -> FontCacheLookup<'static> {
        FontCacheLookup {
            cwd: Path::new("/home/u/videos"),
            exe_dir: Some(Path::new("/usr/lib/kinekino")),
            macos: false,
        }
    }

    #[test]
    fn explicit_override_wins() {
        let root = font_cache_root(
            lookup(&[(CACHE_ENV, "/o/fonts"), ("XDG_CACHE_HOME", "/x")]),
            dirs(&["/home/u/videos/runtime/fonts"]),
            &installed(),
        );
        assert_eq!(root, Path::new("/o/fonts"));
    }

    #[test]
    fn development_tree_above_cwd_comes_before_user_cache() {
        let at = FontCacheLookup {
            cwd: Path::new("/w/kinekino/crates/app"),
            ..installed()
        };
        let root = font_cache_root(
            lookup(&[("XDG_CACHE_HOME", "/x")]),
            dirs(&["/w/kinekino/runtime/fonts"]),
            &at,
        );
        assert_eq!(root, Path::new("/w/kinekino/runtime/fonts"));
    }

    #[test]
    fn existing_tree_above_the_executable_is_still_found() {
        let at = FontCacheLookup {
            exe_dir: Some(Path::new("/w/kinekino/target/debug")),
            ..installed()
        };
        let root = font_cache_root(
            lookup(&[("HOME", "/home/u")]),
            dirs(&["/w/kinekino/runtime/fonts"]),
            &at,
        );
        assert_eq!(root, Path::new("/w/kinekino/runtime/fonts"));
    }

    #[test]
    fn installed_binary_uses_the_per_user_cache_not_its_own_directory() {
        let root = font_cache_root(lookup(&[("HOME", "/home/u")]), dirs(&[]), &installed());
        assert_eq!(root, Path::new("/home/u/.cache/kinekino/fonts"));
        assert!(!root.starts_with("/usr/lib/kinekino"));
    }

    #[test]
    fn xdg_cache_home_beats_home() {
        let root = font_cache_root(
            lookup(&[("XDG_CACHE_HOME", "/x/cache"), ("HOME", "/home/u")]),
            dirs(&[]),
            &installed(),
        );
        assert_eq!(root, Path::new("/x/cache/kinekino/fonts"));
    }

    #[test]
    fn windows_uses_local_app_data() {
        let root = font_cache_root(
            lookup(&[("LOCALAPPDATA", "/l/app"), ("HOME", "/home/u")]),
            dirs(&[]),
            &installed(),
        );
        assert_eq!(root, Path::new("/l/app/kinekino/fonts"));
    }

    #[test]
    fn macos_uses_library_caches() {
        let at = FontCacheLookup {
            macos: true,
            ..installed()
        };
        let root = font_cache_root(lookup(&[("HOME", "/Users/u")]), dirs(&[]), &at);
        assert_eq!(root, Path::new("/Users/u/Library/Caches/kinekino/fonts"));
    }

    #[test]
    fn nothing_set_stays_under_cwd_never_beside_the_executable() {
        let root = font_cache_root(
            lookup(&[(CACHE_ENV, ""), ("HOME", "")]),
            dirs(&[]),
            &installed(),
        );
        assert_eq!(root, Path::new("/home/u/videos/runtime/fonts"));
    }
}
