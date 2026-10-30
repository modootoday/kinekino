//! Enabled synthetic scope only. No registered window, Ready implementation or child process.
use super::super::*;

pub(super) fn engine() -> Arc<NativeEpisodeEngine> {
    use sha2::{Digest, Sha256};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "failed-parent-cleanup-{}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    eprintln!("failed_parent_cleanup_fixture={}", root.display());
    let executable = root.join("mpv.exe");
    let bytes = b"inert failed original engine cleanup fixture; never executed";
    std::fs::write(&executable, bytes).unwrap();
    let trusted = crate::infrastructure::mpv::TrustedMpvExecutable::open(
        &executable,
        &format!("{:x}", Sha256::digest(bytes)),
    )
    .unwrap();
    Arc::new(
        NativeEpisodeEngine::new_with_scope(
            trusted,
            Some(1),
            Arc::new(UnusedArtifacts),
            crate::infrastructure::mpv::guarded_command::HostScope::testing(),
        )
        .unwrap(),
    )
}
