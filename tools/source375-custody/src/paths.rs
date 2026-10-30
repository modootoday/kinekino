use std::path::{Component, Path, PathBuf};
fn safe_relative(raw: &str) -> Result<PathBuf, String> {
    let normalized = raw.replace('\\', "/");
    if normalized.is_empty() || normalized.contains([':', '\t', '\n', '\r']) {
        return Err("invalid relative path".into());
    }
    let path = PathBuf::from(normalized);
    if path
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("relative path escapes or aliases".into());
    }
    Ok(path)
}
pub fn relative(raw: &str) -> Result<PathBuf, String> {
    let path = safe_relative(raw)?;
    if !matches!(path.components().next(),Some(Component::Normal(p)) if ["src","tests","packages","assets","Cargo.toml","Cargo.lock"].iter().any(|allowed| p == std::ffi::OsStr::new(allowed)))
    {
        return Err("unapproved source responsibility".into());
    }
    Ok(path)
}
pub fn source(root: &Path, raw: &str, relative: &Path) -> Result<PathBuf, String> {
    let normalized = raw.replace('\\', "/");
    if let Some(tail) = normalized.strip_prefix("D:/workspace/@myso-kr/com.kinekino.player/") {
        return Ok(root.join(safe_relative(tail)?));
    }
    if is_user_build_cache(&normalized)
        && relative.starts_with("vendor/meari/crates/meari/assets/fonts/pretendard")
    {
        return Ok(root.join(relative));
    }
    Err(format!("unmapped historical source: {raw}"))
}
/// `C:/Users/<any user>/AppData/Local/CodexBuildCache/...`, without naming an account.
fn is_user_build_cache(normalized: &str) -> bool {
    normalized
        .strip_prefix("C:/Users/")
        .and_then(|rest| rest.split_once('/'))
        .is_some_and(|(user, tail)| {
            !user.is_empty() && tail.starts_with("AppData/Local/CodexBuildCache/")
        })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boundaries_and_slashes() {
        assert_eq!(
            relative("src\\lib.rs").unwrap(),
            PathBuf::from("src/lib.rs")
        );
        for path in ["../src/lib.rs", "C:/src/lib.rs", "runtime/x", "/src/lib.rs"] {
            assert!(relative(path).is_err());
        }
        assert!(
            source(
                Path::new("C:/root"),
                "D:/other/src/lib.rs",
                Path::new("src/lib.rs")
            )
            .is_err()
        );
        assert!(
            source(
                Path::new("C:/root"),
                "C:/Users/someone/AppData/Local/CodexBuildCache/x",
                Path::new("src/lib.rs")
            )
            .is_err()
        );
    }
    #[test]
    fn any_user_build_cache_maps_only_pinned_fonts() {
        let root = Path::new("C:/root");
        let font = Path::new("vendor/meari/crates/meari/assets/fonts/pretendard/a.otf");
        let raw = "C:/Users/someone/AppData/Local/CodexBuildCache/f/a.otf";
        assert_eq!(source(root, raw, font).unwrap(), root.join(font));
        assert!(source(root, "C:/Users//AppData/Local/CodexBuildCache/a", font).is_err());
        assert!(source(root, "C:/Users/someone/Desktop/a.otf", font).is_err());
    }
    #[test]
    fn pinned_private_payload_is_not_a_public_destination() {
        let root = Path::new("C:/root");
        assert_eq!(
            source(
                root,
                "D:/workspace/@myso-kr/com.kinekino.player/runtime/development/pin/raw/src/lib.rs",
                Path::new("src/lib.rs")
            )
            .unwrap(),
            root.join("runtime/development/pin/raw/src/lib.rs")
        );
        assert!(
            source(
                root,
                "D:/workspace/@myso-kr/com.kinekino.player/../outside",
                Path::new("src/lib.rs")
            )
            .is_err()
        );
        assert!(relative("runtime/development/pin/raw/src/lib.rs").is_err());
    }
}
