//! Compare direct SDK and parent dispatch against the common context policy.
//! Uses the real SDK's rejection as the oracle, not a copied ID policy.
use super::*;
use crate::application::api::Request;
use meari::application::plugins::{PluginDispatch, PluginPayload};

#[test]
fn invalid_original_request_id_is_rejected_equally_by_sdk_and_direct_parent_dispatch() {
    let f = fixture();
    let mut lease = ProposalSdkRegistration::register(
        f.client.clone(),
        f.registry.clone(),
        &f.scope,
        f.context.clone(),
    )
    .unwrap();
    for request_id in [
        String::new(),
        "blocked-raw-correlation\n".into(),
        "x".repeat(129),
    ] {
        f.context.borrow_mut().request_id = request_id;
        let mut direct = Request::new("proposals.inspect");
        direct.context = f.context.borrow().clone();
        direct.payload = serde_json::json!({"proposal_id":"missing"});
        let expected = f.client.execute(direct.clone()).error.unwrap();
        assert_eq!(expected.code, ErrorCode::InvalidArgument);
        let receipt = f
            .registry
            .borrow_mut()
            .dispatch(
                lease.handle().unwrap(),
                "proposals.inspect",
                PluginPayload {
                    schema: proposal_manifest().capabilities["proposals.inspect"]
                        .request_schema
                        .clone(),
                    value: direct.payload,
                },
            )
            .unwrap();
        let PluginDispatch::Immediate(Err(error)) = receipt.outcome else {
            panic!("Invalid caller context must not succeed");
        };
        let actual: ApiError = serde_json::from_value(error.details.unwrap().value).unwrap();
        assert_eq!(
            actual.code, expected.code,
            "Derived ticket correlation must not sanitize an invalid caller ID into acceptance"
        );
        assert!(
            !serde_json::to_string(&actual)
                .unwrap()
                .contains("blocked-raw-correlation")
        );
    }
    lease.close().unwrap();
}

#[test]
fn valid_context_has_stable_correlation_for_the_same_ticket_without_changing_options() {
    let f = fixture();
    let mut lease = ProposalSdkRegistration::register(
        f.client.clone(),
        f.registry.clone(),
        &f.scope,
        f.context.clone(),
    )
    .unwrap();
    let receipt = f
        .registry
        .borrow_mut()
        .dispatch(
            lease.handle().unwrap(),
            "proposals.inspect",
            PluginPayload {
                schema: proposal_manifest().capabilities["proposals.inspect"]
                    .request_schema
                    .clone(),
                value: serde_json::json!({"proposal_id":"missing"}),
            },
        )
        .unwrap();
    let caller = RequestContext {
        request_id: "유효한-명시-ID".into(),
        expected_revision: Some("9007199254740993".into()),
        idempotency_key: Some("explicit".into()),
        timeout_ms: Some(100),
        ..Default::default()
    };
    let first = scoped_context(&caller, &receipt.ticket).unwrap();
    let second = scoped_context(&caller, &receipt.ticket).unwrap();
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(second).unwrap()
    );
    assert_ne!(first.request_id, caller.request_id);
    let mut actual = serde_json::to_value(first).unwrap();
    actual["request_id"] = serde_json::json!(caller.request_id);
    assert_eq!(actual, serde_json::to_value(caller).unwrap());
    lease.close().unwrap();
}
