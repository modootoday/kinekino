//! Same freshly published authority through actual CLI storage and headless playback.
use kinekino_player::{
    application::{
        api::PlayerSdk,
        review_playback::{
            ReviewChildOwnership, ReviewRuntimeBackend, ReviewRuntimeProfile, sessions::Selection,
        },
        review_source::timeline_evidence::TimelineEvidenceResolver,
    },
    domain::{
        api_values::DecimalU64,
        review_source::{EvidenceReference, MediaReference, ReviewCase},
    },
    infrastructure::review_playback_sessions::LocalReviewRuntimeConfig,
};
use kinekino_process::infrastructure::family::JobBudget;
use serde_json::{Value, json};
use std::{fs, path::Path, sync::Arc, time::Duration};

pub(super) fn verify(
    review_root: &Path,
    evidence: Arc<dyn TimelineEvidenceResolver>,
    case: &ReviewCase,
    media: &MediaReference,
    reference: &EvidenceReference,
    output: &Path,
) -> Value {
    let output = output.join("live-sdk-cli");
    fs::create_dir(&output).unwrap();
    let user_root = output.join("user");
    let runtime_root = output.join("runtime");
    fs::create_dir(&user_root).unwrap();
    fs::create_dir(&runtime_root).unwrap();
    assert!(fs::read_dir(&user_root).unwrap().next().is_none());
    let tool = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .join("runtime/toolchains/mpv-dde0671072-review-r1/mpv.exe");
    let guard = super::custody::Guard::acquire(&tool);
    assert_eq!(
        guard.sha256,
        "f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790"
    );
    let (mut mask, mut system) = (0usize, 0usize);
    unsafe {
        windows::Win32::System::Threading::GetProcessAffinityMask(
            windows::Win32::System::Threading::GetCurrentProcess(),
            &mut mask,
            &mut system,
        )
        .unwrap();
    }
    let budget = JobBudget {
        max_committed_bytes: 1024 * 1024 * 1024,
        affinity_mask: mask.isolate_lowest_one(),
    };
    let config = LocalReviewRuntimeConfig {
        review_root: review_root.into(),
        user_root,
        runtime_root,
        executable: tool,
        budget,
        profile: ReviewRuntimeProfile {
            backend: ReviewRuntimeBackend::OwnedLocalMpv,
            engine_sha256: guard.sha256.clone(),
            max_source_bytes: 128 * 1024 * 1024,
            max_job_committed_bytes: budget.max_committed_bytes as u64,
            cpu_threads: 1,
            startup_timeout_ms: 10000,
            child_ownership: ReviewChildOwnership::OwnedJobUntilPhysicalExit,
        },
        total: Duration::from_secs(35),
        cleanup: Duration::from_secs(2),
        raw_output_bytes: 1024 * 1024,
    };
    super::write_new(
        &output.join("WITNESS.json"),
        &json!({
            "case":case,"media":media,"evidence":reference,"tool":guard.evidence(),
            "fresh_user_root":true,"headless":true,"quality_approved":false,
        }),
    );
    let assembly = kinekino_player::assembly::review_playback::assemble_sdk_with_evidence(
        PlayerSdk::default(),
        config,
        evidence,
        true,
        4,
    )
    .unwrap();
    let host = kinekino_player::assembly::review_playback::start_transport(
        assembly.client,
        assembly.sessions.clone(),
        kinekino_player::infrastructure::local_host::TransportOptions::default(),
    )
    .unwrap();
    let result = super::cli_sessions::verify(
        host,
        assembly.sessions,
        Selection {
            case: case.clone(),
            association_id: "cli-configured".into(),
            association_revision: DecimalU64(2),
        },
        &output,
    );
    guard.assert_unchanged();
    super::write_new(&output.join("RETURN.json"), &result);
    result
}
