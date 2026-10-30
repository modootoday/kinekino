use super::{PlaybackWorkspaceConfiguration, PlaybackWorkspaceHost};
use crate::application::api::ErrorCode;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture {
    root: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "player-workspace-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("mpv.exe"), b"Synthetic trusted identity").unwrap();
        Self { root }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn guarded_workspace_rejects_any_parent_before_sha_files_or_provider_work() {
    let fixture = Fixture::new();
    let absent = fixture.root.join("uncreated-workspace");
    let executable = fixture.root.join("mpv.exe");
    for parent in [Some(0), Some(1)] {
        let error =
            PlaybackWorkspaceHost::start_trusted_standalone(PlaybackWorkspaceConfiguration {
                job_root: &absent,
                catalog_root: &absent,
                profile_root: &absent,
                mpv_executable: &executable,
                mpv_sha256: &"0".repeat(64),
                parent,
            })
            .err()
            .expect("Any parent must fail before digest or provider access");
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        assert!(error.message.contains("cannot have a native parent"));
        assert!(!absent.exists());
    }
}

#[test]
fn guarded_workspace_verifies_sha_before_opening_catalog_or_provider() {
    let fixture = Fixture::new();
    let absent = fixture.root.join("uncreated-workspace");
    let executable = fixture.root.join("mpv.exe");
    let error = PlaybackWorkspaceHost::start_trusted_standalone(PlaybackWorkspaceConfiguration {
        job_root: &absent,
        catalog_root: &absent,
        profile_root: &absent,
        mpv_executable: &executable,
        mpv_sha256: &"0".repeat(64),
        parent: None,
    })
    .err()
    .expect("Mismatched hash must precede missing catalog or provider root");
    assert_eq!(error.code, ErrorCode::SourceExpired);
    assert!(!absent.exists());
}
