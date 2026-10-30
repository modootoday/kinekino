#![cfg(windows)]
#[path = "support/local_host_cli.rs"]
pub mod cli;
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "review_mapping_proof_actual/cli_sessions.rs"]
mod cli_sessions;
#[path = "sprite_capture_cli/custody.rs"]
mod custody;
#[path = "timeline_video_production_actual/sdk.rs"]
mod sdk;
use kinekino_player::{
    application::{
        api::{ApiError, ErrorCode},
        model_review::ReviewStore,
        review_playback::inspect_review_input,
        review_source::{
            MutationControl, ReviewCaseEvidence, ReviewCases, ReviewPackageMedia,
            timeline_evidence::{self, TimelineEvidenceResolver},
        },
    },
    domain::{
        media_timeline::TimeBase,
        review_source::{MediaReference, ReviewCase},
    },
    infrastructure::{
        model_review::ReviewFiles,
        review_content_classification::IsoBmffReviewClassifier,
        timeline_evidence::production::{
            PcmClockRange, PinnedTimelineAudioDecoder, PinnedTimelineCaptureDecoder,
            PinnedTimelineProbe, TimelineBundleInputs, TimelineCapturePoint,
            TimelineProcessFailure, TimelineProductionConfig, TimelineProductionSession,
            VideoProbeWindow,
        },
        timeline_evidence::publication::{
            PublishedTimelineResolver, TimelinePackageDestination, publish,
        },
    },
};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
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
struct Continue;
struct CancelAfterObject(PathBuf);
impl MutationControl for CancelAfterObject {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if self.0.join("objects").is_dir()
            && fs::read_dir(self.0.join("objects"))
                .unwrap()
                .next()
                .is_some()
        {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "cancelled after first object creation, before package commit",
            ));
        }
        Ok(())
    }
}
impl MutationControl for Continue {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
struct CancelAfterCreate(PathBuf, &'static str);
impl MutationControl for CancelAfterCreate {
    fn checkpoint(&self) -> Result<(), ApiError> {
        for entry in fs::read_dir(&self.0).unwrap() {
            let path = entry.unwrap().path();
            if path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(self.1)
                && path.join("STARTED.json").is_file()
                && !path.join("RETURN.json").exists()
            {
                return Err(ApiError::new(
                    ErrorCode::Cancelled,
                    "actual producer cancel after original child creation",
                ));
            }
        }
        Ok(())
    }
}
fn observe_cancelled(cancelled: &mut TimelineProcessFailure<'_, '_>) -> Value {
    let pending = cancelled
        .pending
        .as_mut()
        .expect("original post-create owner retained");
    let cutoff = Instant::now() + Duration::from_secs(5);
    while !pending.poll_cleanup() {
        assert!(
            Instant::now() < cutoff,
            "original probe cleanup not complete"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    let receipt = pending.receipt().unwrap();
    assert!(receipt.child_exit.is_some());
    assert_eq!(
        receipt.family,
        kinekino_process::domain::observation::FamilyObservation::Empty
    );
    assert!(!pending.output_root().join("RETURN.json").exists());
    json!({"error":cancelled.error,"output_root":pending.output_root(),"pid":receipt.identity.pid,"creation_filetime":receipt.identity.creation_filetime,"child_exit":receipt.child_exit,"family_empty":true,"forced":receipt.forced,"cleanup_error":pending.cleanup_error(),"observation_published":false})
}
#[test]
#[ignore = "central frozen input actual source-bound FFprobe Job/output and post-create cancellation; no model/network/GUI"]
fn actual_source_bound_video_probe_and_cancel_preserve_original_owners() {
    kinekino_platform::install();
    let workspace = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let input = PathBuf::from(std::env::var("KINEKINO_REVIEW_SOURCE_RECEIPT_DIR").unwrap());
    assert!(input.starts_with(workspace.join("runtime/verification")));
    let source_guard = custody::Guard::acquire(&input.join("SOURCE.md"));
    assert_eq!(
        source_guard.sha256,
        std::env::var("KINEKINO_REVIEW_SOURCE_RECEIPT_SHA256")
            .unwrap()
            .to_ascii_lowercase()
    );
    let producer = custody::Guard::acquire(&std::env::current_exe().unwrap());
    let output = PathBuf::from(std::env::var("KINEKINO_REVIEW_OUTPUT_DIR").unwrap());
    assert!(output.starts_with(workspace.join("runtime/verification")));
    fs::create_dir(&output).unwrap();
    let review = workspace.join("runtime/verification/sprite-timeline-actual-r1");
    let nomination = custody::Guard::acquire(&review.join("NOMINATION.json"));
    assert_eq!(
        nomination.sha256,
        "b1a47d17ad348d6ac7c4c5ec9eb246a54f561babc442cf4310274cdb349be201"
    );
    let data: Value =
        serde_json::from_slice(&fs::read(review.join("NOMINATION.json")).unwrap()).unwrap();
    let case: ReviewCase = serde_json::from_value(data["case"].clone()).unwrap();
    let media: MediaReference = serde_json::from_value(data["media"].clone()).unwrap();
    let reader = ReviewCaseEvidence::new(ReviewFiles::open(&review).unwrap());
    let selected = inspect_review_input(&reader, &case, &media, &IsoBmffReviewClassifier).unwrap();
    let files = ReviewFiles::open(&review).unwrap();
    let source = files.bind_playback_media(selected).unwrap();
    let tool_path = PathBuf::from(std::env::var("KINEKINO_TIMELINE_FFPROBE").unwrap());
    let tool_guard = custody::Guard::acquire(&tool_path);
    assert_eq!(
        tool_guard.sha256,
        "a6618e99bb58869ded3c6f37b53aa1a8d701c3591dbb7b5b317d47369c112be2"
    );
    let tool = PinnedTimelineProbe::open(&tool_path, &tool_guard.sha256).unwrap();
    let (mut mask, mut system) = (0usize, 0usize);
    unsafe {
        windows::Win32::System::Threading::GetProcessAffinityMask(
            windows::Win32::System::Threading::GetCurrentProcess(),
            &mut mask,
            &mut system,
        )
        .unwrap();
    }
    let runtime = output.join("producer");
    fs::create_dir(&runtime).unwrap();
    let decoder_path = PathBuf::from(std::env::var("KINEKINO_TIMELINE_FFMPEG").unwrap());
    let decoder_guard = custody::Guard::acquire(&decoder_path);
    assert_eq!(
        decoder_guard.sha256,
        "71414777922d05f2639a5cf4a0f9a405f169809cf459000a6f976c9add61f060"
    );
    let decoder = PinnedTimelineAudioDecoder::open(&decoder_path, &decoder_guard.sha256).unwrap();
    let capture_path = PathBuf::from(std::env::var("KINEKINO_TIMELINE_CAPTURE_FFMPEG").unwrap());
    let capture_guard = custody::Guard::acquire(&capture_path);
    assert_eq!(
        capture_guard.sha256,
        "09948d4cdd0650da6ff5a87577469f2a218dc2615ae379f8f734d24c49de0f73"
    );
    let capture_tool =
        PinnedTimelineCaptureDecoder::open(&capture_path, &capture_guard.sha256).unwrap();
    let configuration = || TimelineProductionConfig {
        runtime_root: runtime.clone(),
        budget: kinekino_process::infrastructure::family::JobBudget {
            max_committed_bytes: 1024 * 1024 * 1024,
            affinity_mask: mask.isolate_lowest_one(),
        },
        total: Duration::from_secs(60),
        cleanup: Duration::from_secs(2),
        max_source_bytes: 128 * 1024 * 1024,
        max_output_bytes: 64 * 1024 * 1024,
    };
    write_new(
        &output.join("WITNESS.json"),
        &json!({"source":source_guard.evidence(),"producer":producer.evidence(),"tool":tool_guard.evidence(),"capture_tool":capture_guard.evidence(),"case":case,"media":media,"requested_video_probes":2,"requested_capture_processes":4,"new_model_execution":false,"new_capture_execution":true,"quality_approved":false}),
    );
    let session = TimelineProductionSession::reserve(&source, configuration()).unwrap();
    let blocked = match TimelineProductionSession::reserve(&source, configuration()) {
        Ok(_) => panic!("second producer slot admitted"),
        Err(error) => error,
    };
    assert_eq!(blocked.code, ErrorCode::RevisionConflict);
    let observed = session
        .video(
            &tool,
            VideoProbeWindow::new(16_000_000, 26_000_000).unwrap(),
            &Continue,
        )
        .unwrap();
    observed.check_current().unwrap();
    assert_eq!(observed.frames().frames().len(), 240);
    assert_eq!(observed.frames().frames().first().unwrap().pts, 196608);
    assert_eq!(observed.frames().frames().last().unwrap().pts, 318976);
    assert_eq!(
        observed.frames().time_base(),
        TimeBase::new(1, 12288).unwrap()
    );
    assert_eq!(observed.receipt().child_exit, Some(0));
    assert!(!observed.receipt().forced);
    assert!(observed.receipt().identity.creation_filetime.is_some());
    let accepted_root = observed.output_root().to_path_buf();
    let accepted_sha = observed.frames().sha256().to_owned();
    let mut cancelled = match session.video(
        &tool,
        VideoProbeWindow::new(16_000_000, 26_000_000).unwrap(),
        &CancelAfterCreate(runtime.clone(), "video-"),
    ) {
        Ok(_) => panic!("post-create cancellation published observation"),
        Err(error) => error,
    };
    assert_eq!(cancelled.error.code, ErrorCode::Cancelled);
    let busy = match session.video(
        &tool,
        VideoProbeWindow::new(16_000_000, 26_000_000).unwrap(),
        &Continue,
    ) {
        Ok(_) => panic!("replacement started before original pending cleanup"),
        Err(error) => error,
    };
    assert_eq!(busy.error.code, ErrorCode::ResourceExhausted);
    assert!(busy.pending.is_none());
    drop(busy);
    let cancellation = observe_cancelled(&mut cancelled);
    write_new(&output.join("CANCEL.json"), &cancellation);
    drop(cancelled);
    let audio = session.audio_clock(&decoder, &Continue).unwrap();
    audio.check_current().unwrap();
    let pcm = session.audio_pcm(&decoder, &Continue).unwrap();
    pcm.clock().check_current().unwrap();
    assert_eq!(pcm.pcm().samples(), 10078204);
    assert_eq!(pcm.pcm().first_pts(), 0);
    assert_eq!(pcm.pcm().end_pts(), 10078204);
    assert_eq!(pcm.clock().clock().frames(), audio.clock().frames());
    let pcm_observation = json!({"root":pcm.clock().output_root(),"samples":pcm.pcm().samples(),"first_pts":pcm.pcm().first_pts(),"end_pts":pcm.pcm().end_pts(),"sha256":pcm.pcm().sha256(),"child_exit":pcm.clock().receipt().child_exit,"forced":pcm.clock().receipt().forced,"pid":pcm.clock().receipt().identity.pid,"creation_filetime":pcm.clock().receipt().identity.creation_filetime,"pcm_lineage_validated":false});
    let mut pcm_cancelled =
        match session.audio_pcm(&decoder, &CancelAfterCreate(runtime.clone(), "audio-pcm-")) {
            Ok(_) => panic!("PCM post-create cancel published observation"),
            Err(error) => error,
        };
    assert_eq!(pcm_cancelled.error.code, ErrorCode::Cancelled);
    let pcm_cancellation = observe_cancelled(&mut pcm_cancelled);
    write_new(&output.join("PCM-CANCEL.json"), &pcm_cancellation);
    drop(pcm_cancelled);
    let rejected_range = match session.pcm_lineage(
        &pcm,
        &decoder,
        PcmClockRange::new(-1, 1).unwrap(),
        &Continue,
    ) {
        Ok(_) => panic!("out-of-source PCM range admitted"),
        Err(error) => error,
    };
    assert_eq!(rejected_range.error.code, ErrorCode::InvalidArgument);
    assert!(rejected_range.pending.is_none());
    drop(rejected_range);
    let lineage = session
        .pcm_lineage(
            &pcm,
            &decoder,
            PcmClockRange::new(272000, 408000).unwrap(),
            &Continue,
        )
        .unwrap();
    lineage.check_current().unwrap();
    assert_eq!(lineage.facts().full_samples, 10078204);
    assert_eq!(lineage.facts().clip_start_sample, 272000);
    assert_eq!(lineage.facts().clip_samples, 136000);
    assert_eq!(lineage.facts().full_data_sha256, pcm.pcm().sha256());
    assert_eq!(lineage.facts().replay_data_sha256, pcm.pcm().sha256());
    assert_eq!(lineage.receipt().child_exit, Some(0));
    assert!(!lineage.receipt().forced);
    let lineage_observation = json!({"root":lineage.output_root(),"full_samples":lineage.facts().full_samples,"clip_start_sample":lineage.facts().clip_start_sample,"clip_samples":lineage.facts().clip_samples,"full_sha256":lineage.facts().full_data_sha256,"replay_sha256":lineage.facts().replay_data_sha256,"clip_sha256":lineage.facts().clip_data_sha256,"pid":lineage.receipt().identity.pid,"creation_filetime":lineage.receipt().identity.creation_filetime,"child_exit":lineage.receipt().child_exit,"forced":lineage.receipt().forced,"pcm_lineage_validated":true,"verified_timeline":false});
    let mut replay_cancelled = match session.pcm_lineage(
        &pcm,
        &decoder,
        PcmClockRange::new(272000, 408000).unwrap(),
        &CancelAfterCreate(runtime.clone(), "pcm-replay-"),
    ) {
        Ok(_) => panic!("replay post-create cancel published lineage"),
        Err(error) => error,
    };
    assert_eq!(replay_cancelled.error.code, ErrorCode::Cancelled);
    let replay_cancellation = observe_cancelled(&mut replay_cancelled);
    write_new(&output.join("REPLAY-CANCEL.json"), &replay_cancellation);
    drop(replay_cancelled);
    lineage.check_current().unwrap();
    assert_eq!(audio.receipt().child_exit, Some(0));
    assert!(!audio.receipt().forced);
    assert!(audio.receipt().identity.creation_filetime.is_some());
    let first_audio_pts = audio.clock().frames().first().unwrap().pts;
    let last = audio.clock().frames().last().unwrap();
    let audio_end = i128::from(last.pts) + i128::from(last.samples);
    assert!(first_audio_pts <= 272000 && audio_end >= 408000);
    let audio_observation = json!({"root":audio.output_root(),"frames":audio.clock().frames().len(),"first_pts":first_audio_pts,"end_sample":audio_end,"sample_rate":audio.clock().sample_rate(),"stderr_sha256":audio.clock().sha256(),"pid":audio.receipt().identity.pid,"creation_filetime":audio.receipt().identity.creation_filetime,"child_exit":audio.receipt().child_exit,"forced":audio.receipt().forced,"pcm_lineage_validated":false});
    let mut audio_cancelled = match session.audio_clock(
        &decoder,
        &CancelAfterCreate(runtime.clone(), "audio-clock-"),
    ) {
        Ok(_) => panic!("audio post-create cancel published observation"),
        Err(error) => error,
    };
    assert_eq!(audio_cancelled.error.code, ErrorCode::Cancelled);
    let audio_cancellation = observe_cancelled(&mut audio_cancelled);
    write_new(&output.join("AUDIO-CANCEL.json"), &audio_cancellation);
    drop(audio_cancelled);
    audio.check_current().unwrap();
    drop(audio);
    let mut captures = Vec::new();
    let mut retained_captures = Vec::new();
    for (point, expected) in [
        (TimelineCapturePoint::Origin, 0),
        (
            TimelineCapturePoint::AtOrAfter {
                source_us: 17_000_000,
            },
            208896,
        ),
        (
            TimelineCapturePoint::AtOrAfter {
                source_us: 25_500_000,
            },
            313344,
        ),
    ] {
        let capture = session
            .capture_frame(&capture_tool, &observed, point, &Continue)
            .unwrap();
        capture.check_current().unwrap();
        assert_eq!(capture.observation().actual_pts, expected);
        assert_eq!(
            capture.observation().time_base,
            TimeBase::new(1, 12288).unwrap()
        );
        assert_eq!(capture.receipt().child_exit, Some(0));
        assert!(!capture.receipt().forced);
        if expected == 0 {
            assert_eq!(capture.decoded_origin().unwrap().pts, 0);
        } else {
            assert!(capture.decoded_origin().is_err());
        }
        captures.push(json!({"root":capture.output_root(),"pts":capture.observation().actual_pts,"time_base":"1/12288","png_sha256":capture.observation().png_sha256,"dimensions":capture.dimensions(),"pid":capture.receipt().identity.pid,"creation_filetime":capture.receipt().identity.creation_filetime,"child_exit":capture.receipt().child_exit,"forced":capture.receipt().forced}));
        retained_captures.push(capture);
    }
    let mut capture_cancelled = match session.capture_frame(
        &capture_tool,
        &observed,
        TimelineCapturePoint::AtOrAfter {
            source_us: 18_000_000,
        },
        &CancelAfterCreate(runtime.clone(), "capture-endpoint-"),
    ) {
        Ok(_) => panic!("capture post-create cancel published observation"),
        Err(error) => error,
    };
    assert_eq!(capture_cancelled.error.code, ErrorCode::Cancelled);
    let capture_cancellation = observe_cancelled(&mut capture_cancelled);
    write_new(&output.join("CAPTURE-CANCEL.json"), &capture_cancellation);
    drop(capture_cancelled);
    let inputs = TimelineBundleInputs {
        video: &observed,
        pcm: &lineage,
        origin: &retained_captures[0],
        start: &retained_captures[1],
        end: &retained_captures[2],
    };
    let swapped = TimelineBundleInputs {
        start: inputs.end,
        end: inputs.start,
        ..inputs
    };
    let rejected = match session.bundle(swapped, &Continue) {
        Ok(_) => panic!("swapped capture bounds admitted"),
        Err(error) => error,
    };
    assert_eq!(rejected.code, ErrorCode::InvalidArgument);
    let bundle = session.bundle(inputs, &Continue).unwrap();
    bundle.check_current().unwrap();
    assert_eq!(bundle.mapping().source_start_us, 17_000_000);
    assert_eq!(bundle.mapping().source_end_us, 25_500_000);
    assert_eq!(bundle.mapping().start_frame_pts, 208896);
    assert_eq!(bundle.mapping().end_frame_pts, 313344);
    assert_eq!(bundle.content().pcm.frames.len(), 27128);
    let bundle_guard = custody::Guard::acquire(&bundle.output_root().join("BUNDLE.json"));
    assert_eq!(bundle_guard.sha256, bundle.receipt_sha256());
    assert!(
        fs::OpenOptions::new()
            .write(true)
            .open(bundle.output_root().join("BUNDLE.json"))
            .is_err()
    );
    let bundle_observation = json!({"root":bundle.output_root(),"sha256":bundle.receipt_sha256(),"mapping_start_us":bundle.mapping().source_start_us,"mapping_end_us":bundle.mapping().source_end_us,"content_validated":true,"custody_registered":false,"authority_granted":false,"verified_timeline":false});
    bundle_guard.assert_unchanged();
    let source_store = ReviewFiles::open(&review).unwrap();
    let source_session = ReviewStore::open(&source_store, &case.package_sha256).unwrap();
    let source_package = source_session.package().clone();
    let source_decisions =
        serde_json::to_value(source_session.decisions(&case.item_id).unwrap()).unwrap();
    let destination = |root: PathBuf| TimelinePackageDestination {
        root,
        max_total_bytes: 1024 * 1024 * 1024,
        excluded_roots: Vec::new(),
    };
    let overlap = match publish(
        &bundle,
        &source_store,
        destination(review.clone()),
        &Continue,
    ) {
        Ok(_) => panic!("original root admitted for publication"),
        Err(error) => error,
    };
    assert_eq!(overlap.code, ErrorCode::InvalidArgument);
    assert!(!review.join(".timeline-package-publish.lock").exists());
    let cancelled_root = output.join("cancelled-review");
    fs::create_dir(&cancelled_root).unwrap();
    let cancelled_publication = match publish(
        &bundle,
        &source_store,
        destination(cancelled_root.clone()),
        &CancelAfterObject(cancelled_root.clone()),
    ) {
        Ok(_) => panic!("cancelled copy published package"),
        Err(error) => error,
    };
    assert_eq!(cancelled_publication.code, ErrorCode::Cancelled);
    assert_eq!(
        fs::read_dir(cancelled_root.join("packages"))
            .unwrap()
            .count(),
        0
    );
    let published_root = output.join("published-review");
    fs::create_dir(&published_root).unwrap();
    let published = publish(
        &bundle,
        &source_store,
        destination(published_root.clone()),
        &Continue,
    )
    .unwrap();
    published.check_current().unwrap();
    assert_ne!(published.case().package_sha256, case.package_sha256);
    assert_eq!(published.base_case(), &case);
    assert_eq!(published.case().item_id, case.item_id);
    assert_eq!(
        published.case().source_request_sha256,
        case.source_request_sha256
    );
    assert!(!published.replayed());
    let new_store = ReviewFiles::open(&published_root).unwrap();
    let new_session = ReviewStore::open(&new_store, &published.case().package_sha256).unwrap();
    let mut preserved = new_session.package().clone();
    let original_check_count = source_package
        .items
        .iter()
        .find(|item| item.item_id == case.item_id)
        .unwrap()
        .candidate
        .automated_checks
        .len();
    preserved
        .items
        .iter_mut()
        .find(|item| item.item_id == case.item_id)
        .unwrap()
        .candidate
        .automated_checks
        .truncate(original_check_count);
    assert_eq!(
        serde_json::to_value(&preserved).unwrap(),
        serde_json::to_value(&source_package).unwrap()
    );
    assert!(new_session.decisions(&case.item_id).unwrap().is_empty());
    assert_eq!(
        serde_json::to_value(source_session.decisions(&case.item_id).unwrap()).unwrap(),
        source_decisions
    );
    let declared = ReviewCaseEvidence::new(ReviewFiles::open(&published_root).unwrap());
    let new_case = declared.retain(published.case()).unwrap();
    let manifest: Value =
        serde_json::from_slice(&new_case.read(published.reference(), 65536).unwrap()).unwrap();
    assert_eq!(manifest["schema_version"], "live_2_1");
    assert_eq!(manifest["mapping_revision"], "1");
    assert_eq!(manifest["producer"]["sha256"], producer.sha256);
    assert_eq!(manifest["bundle_sha256"], bundle.receipt_sha256());
    assert!(
        reader
            .retain(&case)
            .unwrap()
            .inspect(published.reference())
            .is_err()
    );
    let replayed = publish(
        &bundle,
        &source_store,
        destination(published_root.clone()),
        &Continue,
    )
    .unwrap();
    assert!(replayed.replayed());
    assert_eq!(replayed.case(), published.case());
    let live_case = published.case().clone();
    let live_reference = published.reference().clone();
    let resolver = PublishedTimelineResolver::new(replayed).unwrap();
    let foreign = match resolver.retain(&case, &media, &live_reference) {
        Ok(_) => panic!("old case admitted to live proof"),
        Err(error) => error,
    };
    assert_eq!(foreign.code, ErrorCode::RevisionConflict);
    let cases = ReviewCases::new(ReviewFiles::open(&published_root).unwrap());
    let media_provider = ReviewPackageMedia::new(ReviewFiles::open(&published_root).unwrap());
    let authority = timeline_evidence::retain(
        &cases,
        &media_provider,
        &resolver,
        &live_case,
        &media,
        &live_reference,
    )
    .unwrap();
    authority.check_current().unwrap();
    assert_eq!(authority.mapping().source_start_us, 17_000_000);
    assert_eq!(authority.mapping().source_end_us, 25_500_000);
    assert!(!authority.playback_ready());
    let reader_observation = json!({"case":live_case,"reference":live_reference,"mapping_start_us":authority.mapping().source_start_us,"mapping_end_us":authority.mapping().source_end_us,"retained_application_evidence":true,"foreign_old_case_rejected":true,"playback_ready":false,"quality_approved":false});
    drop(authority);
    let sdk_observation = sdk::verify(
        &published_root,
        std::sync::Arc::new(resolver),
        &live_case,
        &media,
        &live_reference,
        &output,
    );
    let publication_observation = json!({"root":published.root(),"base_case":published.base_case(),"new_case":published.case(),"manifest":published.reference(),"bundle_sha256":published.bundle_sha256(),"source_package_preserved":true,"source_decisions_preserved":true,"new_decisions":0,"cancel_before_commit_packages":0,"exact_package_replay":true,"custody_published":true,"verified_timeline":false,"authority_granted":false});
    drop(new_case);
    drop(new_session);
    drop(published);
    drop(source_session);
    drop(bundle_guard);
    drop(bundle);
    drop(retained_captures);
    drop(lineage);
    drop(pcm);
    observed.check_current().unwrap();
    drop(observed);
    drop(session);
    let reacquired = TimelineProductionSession::reserve(&source, configuration()).unwrap();
    drop(reacquired);
    assert!(!runtime.join("ABANDONED.json").exists());
    source.check_current().unwrap();
    source_guard.assert_unchanged();
    producer.assert_unchanged();
    tool_guard.assert_unchanged();
    decoder_guard.assert_unchanged();
    capture_guard.assert_unchanged();
    nomination.assert_unchanged();
    write_new(
        &output.join("RETURN.json"),
        &json!({"source":source_guard.evidence(),"producer":producer.evidence(),"tool":tool_guard.evidence(),"accepted_root":accepted_root,"accepted_stdout_sha256":accepted_sha,"frames":240,"time_base":"1/12288","first_pts":196608,"last_pts":318976,"cancel":cancellation,"audio":audio_observation,"audio_cancel":audio_cancellation,"pcm":pcm_observation,"pcm_cancel":pcm_cancellation,"pcm_lineage":lineage_observation,"replay_cancel":replay_cancellation,"captures":captures,"capture_cancel":capture_cancellation,"bundle":bundle_observation,"publication":publication_observation,"live_reader":reader_observation,"live_sdk_cli":sdk_observation,"capture_tool":capture_guard.evidence(),"decoder":decoder_guard.evidence(),"producer_slot_exclusive_and_reacquired":true,"video_observation_accepted":true,"verified_timeline":false,"actual9_binding":false,"quality_approved":false}),
    );
}
