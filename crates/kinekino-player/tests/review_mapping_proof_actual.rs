#![cfg(windows)]
#[path = "support/local_host_cli.rs"]
pub mod cli;
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "review_mapping_proof_actual/cli_sessions.rs"]
mod cli_sessions;
#[path = "sprite_capture_cli/custody.rs"]
mod custody;
#[path = "review_mapping_proof_actual/process_hosts.rs"]
mod process_hosts;
#[path = "review_mapping_proof_actual/sessions.rs"]
mod sessions;
use kinekino_player::{
    application::{
        api::{ErrorCode, PlayerSdk, Request},
        review_source::{
            ReviewCases, ReviewPackageMedia, api::ReviewSourceService, timeline_evidence,
        },
    },
    domain::{
        api_values::DecimalU64,
        review_source::{
            EvidenceReference, MediaReference, ReviewCase, UserRange,
            timeline_mapping::TimelineMappingRecord,
        },
    },
    infrastructure::{
        model_review::ReviewFiles,
        review_source::UserReviewFiles,
        timeline_evidence::{ReviewTimelineEvidence, TrustedTimelineAdmission},
    },
};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
fn write_new(path: &Path, value: &Value) {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    file.write_all(&serde_json::to_vec_pretty(value).unwrap())
        .unwrap();
    file.sync_all().unwrap();
}
#[test]
#[ignore = "central frozen source read-only actual sealed timeline proof/range admission; no decoder/model/GUI"]
fn actual_sealed_mapping_record_never_replaces_live_scoped_range_authority() {
    kinekino_platform::install();
    run(false, false, false);
}
#[test]
#[ignore = "central frozen source actual sealed timeline Verified SDK publication/reopen; no decoder/model/GUI"]
fn actual_sealed_mapping_verified_sdk_publication_reopens_without_evidence_adapter() {
    kinekino_platform::install();
    run(true, false, false);
}
#[test]
#[ignore = "central frozen source actual Verified association local session open/status/close; CPU runtime, no GUI/model/network"]
fn actual_verified_association_local_session_opens_replays_and_physically_closes() {
    kinekino_platform::install();
    run(true, true, false);
}
#[test]
#[ignore = "central frozen source actual Verified association normal CLI open/status/close; CPU runtime, no GUI/model/network"]
fn actual_verified_association_normal_cli_opens_replays_and_physically_closes() {
    kinekino_platform::install();
    run(true, true, true);
}
#[test]
#[ignore = "central frozen source actual SessionRef controls and observed-window pause through normal CLI; CPU runtime, no GUI/model/network"]
fn actual_verified_session_cli_controls_and_observed_window_pause() {
    kinekino_platform::install();
    run(true, true, true);
}
#[test]
#[ignore = "central frozen source actual review host shutdown and original physical owner completion; CPU runtime, no GUI/model/network"]
fn actual_verified_review_host_shutdown_waits_for_original_physical_owner() {
    kinekino_platform::install();
    run(true, true, false);
}
#[test]
#[ignore = "central frozen source separate host process EOF/restart and stale references; CPU runtime, no GUI/model/network"]
fn actual_review_host_process_eof_restart_rejects_stale_references() {
    kinekino_platform::install();
    run(true, false, false);
    process_hosts::verify();
}
#[test]
#[ignore = "central frozen source real broken stdout pipe and original media/transport cleanup; CPU runtime, no GUI/model/network"]
fn actual_review_host_broken_stdout_retains_first_error_and_physical_cleanup() {
    kinekino_platform::install();
    run(true, false, false);
    process_hosts::verify_broken_stdout();
}
fn trusted_admission(
    case: ReviewCase,
    media: MediaReference,
    evidence: EvidenceReference,
) -> TrustedTimelineAdmission {
    TrustedTimelineAdmission {
        case,
        media,
        evidence,
        audio_engine_sha256: "71414777922d05f2639a5cf4a0f9a405f169809cf459000a6f976c9add61f060"
            .into(),
        video_engine_sha256: "a6618e99bb58869ded3c6f37b53aa1a8d701c3591dbb7b5b317d47369c112be2"
            .into(),
        capture_engine_sha256: "09948d4cdd0650da6ff5a87577469f2a218dc2615ae379f8f734d24c49de0f73"
            .into(),
        capture_cli_sha256: "8d06f1919d05abedcdaa799c9a7b43fa42cf8f14b5c092fd6eb4d6d058cf99f9"
            .into(),
    }
}
fn run(store_verified: bool, run_sessions: bool, use_cli: bool) {
    let workspace = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let receipt_dir = PathBuf::from(
        std::env::var("KINEKINO_REVIEW_SOURCE_RECEIPT_DIR")
            .expect("explicit frozen input directory"),
    );
    assert!(receipt_dir.starts_with(workspace.join("runtime/verification")));
    let source_receipt = custody::Guard::acquire(&receipt_dir.join("SOURCE.md"));
    assert_eq!(
        source_receipt.sha256,
        std::env::var("KINEKINO_REVIEW_SOURCE_RECEIPT_SHA256")
            .expect("explicit input SHA")
            .to_ascii_lowercase()
    );
    let witness = custody::Guard::acquire(&std::env::current_exe().unwrap());
    let output = PathBuf::from(
        std::env::var("KINEKINO_REVIEW_OUTPUT_DIR").expect("explicit fresh private output"),
    );
    assert!(output.starts_with(workspace.join("runtime/verification")));
    fs::create_dir(&output).unwrap();
    let root = workspace.join("runtime/verification/sprite-timeline-actual-r1");
    let mut originals = Vec::new();
    for (file, sha) in [
        (
            "NOMINATION.json",
            "b1a47d17ad348d6ac7c4c5ec9eb246a54f561babc442cf4310274cdb349be201",
        ),
        (
            "SEALED.json",
            "4605c386980fda3b0af03b931683d82ead0b2554d53bdb9f4bbe15e3a4f3641f",
        ),
        (
            "ORIGINALS.json",
            "5f2428182d0f8f698f6ed1f6c3a3594a33925fac1b371024a7c8b442b72e527f",
        ),
        (
            "RETURN.json",
            "882868f1bd65be519679e2d5a1cde4baeafde714692db79c90ba6ca8ff1f5925",
        ),
    ] {
        let guard = custody::Guard::acquire(&root.join(file));
        assert_eq!(guard.sha256, sha);
        originals.push(guard);
    }
    let nomination: Value =
        serde_json::from_slice(&fs::read(root.join("NOMINATION.json")).unwrap()).unwrap();
    let case: ReviewCase = serde_json::from_value(nomination["case"].clone()).unwrap();
    let media: MediaReference = serde_json::from_value(nomination["media"].clone()).unwrap();
    let evidence: EvidenceReference =
        serde_json::from_value(nomination["evidence"].clone()).unwrap();
    write_new(
        &output.join("WITNESS.json"),
        &json!({"source":source_receipt.evidence(),"producer":witness.evidence(),"case":case,"media":media,"evidence":evidence,"requested_stored_verified_transition":store_verified,"requested_owned_local_runtime":run_sessions,"requested_normal_cli":use_cli,"requested_decode_execution":run_sessions,"new_decode_execution":false,"new_capture_execution":false,"new_model_execution":false}),
    );
    let witness_record = custody::Guard::acquire(&output.join("WITNESS.json"));
    let cases = ReviewCases::new(ReviewFiles::open(&root).unwrap());
    let medias = ReviewPackageMedia::new(ReviewFiles::open(&root).unwrap());
    let admission = trusted_admission(case.clone(), media.clone(), evidence.clone());
    let resolver =
        ReviewTimelineEvidence::new(ReviewFiles::open(&root).unwrap(), admission.clone()).unwrap();
    let retained =
        timeline_evidence::retain(&cases, &medias, &resolver, &case, &media, &evidence).unwrap();
    let record = retained.mapping_record().unwrap();
    let decoded: TimelineMappingRecord =
        serde_json::from_slice(&serde_json::to_vec(&record).unwrap()).unwrap();
    assert_eq!(decoded.source_time_base.numerator, "1");
    assert_eq!(decoded.source_time_base.denominator, "12288");
    assert_eq!(decoded.origin_pts, "0");
    assert_eq!(decoded.origin_time_base.denominator, "12288");
    let full = retained
        .retain_range(
            &decoded,
            &UserRange {
                start_us: DecimalU64(17_000_000),
                end_us: DecimalU64(25_500_000),
            },
        )
        .unwrap();
    assert_eq!(full.plan().first_frame_pts, "208896");
    assert_eq!(full.plan().end_frame_pts, "313344");
    let inner = retained
        .retain_range(
            &decoded,
            &UserRange {
                start_us: DecimalU64(18_000_000),
                end_us: DecimalU64(19_000_000),
            },
        )
        .unwrap();
    assert_eq!(inner.plan().first_frame_pts, "221184");
    assert_eq!(inner.plan().end_frame_pts, "233472");
    let outside = UserRange {
        start_us: DecimalU64(17_000_000),
        end_us: DecimalU64(25_500_001),
    };
    assert!(
        matches!(retained.retain_range(&decoded, &outside), Err(e) if e.code == ErrorCode::NeedsInput)
    );
    let mut forged = decoded.clone();
    forged.evidence.sha256 = "e".repeat(64);
    assert!(
        matches!(retained.retain_range(&forged, &inner.plan().range), Err(e) if e.code == ErrorCode::RevisionConflict)
    );
    full.check_current().unwrap();
    inner.check_current().unwrap();
    assert!(!retained.playback_ready());
    let full_plan = full.plan().clone();
    let inner_plan = inner.plan().clone();
    drop(full);
    drop(inner);
    drop(retained);
    let mut publication = Value::Null;
    let mut session_result = Value::Null;
    if store_verified {
        let user_root = output.join("user-review");
        fs::create_dir(&user_root).unwrap();
        let sdk = PlayerSdk::default().with_review_source(
            ReviewSourceService::new(
                cases,
                medias,
                UserReviewFiles::open(&user_root, true).unwrap(),
                true,
            )
            .with_timeline_evidence(resolver),
        );
        let payload = json!({"case":case,"association_id":"sealed-sprite","media":media,"note":"sealed mapping technical validation"});
        let mut create = Request::new("reviews.association.create");
        create.context.idempotency_key = Some("sealed-create".into());
        create.payload = payload.clone();
        let created = sdk.execute(create);
        write_new(
            &output.join("CREATE.json"),
            &serde_json::to_value(&created).unwrap(),
        );
        assert!(created.ok, "{:?}", created.error);
        let mut verify = Request::new("reviews.association.revise");
        verify.context.expected_revision = Some("1".into());
        verify.context.idempotency_key = Some("sealed-verify".into());
        verify.payload = payload;
        verify.payload["timeline_evidence"] = json!(evidence);
        write_new(
            &output.join("VERIFY-REQUEST.json"),
            &serde_json::to_value(&verify).unwrap(),
        );
        let result = sdk.execute(verify.clone());
        write_new(
            &output.join("VERIFY.json"),
            &serde_json::to_value(&result).unwrap(),
        );
        assert!(result.ok, "{:?}", result.error);
        publication = result.data.unwrap();
        assert_eq!(
            publication["receipt"]["body"]["association"]["timeline_status"]["verified"],
            json!(record)
        );
        assert_eq!(
            publication["receipt"]["body"]["association"]["revision"],
            "2"
        );
        assert_eq!(publication["playback_ready"], false);
        drop(sdk);
        let reopened = PlayerSdk::default().with_review_source(
            kinekino_player::infrastructure::review_source::open_api(&root, &user_root, true)
                .unwrap(),
        );
        let replay = reopened.execute(verify.clone());
        write_new(
            &output.join("REPLAY.json"),
            &serde_json::to_value(&replay).unwrap(),
        );
        assert!(replay.ok, "{:?}", replay.error);
        assert_eq!(replay.data.unwrap(), publication);
        let mut inspect = Request::new("reviews.association.inspect");
        inspect.payload = json!({"case":case,"association_id":"sealed-sprite"});
        let inspected = reopened.execute(inspect);
        write_new(
            &output.join("INSPECT.json"),
            &serde_json::to_value(&inspected).unwrap(),
        );
        assert!(inspected.ok);
        assert_eq!(
            inspected.data.unwrap()["stored"]["association"],
            publication["receipt"]["body"]["association"]
        );
        verify.context.expected_revision = Some("2".into());
        verify.context.idempotency_key = Some("new-proof-unconfigured".into());
        let unsupported = reopened.execute(verify);
        write_new(
            &output.join("UNCONFIGURED.json"),
            &serde_json::to_value(&unsupported).unwrap(),
        );
        assert_eq!(unsupported.error.unwrap().code, ErrorCode::Unsupported);
        if run_sessions {
            session_result =
                sessions::verify(workspace, &root, &user_root, admission, &output, use_cli);
        }
    }
    for guard in originals
        .iter()
        .chain([&source_receipt, &witness, &witness_record])
    {
        guard.assert_unchanged();
    }
    write_new(
        &output.join("RETURN.json"),
        &json!({"mapping_record":record,"full_range_plan":full_plan,"inner_range_plan":inner_plan,"outside_coverage_denied":true,"forged_record_denied":true,"originals_unchanged":true,"serialized_record_is_authority":false,"playback_ready":false,"stored_verified_transition":store_verified,"publication":publication,"session_result":session_result,"new_decode_execution":run_sessions,"new_capture_execution":false,"new_model_execution":false,"actual9_binding":false,"quality_approved":false,"source":source_receipt.evidence(),"producer":witness.evidence()}),
    );
    println!(
        "actual retained mapping/range evidence {}",
        output.display()
    );
}
