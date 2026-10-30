use super::*;
use serde_json::{Value, json};

fn source_ref() -> Value {
    json!({"source_asset_sha256":"a".repeat(64),"outer_pointer":"/messages/0/content",
        "extraction":"asr_request_after_no_think_lf_v1","inner_pointer":"/2"})
}

fn request(target: Value) -> Value {
    json!({"schema_version":"1", "context":{
        "request_id":"launch-명시", "api_version":crate::application::api::API_VERSION,
        "expected_revision":"18446744073709551615", "idempotency_key":"retain-key", "timeout_ms":17
    }, "selection": {"proposal_id":"proposal-1", "document_revision":"18446744073709551615",
        "source_id":"source-1", "source_revision":"9007199254740993",
        "source_sha256":"b".repeat(64), "target":target}})
}

fn decode(value: &Value) -> Result<NativeProposalLaunchRequest, ApiError> {
    decode_native_proposal_request(&serde_json::to_vec(value).unwrap())
}

#[test]
fn all_targets_preserve_explicit_selection_full_reference_and_context() {
    for target in [
        json!({"kind":"card","index":1}),
        json!({"kind":"highlight","index":2}),
        json!({"kind":"cue_correction","index":3,"cue_id":"cue-名"}),
        json!({"kind":"document_correction","index":2,"source_ref":source_ref()}),
        json!({"kind":"document_repair","selection":{"kind":"unselected"}}),
        json!({"kind":"document_repair","selection":{"kind":"selected","source_ref":source_ref()}}),
    ] {
        let original = request(target);
        let decoded = decode(&original).unwrap();
        assert_eq!(serde_json::to_value(&decoded).unwrap(), original);
        assert_eq!(decoded.selection.document_revision, DecimalU64(u64::MAX));
        assert_eq!(
            decoded.selection.source_revision,
            DecimalU64(9_007_199_254_740_993)
        );
        assert_eq!(decoded.context.timeout_ms, Some(17));
        assert_eq!(
            decoded.context.idempotency_key.as_deref(),
            Some("retain-key")
        );
        assert_eq!(
            decoded.context.expected_revision.as_deref(),
            Some("18446744073709551615")
        );
        let index = decoded.selection.target.checked_item_index().unwrap();
        assert_eq!(
            index,
            original["selection"]["target"]
                .get("index")
                .map(|value| value.as_u64().unwrap() as usize)
        );
    }
}

#[test]
fn unknown_fields_are_rejected_at_every_input_boundary() {
    for pointer in [
        "",
        "/context",
        "/selection",
        "/selection/target",
        "/selection/target/source_ref",
    ] {
        let mut value =
            request(json!({"kind":"document_correction","index":2,"source_ref":source_ref()}));
        value
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("tsx_or_executable".into(), json!("ignored"));
        assert_eq!(
            decode(&value).unwrap_err().code,
            ErrorCode::InvalidArgument,
            "{pointer}"
        );
    }
    let mut value = request(
        json!({"kind":"document_repair","selection":{"kind":"unselected","source_ref":source_ref()}}),
    );
    assert!(decode(&value).is_err());
    value["selection"]["target"] = json!({"kind":"document_repair"});
    assert!(decode(&value).is_err());
    value["selection"]["target"] =
        json!({"kind":"document_repair","selection":{"kind":"selected"}});
    assert!(decode(&value).is_err());
}

#[test]
fn context_is_validated_without_normalization_or_silently_dropping_fields() {
    for (field, invalid) in [
        ("request_id", json!("bad\nrequest")),
        ("api_version", json!("unsupported")),
        ("expected_revision", json!("-1")),
        ("idempotency_key", json!("")),
        ("timeout_ms", json!(0)),
    ] {
        let mut value = request(json!({"kind":"card","index":1}));
        value["context"][field] = invalid;
        assert_eq!(
            decode(&value).unwrap_err().code,
            ErrorCode::InvalidArgument,
            "{field}"
        );
    }
    let mut value = request(json!({"kind":"card","index":1}));
    value["schema_version"] = json!("2");
    let error = decode(&value).unwrap_err();
    assert_eq!(error.code, ErrorCode::Unsupported);
    assert_eq!(error.request_id, "launch-명시");
}

#[test]
fn duplicate_json_fields_are_rejected_instead_of_last_value_winning() {
    let value = request(json!({"kind":"card","index":1}));
    let wire = serde_json::to_string(&value).unwrap();
    for duplicate in [
        wire.replace(
            "\"schema_version\":\"1\"",
            "\"schema_version\":\"1\",\"schema_version\":\"1\"",
        ),
        wire.replace("\"timeout_ms\":17", "\"timeout_ms\":17,\"timeout_ms\":18"),
        wire.replace("\"index\":1", "\"index\":1,\"index\":2"),
    ] {
        assert_ne!(duplicate, wire);
        assert_eq!(
            decode_native_proposal_request(duplicate.as_bytes())
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
    }
}

#[test]
fn wire_numbers_and_targets_cannot_be_coerced_or_defaulted() {
    for target in [
        json!({"kind":"card"}),
        json!({"kind":"card","index":-1}),
        json!({"kind":"card","index":4294967296_u64}),
        json!({"kind":"card","index":"1"}),
        json!({"kind":"cue_correction","index":1}),
        json!({"kind":"unknown","index":1}),
    ] {
        assert!(decode(&request(target)).is_err());
    }
    let decoded = decode(&request(json!({"kind":"card","index":u32::MAX}))).unwrap();
    assert_eq!(
        decoded.selection.target.checked_item_index().unwrap(),
        Some(usize::try_from(u32::MAX).unwrap())
    );
    for bad in [json!(1), json!("-1"), json!("18446744073709551616")] {
        let mut value = request(json!({"kind":"card","index":1}));
        value["selection"]["document_revision"] = bad;
        assert!(decode(&value).is_err());
    }
}

#[test]
fn exact_byte_limit_is_accepted_but_overflow_and_malformed_bytes_are_rejected() {
    let mut bytes = serde_json::to_vec(&request(json!({"kind":"card","index":1}))).unwrap();
    bytes.resize(MAX_NATIVE_PROPOSAL_REQUEST_BYTES, b' ');
    assert!(decode_native_proposal_request(&bytes).is_ok());
    bytes.push(b' ');
    assert_eq!(
        decode_native_proposal_request(&bytes).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    for bytes in [&b"{"[..], &b"\xff"[..], &b"{} {}"[..]] {
        assert_eq!(
            decode_native_proposal_request(bytes).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
}
