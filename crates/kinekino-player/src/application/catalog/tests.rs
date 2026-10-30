use super::*;
use kinekino_core::source::document::{CatalogFetchPort, ErrorKind, Provenance};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

struct Fetch {
    corrupt: bool,
    fail: bool,
}
impl CatalogFetchPort for Fetch {
    fn fetch(&self, location: &PageLocation) -> Result<RawDocument, CatalogError> {
        if self.fail {
            return Err(CatalogError::new(ErrorKind::Transport, "connection failed"));
        }
        let body = b"exact challenge bytes".to_vec();
        Ok(RawDocument {
            status: 403,
            content_type: "text/html".into(),
            content_encoding: None,
            provenance: Provenance {
                requested_url: location.as_str().into(),
                final_url: location.as_str().into(),
                redirects: vec![],
                sha256: if self.corrupt {
                    "bad".into()
                } else {
                    format!("{:x}", Sha256::digest(&body))
                },
                bytes: body.len(),
                observed_unix_ms: 1_000,
                parser_revision: "fixture".into(),
            },
            body,
        })
    }
}
#[derive(Default)]
struct Artifacts(Mutex<Vec<Vec<u8>>>);
impl CatalogArtifacts for Artifacts {
    fn save(&self, raw: &[u8], metadata: &CaptureMetadata) -> Result<RawArtifact, ApiError> {
        self.0.lock().unwrap().push(raw.to_vec());
        assert_eq!(metadata.observed_at, "1970-01-01T00:00:01Z");
        Ok(RawArtifact {
            artifact_id: "fixture".into(),
            body_sha256: metadata.body_sha256.clone(),
            body_bytes: metadata.body_bytes,
            body_path: "objects/fixture.body".into(),
            metadata_path: "captures/fixture.json".into(),
        })
    }
}
struct Parser(Arc<Artifacts>);
impl CatalogParser for Parser {
    fn parse(&self, _: &RawDocument) -> Result<CatalogPage, CatalogError> {
        assert_eq!(
            self.0.0.lock().unwrap().as_slice(),
            [b"exact challenge bytes".to_vec()]
        );
        Err(CatalogError::new(ErrorKind::AccessRequired, "challenge"))
    }
}
fn service(corrupt: bool, fail: bool) -> (CatalogService, Arc<Artifacts>) {
    let artifacts = Arc::new(Artifacts::default());
    (
        CatalogService::new(
            Arc::new(Fetch { corrupt, fail }),
            Arc::new(Parser(artifacts.clone())),
            artifacts.clone(),
        ),
        artifacts,
    )
}
#[test]
fn complete_error_response_is_saved_before_parser_and_returned_as_reference() {
    let (service, artifacts) = service(false, false);
    let error = service
        .search(SearchCatalog {
            category: Category::Airing,
            page: 1,
        })
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::NeedsInput);
    assert_eq!(error.details["raw_artifact"]["artifact_id"], "fixture");
    assert_eq!(error.details["capture"]["status"], 403);
    assert_eq!(artifacts.0.lock().unwrap().len(), 1);
}
#[test]
fn long_source_metadata_keeps_raw_reference_within_durable_error_budget() {
    let (service, _) = service(false, false);
    let error = service
        .episodes(ListEpisodes {
            series_url: format!("https://catalog.example.test/c/{}", "x".repeat(7000)),
        })
        .unwrap_err();
    assert_eq!(error.details["raw_artifact"]["artifact_id"], "fixture");
    assert_eq!(error.details["capture_metadata_in_artifact"], true);
    assert!(serde_json::to_vec(&error.details).unwrap().len() < 16384);
}
#[test]
fn incomplete_transport_or_tampered_identity_never_creates_fake_artifact() {
    for (corrupt, fail) in [(false, true), (true, false)] {
        let (service, artifacts) = service(corrupt, fail);
        let error = service
            .search(SearchCatalog {
                category: Category::Airing,
                page: 1,
            })
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::EngineFailed);
        assert!(artifacts.0.lock().unwrap().is_empty());
        assert!(error.details["raw_artifact"].is_null());
    }
}
#[test]
fn catalog_default_unavailable_and_explicit_instance_experimental() {
    use crate::application::api::{PlayerSdk, Request};
    let default = PlayerSdk::default().execute(Request::new("catalog.search"));
    assert_eq!(default.error.unwrap().code, ErrorCode::Unsupported);
    let (service, _) = service(false, false);
    let sdk = PlayerSdk::default().with_catalog(service);
    let caps = sdk.execute(Request::new("capabilities")).data.unwrap();
    assert_eq!(caps["catalog"]["site_verified"], false);
    assert!(
        caps["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["command"] == "catalog.search" && v["status"] == "experimental")
    );
    for payload in [
        serde_json::json!({"category":"airing","query":"x"}),
        serde_json::json!({"category":"airing","genres":["x"]}),
        serde_json::json!({"category":"airing","page":0}),
    ] {
        let mut request = Request::new("catalog.search");
        request.payload = payload;
        assert_eq!(
            sdk.execute(request).error.unwrap().code,
            ErrorCode::InvalidArgument
        );
    }
    let mut request = Request::new("episodes.list");
    request.payload = serde_json::json!({"series_url":"https://catalog.example.test/e/a"});
    assert_eq!(
        sdk.execute(request).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
}
