#![cfg(windows)]
#[path = "sprite_timeline_bundle/assemble.rs"]
mod assemble;
#[path = "sprite_capture_cli/custody.rs"]
mod custody;
#[path = "sprite_timeline_bundle/storage.rs"]
mod storage;
use kinekino_player::{
    application::review_source::{ReviewCases, ReviewPackageMedia, timeline_evidence},
    infrastructure::{
        model_review::ReviewFiles,
        timeline_evidence::{ReviewTimelineEvidence, TrustedTimelineAdmission},
    },
};
use serde_json::json;

#[test]
#[ignore = "explicit central allocation: copies/verifies pinned historical media/tool evidence; no decoder or inference execution"]
fn publish_and_verify_existing_sprite_timeline_bundle_preserving_originals() {
    kinekino_platform::install();
    let assemble::Prepared {
        mut bundle,
        case,
        media,
        evidence,
        pins,
    } = assemble::prepare_from_pinned_sprite_history();
    let cases = ReviewCases::new(ReviewFiles::open(&bundle.root).unwrap());
    let medias = ReviewPackageMedia::new(ReviewFiles::open(&bundle.root).unwrap());
    let resolver = ReviewTimelineEvidence::new(
        ReviewFiles::open(&bundle.root).unwrap(),
        TrustedTimelineAdmission {
            case: case.clone(),
            media: media.clone(),
            evidence: evidence.clone(),
            audio_engine_sha256: pins.audio,
            video_engine_sha256: pins.probe,
            capture_engine_sha256: pins.capture,
            capture_cli_sha256: pins.cli,
        },
    )
    .unwrap();
    let retained =
        timeline_evidence::retain(&cases, &medias, &resolver, &case, &media, &evidence).unwrap();
    let mapping = retained.mapping();
    assert_eq!(
        (mapping.source_start_us, mapping.source_end_us),
        (17_000_000, 25_500_000)
    );
    assert_eq!(
        (mapping.start_frame_pts, mapping.end_frame_pts),
        (208896, 313344)
    );
    assert!(!retained.playback_ready());
    retained.check_current().unwrap();
    bundle.assert_all_unchanged();
    let receipt=bundle.seal_file("RETURN.json",&json!({"case":case,"media":media,"evidence":evidence,"source_start_us":mapping.source_start_us.to_string(),"source_end_us":mapping.source_end_us.to_string(),"start_frame_pts":mapping.start_frame_pts,"end_frame_pts":mapping.end_frame_pts,"resolver_observed":true,"originals_unchanged":true,"playback_ready":retained.playback_ready(),"new_capture_execution":false,"new_decode_execution":false,"actual9_binding":false,"quality_approved":false,"scope":"new-host read-only revalidation of historical producer evidence"}));
    bundle.assert_all_unchanged();
    println!(
        "actual bundle {} RETURN SHA256 {receipt}",
        bundle.root.display()
    );
}
