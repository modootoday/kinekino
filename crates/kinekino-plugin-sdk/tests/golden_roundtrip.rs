//! Every golden line decodes into the SDK type and re-encodes to the same JSON value.
use kinekino_plugin_sdk::golden::{self, lines};
use kinekino_plugin_sdk::wire::document::{DocRequest, DocResponse};
use kinekino_plugin_sdk::wire::source::{Request, Response};
use kinekino_plugin_sdk::{DOCUMENT_PROTOCOL, Manifest, PROTOCOL_FIELD};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

fn roundtrip<T: Serialize + DeserializeOwned>(golden: &str, extra: Option<(&str, &str)>) -> usize {
    let mut n = 0;
    for line in lines(golden) {
        let expected: Value = serde_json::from_str(line).unwrap();
        let typed: T = serde_json::from_str(line).unwrap_or_else(|e| panic!("{line}: {e}"));
        let mut got = serde_json::to_value(&typed).unwrap();
        if let Some((k, v)) = extra {
            got[k] = v.into();
        }
        assert_eq!(got, expected, "{line}");
        n += 1;
    }
    n
}

#[test]
fn source_wire_goldens_roundtrip() {
    assert_eq!(roundtrip::<Request>(golden::SOURCE_REQUESTS, None), 5);
    assert_eq!(roundtrip::<Response>(golden::SOURCE_RESPONSES, None), 5);
}

#[test]
fn document_wire_goldens_roundtrip() {
    let tag = Some((PROTOCOL_FIELD, DOCUMENT_PROTOCOL));
    assert_eq!(roundtrip::<DocRequest>(golden::DOCUMENT_REQUESTS, tag), 5);
    assert_eq!(
        roundtrip::<DocResponse>(golden::DOCUMENT_RESPONSES, None),
        5
    );
}

#[test]
fn manifest_golden_validates_and_roundtrips() {
    let m = Manifest::parse(golden::MANIFEST).unwrap();
    assert_eq!(m.api, kinekino_plugin_sdk::SOURCE_API);
    let expected: Value = serde_json::from_str(golden::MANIFEST).unwrap();
    assert_eq!(serde_json::to_value(&m).unwrap(), expected);
}

#[test]
fn manifest_validation_rejects_bad_fields() {
    let base: Value = serde_json::from_str(golden::MANIFEST).unwrap();
    let bad = |path: &[&str], v: Value| {
        let mut m = base.clone();
        let mut slot = &mut m;
        for p in path {
            slot = &mut slot[*p];
        }
        *slot = v;
        Manifest::parse(&m.to_string()).unwrap_err()
    };
    assert!(bad(&["executable"], "../x".into()).contains("relative"));
    assert!(bad(&["executable"], "/abs".into()).contains("relative"));
    assert!(bad(&["id"], "a/b".into()).contains("id"));
    assert!(bad(&["capabilities"], serde_json::json!(["teleport"])).contains("teleport"));
    assert!(bad(&["document", "kinds", "show"], "movie".into()).contains("kinds"));
    assert!(bad(&["signature", "key_id"], "xyz".into()).contains("key_id"));
    assert!(bad(&["entry_sha256"], "AA".into()).contains("entry_sha256"));
    assert!(bad(&["surprise"], 1.into()).contains("unknown field"));
}
