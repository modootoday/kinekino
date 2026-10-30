#![cfg(windows)]
use kinekino_player::application::api::{AssetClient, ReviewClient};
#[path = "support/cli_exe.rs"]
mod cli_exe;
use kinekino_player::{
    application::{
        api::{ApplicationClientHandle, ErrorCode, PlayerSdk, Request, RequestContext},
        model_review::{InspectReviewPackage, ListReviewDecisions, ReviewStore},
    },
    domain::{
        api_values::DecimalU64,
        model_review::{CheckOutcome, RecordReviewDecision, ReviewPackage, ReviewVerdict},
    },
    infrastructure::{content_digest, model_review::ReviewFiles},
};
use serde_json::{Value, json};
#[path = "../src/test_support/review_package.rs"]
mod review_package;
use std::{
    fs,
    io::{Cursor, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
struct Fixture {
    root: PathBuf,
    sha: String,
    package: ReviewPackage,
}
fn hash(bytes: &[u8]) -> String {
    content_digest::sha256(&mut Cursor::new(bytes), 4_294_967_296)
        .unwrap()
        .1
}
#[test]
fn bounded_text_reads_match_actual_cli_and_reject_unreferenced_or_invalid_evidence() {
    use kinekino_player::application::model_review::ReadReviewTextAsset;
    let mut fixture = Fixture::new();
    let input = ReadReviewTextAsset {
        package_sha256: fixture.sha.clone(),
        asset_sha256: fixture.package.items[0].source.sha256.clone(),
    };
    let result = fixture
        .sdk()
        .read_review_text_asset(RequestContext::default(), input.clone())
        .unwrap();
    assert_eq!(result.utf8, "Original fixed cue");
    assert_eq!(result.bytes.0, result.utf8.len() as u64);
    let mut request = Request::new("reviews.read_text_asset");
    request.payload = serde_json::to_value(&input).unwrap();
    let (code, response) = fixture.invoke(&request);
    assert_eq!(code, 0);
    assert_eq!(response["data"], serde_json::to_value(result).unwrap());
    assert_eq!(
        PlayerSdk::default()
            .execute(request.clone())
            .error
            .unwrap()
            .code,
        ErrorCode::Unsupported
    );
    request.payload["asset_sha256"] = json!("../escape");
    assert_eq!(
        fixture.sdk().execute(request.clone()).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
    let unreferenced = b"An unreferenced object";
    let unrelated_sha = hash(unreferenced);
    fs::write(
        fixture.root.join("objects").join(&unrelated_sha),
        unreferenced,
    )
    .unwrap();
    request.payload["asset_sha256"] = json!(unrelated_sha);
    assert_eq!(
        fixture.sdk().execute(request.clone()).error.unwrap().code,
        ErrorCode::NotFound
    );
    request.payload["asset_sha256"] = json!(input.asset_sha256);
    request.payload["path"] = json!("outside.txt");
    assert_eq!(
        fixture.sdk().execute(request.clone()).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
    for (body, media_type, expected) in [
        (
            vec![b'x'; 65_537],
            "text/plain",
            Some(ErrorCode::Unsupported),
        ),
        (vec![0xff, 0xfe], "text/plain", Some(ErrorCode::NeedsInput)),
        (
            b"Not a video".to_vec(),
            "video/mp4",
            Some(ErrorCode::Unsupported),
        ),
        (
            "한국어\n💬".as_bytes().to_vec(),
            "text/plain; charset=utf-8",
            None,
        ),
        (b"{truncated JSON".to_vec(), "application/json", None),
        (vec![b'x'; 65_536], "text/plain", None),
    ] {
        let asset = &mut fixture.package.items[0].source;
        asset.sha256 = hash(&body);
        asset.bytes = DecimalU64(body.len() as u64);
        asset.media_type = media_type.into();
        fs::write(fixture.root.join("objects").join(&asset.sha256), &body).unwrap();
        fixture.publish_package();
        request.payload = json!({"package_sha256":fixture.sha,"asset_sha256":fixture.package.items[0].source.sha256});
        let response = fixture.sdk().execute(request.clone());
        if let Some(expected) = expected {
            assert_eq!(response.error.unwrap().code, expected);
        } else {
            assert!(response.ok);
            assert_eq!(
                response.data.unwrap()["utf8"],
                String::from_utf8(body).unwrap()
            );
        }
    }
    fs::write(
        fixture
            .root
            .join("objects")
            .join(&fixture.package.items[0].source.sha256),
        b"tampered",
    )
    .unwrap();
    assert_eq!(
        fixture.sdk().execute(request).error.unwrap().code,
        ErrorCode::RevisionConflict
    );
}
impl Fixture {
    fn new() -> Self {
        kinekino_platform::install();
        // Workspace root without `..` components: the store rejects parent-dir roots.
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .join("runtime/review-sdk-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        for folder in ["packages", "objects", "decisions"] {
            fs::create_dir_all(root.join(folder)).unwrap();
        }
        let mut package = review_package::package();
        let source = review_package::SOURCE;
        let baseline = review_package::BASELINE;
        let candidate = review_package::CANDIDATE;
        let item = &mut package.items[0];
        for (asset, bytes) in [
            (&mut item.source, source),
            (&mut item.baseline.as_mut().unwrap().assets[0], baseline),
            (&mut item.candidate.assets[0], candidate),
        ] {
            asset.sha256 = hash(bytes);
            asset.bytes = DecimalU64(bytes.len() as u64);
            asset.media_type = "text/plain".into();
            fs::write(root.join("objects").join(&asset.sha256), bytes).unwrap();
        }
        item.candidate.automated_checks[0].outcome = CheckOutcome::Passed;
        item.known_issues.clear();
        let mut fixture = Self {
            root,
            sha: String::new(),
            package,
        };
        fixture.publish_package();
        fixture
    }
    fn publish_package(&mut self) {
        let bytes = serde_json::to_vec_pretty(&self.package).unwrap();
        self.sha = hash(&bytes);
        fs::write(
            self.root
                .join("packages")
                .join(format!("{}.json", self.sha)),
            bytes,
        )
        .unwrap();
    }
    fn decision(&self, id: &str, verdict: ReviewVerdict) -> RecordReviewDecision {
        RecordReviewDecision {
            package_sha256: self.sha.clone(),
            item_id: "sample-001".into(),
            decision_id: id.into(),
            verdict,
            note: "Synthetic test observation only; not user/model approval.".into(),
            issue_ranges: vec![],
        }
    }
    fn sdk(&self) -> ApplicationClientHandle {
        Arc::new(PlayerSdk::default().with_review_store(ReviewFiles::open(&self.root).unwrap()))
    }
    fn invoke(&self, request: &Request) -> (i32, Value) {
        let mut child = Command::new(crate::cli_exe::kinekino_cli_exe())
            .args(["--review-root", self.root.to_str().unwrap(), "--input", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&serde_json::to_vec(request).unwrap())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.stderr.is_empty());
        (
            output.status.code().unwrap(),
            serde_json::from_slice(&output.stdout).unwrap(),
        )
    }
}
#[test]
fn actual_cli_inspects_records_and_reads_same_immutable_decisions_as_shared_sdk() {
    let fixture = Fixture::new();
    let sdk = fixture.sdk();
    let view = sdk
        .inspect_review(
            RequestContext::default(),
            InspectReviewPackage {
                package_sha256: fixture.sha.clone(),
            },
        )
        .unwrap();
    assert!(view.evidence_verified && view.review_ready);
    assert!(!view.automatic_application_enabled);
    let mut request = Request::new("reviews.inspect");
    request.context.request_id = "review-요청".into();
    request.payload = json!({"package_sha256":fixture.sha});
    let (code, remote) = fixture.invoke(&request);
    assert_eq!(code, 0);
    assert_eq!(remote, serde_json::to_value(sdk.execute(request)).unwrap());
    for (id, verdict, passed) in [
        ("review-a", ReviewVerdict::Deferred, false),
        ("review-b", ReviewVerdict::NeedsChanges, false),
        ("review-c", ReviewVerdict::Usable, true),
    ] {
        let input = fixture.decision(id, verdict);
        let mut request = Request::new("reviews.record_decision");
        request.payload = serde_json::to_value(&input).unwrap();
        let (code, result) = fixture.invoke(&request);
        assert_eq!(code, 0, "{result}");
        assert_eq!(result["data"]["decision"]["sample_review_passed"], passed);
        assert_eq!(
            result["data"]["decision"]["automatic_application_enabled"],
            false
        );
        assert_eq!(
            sdk.record_review_decision(RequestContext::default(), input)
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
    }
    let rows = sdk
        .review_decisions(
            RequestContext::default(),
            ListReviewDecisions {
                package_sha256: fixture.sha.clone(),
                item_id: "sample-001".into(),
            },
        )
        .unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].decision.verdict, ReviewVerdict::Deferred);
    assert!(!rows[0].decision.sample_review_passed);
    let mut list = Request::new("reviews.decisions");
    list.payload = json!({"package_sha256":fixture.sha,"item_id":"sample-001"});
    assert_eq!(
        fixture.invoke(&list).1,
        serde_json::to_value(sdk.execute(list)).unwrap()
    );
}
#[test]
fn deferred_missing_baseline_or_failed_checks_never_pass_and_conditions_are_pinned() {
    let mut fixture = Fixture::new();
    let baseline = fixture.package.items[0].baseline.clone();
    fixture.package.items[0].baseline = None;
    fixture.publish_package();
    let sdk = fixture.sdk();
    let view = sdk
        .inspect_review(
            RequestContext::default(),
            InspectReviewPackage {
                package_sha256: fixture.sha.clone(),
            },
        )
        .unwrap();
    assert!(!view.review_ready);
    let record = sdk
        .record_review_decision(
            RequestContext::default(),
            fixture.decision("not-ready", ReviewVerdict::Usable),
        )
        .unwrap();
    assert!(!record.decision.sample_review_passed && !record.decision.review_ready);
    drop(sdk);
    fixture.package.items[0].baseline = baseline;
    fixture.package.items[0].candidate.automated_checks[0].outcome = CheckOutcome::Failed;
    fixture.publish_package();
    let sdk = fixture.sdk();
    let record = sdk
        .record_review_decision(
            RequestContext::default(),
            fixture.decision("failed-check", ReviewVerdict::Usable),
        )
        .unwrap();
    assert!(
        record.decision.review_ready
            && !record.decision.automated_prerequisites_passed
            && !record.decision.sample_review_passed
    );
    drop(sdk);
    fixture.package.items[0].candidate.condition.revision = "main".into();
    fixture.publish_package();
    let sdk = fixture.sdk();
    assert_eq!(
        sdk.inspect_review(
            RequestContext::default(),
            InspectReviewPackage {
                package_sha256: fixture.sha.clone()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidArgument
    );
}

#[test]
fn non_temporal_issues_and_missing_review_methods_are_rejected() {
    let mut fixture = Fixture::new();
    let sdk = fixture.sdk();
    let mut input = fixture.decision("bad-range", ReviewVerdict::Deferred);
    input
        .issue_ranges
        .push(kinekino_player::domain::model_review::ReviewRange {
            start_us: DecimalU64(0),
            end_us: DecimalU64(1),
        });
    assert_eq!(
        sdk.record_review_decision(RequestContext::default(), input)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    drop(sdk);
    fixture.package.items[0].candidate.automated_checks[0].outcome = CheckOutcome::NotRun;
    fixture.publish_package();
    let sdk = fixture.sdk();
    let record = sdk
        .record_review_decision(
            RequestContext::default(),
            fixture.decision("not-run", ReviewVerdict::Usable),
        )
        .unwrap();
    assert!(record.decision.review_ready && !record.decision.sample_review_passed);
    drop(sdk);
    fixture.package.items[0].criteria[0].how_to_review.clear();
    fixture.publish_package();
    assert_eq!(
        fixture
            .sdk()
            .inspect_review(
                RequestContext::default(),
                InspectReviewPackage {
                    package_sha256: fixture.sha.clone()
                }
            )
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
}

#[test]
fn concurrent_same_identity_has_one_immutable_record_and_reproducible_digest() {
    let fixture = Fixture::new();
    let sdk = fixture.sdk();
    let input = fixture.decision("one-identity", ReviewVerdict::Deferred);
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let workers: Vec<_> = (0..2)
        .map(|_| {
            let sdk = sdk.clone();
            let barrier = barrier.clone();
            let input = input.clone();
            std::thread::spawn(move || {
                barrier.wait();
                sdk.record_review_decision(RequestContext::default(), input)
            })
        })
        .collect();
    let results: Vec<_> = workers.into_iter().map(|w| w.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    let rejected = results.iter().find_map(|r| r.as_ref().err()).unwrap();
    assert!(matches!(
        rejected.code,
        ErrorCode::RevisionConflict | ErrorCode::ResourceExhausted | ErrorCode::PermissionDenied
    ));
    let records = sdk
        .review_decisions(
            RequestContext::default(),
            ListReviewDecisions {
                package_sha256: fixture.sha.clone(),
                item_id: "sample-001".into(),
            },
        )
        .unwrap();
    assert_eq!(records.len(), 1);
    assert!(!records[0].decision.sample_review_passed);
    assert_eq!(
        hash(&serde_json::to_vec_pretty(&records[0].decision).unwrap()),
        records[0].decision_sha256
    );
}
#[test]
fn package_asset_and_decision_tampering_is_rejected_before_a_pass_can_be_read() {
    let fixture = Fixture::new();
    let sdk = fixture.sdk();
    let stored = sdk
        .record_review_decision(
            RequestContext::default(),
            fixture.decision("decision-1", ReviewVerdict::Usable),
        )
        .unwrap();
    let decision_file = fixture
        .root
        .join("decisions")
        .join(&fixture.sha)
        .join("i-73616d706c652d303031")
        .join("d-6465636973696f6e2d31")
        .join(format!("{}.json", stored.decision_sha256));
    let mut permissions = fs::metadata(&decision_file).unwrap().permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    {
        permissions.set_readonly(false);
    }
    fs::set_permissions(&decision_file, permissions).unwrap();
    let mut value: Value = serde_json::from_slice(&fs::read(&decision_file).unwrap()).unwrap();
    value["note"] = json!("tampered");
    fs::write(&decision_file, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    assert_eq!(
        sdk.review_decisions(
            RequestContext::default(),
            ListReviewDecisions {
                package_sha256: fixture.sha.clone(),
                item_id: "sample-001".into()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    fs::write(
        fixture
            .root
            .join("objects")
            .join(&fixture.package.items[0].source.sha256),
        b"tampered",
    )
    .unwrap();
    assert_eq!(
        sdk.inspect_review(
            RequestContext::default(),
            InspectReviewPackage {
                package_sha256: fixture.sha.clone()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
    fs::write(
        fixture
            .root
            .join("packages")
            .join(format!("{}.json", fixture.sha)),
        b"{}",
    )
    .unwrap();
    assert_eq!(
        sdk.inspect_review(
            RequestContext::default(),
            InspectReviewPackage {
                package_sha256: fixture.sha.clone()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::RevisionConflict
    );
}
#[test]
fn verified_session_holds_assets_against_writes_and_reparse_escape_is_denied() {
    let fixture = Fixture::new();
    let store = ReviewFiles::open(&fixture.root).unwrap();
    let session = ReviewStore::open(&store, &fixture.sha).unwrap();
    let object = fixture
        .root
        .join("objects")
        .join(&fixture.package.items[0].source.sha256);
    assert!(fs::OpenOptions::new().write(true).open(&object).is_err());
    drop(session);
    drop(store);
    fs::remove_file(&object).unwrap();
    let outside = fixture.root.join("outside.txt");
    fs::write(&outside, b"Original fixed cue").unwrap();
    std::os::windows::fs::symlink_file(&outside, &object).unwrap();
    let sdk = fixture.sdk();
    assert_eq!(
        sdk.inspect_review(
            RequestContext::default(),
            InspectReviewPackage {
                package_sha256: fixture.sha.clone()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::PermissionDenied
    );
    let mut request = Request::new("reviews.record_decision");
    let mut value =
        serde_json::to_value(fixture.decision("../escape", ReviewVerdict::Usable)).unwrap();
    request.payload = value.clone();
    assert_eq!(
        sdk.execute(request.clone()).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
    value["package_sha256"] = json!("../../private");
    request.payload = value;
    assert_eq!(
        sdk.execute(request).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
}
#[test]
fn absent_backend_schema_context_and_missing_material_are_explicit() {
    let fixture = Fixture::new();
    let mut request = Request::new("reviews.inspect");
    request.payload = json!({"package_sha256":fixture.sha});
    assert_eq!(
        PlayerSdk::default()
            .execute(request.clone())
            .error
            .unwrap()
            .code,
        ErrorCode::Unsupported
    );
    let sdk = fixture.sdk();
    let mut schema = Request::new("schema");
    schema.payload = json!({"command":"reviews.record_decision"});
    let data = sdk.execute(schema).data.unwrap();
    assert_eq!(data["status"], "supported");
    assert_eq!(
        data["request"]["properties"]["payload"]["properties"]["verdict"]["enum"],
        json!(["usable", "needs_changes", "deferred"])
    );
    assert!(data["review_package_schema"]["properties"]["items"].is_object());
    request.context.timeout_ms = Some(100);
    assert_eq!(
        sdk.execute(request).error.unwrap().code,
        ErrorCode::Unsupported
    );
    let error = sdk
        .inspect_review(
            RequestContext::default(),
            InspectReviewPackage {
                package_sha256: "f".repeat(64),
            },
        )
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::NotFound);
    assert_eq!(error.remediation, "prepare_review_package");
    assert!(!error.message.contains("approval"));
    let mut forged =
        serde_json::to_value(fixture.decision("forged", ReviewVerdict::Deferred)).unwrap();
    forged["sample_review_passed"] = json!(true);
    let mut request = Request::new("reviews.record_decision");
    request.payload = forged;
    assert_eq!(
        sdk.execute(request).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
}
