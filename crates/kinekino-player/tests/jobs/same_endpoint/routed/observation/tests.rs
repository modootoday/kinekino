//! Synthetic schema fixtures only; never GUI/native/cleanup acceptance evidence.
use super::*;
use serde_json::{Value, json};

fn line(data: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(data).unwrap();
    bytes.push(b'\n');
    bytes
}
fn ready_value() -> Value {
    json!({"kind":"native-routed-frame-ready","elapsed_ms":1,"elapsed_scope":"native-viewer",
        "data":{"schema_version":1,"status":"activated",
        "document":{"host_id":"1","instance_document":"original-document","instance_epoch":"1",
            "instance_id":"1","source_generation":"1","definition_revision":"1","document_revision":"1","turn_sequence":"2"},
        "window":{"owner":"1","epoch":"1","surface_generation":"1"},
        "frame":{"context_id":"original-frame","context_generation":"1","scene_generation":"2"},
        "child_id":"1","route_generation":"1"}})
}
fn final_value() -> Value {
    json!({"kind":"native-viewer-outcome","elapsed_ms":2,"elapsed_scope":"native-viewer",
        "data":{"success":true,"compatible_older_key":"preserved","native_routed":{
            "schema_version":2,"native_closed":true,"viewer_workers_quiescent":true,
            "execution":{"worker_stopped":true,"unacknowledged":"0"},
            "local_host":{"pipe":"joined","sdk":"joined","dispatch":"completed"},
            "transition":{"configured":true,"admission_closed":true,"ended":true,"remaining_job_id":null,"error":null},
            "provider":{"configured":true,"completed":true,"pending":false,
                "jobs_receipt":{"completed_total":"9","cancelled_total":"0","interrupted_total":"0"},
                "browser_worker_exited":true,"first_error":null,"secondary_errors":[],"secondary_overflow":false},
            "dispatcher":{"joined":true,"worker_panicked":false,"faults":"0"},
            "cleanup_complete":true,"first_error":null,"diagnostic_write_error":null}}})
}
#[test]
fn complete_native_cleanup_with_wrong_job_outcome_remains_a_functional_failure() {
    kinekino_platform::install();
    let mut report = final_value();
    report["data"]["native_routed"]["provider"]["jobs_receipt"]["completed_total"] = json!("2");
    let bytes = line(&report);
    let error = final_report(&bytes, 3).unwrap_err();
    assert!(error.to_string().contains("provider job outcomes"));
    assert!(error.to_string().contains("expected completed=3"));
    assert!(error.to_string().contains("observed completed=2"));
    assert!(error.to_string().contains("native cleanup is complete"));
    assert!(!error.to_string().contains("cleanup is incomplete"));
    assert!(final_report(&bytes, 2).unwrap().cleanup_complete);
    assert_eq!(bytes, line(&report));
}
#[test]
fn actual_cleanup_fault_keeps_priority_over_job_outcome_mismatch() {
    kinekino_platform::install();
    let mut report = final_value();
    report["data"]["native_routed"]["execution"]["worker_stopped"] = json!(false);
    report["data"]["native_routed"]["provider"]["jobs_receipt"]["completed_total"] = json!("2");
    let error = final_report(&line(&report), 3).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("cleanup is incomplete or faulted")
    );
    assert!(!error.to_string().contains("native cleanup is complete"));
}

#[test]
fn complete_ready_schema_rejects_partial_oversized_control_text_and_numeric_counters() {
    kinekino_platform::install();
    let original = ready_value();
    let bytes = line(&original);
    assert_eq!(ready(&bytes).unwrap().child_id.0, 1);
    assert!(ready(&bytes[..bytes.len() - 1]).is_err());
    let mut oversized = vec![b'x'; READY_MAX_BYTES + 1];
    oversized.push(b'\n');
    assert!(ready(&oversized).is_err());
    for replacement in [json!("wrong-kind"), json!("native-viewer-outcome")] {
        let mut changed = original.clone();
        changed["kind"] = replacement;
        assert!(ready(&line(&changed)).is_err());
    }
    for text in [
        "x".repeat(129),
        "한".repeat(43),
        "original\nframe".into(),
        String::new(),
    ] {
        let mut changed = original.clone();
        changed["data"]["frame"]["context_id"] = json!(text);
        assert!(ready(&line(&changed)).is_err());
    }
    let mut changed = original.clone();
    changed["data"]["child_id"] = json!(1);
    assert!(ready(&line(&changed)).is_err());
    changed = original.clone();
    changed["data"]["schema_version"] = json!(2);
    assert!(ready(&line(&changed)).is_err());
    changed = original.clone();
    changed["data"]["unknown_authority"] = json!(true);
    assert!(ready(&line(&changed)).is_err());
}

