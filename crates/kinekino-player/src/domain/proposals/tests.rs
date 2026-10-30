use super::*;
use crate::domain::api_values::DecimalU64;
use serde_json::json;
use sha2::{Digest, Sha256};
fn asset() -> String {
    serde_json::to_string(&json!({"messages":[{"role":"user","content":concat!("Edit this ASR draft. Correct punctuation only. Do not guess unclear words. JSON only: {\"proposed\":string,\"uncertain_words\":[string]}. Under100words. /no_think\n",r#"["Actual draft.","Keep literal \\n and 한국어."]"#)}]})).unwrap()
}
fn document() -> DocumentSourceSnapshot {
    let text = asset();
    DocumentSourceSnapshot::from_asset(
        "document-1".into(),
        DecimalU64(1),
        format!("{:x}", Sha256::digest(text.as_bytes())),
        text,
    )
    .unwrap()
}
#[test]
fn document_references_bind_actual_array_positions_and_exact_source_bytes() {
    let doc = document();
    assert_eq!(doc.texts[0].inner_pointer, "/0");
    assert_eq!(doc.texts[1].text, "Keep literal \\n and 한국어.");
    let r = doc.text_reference("/0").unwrap();
    assert!(doc.contains_reference(&r));
    assert!(doc.text_reference("/00").is_none());
    let source = SourceSnapshot::Document(doc.clone());
    let p = ProposalPayload::DocumentCorrection {
        edits: vec![DocumentCorrection {
            source_ref: r.clone(),
            text: "User punctuation!".into(),
        }],
    };
    assert!(p.validate(&source, ProposalTask::Correction).is_ok());
    let mut foreign = r;
    foreign.source_asset_sha256 = "0".repeat(64);
    assert!(!doc.contains_reference(&foreign));
    let mut wrong = doc.clone();
    wrong.texts[0].text.push_str(" invented");
    assert!(wrong.validate().is_err());
    let mut wrong = doc.clone();
    wrong.asset_utf8.push(' ');
    assert!(wrong.validate().is_err());
    let mut wrong = doc.clone();
    wrong.selector.outer_pointer = "/messages/1/content".into();
    assert!(wrong.validate().is_err());
    let mut wrong = doc;
    wrong.texts.reverse();
    assert!(wrong.validate().is_err());
}
#[test]
fn source_profile_never_repairs_literal_markers_truncation_or_changed_instructions() {
    let huge = "x".repeat(MAX_DOCUMENT_BYTES + 1);
    assert_eq!(
        DocumentSourceSnapshot::from_asset("doc".into(), DecimalU64(1), "0".repeat(64), huge)
            .unwrap_err(),
        "Document source exceeds 64 KiB"
    );
    for changed in [
        asset().replace("/no_think\\n", "/no_think\\\\n"),
        asset().replace("Correct punctuation only.", "Replace source words."),
        asset().replace("Actual draft.", "Actual draft. /no_think\\n"),
        asset().replace("/no_think", "/no_marker"),
        format!(r#"{{"messages":[],{}"#, &asset()[1..]),
    ] {
        let sha = format!("{:x}", Sha256::digest(changed.as_bytes()));
        assert!(
            DocumentSourceSnapshot::from_asset("doc".into(), DecimalU64(1), sha, changed).is_err()
        );
    }
    let mut request: serde_json::Value = serde_json::from_str(&asset()).unwrap();
    let content = request["messages"][0]["content"].as_str().unwrap();
    request["messages"][0]["content"] = json!(&content[..content.len() - 1]);
    let raw = serde_json::to_string(&request).unwrap();
    let sha = format!("{:x}", Sha256::digest(raw.as_bytes()));
    assert!(DocumentSourceSnapshot::from_asset("doc".into(), DecimalU64(1), sha, raw).is_err());
    let mut request: serde_json::Value = serde_json::from_str(&asset()).unwrap();
    request["messages"][0]["content"] = json!(format!(
        "{}{{}}",
        request["messages"][0]["content"].as_str().unwrap()
    ));
    let raw = serde_json::to_string(&request).unwrap();
    let sha = format!("{:x}", Sha256::digest(raw.as_bytes()));
    assert!(DocumentSourceSnapshot::from_asset("doc".into(), DecimalU64(1), sha, raw).is_err());
}
#[test]
fn legacy_cue_source_wire_round_trips_without_migration_or_document_fields() {
    let old = format!(
        r#"{{"source_id":"legacy","source_revision":"7","source_sha256":"{}","cues":[{{"cue_id":"c1","text":"Original","start_us":null,"end_us":null}}]}}"#,
        "a".repeat(64)
    );
    let source: SourceSnapshot = serde_json::from_str(&old).unwrap();
    source.validate().unwrap();
    assert_eq!(source.schema_version(), "1");
    assert_eq!(serde_json::to_string(&source).unwrap(), old);
    assert!(source.as_document().is_none());
    assert_eq!(source.cues()[0].cue_id, "c1");
    let serialized = serde_json::to_string(&SourceSnapshot::Document(document())).unwrap();
    assert!(!serialized.contains("cue_id"));
    assert!(!serialized.contains("start_us"));
    assert!(!serialized.contains("end_us"));
}
