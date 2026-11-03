//! Resume keys follow symlinks to their target; symlink creation is unix-specific,
//! so this check lives in the unix adapter rather than in kinekino-core.
use kinekino_core::playback::ResumeKey;

fn key(s: &str) -> Option<String> {
    ResumeKey::for_source(s).map(|k| k.as_str().to_owned())
}

#[test]
fn symlink_resolves_to_target_key() {
    let d = std::env::temp_dir().join(format!("kk-reskey-{}-link", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    let target = d.join("real.mkv");
    std::fs::write(&target, b"x").unwrap();
    let link = d.join("link.mkv");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let canon = std::fs::canonicalize(&target).unwrap();
    let expected = format!("file:{}", canon.display());
    let via_link = key(link.to_str().unwrap());
    let via_target = key(target.to_str().unwrap());
    let _ = std::fs::remove_dir_all(&d);
    assert_eq!(via_target.as_deref(), Some(expected.as_str()));
    assert_eq!(via_link, via_target);
}
