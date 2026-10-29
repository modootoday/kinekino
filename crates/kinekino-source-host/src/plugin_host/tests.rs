use super::*;
use std::path::Path;

#[test]
fn manifest_rejects_escaping_executable_and_unknown_capability() {
    let dir = Path::new("/p");
    let ok = r#"{"id":"com.example.fixture","version":"1","api":"source/1","capabilities":["browse"],"executable":"bin/plugin"}"#;
    let m = PluginManifest::parse(dir, ok).unwrap();
    assert_eq!(m.executable, dir.join("bin/plugin"));
    assert!(PluginManifest::parse(dir, &ok.replace("bin/plugin", "../x")).is_err());
    assert!(PluginManifest::parse(dir, &ok.replace("browse", "teleport")).is_err());
}

#[test]
fn fixture_serves_browse_and_rejects_others() {
    let reply = wire::serve_fixture(
        r#"{"method":"browse","category":{"provider":"p","id":"airing"},"page":2}"#,
        "p",
    );
    assert!(reply.contains("airing-2"), "{reply}");
    assert!(
        wire::serve_fixture(r#"{"method":"search","query":"x","page":1}"#, "p")
            .contains("unsupported")
    );
}

/// Host side of the SDK golden contract: the same files the SDK tests read.
mod golden {
    use super::*;
    use kinekino_core::source::SourceCapability;
    use kinekino_plugin_sdk::golden::{self, lines};

    #[test]
    fn host_parses_golden_manifest_into_core_values() {
        let m = PluginManifest::parse(Path::new("/p"), golden::MANIFEST).unwrap();
        assert_eq!(m.source.id, "com.example.fixture");
        assert_eq!(m.source.capabilities.len(), 5);
        assert_eq!(m.source.capabilities[0], SourceCapability::Browse);
        assert_eq!(m.executable, Path::new("/p").join("bin/fixture-source"));
        assert_eq!(m.document.unwrap().kinds["watch"], "episode");
    }

    #[test]
    fn host_wire_reads_every_golden_line() {
        for l in lines(golden::SOURCE_REQUESTS) {
            serde_json::from_str::<wire::Request>(l).unwrap();
        }
        for l in lines(golden::SOURCE_RESPONSES) {
            serde_json::from_str::<wire::Response>(l).unwrap();
        }
        for l in lines(golden::DOCUMENT_REQUESTS) {
            let r: document_wire::DocRequest = serde_json::from_str(l).unwrap();
            let mut v = serde_json::to_value(r).unwrap();
            v[document_wire::PROTOCOL_FIELD] = document_wire::DOCUMENT_PROTOCOL.into();
            assert_eq!(v, serde_json::from_str::<serde_json::Value>(l).unwrap());
        }
        for l in lines(golden::DOCUMENT_RESPONSES) {
            serde_json::from_str::<document_wire::DocResponse>(l).unwrap();
        }
    }

    #[test]
    fn fixture_plugin_answers_golden_browse() {
        let first = lines(golden::SOURCE_REQUESTS).next().unwrap();
        let reply: wire::Response =
            serde_json::from_str(&wire::serve_fixture(first, "com.example.fixture")).unwrap();
        assert!(matches!(
            reply,
            wire::Response::Page {
                next_page: Some(2),
                ..
            }
        ));
    }
}