#[test]
fn final_summary_alone_cannot_replace_each_original_configured_owner_receipt() {
    kinekino_platform::install();
    let original = final_value();
    assert!(final_report(&line(&original), 9).is_ok());
    assert!(final_report(&line(&original), 10).is_err());
    let paths = [
        "/native_closed",
        "/viewer_workers_quiescent",
        "/execution/worker_stopped",
        "/execution/unacknowledged",
        "/local_host/pipe",
        "/local_host/sdk",
        "/local_host/dispatch",
        "/transition/configured",
        "/transition/admission_closed",
        "/transition/ended",
        "/provider/configured",
        "/provider/completed",
        "/provider/pending",
        "/provider/jobs_receipt",
        "/provider/browser_worker_exited",
        "/provider/secondary_overflow",
        "/dispatcher/joined",
        "/dispatcher/worker_panicked",
        "/dispatcher/faults",
    ];
    for path in paths {
        let mut changed = original.clone();
        *changed["data"]["native_routed"].pointer_mut(path).unwrap() = Value::Null;
        assert!(
            final_report(&line(&changed), 9).is_err(),
            "Missing original receipt {path}"
        );
    }
    let mut changed = original.clone();
    changed["data"]["native_routed"]["transition"]["configured"] = json!(false);
    assert!(final_report(&line(&changed), 9).is_err());
    changed = original.clone();
    changed["data"]["native_routed"]["diagnostic_write_error"] = json!("original flush failed");
    assert!(final_report(&line(&changed), 9).is_err());
    changed = original.clone();
    changed["data"]["native_routed"]["transition"]["remaining_job_id"] =
        json!("still-needs-terminal-evidence");
    assert!(final_report(&line(&changed), 9).is_err());
    changed = original.clone();
    changed["data"]
        .as_object_mut()
        .unwrap()
        .remove("native_routed");
    assert!(final_report(&line(&changed), 9).is_err());
    let bytes = line(&original);
    assert!(final_report(&bytes[..bytes.len() - 1], 9).is_err());
    let mut oversized = vec![b'x'; FINAL_MAX_BYTES + 1];
    oversized.push(b'\n');
    assert!(final_report(&oversized, 9).is_err());
}

#[test]
fn final_v2_rejects_historical_schema_counts_and_unobserved_overflow() {
    kinekino_platform::install();
    let original = final_value();
    let observed = final_report(&line(&original), 9).unwrap();
    assert_eq!(observed.schema_version, 2);
    assert_eq!(observed.provider.secondary_overflow, Some(false));
    for version in [0, 1, 3] {
        let mut changed = original.clone();
        changed["data"]["native_routed"]["schema_version"] = json!(version);
        assert!(final_report(&line(&changed), 9).is_err());
    }
    for overflow in [json!("0"), json!(0), json!(1), json!(true), Value::Null] {
        let mut changed = original.clone();
        changed["data"]["native_routed"]["provider"]["secondary_overflow"] = overflow;
        assert!(final_report(&line(&changed), 9).is_err());
    }
    let mut changed = original.clone();
    changed["data"]["native_routed"]["provider"]
        .as_object_mut()
        .unwrap()
        .remove("secondary_overflow");
    assert!(final_report(&line(&changed), 9).is_err());
}

#[test]
fn original_secondary_diagnostics_preserve_prose_and_cannot_be_upgraded_to_api_errors() {
    kinekino_platform::install();
    let mut report = final_value();
    let diagnostic = json!({"code":"timeout","message":"원 오류\nsecond line\t한글",
        "remediation":"원 조치\r\nretry after inspection"});
    report["data"]["native_routed"]["provider"]["secondary_errors"] = json!([diagnostic]);
    let bytes = line(&report);
    let decoded: Envelope<FinalData> = decode(&bytes, FINAL_MAX_BYTES).unwrap();
    let original = &decoded.data.native_routed.provider.secondary_errors[0];
    assert_eq!(original.message, "원 오류\nsecond line\t한글");
    assert_eq!(original.remediation, "원 조치\r\nretry after inspection");
    assert_eq!(serde_json::to_value(original).unwrap(), diagnostic);
    assert!(final_report(&bytes, 9).is_err());
    for (field, value) in [
        ("details", json!({})),
        ("retryable", json!(true)),
        ("request_id", json!("fabricated-request")),
    ] {
        let mut changed = report.clone();
        changed["data"]["native_routed"]["provider"]["secondary_errors"][0][field] = value;
        assert!(decode::<Envelope<FinalData>>(&line(&changed), FINAL_MAX_BYTES).is_err());
    }
    for field in ["code", "message", "remediation"] {
        let mut changed = report.clone();
        changed["data"]["native_routed"]["provider"]["secondary_errors"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(decode::<Envelope<FinalData>>(&line(&changed), FINAL_MAX_BYTES).is_err());
    }
}
