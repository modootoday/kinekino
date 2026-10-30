use super::*;
use serde_json::json;
fn fixture() -> NativeRoutedFrameReadyV1 {
    serde_json::from_value(json!({"schema_version":1,"status":"activated",
        "document":{"host_id":"1","instance_document":"original","instance_epoch":"2","instance_id":"3",
            "source_generation":"4","definition_revision":"5","document_revision":"6","turn_sequence":"7"},
        "window":{"owner":"1","epoch":"2","surface_generation":"3"},
        "frame":{"context_id":"original-frame","context_generation":"1","scene_generation":"2"},
        "child_id":"8","route_generation":"9"})).unwrap()
}
#[test]
fn ready_preserves_string_counters_and_rejects_version_or_identity_overflow() {
    let mut record = fixture();
    let encoded = ready(&record, 10).unwrap();
    let value: Value = serde_json::from_slice(encoded.as_bytes()).unwrap();
    assert_eq!(value["data"]["child_id"], "8");
    assert!(encoded.as_bytes().len() <= READY_MAX_BYTES);
    record.schema_version = 2;
    assert!(ready(&record, 10).is_err());
    record.schema_version = 1;
    record.frame.context_id = "x".repeat(IDENTITY_TEXT_MAX_BYTES + 1);
    assert!(ready(&record, 10).is_err());
    record.frame.context_id = "line\nbreak".into();
    assert!(ready(&record, 10).is_err());
}
#[test]
fn envelope_checks_the_entire_record_before_any_output() {
    let oversized = json!({"legacy": "x".repeat(FINAL_MAX_BYTES)});
    assert!(
        envelope(
            VIEWER_OUTCOME_KIND,
            &oversized,
            0,
            "native-viewer",
            FINAL_MAX_BYTES
        )
        .is_err()
    );
    let encoded = ready(&fixture(), 0).unwrap();
    let mut bytes = Vec::new();
    write_line(&encoded, &mut bytes).unwrap();
    assert_eq!(bytes.last(), Some(&b'\n'));
    assert_eq!(&bytes[..bytes.len() - 1], encoded.as_bytes());
}

#[test]
fn bounded_buffer_accepts_exact_encoded_boundary_and_rejects_one_byte_less() {
    let data = json!({"text":"한글\n\"\\"});
    let encoded = envelope("fixture", &data, 0, "native-viewer", 1024).unwrap();
    let bound = encoded.as_bytes().len();
    assert!(envelope("fixture", &data, 0, "native-viewer", bound).is_ok());
    assert!(envelope("fixture", &data, 0, "native-viewer", bound - 1).is_err());
    let mut buffer = BoundedBuffer {
        bytes: Vec::with_capacity(3),
        limit: 3,
    };
    buffer.write_all(b"ab").unwrap();
    assert!(buffer.write_all(b"cd").is_err());
    assert_eq!(buffer.bytes, b"ab");
}

#[test]
fn serialization_failure_exposes_no_encoded_token_or_output() {
    struct Fails;
    impl Serialize for Fails {
        fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("fixture encoding failure"))
        }
    }
    let mut output = Vec::new();
    let result = envelope("fixture", &Fails, 0, "native-viewer", READY_MAX_BYTES);
    assert!(result.is_err());
    if let Ok(encoded) = result {
        write_line(&encoded, &mut output).unwrap();
    }
    assert!(output.is_empty());
}

#[test]
fn final_v2_preserves_original_diagnostic_fields_boolean_or_unobserved_overflow() {
    let mut record: NativeRoutedFinalV2 = serde_json::from_value(json!({
        "schema_version":2,"native_closed":null,"viewer_workers_quiescent":null,
        "execution":{"worker_stopped":null,"unacknowledged":null},
        "local_host":{"pipe":"unobserved","sdk":"unobserved","dispatch":"unobserved"},
        "transition":{"configured":true,"admission_closed":null,"ended":null,"remaining_job_id":null,"error":null},
        "provider":{"configured":true,"completed":null,"pending":null,"jobs_receipt":null,
            "browser_worker_exited":null,"first_error":null,"secondary_errors":[],"secondary_overflow":null},
        "dispatcher":{"joined":null,"worker_panicked":null,"faults":null},
        "cleanup_complete":false,"first_error":null,"diagnostic_write_error":null
    })).unwrap();
    record
        .provider
        .secondary_errors
        .push(ProviderShutdownDiagnostic {
            code: crate::application::api::ErrorCode::Internal,
            message: "original\nmessage".into(),
            remediation: "original remedy".into(),
        });
    let legacy = json!({"original":"retained"});
    let encoded = final_outcome(&legacy, &record, 0).unwrap();
    let data: Value = serde_json::from_slice(encoded.as_bytes()).unwrap();
    assert_eq!(data["data"]["original"], "retained");
    assert_eq!(
        data["data"]["native_routed"]["provider"]["secondary_overflow"],
        Value::Null
    );
    assert_eq!(
        data["data"]["native_routed"]["provider"]["secondary_errors"][0]["message"],
        "original\nmessage"
    );
    assert!(
        data["data"]["native_routed"]["provider"]["secondary_errors"][0]
            .get("details")
            .is_none()
    );
    record.provider.secondary_overflow = Some(false);
    assert!(final_outcome(&legacy, &record, 0).is_ok());
    record.schema_version = 1;
    assert!(final_outcome(&legacy, &record, 0).is_err());
    record.schema_version = 2;
    record.provider.secondary_errors[0].message = "한".repeat(1366);
    assert!(final_outcome(&legacy, &record, 0).is_err());
    assert_eq!(legacy, json!({"original":"retained"}));
}
