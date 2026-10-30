//! Existing actual media custody and raw classification only. No engine/GUI/model launch.
#![cfg(windows)] // Windows-only infrastructure (review_source/model_review/timeline_evidence); Linux port pending platform split.
use kinekino_player::{
    application::review_playback::{ReviewContentClass, inspect_review_input},
    application::review_source::ReviewCaseEvidence,
    domain::review_source::{MediaReference, ReviewCase},
    infrastructure::{
        model_review::ReviewFiles, review_content_classification::LocalReviewContentClassifier,
    },
};
#[test]
#[ignore = "explicit actual9 prepared package root; full content verification, no decode/playback"]
fn actual_sintel_matroska_retains_exact_case_media_and_header_classification() {
    kinekino_platform::install();
    let root = std::path::PathBuf::from(
        std::env::var_os("REVIEW_MATROSKA_PACKAGE_ROOT").expect("explicit prepared package root"),
    );
    assert!(root.is_absolute());
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
    let reader = ReviewCaseEvidence::new(ReviewFiles::open(&root).unwrap());
    let input =
        inspect_review_input(&reader, &case, &media, &LocalReviewContentClassifier).unwrap();
    assert_eq!(input.identity().case, case);
    assert_eq!(input.identity().media, media);
    assert_eq!(
        input.identity().classification,
        ReviewContentClass::MatroskaHeader
    );
    assert_eq!(input.identity().declared_media_type, "video/x-matroska");
    assert_eq!(input.identity().bytes, 681285280);
    let files = ReviewFiles::open(&root).unwrap();
    let guarded = files.bind_playback_media(input).unwrap();
    guarded.check_current().unwrap();
    println!("actual Matroska input retained; no runtime admission, timeline or quality grant");
}
