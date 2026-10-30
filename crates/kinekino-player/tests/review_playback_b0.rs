#![cfg(windows)]
#[path = "sprite_capture_cli/custody.rs"]
mod custody;
use kinekino_player::{
    application::{
        api::ErrorCode,
        review_playback::{
            ReviewContentClass, UnavailableReviewRuntime, inspect_review_input,
            reserve_review_runtime,
        },
        review_source::ReviewCaseEvidence,
    },
    infrastructure::{
        model_review::ReviewFiles, review_content_classification::IsoBmffReviewClassifier,
    },
};
use serde_json::{Value, json};
use std::{fs, path::Path};

#[test]
#[ignore = "central actual Sprite source + existing pinned mpv + empty Job admission; no engine starts"]
fn actual_source_tool_and_job_are_retained_by_one_admitted_owner() {
    kinekino_platform::install();
    use kinekino_player::{
        application::review_playback::{
            ReviewChildOwnership, ReviewRuntimeBackend, ReviewRuntimeProfile,
        },
        infrastructure::{mpv::TrustedMpvExecutable, review_runtime::ReviewRuntimePool},
    };
    use kinekino_process::infrastructure::family::JobBudget;
    let workspace = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let root = workspace.join("runtime/verification/sprite-timeline-actual-r1");
    let nomination_guard = custody::Guard::acquire(&root.join("NOMINATION.json"));
    assert_eq!(
        nomination_guard.sha256,
        "b1a47d17ad348d6ac7c4c5ec9eb246a54f561babc442cf4310274cdb349be201"
    );
    let nomination: Value =
        serde_json::from_slice(&fs::read(root.join("NOMINATION.json")).unwrap()).unwrap();
    let case = serde_json::from_value(nomination["case"].clone()).unwrap();
    let media = serde_json::from_value(nomination["media"].clone()).unwrap();
    let reader = ReviewCaseEvidence::new(ReviewFiles::open(&root).unwrap());
    let files = ReviewFiles::open(&root).unwrap();
    let input = inspect_review_input(&reader, &case, &media, &IsoBmffReviewClassifier).unwrap();
    let source = files.bind_playback_media(input).unwrap();
    drop(files);
    drop(reader);
    let tool_path = workspace.join("runtime/toolchains/mpv-dde0671072-review-r1/mpv.exe");
    let tool_sha = "f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790";
    let tool = TrustedMpvExecutable::open(&tool_path, tool_sha).unwrap();
    let allocation = workspace.join("runtime/build-cache/tmp").join(format!(
        "review-admitted-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&allocation).unwrap();
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
        max_committed_bytes: 512 * 1024 * 1024,
        affinity_mask: mask.isolate_lowest_one(),
    };
    let profile = ReviewRuntimeProfile {
        backend: ReviewRuntimeBackend::OwnedLocalMpv,
        engine_sha256: tool_sha.into(),
        max_source_bytes: 128 * 1024 * 1024,
        max_job_committed_bytes: budget.max_committed_bytes as u64,
        cpu_threads: 1,
        startup_timeout_ms: 30000,
        child_ownership: ReviewChildOwnership::OwnedJobUntilPhysicalExit,
    };
    let pool = ReviewRuntimePool::open(&allocation, tool, profile, budget).unwrap();
    let admitted = source.admit(pool.reserve_owned().unwrap()).unwrap();
    assert!(matches!(pool.reserve_owned(), Err(e) if e.code == ErrorCode::RevisionConflict));
    drop(pool);
    admitted.check_current().unwrap();
    assert_eq!(admitted.identity().bytes, 102392846);
    for path in [
        tool_path.clone(),
        root.join("objects").join(&admitted.identity().media.sha256),
        allocation.join("review-runtime.lock"),
    ] {
        assert_eq!(
            fs::OpenOptions::new()
                .write(true)
                .open(path)
                .unwrap_err()
                .raw_os_error(),
            Some(32)
        );
    }
    drop(admitted);
    assert!(
        fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(allocation.join("review-runtime.lock"))
            .is_ok()
    );
    let tool_after = custody::Guard::acquire(&tool_path);
    assert_eq!(tool_after.sha256, tool_sha);
    nomination_guard.assert_unchanged();
    println!(
        "admission only; no media process/playback_ready/quality approval; preserved allocation {}",
        allocation.display()
    );
}

#[test]
#[ignore = "central read-only actual source custody handoff; no media process"]
fn actual_review_engine_object_remains_guarded_after_reader_drop() {
    kinekino_platform::install();
    let workspace = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let root = workspace.join("runtime/verification/sprite-timeline-actual-r1");
    let nomination_guard = custody::Guard::acquire(&root.join("NOMINATION.json"));
    assert_eq!(
        nomination_guard.sha256,
        "b1a47d17ad348d6ac7c4c5ec9eb246a54f561babc442cf4310274cdb349be201"
    );
    let nomination: Value =
        serde_json::from_slice(&fs::read(root.join("NOMINATION.json")).unwrap()).unwrap();
    let case = serde_json::from_value(nomination["case"].clone()).unwrap();
    let media = serde_json::from_value(nomination["media"].clone()).unwrap();
    let reader = ReviewCaseEvidence::new(ReviewFiles::open(&root).unwrap());
    let files = ReviewFiles::open(&root).unwrap();
    let wrong_root = workspace.join("runtime/build-cache/tmp").join(format!(
        "review-playback-wrong-root-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&wrong_root).unwrap();
    for directory in ["packages", "objects", "decisions"] {
        fs::create_dir(wrong_root.join(directory)).unwrap();
    }
    let wrong_input =
        inspect_review_input(&reader, &case, &media, &IsoBmffReviewClassifier).unwrap();
    fs::write(
        wrong_root
            .join("objects")
            .join(&wrong_input.identity().media.sha256),
        b"not the selected source",
    )
    .unwrap();
    let wrong_files = ReviewFiles::open(&wrong_root).unwrap();
    let rejected = wrong_files.bind_playback_media(wrong_input);
    assert!(matches!(rejected, Err(ref e) if e.code == ErrorCode::RevisionConflict));
    println!(
        "preserved wrong-root negative fixture {}",
        wrong_root.display()
    );
    let input = inspect_review_input(&reader, &case, &media, &IsoBmffReviewClassifier).unwrap();
    let bound = files.bind_playback_media(input).unwrap();
    drop(files);
    drop(reader);
    bound.check_current().unwrap();
    assert_eq!(bound.identity().bytes, 102392846);
    let source_path = root.join("objects").join(&bound.identity().media.sha256);
    // Open without truncate/write: even acquiring a writer must be denied.
    let writer = fs::OpenOptions::new().write(true).open(&source_path);
    assert_eq!(writer.unwrap_err().raw_os_error(), Some(32));
    bound.check_current().unwrap();
    nomination_guard.assert_unchanged();
}

#[test]
#[ignore = "explicit central allocation: read-only actual package source classification; no playback engine starts"]
fn actual_review_source_has_independent_raw_identity_and_runtime_remains_unadmitted() {
    kinekino_platform::install();
    let witness = custody::Guard::acquire(&std::env::current_exe().unwrap());
    let workspace = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let root = workspace.join("runtime/verification/sprite-timeline-actual-r1");
    let mut guards = Vec::new();
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
        guards.push(guard);
    }
    let nomination: Value =
        serde_json::from_slice(&fs::read(root.join("NOMINATION.json")).unwrap()).unwrap();
    let case = serde_json::from_value(nomination["case"].clone()).unwrap();
    let media = serde_json::from_value(nomination["media"].clone()).unwrap();
    let reader = ReviewCaseEvidence::new(ReviewFiles::open(&root).unwrap());
    let input = inspect_review_input(&reader, &case, &media, &IsoBmffReviewClassifier).unwrap();
    assert_eq!(
        input.identity().classification,
        ReviewContentClass::IsoBmffHeader
    );
    assert_eq!(input.identity().bytes, 102392846);
    assert_eq!(input.identity().declared_media_type, "video/mp4");
    assert_eq!(
        input.identity().media.sha256,
        "7cfc8b8c6eeb19685048a289aca208a01356f5082b470fb9f4d933fe4b2087ef"
    );
    input.check_current().unwrap();
    let identity = input.identity().clone();
    let result = reserve_review_runtime(input, &UnavailableReviewRuntime);
    assert!(matches!(result,Err(ref e) if e.code==ErrorCode::Unsupported));
    for guard in &guards {
        guard.assert_unchanged();
    }
    witness.assert_unchanged();
    let output = workspace.join("runtime/verification/review-playback-b0-datalab-r1");
    fs::create_dir(&output).unwrap();
    let receipt = json!({"source_case":identity.case,"media":identity.media,"full_source_bytes":identity.bytes,"declared_media_type":identity.declared_media_type,"raw_classification":"iso_bmff_header; not codec/decode proof","runtime_result":"unsupported; concrete local owned runtime not admitted","media_runtime_start_attempted":false,"playback_ready":false,"source_and_bundle_unchanged":true,"witness":witness.evidence(),"actual9_binding":false,"quality_approved":false});
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join("RETURN.json"))
        .unwrap();
    use std::io::Write;
    file.write_all(&serde_json::to_vec_pretty(&receipt).unwrap())
        .unwrap();
    file.sync_all().unwrap();
    witness.assert_unchanged();
    println!("review playback B0 evidence {}", output.display());
}
