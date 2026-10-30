//! Mock ports test admission ordering and lifetime only; real bytes use the separate actual test.
use super::*;
use crate::{
    application::model_review::{ReviewSession, ReviewTextAsset, StoredDecision},
    domain::model_review::{ReviewDecision, ReviewPackage},
    infrastructure::review_content_classification::IsoBmffReviewClassifier,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
struct Store {
    package: ReviewPackage,
    bytes: Vec<u8>,
    live: Arc<AtomicUsize>,
}
struct Session {
    package: ReviewPackage,
    bytes: Vec<u8>,
    live: Arc<AtomicUsize>,
}
impl Drop for Session {
    fn drop(&mut self) {
        self.live.fetch_sub(1, Ordering::SeqCst);
    }
}
impl ReviewStore for Store {
    fn open(&self, _sha: &str) -> Result<Box<dyn ReviewSession>, ApiError> {
        self.live.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(Session {
            package: self.package.clone(),
            bytes: self.bytes.clone(),
            live: self.live.clone(),
        }))
    }
}
impl ReviewSession for Session {
    fn package(&self) -> &ReviewPackage {
        &self.package
    }
    fn read_text_asset(&self, _sha: &str) -> Result<ReviewTextAsset, ApiError> {
        Err(invalid())
    }
    fn append(&self, _decision: &ReviewDecision) -> Result<StoredDecision, ApiError> {
        Err(invalid())
    }
    fn decisions(&self, _item: &str) -> Result<Vec<StoredDecision>, ApiError> {
        Err(invalid())
    }
    fn read_evidence_prefix(&self, _id: &str, _sha: &str, max: u64) -> Result<Vec<u8>, ApiError> {
        Ok(self.bytes[..self.bytes.len().min(max as usize)].to_vec())
    }
}
fn fixture(
    bytes: Vec<u8>,
    mime: &str,
) -> (
    ReviewCaseEvidence<Store>,
    ReviewCase,
    MediaReference,
    Arc<AtomicUsize>,
) {
    let package=serde_json::from_value(serde_json::json!({"schema_version":"1","package_id":"fixture","items":[{"item_id":"item","task":"media_timeline","source":{"asset_id":"request","sha256":"c".repeat(64),"bytes":"1","media_type":"application/json"},"range":null,"baseline":null,"candidate":{"condition":{"model_id":"fixture","revision":"r1","precision":"fp32","engine_id":"fixture","engine_revision":"r1","input_conditions":{"sample":"synthetic"}},"assets":[{"asset_id":"media","sha256":"b".repeat(64),"bytes":bytes.len().to_string(),"media_type":mime}],"automated_checks":[{"check_id":"quality","outcome":"not_run","summary":"Synthetic only","evidence":[]}],"measured_resources":{"kind":"synthetic"}},"criteria":[{"criterion_id":"compare","how_to_review":"Compare source","expected":"Faithful"}],"known_issues":[]}]})).unwrap();
    let live = Arc::new(AtomicUsize::new(0));
    (
        ReviewCaseEvidence::new(Store {
            package,
            bytes,
            live: live.clone(),
        }),
        ReviewCase {
            package_sha256: "a".repeat(64),
            item_id: "item".into(),
            source_request_sha256: "c".repeat(64),
        },
        MediaReference {
            asset_id: "media".into(),
            sha256: "b".repeat(64),
        },
        live,
    )
}
fn mp4_header() -> Vec<u8> {
    let mut value = 24u32.to_be_bytes().to_vec();
    value.extend_from_slice(b"ftypisom\0\0\0\0isommp41");
    value
}
struct Runtime {
    calls: AtomicUsize,
    live: Arc<AtomicUsize>,
    revoked: Arc<AtomicBool>,
    source_limit: u64,
}
struct Permit {
    profile: ReviewRuntimeProfile,
    live: Arc<AtomicUsize>,
    revoked: Arc<AtomicBool>,
}
impl Drop for Permit {
    fn drop(&mut self) {
        self.live.fetch_sub(1, Ordering::SeqCst);
    }
}
impl ReviewRuntimeLease for Permit {
    fn profile(&self) -> &ReviewRuntimeProfile {
        &self.profile
    }
    fn check_current(&self) -> Result<(), ApiError> {
        if self.revoked.load(Ordering::SeqCst) {
            Err(ApiError::new(ErrorCode::Cancelled, "fixture revoked"))
        } else {
            Ok(())
        }
    }
}
impl ReviewRuntimeAdmission for Runtime {
    fn reserve(
        &self,
        _input: &ReviewInputIdentity,
    ) -> Result<Box<dyn ReviewRuntimeLease + '_>, ApiError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.live.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(Permit {
            profile: ReviewRuntimeProfile {
                backend: ReviewRuntimeBackend::OwnedLocalMpv,
                engine_sha256: "e".repeat(64),
                max_source_bytes: self.source_limit,
                max_job_committed_bytes: 64 * 1024 * 1024,
                cpu_threads: 2,
                startup_timeout_ms: 30000,
                child_ownership: ReviewChildOwnership::OwnedJobUntilPhysicalExit,
            },
            live: self.live.clone(),
            revoked: self.revoked.clone(),
        }))
    }
}
fn runtime(limit: u64) -> Runtime {
    Runtime {
        calls: AtomicUsize::new(0),
        live: Arc::new(AtomicUsize::new(0)),
        revoked: Arc::new(AtomicBool::new(false)),
        source_limit: limit,
    }
}
#[test]
fn classified_input_and_explicit_runtime_leases_survive_until_release() {
    let (reader, case, media, live) = fixture(mp4_header(), "video/mp4");
    let runtime = runtime(1000);
    let admitted =
        prepare_review_input(&reader, &case, &media, &IsoBmffReviewClassifier, &runtime).unwrap();
    assert_eq!(
        admitted.identity().classification,
        ReviewContentClass::IsoBmffHeader
    );
    assert_eq!(live.load(Ordering::SeqCst), 1);
    assert_eq!(runtime.live.load(Ordering::SeqCst), 1);
    runtime.revoked.store(true, Ordering::SeqCst);
    assert!(admitted.check_current().is_err());
    drop(admitted);
    assert_eq!(live.load(Ordering::SeqCst), 0);
    assert_eq!(runtime.live.load(Ordering::SeqCst), 0);
}
#[test]
fn raw_bytes_and_declared_mime_are_independent_gates_before_runtime() {
    for (bytes, mime) in [
        (b"not an ISO BMFF source".to_vec(), "video/mp4"),
        (mp4_header(), "image/png"),
    ] {
        let (reader, case, media, live) = fixture(bytes, mime);
        let runtime = runtime(1000);
        assert!(
            prepare_review_input(&reader, &case, &media, &IsoBmffReviewClassifier, &runtime)
                .is_err()
        );
        assert_eq!(runtime.calls.load(Ordering::SeqCst), 0);
        assert_eq!(live.load(Ordering::SeqCst), 0);
    }
}
#[test]
fn matroska_header_and_mime_match_before_any_runtime_grant() {
    use crate::infrastructure::review_content_classification::LocalReviewContentClassifier;
    let mkv = b"\x1a\x45\xdf\xa3\x8b\x42\x82\x88matroska".to_vec();
    for (bytes, mime) in [
        (mkv.clone(), "video/mp4"),
        (mp4_header(), "video/x-matroska"),
    ] {
        let (reader, case, media, live) = fixture(bytes, mime);
        let runtime = runtime(1000);
        assert!(
            prepare_review_input(
                &reader,
                &case,
                &media,
                &LocalReviewContentClassifier,
                &runtime
            )
            .is_err()
        );
        assert_eq!(runtime.calls.load(Ordering::SeqCst), 0);
        assert_eq!(live.load(Ordering::SeqCst), 0);
    }
    let (reader, case, media, live) = fixture(mkv, "video/x-matroska");
    let runtime = runtime(1000);
    let admitted = prepare_review_input(
        &reader,
        &case,
        &media,
        &LocalReviewContentClassifier,
        &runtime,
    )
    .unwrap();
    assert_eq!(
        admitted.identity().classification,
        ReviewContentClass::MatroskaHeader
    );
    assert_eq!(runtime.calls.load(Ordering::SeqCst), 1);
    assert_eq!(live.load(Ordering::SeqCst), 1);
    drop(admitted);
    assert_eq!(live.load(Ordering::SeqCst), 0);
    assert_eq!(runtime.live.load(Ordering::SeqCst), 0);
}
#[test]
fn absent_runtime_or_insufficient_grant_never_becomes_ready_by_default() {
    let (reader, case, media, live) = fixture(mp4_header(), "video/mp4");
    assert!(
        matches!(prepare_review_input(&reader,&case,&media,&IsoBmffReviewClassifier,&UnavailableReviewRuntime),Err(e) if e.code==ErrorCode::Unsupported)
    );
    assert_eq!(live.load(Ordering::SeqCst), 0);
    let runtime = runtime(1);
    assert!(
        matches!(prepare_review_input(&reader,&case,&media,&IsoBmffReviewClassifier,&runtime),Err(e) if e.code==ErrorCode::ResourceExhausted)
    );
    assert_eq!(runtime.live.load(Ordering::SeqCst), 0);
    assert_eq!(live.load(Ordering::SeqCst), 0);
}
