//! `serve`/`SourcePlugin` dispatch against the goldens.
use kinekino_plugin_sdk::fixture::{FixturePlugin, serve_document_fixture, serve_fixture};
use kinekino_plugin_sdk::golden::{self, lines};
use kinekino_plugin_sdk::serve::serve_io;
use kinekino_plugin_sdk::wire::document::DocResponse;
use kinekino_plugin_sdk::wire::source::Response;
use kinekino_plugin_sdk::{SourcePlugin, handle_line};

struct Empty;
impl SourcePlugin for Empty {}

#[test]
fn default_methods_answer_unsupported_for_every_golden_request() {
    for l in lines(golden::SOURCE_REQUESTS) {
        let r: Response = serde_json::from_str(&handle_line(&mut Empty, l)).unwrap();
        assert!(
            matches!(r, Response::Error { ref kind, .. } if kind == "unsupported"),
            "{l}"
        );
    }
    for l in lines(golden::DOCUMENT_REQUESTS) {
        let r: DocResponse = serde_json::from_str(&handle_line(&mut Empty, l)).unwrap();
        assert!(matches!(r, DocResponse::Error(_)), "{l}");
    }
    let r: Response = serde_json::from_str(&handle_line(&mut Empty, "not json")).unwrap();
    assert!(matches!(r, Response::Error { ref kind, .. } if kind == "invalid"));
}

#[test]
fn fixture_plugin_routes_source_and_document_lines() {
    let mut p = FixturePlugin {
        provider: "p".into(),
    };
    let mut out = Vec::new();
    let first = lines(golden::SOURCE_REQUESTS).next().unwrap();
    serve_io(&mut p, &mut format!("{first}\n").as_bytes(), &mut out).unwrap();
    let text = String::from_utf8(out).unwrap();
    assert!(text.ends_with('\n') && text.contains("airing-1"), "{text}");
    let revision = lines(golden::DOCUMENT_REQUESTS).next().unwrap();
    assert!(handle_line(&mut p, revision).contains("fixture-parser/1"));
    assert!(serve_document_fixture(revision).contains("fixture-parser/1"));
    assert!(
        serve_fixture(r#"{"method":"search","query":"x","page":1}"#, "p").contains("unsupported")
    );
}
