//! Fake plugin for end-to-end tests (`fixture` feature), written only against the SDK's
//! public [`SourcePlugin`] API. Synthetic data only.
use crate::document::{CatalogError, ErrorKind};
use crate::serve::{SourcePlugin, handle_line};
use crate::wire::document::{DocRequest, DocResponse};
use crate::wire::source::{Item, Ref, Response};

/// Serves one page per browse; everything else is `unsupported`. `document/1` reports a
/// fixed parser revision only.
pub struct FixturePlugin {
    pub provider: String,
}

impl SourcePlugin for FixturePlugin {
    fn browse(&mut self, category: &Ref, page: u32) -> Response {
        Response::Page {
            items: vec![Item {
                reference: Ref {
                    provider: self.provider.clone(),
                    id: format!("{}-{page}", category.id),
                },
                title: format!("Fixture {} page {page}", category.id),
                kind: None,
            }],
            next_page: Some(page + 1),
        }
    }

    fn document(&mut self, request: DocRequest) -> DocResponse {
        match request {
            DocRequest::Revision => DocResponse::Revision("fixture-parser/1".into()),
            _ => DocResponse::Error(CatalogError::new(
                ErrorKind::UnsupportedContent,
                "fixture document plugin serves revision only",
            )),
        }
    }
}

/// `source/1` line → response line for the fixture plugin.
pub fn serve_fixture(input: &str, provider: &str) -> String {
    handle_line(
        &mut FixturePlugin {
            provider: provider.into(),
        },
        input,
    )
}

/// `document/1` line (with or without the protocol field) → response line.
pub fn serve_document_fixture(input: &str) -> String {
    let mut plugin = FixturePlugin {
        provider: String::new(),
    };
    match serde_json::from_str::<DocRequest>(input.trim()) {
        Ok(request) => serde_json::to_string(&plugin.document(request)).unwrap_or_default(),
        Err(e) => serde_json::to_string(&DocResponse::Error(CatalogError::new(
            ErrorKind::UnsupportedContent,
            e.to_string(),
        )))
        .unwrap_or_default(),
    }
}
