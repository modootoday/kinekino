use super::*;
use crate::{
    application::card_projects::{StoredCardProject, prepare_publication, publication_sha256},
    domain::card_document::CardDocument,
};
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
#[ignore = "Explicit read-only allocation of the original R115 source blobs"]
fn historical_r115_prepared_blobs_round_trip_without_changing_descriptor_bytes() {
    use sha2::{Digest, Sha256};
    let root = std::path::PathBuf::from(
        std::env::var_os("CARD_HISTORICAL_SOURCE_ROOT").expect("Original R115 sources"),
    );
    assert!(root.is_absolute());
    for sha in [
        "08bca41f8336aa0f4f5087787a42f8bffed558c227366013576850b4120497be",
        "1c1d0ffdf62fe9017c8d4f378f3bcf8b17dd2043e1f461ec16c55c4472c499f7",
    ] {
        let bytes = std::fs::read(root.join(format!("{sha}.utf8"))).unwrap();
        assert_eq!(format!("{:x}", Sha256::digest(&bytes)), sha);
        let prepared: PreparedCardBundle = serde_json::from_slice(&bytes).unwrap();
        assert!(prepared.template.frame_required);
        assert!(
            prepared
                .cards
                .iter()
                .all(|card| card.frame.is_some() && card.observation.is_some())
        );
        assert_eq!(serde_json::to_vec(&prepared).unwrap(), bytes);
    }
}
struct Store(StoredCardProject);
impl CardProjectStore for Store {
    fn load(&self, _: &str, _: Option<DecimalU64>) -> Result<StoredCardProject, ApiError> {
        Ok(self.0.clone())
    }
    fn publish(
        &self,
        _: Option<DecimalU64>,
        _: &CardDocument,
    ) -> Result<StoredCardProject, ApiError> {
        panic!("Preparation must never mutate the project")
    }
}
struct Assets {
    calls: AtomicUsize,
    wrong_png: bool,
}
impl CardBundleAssets for Assets {
    fn verify_template(&self, canvas: &CardCanvas) -> Result<BundleTemplatePin, ApiError> {
        Ok(BundleTemplatePin {
            role_layouts_and_pages: false,
            frame_required: true,
            canvas: canvas.clone(),
            common_font_sha256: vec!["d".repeat(64)],
            license_refs: vec!["fixture-license".into()],
            limitations: vec!["fixture_only_not_native_rendering".into()],
        })
    }
    fn inspect_frame(
        &self,
        selection: &SelectedCardFrame,
    ) -> Result<BundleFrameObservation, ApiError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(BundleFrameObservation {
            sha256: if self.wrong_png {
                "f".repeat(64)
            } else {
                selection.png_sha256.clone()
            },
            bytes: DecimalU64(100),
            width: 8,
            height: 8,
        })
    }
    fn verify_new_output_directory(&self, path: &str) -> Result<(), ApiError> {
        if path == "fixture-existing" {
            Err(error(ErrorCode::RevisionConflict, "Fixture output exists"))
        } else {
            Ok(())
        }
    }
}
fn fixture() -> (Store, CardBundleRequest, Assets) {
    let document: CardDocument = serde_json::from_value(serde_json::json!({"schema_version":"1","document_id":"bundle-1","revision":"1",
        "source":{"source_id":"source-1","source_revision":"1","source_sha256":"a".repeat(64),"cues":[{"cue_id":"cue-1","text":"원문","start_us":null,"end_us":null}]},
        "layout":{"canvas":{"width":1080,"height":1080,"template_id":"card.frame-two-slot","template_version":"0.1.0-proposal.1","template_sha256":"b".repeat(64)},
            "cards":[{"card_id":"cover","role":"cover","text_kind":"summary","content":{"front":"표지","back":"요약","source_refs":["cue-1"]},"frame":{"artifact_id":"frame-1","png_sha256":"c".repeat(64),"source_sha256":"a".repeat(64),"declared_source_pts_us":null},"alt_text_draft":"초안 설명"},
                     {"card_id":"last","role":"closing","text_kind":"quotation","content":{"front":"마지막","back":"원문","source_refs":["cue-1"]},"frame":{"artifact_id":"frame-1","png_sha256":"c".repeat(64),"source_sha256":"a".repeat(64),"declared_source_pts_us":null},"alt_text_draft":"마지막 설명"}],"caption_draft":"게시문"}})).unwrap();
    let publication = prepare_publication(None, &document).unwrap();
    let sha = publication_sha256(&publication).unwrap();
    let request = CardBundleRequest {
        project: SavedCardProjectPin {
            document_id: document.document_id.clone(),
            publication_revision: DecimalU64(1),
            publication_sha256: sha.clone(),
        },
        frames: vec![SelectedCardFrame {
            artifact_id: "frame-1".into(),
            path: "fixture-selected.png".into(),
            png_sha256: "c".repeat(64),
        }],
        output_directory: "fixture-new".into(),
        review_artifact_only: true,
    };
    (
        Store(StoredCardProject {
            publication_sha256: sha,
            latest_publication_revision: DecimalU64(1),
            publication,
        }),
        request,
        Assets {
            calls: AtomicUsize::new(0),
            wrong_png: false,
        },
    )
}
#[test]
fn prepares_ordered_complete_project_without_creating_accepted_output_or_publishing() {
    let (store, request, assets) = fixture();
    let before = serde_json::to_vec(&store.0).unwrap();
    let prepared = prepare(&store, &assets, request).unwrap();
    assert_eq!(prepared.cards[0].output_name, "001-cover.png");
    assert_eq!(prepared.cards[1].output_name, "002-closing.png");
    assert_eq!(prepared.cards[1].position, DecimalU64(1));
    assert_eq!(prepared.publication.document.layout.caption_draft, "게시문");
    assert_eq!(
        prepared.publication.document.layout.cards[1].alt_text_draft,
        "마지막 설명"
    );
    assert!(prepared.review_artifact_only);
    assert_eq!(assets.calls.load(Ordering::SeqCst), 2);
    assert_eq!(serde_json::to_vec(&store.0).unwrap(), before);
    let value = serde_json::to_value(prepared).unwrap();
    assert!(value.get("accepted_output_revision").is_none());
    assert!(value.get("quality_approved").is_none());
}
#[test]
fn rejects_wrong_publication_and_adapter_tampering_before_frame_io() {
    let (mut store, mut request, assets) = fixture();
    request.project.publication_sha256 = "e".repeat(64);
    assert_eq!(
        prepare(&store, &assets, request).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let (_, request, _) = fixture();
    store.0.publication.document.layout.caption_draft = "tampered".into();
    assert_eq!(
        prepare(&store, &assets, request).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(assets.calls.load(Ordering::SeqCst), 0);
}
#[test]
fn rejects_duplicate_unused_missing_and_changed_png_selections() {
    let (store, request, assets) = fixture();
    let mut duplicate = request.clone();
    duplicate.frames.push(duplicate.frames[0].clone());
    assert_eq!(
        prepare(&store, &assets, duplicate).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let mut unused = request.clone();
    let mut extra = unused.frames[0].clone();
    extra.artifact_id = "unused".into();
    unused.frames.push(extra);
    assert_eq!(
        prepare(&store, &assets, unused).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let mut missing = request.clone();
    missing.frames.clear();
    assert_eq!(
        prepare(&store, &assets, missing).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    let mut wrong = request.clone();
    wrong.frames[0].png_sha256 = "e".repeat(64);
    assert_eq!(
        prepare(&store, &assets, wrong).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let changed = Assets {
        calls: AtomicUsize::new(0),
        wrong_png: true,
    };
    assert_eq!(
        prepare(&store, &changed, request).unwrap_err().code,
        ErrorCode::SourceExpired
    );
}
#[test]
fn rejects_nonreview_and_existing_output_without_modifying_saved_content() {
    let (store, mut request, assets) = fixture();
    request.review_artifact_only = false;
    assert_eq!(
        prepare(&store, &assets, request).unwrap_err().code,
        ErrorCode::NeedsInput
    );
    let (_, mut request, _) = fixture();
    request.output_directory = "fixture-existing".into();
    assert_eq!(
        prepare(&store, &assets, request).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
}
