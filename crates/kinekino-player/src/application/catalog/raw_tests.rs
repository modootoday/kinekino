use super::*;
use sha2::{Digest, Sha256};

struct Source {
    bytes: Vec<u8>,
}
impl CatalogRawSource for Source {
    fn load_raw(&self, artifact_id: &str) -> Result<LoadedCatalogRaw, ApiError> {
        let sha = format!("{:x}", Sha256::digest(&self.bytes));
        let reference = RawArtifact {
            artifact_id: artifact_id.into(),
            body_sha256: sha.clone(),
            body_bytes: DecimalU64(self.bytes.len() as u64),
            body_path: format!("objects/{sha}.body"),
            metadata_path: format!("captures/{artifact_id}.json"),
        };
        let capture = CaptureMetadata {
            requested_url: Category::Airing.location().unwrap().as_str().into(),
            final_url: Category::Airing.location().unwrap().as_str().into(),
            redirects: vec![],
            observed_at: "1970-01-01T00:00:01Z".into(),
            parser_revision: "fixture".into(),
            capture_kind: CatalogCaptureKind::HttpEntity,
            status: Some(200),
            content_type: "text/html".into(),
            content_encoding: None,
            body_sha256: sha,
            body_bytes: reference.body_bytes,
            browser: None,
        };
        Ok(LoadedCatalogRaw {
            body: self.bytes.clone(),
            reference,
            capture,
        })
    }
}
fn input(offset: u64, limit: usize, encoding: RawEncoding) -> ReadCatalogRaw {
    ReadCatalogRaw {
        artifact_id: "0".repeat(64),
        offset: DecimalU64(offset),
        limit,
        encoding,
    }
}
#[test]
fn raw_utf8_chunks_preserve_multibyte_boundaries_and_explicit_eof() {
    let source = Source {
        bytes: "가나다".as_bytes().to_vec(),
    };
    let first = super::raw::read(&source, input(0, 4, RawEncoding::Utf8)).unwrap();
    assert_eq!(first.text, "가");
    assert_eq!(first.next_offset.unwrap().0, 3);
    assert_eq!(first.chunk_bytes.0, 3);
    assert_eq!(
        super::raw::read(&source, input(1, 4, RawEncoding::Utf8))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        super::raw::read(&source, input(0, 2, RawEncoding::Utf8))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    let eof = super::raw::read(&source, input(9, 4, RawEncoding::Utf8)).unwrap();
    assert!(eof.text.is_empty());
    assert!(eof.next_offset.is_none());
}
#[test]
fn non_utf8_requires_explicit_hex_without_normalizing_original_bytes() {
    let source = Source {
        bytes: vec![0xff, 0, 0x42],
    };
    assert_eq!(
        super::raw::read(&source, input(0, 3, RawEncoding::Utf8))
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    let hex = super::raw::read(&source, input(0, 3, RawEncoding::Hex)).unwrap();
    assert_eq!(hex.text, "ff0042");
    assert_eq!(hex.chunk_bytes.0, 3);
    assert!(hex.next_offset.is_none());
}
