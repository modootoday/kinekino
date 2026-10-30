use super::*;
use crate::{
    application::review_playback::{
        ReviewChildOwnership, ReviewRuntimeBackend, ReviewRuntimeProfile,
    },
    domain::review_source::{EvidenceReference, MediaReference, ReviewCase},
    infrastructure::timeline_evidence::TrustedTimelineAdmission,
};
use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
struct Tree(PathBuf);
impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn configuration() -> (Tree, LocalReviewSessionConfig) {
    let root = std::env::temp_dir().join(format!(
        "review-sdk-layout-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    for name in ["review", "user", "runtime"] {
        fs::create_dir(root.join(name)).unwrap();
    }
    let configuration = LocalReviewSessionConfig {
        review_root: root.join("review"),
        user_root: root.join("user"),
        runtime_root: root.join("runtime"),
        executable: root.join("unreachable.exe"),
        admission: TrustedTimelineAdmission {
            case: ReviewCase {
                package_sha256: "a".repeat(64),
                item_id: "item".into(),
                source_request_sha256: "b".repeat(64),
            },
            media: MediaReference {
                asset_id: "media".into(),
                sha256: "c".repeat(64),
            },
            evidence: EvidenceReference {
                asset_id: "proof".into(),
                sha256: "d".repeat(64),
            },
            audio_engine_sha256: "e".repeat(64),
            video_engine_sha256: "f".repeat(64),
            capture_engine_sha256: "a".repeat(64),
            capture_cli_sha256: "b".repeat(64),
        },
        profile: ReviewRuntimeProfile {
            backend: ReviewRuntimeBackend::OwnedLocalMpv,
            engine_sha256: "a".repeat(64),
            max_source_bytes: 1024,
            max_job_committed_bytes: 1024,
            cpu_threads: 1,
            startup_timeout_ms: 1000,
            child_ownership: ReviewChildOwnership::OwnedJobUntilPhysicalExit,
        },
        budget: kinekino_process::infrastructure::family::JobBudget {
            max_committed_bytes: 1024,
            affinity_mask: 1,
        },
        total: Duration::from_secs(3),
        cleanup: Duration::from_secs(1),
        raw_output_bytes: 1024,
    };
    (Tree(root), configuration)
}
#[test]
fn overlapping_runtime_is_rejected_before_writable_store_initialization() {
    let (_tree, mut config) = configuration();
    config.runtime_root = config.user_root.clone();
    let cases = config.user_root.join("cases");
    let error = match assemble_sdk(PlayerSdk::default(), config, true, 4) {
        Ok(_) => panic!("overlapping roots admitted"),
        Err(error) => error,
    };
    assert_eq!(
        error.code,
        crate::application::api::ErrorCode::InvalidArgument
    );
    assert!(!cases.exists());
}
#[test]
fn readonly_empty_store_never_creates_user_cases() {
    let (_tree, config) = configuration();
    let cases = config.user_root.join("cases");
    assert!(assemble_sdk(PlayerSdk::default(), config, false, 4).is_err());
    assert!(!cases.exists());
}
