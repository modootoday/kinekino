//! Real immutable filesystem custody, synthetic bytes; no decode/timeline/AI approval.
#![cfg(windows)] // Windows-only infrastructure (review_source/model_review/timeline_evidence); Linux port pending platform split.
use kinekino_player::application::api::ProposalClient;
use kinekino_player::{
    application::{
        api::{ApiError, ErrorCode},
        model_review::ReviewStore,
        review_source::{
            MutationControl,
            media_preparation::{digest, prepare},
        },
    },
    domain::{
        api_values::DecimalU64,
        model_review::{EvidenceAsset, ReviewPackage},
        review_source::ReviewCase,
    },
    infrastructure::{
        model_review::ReviewFiles,
        timeline_evidence::publication::{
            TimelinePackageDestination,
            preparation::{LocalMediaSelection, prepare_local_media},
        },
    },
};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
struct Cancel;
impl MutationControl for Cancel {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Err(ApiError::new(ErrorCode::Cancelled, "cancel fixture"))
    }
}
struct Fixture {
    root: PathBuf,
    original: PathBuf,
    destination: PathBuf,
    media: PathBuf,
    asset: EvidenceAsset,
    body: Vec<u8>,
    case: ReviewCase,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "review-media-preparation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&root).unwrap();
        let original = root.join("original");
        let destination = root.join("prepared");
        let media_root = root.join("selection");
        for path in [&original, &destination, &media_root] {
            fs::create_dir(path).unwrap();
        }
        for name in ["objects", "packages", "decisions"] {
            fs::create_dir(original.join(name)).unwrap();
        }
        let request = b"original untimed model request";
        let candidate = b"original model output";
        for bytes in [request.as_slice(), candidate.as_slice()] {
            fs::write(original.join("objects").join(digest(bytes)), bytes).unwrap();
        }
        let body = serde_json::to_vec(&json!({"schema_version":"1","package_id":"original","items":[{
            "item_id":"item","task":"correction","source":{"asset_id":"request","sha256":digest(request),"bytes":request.len().to_string(),"media_type":"text/plain"},
            "range":null,"baseline":null,"candidate":{"condition":{"model_id":"fixture","revision":"r1","precision":"f32","engine_id":"fixture","engine_revision":"r1","input_conditions":{"kind":"synthetic"}},
            "assets":[{"asset_id":"candidate","sha256":digest(candidate),"bytes":candidate.len().to_string(),"media_type":"text/plain"}],
            "automated_checks":[{"check_id":"quality","outcome":"not_run","summary":"Unapproved","evidence":[]}],"measured_resources":{"kind":"synthetic"}},
            "criteria":[{"criterion_id":"faithful","how_to_review":"Compare","expected":"Faithful"}],"known_issues":["NeedsRepair"]}]})).unwrap();
        let case = ReviewCase {
            package_sha256: digest(&body),
            item_id: "item".into(),
            source_request_sha256: digest(request),
        };
        fs::write(
            original
                .join("packages")
                .join(format!("{}.json", case.package_sha256)),
            &body,
        )
        .unwrap();
        fs::write(
            original.join("decisions").join("original-marker"),
            b"NeedsRepair",
        )
        .unwrap();
        let bytes = b"synthetic declared media, no decode claim";
        let media = media_root.join("fixture.mkv");
        fs::write(&media, bytes).unwrap();
        let asset = EvidenceAsset {
            asset_id: "explicit-media".into(),
            sha256: digest(bytes),
            bytes: DecimalU64(bytes.len() as u64),
            media_type: "video/x-matroska".into(),
        };
        Self {
            root,
            original,
            destination,
            media,
            asset,
            body,
            case,
        }
    }
    fn selection(&self) -> LocalMediaSelection {
        LocalMediaSelection {
            path: self.media.clone(),
            asset: self.asset.clone(),
        }
    }
    fn destination(&self) -> TimelinePackageDestination {
        TimelinePackageDestination {
            root: self.destination.clone(),
            max_total_bytes: 1_048_576,
            excluded_roots: vec![self.original.clone()],
        }
    }
}
#[test]
fn prepared_package_retains_original_candidate_and_does_not_grant_clock_or_quality() {
    kinekino_platform::install();
    let f = Fixture::new();
    let original = ReviewFiles::open(&f.original).unwrap();
    let prepared =
        prepare_local_media(&original, &f.case, f.selection(), f.destination(), &Control).unwrap();
    assert!(!prepared.replayed());
    assert_eq!(prepared.relation().base_case, f.case);
    assert!(!prepared.relation().historical_media_verified);
    assert!(!prepared.relation().timeline_verified);
    assert!(!prepared.relation().quality_approved);
    assert_ne!(prepared.case().package_sha256, f.case.package_sha256);
    let store = ReviewFiles::open(prepared.root()).unwrap();
    let session = ReviewStore::open(&store, &prepared.case().package_sha256).unwrap();
    let mut package = session.package().clone();
    let added = package.items[0].candidate.automated_checks.pop().unwrap();
    assert!(matches!(
        added.outcome,
        kinekino_player::domain::model_review::CheckOutcome::NotRun
    ));
    let base: ReviewPackage = serde_json::from_slice(&f.body).unwrap();
    assert_eq!(
        serde_json::to_value(package).unwrap(),
        serde_json::to_value(base).unwrap()
    );
    let again =
        prepare_local_media(&original, &f.case, f.selection(), f.destination(), &Control).unwrap();
    assert!(again.replayed());
    assert_eq!(again.case(), prepared.case());
    assert_eq!(
        fs::read(
            f.original
                .join("packages")
                .join(format!("{}.json", f.case.package_sha256))
        )
        .unwrap(),
        f.body
    );
    assert_eq!(
        fs::read(f.original.join("decisions/original-marker")).unwrap(),
        b"NeedsRepair"
    );
    println!("custody fixture={}", f.root.display());
}
#[test]
fn wrong_case_media_budget_and_cancel_publish_no_package() {
    kinekino_platform::install();
    let f = Fixture::new();
    let original = ReviewFiles::open(&f.original).unwrap();
    let mut case = f.case.clone();
    case.source_request_sha256 = "a".repeat(64);
    assert!(prepare(&f.body, &case, f.asset.clone()).is_err());
    let mut selection = f.selection();
    selection.asset.sha256 = "a".repeat(64);
    assert!(prepare_local_media(&original, &f.case, selection, f.destination(), &Control).is_err());
    let mut destination = f.destination();
    destination.max_total_bytes = 1;
    assert!(prepare_local_media(&original, &f.case, f.selection(), destination, &Control).is_err());
    let error = prepare_local_media(&original, &f.case, f.selection(), f.destination(), &Cancel)
        .err()
        .unwrap();
    assert_eq!(error.code, ErrorCode::Cancelled);
    assert_eq!(fs::read_dir(&f.destination).unwrap().count(), 0);
}
#[test]
fn retained_corrupt_object_is_not_overwritten_or_committed() {
    kinekino_platform::install();
    let f = Fixture::new();
    let original = ReviewFiles::open(&f.original).unwrap();
    fs::create_dir(f.destination.join("objects")).unwrap();
    let path = f.destination.join("objects").join(&f.asset.sha256);
    fs::write(&path, b"partial orphan").unwrap();
    assert!(
        prepare_local_media(&original, &f.case, f.selection(), f.destination(), &Control).is_err()
    );
    assert_eq!(fs::read(path).unwrap(), b"partial orphan");
    assert_eq!(
        fs::read_dir(f.destination.join("packages"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn typed_sdk_media_preparation_preserves_configuration_and_readonly_boundaries() {
    kinekino_platform::install();
    use kinekino_player::{
        application::{
            api::{PlayerSdk, Request, RequestContext},
            review_source::media_api::{MediaSelectionRequest, PREPARE},
        },
        infrastructure::review_source::{
            media_preparation::{FixedMediaSelection, MediaPreparationFiles},
            open_api,
        },
    };
    use std::collections::BTreeMap;
    let f = Fixture::new();
    let user = f.root.join("user");
    fs::create_dir(&user).unwrap();
    let config = |writable| {
        MediaPreparationFiles::new(
            f.original.clone(),
            f.destination.clone(),
            BTreeMap::from([(
                "selected".into(),
                FixedMediaSelection {
                    path: f.media.clone(),
                    asset: f.asset.clone(),
                },
            )]),
            1_048_576,
            writable,
        )
        .unwrap()
    };
    let input = MediaSelectionRequest {
        base_case: f.case.clone(),
        selection_id: "selected".into(),
    };
    let sdk = PlayerSdk::default().with_review_source(
        open_api(&f.original, &user, true)
            .unwrap()
            .with_media_preparation(config(true)),
    );
    let receipt = sdk
        .prepare_review_media(RequestContext::default(), &input)
        .unwrap();
    assert!(!receipt.replayed);
    let observed = sdk
        .inspect_review_media(RequestContext::default(), &input)
        .unwrap();
    assert_eq!(observed.case, receipt.case);
    assert!(observed.replayed);
    let caps = sdk.execute(Request::new("capabilities"));
    assert_eq!(
        caps.data.as_ref().unwrap()["commands"]
            .as_array()
            .unwrap()
            .len(),
        119
    );
    assert_eq!(
        caps.data.as_ref().unwrap()["review_media_preparation"]["write_available"],
        true
    );
    let unknown = MediaSelectionRequest {
        selection_id: "foreign".into(),
        ..input.clone()
    };
    assert_eq!(
        sdk.prepare_review_media(RequestContext::default(), &unknown)
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    let mut request = Request::new(PREPARE);
    request.payload = serde_json::to_value(&input).unwrap();
    request.payload["path"] = "C:/caller/source.mkv".into();
    assert_eq!(
        sdk.execute(request).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
    let readonly = PlayerSdk::default().with_review_source(
        open_api(&f.original, &user, false)
            .unwrap()
            .with_media_preparation(config(false)),
    );
    assert_eq!(
        readonly
            .prepare_review_media(RequestContext::default(), &input)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    fs::rename(&f.media, f.media.with_extension("removed")).unwrap();
    assert_eq!(
        readonly
            .inspect_review_media(RequestContext::default(), &input)
            .unwrap()
            .case,
        receipt.case
    );
    let bad_context = RequestContext {
        timeout_ms: Some(1),
        ..RequestContext::default()
    };
    assert_eq!(
        readonly
            .inspect_review_media(bad_context, &input)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        PlayerSdk::default()
            .prepare_review_media(RequestContext::default(), &input)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    assert_eq!(
        fs::read(f.original.join("decisions/original-marker")).unwrap(),
        b"NeedsRepair"
    );
}

#[test]
#[ignore = "explicit original actual9 workspace and new evidence directory; local media copy only"]
fn actual_nine_media_selection_preserves_original_package_and_decisions() {
    kinekino_platform::install();
    use std::{collections::BTreeMap, io::Read, path::Path};
    fn file_sha(path: &Path) -> String {
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        let mut file = fs::File::open(path).unwrap();
        let mut buffer = vec![0u8; 1_048_576];
        loop {
            let count = file.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            hash.update(&buffer[..count]);
        }
        format!("{:x}", hash.finalize())
    }
    fn inventory(root: &Path) -> BTreeMap<PathBuf, String> {
        let mut files = BTreeMap::new();
        for entry in fs::read_dir(root).unwrap() {
            let path = entry.unwrap().path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            assert!(!metadata.file_type().is_symlink());
            if metadata.is_dir() {
                files.extend(inventory(&path));
            } else {
                files.insert(path.clone(), file_sha(&path));
            }
        }
        files
    }
    let workspace = PathBuf::from(
        std::env::var_os("HISTORICAL_INPUT_WORKSPACE").expect("explicit original workspace"),
    );
    let evidence = PathBuf::from(
        std::env::var_os("REVIEW_MEDIA_PREPARATION_EVIDENCE_DIR")
            .expect("explicit new evidence directory"),
    );
    assert!(workspace.is_absolute() && evidence.is_absolute());
    fs::create_dir(&evidence).unwrap();
    let original = workspace.join(".spec/artifacts/quality-review/sdk-text-results-r3");
    let before = inventory(&original);
    let package_sha = "659b3cb6757259d8e770f6d3c96c4a77dafbab43c6ea540bcc7e0c58e4eb0197";
    let body = fs::read(
        original
            .join("packages")
            .join(format!("{package_sha}.json")),
    )
    .unwrap();
    assert_eq!(digest(&body), package_sha);
    let package: ReviewPackage = serde_json::from_slice(&body).unwrap();
    assert_eq!(package.items.len(), 9);
    let mut ids = package
        .items
        .iter()
        .map(|item| item.item_id.as_str())
        .collect::<Vec<_>>();
    ids.sort();
    assert_eq!(
        ids,
        vec![
            "gemma270-cards",
            "gemma270-correction",
            "gemma270-highlights",
            "llama1b-cards",
            "llama1b-correction",
            "llama1b-highlights",
            "qwen06-cards",
            "qwen06-correction",
            "qwen06-highlights"
        ]
    );
    let media = workspace.join("runtime/ai-spike/source/Sintel.2010.720p.mkv");
    let media_sha = "f12c070e295b38cfc94ebd61ac3357c3bac82d1015985f1e8da93a4c6496c46d";
    assert_eq!(file_sha(&media), media_sha);
    assert_eq!(fs::metadata(&media).unwrap().len(), 681285280);
    let destination = evidence.join("prepared");
    fs::create_dir(&destination).unwrap();
    let source = ReviewFiles::open(&original).unwrap();
    let mut receipts = Vec::new();
    for item in &package.items {
        assert!(item.range.is_none());
        let case = ReviewCase {
            package_sha256: package_sha.into(),
            item_id: item.item_id.clone(),
            source_request_sha256: item.source.sha256.clone(),
        };
        let asset = EvidenceAsset {
            asset_id: "explicit-sintel-source".into(),
            sha256: media_sha.into(),
            bytes: DecimalU64(681285280),
            media_type: "video/x-matroska".into(),
        };
        let prepared = prepare_local_media(
            &source,
            &case,
            LocalMediaSelection {
                path: media.clone(),
                asset,
            },
            TimelinePackageDestination {
                root: destination.clone(),
                max_total_bytes: 1_073_741_824,
                excluded_roots: vec![original.clone()],
            },
            &Control,
        )
        .unwrap();
        assert!(!prepared.replayed());
        let reopened = ReviewFiles::open(prepared.root()).unwrap();
        let session = ReviewStore::open(&reopened, &prepared.case().package_sha256).unwrap();
        let mut restored = session.package().clone();
        let selected = restored
            .items
            .iter_mut()
            .find(|candidate| candidate.item_id == item.item_id)
            .unwrap();
        let check = selected.candidate.automated_checks.pop().unwrap();
        assert!(matches!(
            check.outcome,
            kinekino_player::domain::model_review::CheckOutcome::NotRun
        ));
        assert_eq!(
            serde_json::to_value(&restored).unwrap(),
            serde_json::to_value(&package).unwrap()
        );
        let relation = prepared.relation();
        assert_eq!(relation.base_case, case);
        assert_eq!(
            relation.original_candidate_sha256,
            digest(&serde_json::to_vec(&item.candidate).unwrap())
        );
        assert!(
            !relation.historical_media_verified
                && !relation.timeline_verified
                && !relation.quality_approved
        );
        assert!(
            fs::read_dir(destination.join("decisions"))
                .unwrap()
                .next()
                .is_none()
        );
        receipts.push(json!({"case":prepared.case(),"relation":relation,"historical_media_verified":false,"timeline_verified":false,"quality_approved":false}));
        println!(
            "actual9 prepared {} original candidate/range preserved",
            item.item_id
        );
    }
    assert_eq!(inventory(&original), before);
    assert_eq!(file_sha(&media), media_sha);
    // One physical media object is shared by nine immutable packages, never nine media copies.
    assert_eq!(
        fs::metadata(destination.join("objects").join(media_sha))
            .unwrap()
            .len(),
        681285280
    );
    fs::write(evidence.join("actual9-prepared.json"), serde_json::to_vec_pretty(&json!({"schema_version":"actual9-media-preparation-evidence/1","original_package_sha256":package_sha,"items":receipts,"original_inventory":before,"user_quality_approved":false,"gui_accepted":false})).unwrap()).unwrap();
}
