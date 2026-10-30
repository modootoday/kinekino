use super::*;
use crate::application::catalog::CatalogParser;
use crate::application::catalog::{CatalogApi, Category, SearchCatalog};
use kinekino_core::source::document::{
    CatalogError, CatalogFetchPort, CatalogPage, Entry, PageLocation, Provenance, RawDocument,
};

/// Storage-focused stub: real page parsing lives in the source plugin repository.
struct StubParser;
impl CatalogParser for StubParser {
    fn revision(&self) -> &str {
        "stub-v1"
    }
    fn parse(&self, raw: &RawDocument) -> Result<CatalogPage, CatalogError> {
        let location = PageLocation::parse(&raw.provenance.final_url)?;
        let html = String::from_utf8_lossy(&raw.body).into_owned();
        self.parse_html(&location, &html, raw.provenance.clone())
    }
    fn parse_html(
        &self,
        location: &PageLocation,
        _html: &str,
        provenance: Provenance,
    ) -> Result<CatalogPage, CatalogError> {
        let one = location.resolve("/c/one")?;
        Ok(CatalogPage {
            title: "Fixture".into(),
            series: None,
            entries: vec![Entry {
                id: one.stable_id(),
                title: "one".into(),
                url: one.as_str().into(),
            }],
            next_page: None,
            provenance,
        })
    }
}

struct Fetch;
impl CatalogFetchPort for Fetch {
    fn fetch(&self, location: &PageLocation) -> Result<RawDocument, CatalogError> {
        let body =
            br#"<h1>Fixture</h1><div class="wr-subject"><a href="/c/one">one</a></div>"#.to_vec();
        Ok(RawDocument {
            status: 200,
            content_type: "text/html; charset=utf-8".into(),
            content_encoding: None,
            provenance: Provenance {
                requested_url: location.as_str().into(),
                final_url: location.as_str().into(),
                redirects: vec![],
                sha256: format!("{:x}", Sha256::digest(&body)),
                bytes: body.len(),
                observed_unix_ms: 1_000,
                parser_revision: "stub-v1".into(),
            },
            body,
        })
    }
}
#[test]
fn synthetic_sdk_captures_exact_files_reopens_and_rejects_corruption() {
    kinekino_source_host::plugin_host::install_fixture_rules();
    let root = std::env::temp_dir().join(format!(
        "catalog-files-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    {
        let artifacts = Arc::new(CatalogFiles::open(&root).unwrap());
        let service = CatalogService::new(Arc::new(Fetch), Arc::new(StubParser), artifacts);
        let result = service
            .search(SearchCatalog {
                category: Category::Airing,
                page: 1,
            })
            .unwrap();
        assert_eq!(result.entries[0].id, "catalog:/c/one");
        assert!(!result.site_verified);
        let body_path = root.join(&result.raw_artifact.body_path);
        let body = fs::read(&body_path).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&body)),
            result.raw_artifact.body_sha256
        );
        let manifest = fs::read(root.join(&result.raw_artifact.metadata_path)).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&manifest)),
            result.raw_artifact.artifact_id
        );
        assert_eq!(
            serde_json::from_slice::<CaptureMetadata>(&manifest)
                .unwrap()
                .observed_at,
            "1970-01-01T00:00:01Z"
        );
        // Same immutable observation reuses exact files after a fresh storage instance.
        let reopened = CatalogFiles::open(&root).unwrap();
        assert_eq!(
            reopened.save(&body, &result.capture).unwrap().artifact_id,
            result.raw_artifact.artifact_id
        );
        fs::write(&body_path, b"corrupt").unwrap();
        assert_eq!(
            reopened.save(&body, &result.capture).unwrap_err().code,
            ErrorCode::SourceExpired
        );
        assert_eq!(fs::read(&body_path).unwrap(), b"corrupt");
    }
    fs::remove_dir_all(&root).unwrap();
}
