#![cfg(windows)]
//! Actual local producer and SDK association. No inference/network/GUI or playback child.
use kinekino_player::{
    application::{
        api::{ApiError, PlayerSdk, Request},
        review_source::{
            MutationControl, ReviewCases, ReviewPackageMedia, api::ReviewSourceService,
        },
    },
    assembly::review_playback::{
        IncompleteTimelineCleanup, TimelinePreparation, TimelinePreparationTools, prepare_timeline,
    },
    domain::review_source::{MediaReference, ReviewCase},
    infrastructure::{
        model_review::ReviewFiles,
        review_source::UserReviewFiles,
        timeline_evidence::{
            production::*,
            publication::{PublishedTimelineResolver, TimelinePackageDestination},
        },
    },
};
use serde_json::json;
use std::{fs, path::PathBuf, time::Duration};
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
struct Retain;
impl IncompleteTimelineCleanup for Retain {
    fn incomplete(&mut self, error: &ApiError, pending: &mut PendingTimelineProcess<'_, '_>) -> ! {
        eprintln!(
            "{}",
            json!({"error":error,"cleanup_complete":false,"output_root":pending.output_root(),"cleanup_error":pending.cleanup_error()})
        );
        std::process::exit(i32::from(error.exit_code()));
    }
}
fn call(
    sdk: &PlayerSdk,
    root: &std::path::Path,
    name: &str,
    request: Request,
) -> serde_json::Value {
    let response = sdk.execute(request);
    fs::write(
        root.join(format!("{name}.json")),
        serde_json::to_vec_pretty(&response).unwrap(),
    )
    .unwrap();
    assert!(response.ok, "{name}: {:?}", response.error);
    response.data.unwrap()
}
#[test]
#[ignore = "central actual MKV source/probe/PCM/capture/publication/SDK association; no playback/GUI/model/network"]
fn actual_matroska_producer_publishes_fresh_timeline_and_sdk_verified_association() {
    kinekino_platform::install();
    let root = PathBuf::from(
        std::env::var_os("REVIEW_MATROSKA_TIMELINE_OUTPUT").expect("explicit new output"),
    );
    let source = PathBuf::from(
        std::env::var_os("REVIEW_MATROSKA_PACKAGE_ROOT").expect("explicit prepared source"),
    );
    assert!(root.is_absolute() && source.is_absolute());
    fs::create_dir(&root).unwrap();
    for name in ["runtime", "published", "user"] {
        fs::create_dir(root.join(name)).unwrap();
    }
    let case = ReviewCase {
        package_sha256: "b1e2be1056e044cd3a8f9e41d16d768b6292de81c2467890d79eee844268e390".into(),
        item_id: "gemma270-highlights".into(),
        source_request_sha256: "658e5fbded1a1203472c937b8b7188c64915859446cb6c4e9b827d877b0d8093"
            .into(),
    };
    let media = MediaReference {
        asset_id: "explicit-sintel-source".into(),
        sha256: "f12c070e295b38cfc94ebd61ac3357c3bac82d1015985f1e8da93a4c6496c46d".into(),
    };
    let tools = TimelinePreparationTools {
        video: PinnedTimelineProbe::open(
            &PathBuf::from(std::env::var_os("KINEKINO_TIMELINE_FFPROBE").unwrap()),
            "a6618e99bb58869ded3c6f37b53aa1a8d701c3591dbb7b5b317d47369c112be2",
        )
        .unwrap(),
        audio: PinnedTimelineAudioDecoder::open(
            &PathBuf::from(std::env::var_os("KINEKINO_TIMELINE_FFMPEG").unwrap()),
            "71414777922d05f2639a5cf4a0f9a405f169809cf459000a6f976c9add61f060",
        )
        .unwrap(),
        capture: PinnedTimelineCaptureDecoder::open(
            &PathBuf::from(std::env::var_os("KINEKINO_TIMELINE_CAPTURE_FFMPEG").unwrap()),
            "09948d4cdd0650da6ff5a87577469f2a218dc2615ae379f8f734d24c49de0f73",
        )
        .unwrap(),
    };
    let (mut affinity, mut system) = (0usize, 0usize);
    unsafe {
        windows::Win32::System::Threading::GetProcessAffinityMask(
            windows::Win32::System::Threading::GetCurrentProcess(),
            &mut affinity,
            &mut system,
        )
        .unwrap();
    }
    let token = prepare_timeline(
        TimelinePreparation {
            source_root: source,
            case: case.clone(),
            media: media.clone(),
            runtime: TimelineProductionConfig {
                runtime_root: root.join("runtime"),
                budget: kinekino_process::infrastructure::family::JobBudget {
                    max_committed_bytes: 1_073_741_824,
                    affinity_mask: affinity.isolate_lowest_one(),
                },
                total: Duration::from_secs(120),
                cleanup: Duration::from_secs(2),
                max_source_bytes: 1_073_741_824,
                max_output_bytes: 67_108_864,
            },
            destination: TimelinePackageDestination {
                root: root.join("published"),
                max_total_bytes: 2_147_483_648,
                excluded_roots: vec![root.join("user")],
            },
            video_window: VideoProbeWindow::new(107_000_000, 134_000_000).unwrap(),
            pcm_range: PcmClockRange::new(1_716_000, 2_140_800).unwrap(),
            start_us: 107_250_000,
            end_us: 133_800_000,
        },
        &tools,
        &Control,
        &mut Retain,
    )
    .unwrap();
    token.check_current().unwrap();
    assert_eq!(token.base_case(), &case);
    assert_ne!(token.case().package_sha256, case.package_sha256);
    let published_case = token.case().clone();
    let reference = token.reference().clone();
    fs::write(root.join("published-identity.json"), serde_json::to_vec_pretty(&json!({"base_case":case,"case":published_case,"reference":reference,"media":media,"historical_media_verified":false,"quality_approved":false,"playback_executed":false})).unwrap()).unwrap();
    let resolver = PublishedTimelineResolver::new(token).unwrap();
    let service = ReviewSourceService::new(
        ReviewCases::new(ReviewFiles::open(&root.join("published")).unwrap()),
        ReviewPackageMedia::new(ReviewFiles::open(&root.join("published")).unwrap()),
        UserReviewFiles::open(&root.join("user"), true).unwrap(),
        true,
    )
    .with_timeline_evidence(resolver);
    let sdk = PlayerSdk::default().with_review_source(service);
    let mut create = Request::new("reviews.association.create");
    create.payload = json!({"case":published_case,"association_id":"actual-mkv","media":media,"note":"Explicit technical media selection; user quality unapproved"});
    create.context.idempotency_key = Some("create-mkv".into());
    call(&sdk, &root, "association-create", create.clone());
    let mut verify = Request::new("reviews.association.revise");
    verify.payload = create.payload;
    verify.payload["timeline_evidence"] = json!(reference);
    verify.context.expected_revision = Some("1".into());
    verify.context.idempotency_key = Some("verify-mkv".into());
    let verified = call(&sdk, &root, "association-verify", verify.clone());
    assert_eq!(verified["receipt"]["body"]["association"]["revision"], "2");
    let mapping = &verified["receipt"]["body"]["association"]["timeline_status"]["verified"];
    assert!(mapping.is_object(), "{verified}");
    assert_eq!(mapping["source_start_us"], "107250000");
    assert_eq!(mapping["source_end_us"], "133800000");
    assert_eq!(verified["playback_ready"], false);
    let replay = call(&sdk, &root, "association-replay", verify);
    assert_eq!(verified["receipt"], replay["receipt"]);
    println!(
        "actual MKV fresh producer and SDK verified association complete; playback and quality unapproved"
    );
}
