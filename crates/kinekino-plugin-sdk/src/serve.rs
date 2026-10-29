//! Plugin-side loop: read one request line, dispatch to [`SourcePlugin`], write one response
//! line. A line whose [`PROTOCOL_FIELD`] is [`DOCUMENT_PROTOCOL`] goes to `document`.
use crate::document::{CatalogError, ErrorKind};
use crate::wire::document::{DocRequest, DocResponse};
use crate::wire::source::{Ref, Request, Response};
use crate::{DOCUMENT_PROTOCOL, PROTOCOL_FIELD};
use std::io::{BufRead, Write};

/// `Error{kind:"unsupported"}` for a method the plugin does not offer.
pub fn unsupported(method: &str) -> Response {
    Response::Error {
        kind: "unsupported".into(),
        message: format!("{method} is not supported by this plugin"),
    }
}

/// One method per `source/1` capability; every default answers `unsupported`, so a plugin
/// implements only what its manifest declares.
pub trait SourcePlugin {
    fn browse(&mut self, _category: &Ref, _page: u32) -> Response {
        unsupported("browse")
    }
    fn search(&mut self, _query: &str, _page: u32) -> Response {
        unsupported("search")
    }
    fn series(&mut self, _series: &Ref) -> Response {
        unsupported("series")
    }
    fn episode(&mut self, _episode: &Ref) -> Response {
        unsupported("episode")
    }
    fn resolve(&mut self, _episode: &Ref) -> Response {
        unsupported("resolve")
    }
    /// `document/1`; plugins without a `document` manifest section keep the default.
    fn document(&mut self, _request: DocRequest) -> DocResponse {
        DocResponse::Error(CatalogError::new(
            ErrorKind::UnsupportedContent,
            "document/1 is not served by this plugin",
        ))
    }
}

/// Answers one request line; never panics on malformed input.
pub fn handle_line(plugin: &mut dyn SourcePlugin, line: &str) -> String {
    let line = line.trim();
    let document = serde_json::from_str::<serde_json::Value>(line)
        .is_ok_and(|v| v[PROTOCOL_FIELD] == DOCUMENT_PROTOCOL);
    let encoded = if document {
        let response = match serde_json::from_str::<DocRequest>(line) {
            Ok(request) => plugin.document(request),
            Err(e) => DocResponse::Error(CatalogError::new(
                ErrorKind::UnsupportedContent,
                e.to_string(),
            )),
        };
        serde_json::to_string(&response)
    } else {
        let response = match serde_json::from_str::<Request>(line) {
            Ok(Request::Browse { category, page }) => plugin.browse(&category, page),
            Ok(Request::Search { query, page }) => plugin.search(&query, page),
            Ok(Request::Series { series }) => plugin.series(&series),
            Ok(Request::Episode { episode }) => plugin.episode(&episode),
            Ok(Request::Resolve { episode }) => plugin.resolve(&episode),
            Err(e) => Response::Error {
                kind: "invalid".into(),
                message: e.to_string(),
            },
        };
        serde_json::to_string(&response)
    };
    encoded.unwrap_or_default()
}

/// Reads one line from `input`, writes the answer and a newline to `output`.
pub fn serve_io(
    plugin: &mut dyn SourcePlugin,
    input: &mut dyn BufRead,
    output: &mut dyn Write,
) -> std::io::Result<()> {
    let mut line = String::new();
    input.read_line(&mut line)?;
    writeln!(output, "{}", handle_line(plugin, &line))?;
    output.flush()
}

/// Process entry for a plugin executable: one request on stdin, one response on stdout.
pub fn serve(plugin: &mut dyn SourcePlugin) -> std::io::Result<()> {
    serve_io(
        plugin,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
    )
}
