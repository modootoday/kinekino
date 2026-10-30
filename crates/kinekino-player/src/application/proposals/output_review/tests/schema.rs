use super::*;
use crate::application::api::{PlayerSdk, Request};
use serde_json::json;

#[test]
fn new_schema_records_context_and_does_not_claim_optional_legacy_support() {
    let client = PlayerSdk::default().with_proposal_store(LegacyStore::default());
    for command in [
        "proposals.inspect_output_review_target",
        "proposals.record_output_decision",
        "proposals.output_decisions",
    ] {
        let mut request = Request::new("schema");
        request.payload = json!({"command":command});
        let response = client.execute(request);
        assert!(response.ok);
        let schema = response.data.unwrap();
        assert_eq!(schema["status"], "unsupported");
        assert_eq!(
            schema["request"]["properties"]["context"]["properties"]["idempotency_key"]["type"],
            "null"
        );
        assert_eq!(
            schema["request"]["properties"]["payload"]["additionalProperties"],
            false
        );
        if command == "proposals.record_output_decision" {
            assert!(
                schema["request"]["properties"]["context"]["required"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("expected_revision"))
            );
        } else {
            assert_eq!(
                schema["request"]["properties"]["context"]["properties"]["expected_revision"]["type"],
                "null"
            );
        }
    }
    let caps = client.execute(Request::new("capabilities")).data.unwrap();
    assert_eq!(caps["proposals"]["backend_available"], true);
    assert_eq!(caps["output_reviews"]["available"], false);
    for command in caps["commands"].as_array().unwrap() {
        if command["command"].as_str().is_some_and(is_command) {
            assert_eq!(command["status"], "unsupported");
        }
    }
}

#[test]
fn legacy_command_schema_and_proposals_capability_objects_are_not_rewritten() {
    assert!(!crate::application::proposals::is_command(
        "proposals.record_output_decision"
    ));
    let before = crate::application::proposals::schema::describe(
        &json!({"command":"proposals.accept"}),
        true,
    )
    .unwrap();
    let client = PlayerSdk::default().with_proposal_store(LegacyStore::default());
    let mut request = Request::new("schema");
    request.payload = json!({"command":"proposals.accept"});
    assert_eq!(client.execute(request).data.unwrap(), before);
    let mut original = json!({"commands":[]});
    crate::application::proposals::schema::capabilities(&mut original, true, false);
    let legacy = original["proposals"].clone();
    super::super::schema::capabilities(&mut original, false);
    assert_eq!(original["proposals"], legacy);
    assert_eq!(original["output_reviews"]["counts_as_model_sample"], false);
}
