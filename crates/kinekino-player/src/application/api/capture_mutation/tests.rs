use super::*;
use crate::application::api::{PlayerSdk, Request};
use serde_json::json;

#[test]
fn unavailable_capture_commands_remain_explicit_in_capabilities_and_schema() {
    let sdk = PlayerSdk::default();
    let capabilities = sdk.execute(Request::new("capabilities"));
    assert!(capabilities.ok);
    for command in ["proposals.edit_capture", "proposals.accept_capture"] {
        let data = capabilities.data.as_ref().unwrap();
        let entry = data["commands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["command"] == command)
            .unwrap();
        assert_eq!(entry["status"], "unsupported");
        let mut request = Request::new("schema");
        request.payload = json!({"command":command});
        let response = sdk.execute(request);
        assert!(response.ok);
        let schema = response.data.unwrap();
        assert_eq!(schema["status"], "unsupported");
        assert_eq!(
            schema["constraints"]["continuation_is_cas_expectation"],
            true
        );
        assert_eq!(schema["constraints"]["automatic_refresh"], false);
        assert_eq!(
            schema["request"]["properties"]["context"]["properties"]["expected_revision"]["type"],
            "null"
        );
        let response = sdk.execute(Request::new(command));
        assert_eq!(response.error.unwrap().code, ErrorCode::Unsupported);
    }
}

#[test]
fn strict_capture_request_does_not_accept_caller_origin_or_partial_expectation() {
    let mut payload = json!({"anchor":{"project_id":"work","project_revision":"1","project_revision_sha256":"a".repeat(64),"proposal_id":"draft","capture_sha256":"b".repeat(64)},"draft":{"task":"correction","edits":[]}});
    assert!(serde_json::from_value::<EditCapturedProposal>(payload.clone()).is_ok());
    payload["origin"] = json!("caller-issued");
    assert!(serde_json::from_value::<EditCapturedProposal>(payload.clone()).is_err());
    payload.as_object_mut().unwrap().remove("origin");
    payload["expected"] = json!({"draft_revision":"2"});
    assert!(serde_json::from_value::<EditCapturedProposal>(payload).is_err());
}
