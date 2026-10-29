mod support;
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad::domain::ArtifactPin;
use kinekino_vad::protocol::{
    application::*,
    domain::{ExpectedWorkerContext, WorkerOutcomeCandidate, WorkerResult},
};
use serde_json::{Value, json};
use std::{
    cell::Cell,
    io::{self, Read},
};
use support::*;

fn worker_request_value(context: &Value, original: &[u8]) -> Value {
    json!({"schema_version":"1","job_id":"opaque-parent-label","attempt":"1",
        "input_sha256":"a".repeat(64),"source_blob_sha256":"b".repeat(64),
        "submission_request_sha256":null,"descriptor_sha256":pin("descriptor",original).sha256(),
        "context":context})
}
fn worker_request(context: &Value) -> ValidatedWorkerRequest {
    let original = bytes(context);
    let raw = bytes(&worker_request_value(context, &original));
    decode_worker_request(raw.as_slice(), raw.len() as u64, &original).unwrap()
}
#[test]
fn worker_request_preserves_noncanonical_original_descriptor_digest() {
    let context = descriptor();
    let original = serde_json::to_vec_pretty(&context).unwrap();
    let raw = bytes(&worker_request_value(&context, &original));
    let request = decode_worker_request(raw.as_slice(), raw.len() as u64, &original).unwrap();
    assert_eq!(
        request.expected().descriptor().raw().sha256(),
        pin("original", &original).sha256()
    );
    assert_ne!(
        request.expected().descriptor().raw().sha256(),
        pin("canonical", &bytes(&context)).sha256()
    );
}
#[test]
fn worker_request_rejects_changed_original_descriptor() {
    let context = descriptor();
    let original = bytes(&context);
    let raw = bytes(&worker_request_value(&context, &original));
    let mut changed = original.clone();
    changed.push(b' ');
    assert_eq!(
        code(decode_worker_request(
            raw.as_slice(),
            raw.len() as u64,
            &changed
        )),
        ErrorCode::SourceExpired
    );
}
#[test]
fn worker_request_rejects_context_from_other_profile() {
    let context = descriptor();
    let original = bytes(&context);
    let mut value = worker_request_value(&context, &original);
    value["context"]["profile"]["revision"] = json!("2".repeat(40));
    let raw = bytes(&value);
    assert_eq!(
        code(decode_worker_request(
            raw.as_slice(),
            raw.len() as u64,
            &original
        )),
        ErrorCode::RevisionConflict
    );
}
#[test]
fn worker_request_rejects_unknown_duplicate_fields_and_zero_attempt() {
    let context = descriptor();
    let original = bytes(&context);
    let mut value = worker_request_value(&context, &original);
    value["attempt"] = json!("0");
    let raw = bytes(&value);
    assert!(decode_worker_request(raw.as_slice(), raw.len() as u64, &original).is_err());
    value["attempt"] = json!("1");
    value["path"] = json!("caller-path-is-not-a-port");
    let raw = bytes(&value);
    assert!(decode_worker_request(raw.as_slice(), raw.len() as u64, &original).is_err());
    let raw = bytes(&worker_request_value(&context, &original));
    let duplicate = format!(
        "{{\"attempt\":\"2\",{}",
        std::str::from_utf8(&raw[1..]).unwrap()
    );
    assert!(
        decode_worker_request(duplicate.as_bytes(), duplicate.len() as u64, &original).is_err()
    );
}
#[test]
fn worker_request_oversize_rejects_before_read() {
    assert_eq!(
        code(decode_worker_request(
            NeverRead,
            MAX_DESCRIPTOR_BYTES + 1,
            b""
        )),
        ErrorCode::ResourceExhausted
    );
}
#[test]
fn worker_response_complete_reuses_parent_decoder_without_observed_claims() {
    let request = worker_request(&descriptor());
    let raw = encode_worker_response(
        &request,
        &WorkerResult::Complete {
            scores: vec![0.1, 0.9],
        },
    )
    .unwrap();
    let decoded =
        decode_worker_outcome(raw.as_slice(), raw.len() as u64, request.expected()).unwrap();
    assert!(matches!(decoded.result(), WorkerResult::Complete { .. }));
    assert!(decoded.actual_runtime().is_none());
}
#[test]
fn worker_response_partial_has_actual_processed_count_not_completion() {
    let request = worker_request(&descriptor());
    let raw = encode_worker_response(
        &request,
        &WorkerResult::Partial {
            processed_samples: 512,
            scores: vec![0.1],
        },
    )
    .unwrap();
    let decoded =
        decode_worker_outcome(raw.as_slice(), raw.len() as u64, request.expected()).unwrap();
    assert!(matches!(
        decoded.result(),
        WorkerResult::Partial {
            processed_samples: 512,
            ..
        }
    ));
    assert!(
        encode_worker_response(&request, &WorkerResult::Complete { scores: vec![0.1] }).is_err()
    );
}
#[test]
fn worker_response_cancelled_stays_failure_code() {
    let request = worker_request(&descriptor());
    let raw = encode_worker_response(
        &request,
        &WorkerResult::Failure {
            code: ErrorCode::Cancelled,
            message: "Cancelled; not cleanup ACK".into(),
        },
    )
    .unwrap();
    let decoded =
        decode_worker_outcome(raw.as_slice(), raw.len() as u64, request.expected()).unwrap();
    assert!(matches!(
        decoded.result(),
        WorkerResult::Failure {
            code: ErrorCode::Cancelled,
            ..
        }
    ));
}
#[test]
fn worker_response_rejects_nonfinite_scores_and_oversized_failure() {
    let request = worker_request(&descriptor());
    assert!(
        encode_worker_response(
            &request,
            &WorkerResult::Complete {
                scores: vec![f32::NAN, 0.1]
            }
        )
        .is_err()
    );
    assert!(
        encode_worker_response(
            &request,
            &WorkerResult::Failure {
                code: ErrorCode::EngineFailed,
                message: "x".repeat(4097)
            }
        )
        .is_err()
    );
}

fn result(
    value: &Value,
    expected: &ExpectedWorkerContext,
) -> Result<WorkerOutcomeCandidate, ApiError> {
    let raw = bytes(value);
    decode_worker_outcome(raw.as_slice(), raw.len() as u64, expected)
}
fn code<T>(value: Result<T, ApiError>) -> ErrorCode {
    match value {
        Err(error) => error.code,
        Ok(_) => panic!("Unexpected success"),
    }
}
struct NeverRead;
impl Read for NeverRead {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        panic!("Oversized declaration reached reader")
    }
}
struct Infinite<'a>(&'a Cell<usize>);
impl Read for Infinite<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        buffer.fill(b' ');
        self.0.set(self.0.get() + buffer.len());
        Ok(buffer.len())
    }
}
#[test]
fn descriptor_reuses_p0_types_and_preserves_exact_raw_pin() {
    let value = descriptor();
    let raw = bytes(&value);
    let expected_pin = pin("descriptor", &raw);
    let decoded = decode_descriptor(raw.as_slice(), &expected_pin).unwrap();
    assert_eq!(decoded.raw(), &expected_pin);
    assert_eq!(decoded.profile().precision(), "fp32");
    assert_eq!(decoded.input().samples(), 513);
    assert_eq!((decoded.input().start(), decoded.input().end()), (0, 513));
    assert_ne!(decoded.raw().sha256(), decoded.input().wav().sha256());
    assert_ne!(
        decoded.input().wav().sha256(),
        decoded.input().pcm().sha256()
    );
}
#[test]
fn oversized_declarations_are_rejected_before_reading() {
    let too_big = ArtifactPin::new(
        "descriptor".into(),
        "a".repeat(64),
        DecimalU64(MAX_DESCRIPTOR_BYTES + 1),
    )
    .unwrap();
    assert_eq!(
        code(decode_descriptor(NeverRead, &too_big)),
        ErrorCode::ResourceExhausted
    );
    let expected = expected(&descriptor());
    assert_eq!(
        code(decode_worker_outcome(
            NeverRead,
            MAX_OUTCOME_BYTES + 1,
            &expected
        )),
        ErrorCode::ResourceExhausted
    );
    assert_eq!(
        code(decode_worker_outcome(NeverRead, 0, &expected)),
        ErrorCode::InvalidArgument
    );
}
#[test]
fn actual_oversized_streams_stop_at_declaration_plus_one() {
    let counter = Cell::new(0);
    let declared = ArtifactPin::new("descriptor".into(), "a".repeat(64), DecimalU64(32)).unwrap();
    assert_eq!(
        code(decode_descriptor(Infinite(&counter), &declared)),
        ErrorCode::SourceExpired
    );
    assert_eq!(counter.get(), 33);
    counter.set(0);
    assert_eq!(
        code(decode_worker_outcome(
            Infinite(&counter),
            128,
            &expected(&descriptor())
        )),
        ErrorCode::SourceExpired
    );
    assert_eq!(counter.get(), 129);
}
#[test]
fn descriptor_checks_actual_length_sha_and_utf8() {
    let raw = bytes(&descriptor());
    let expected_pin = pin("descriptor", &raw);
    assert_eq!(
        code(decode_descriptor(&raw[..raw.len() - 1], &expected_pin)),
        ErrorCode::SourceExpired
    );
    let mut mutated = raw.clone();
    mutated[0] ^= 1;
    assert_eq!(
        code(decode_descriptor(mutated.as_slice(), &expected_pin)),
        ErrorCode::SourceExpired
    );
    let invalid = [0xff, 0xfe];
    assert_eq!(
        code(decode_descriptor(
            invalid.as_slice(),
            &pin("descriptor", &invalid)
        )),
        ErrorCode::InvalidArgument
    );
}
#[test]
fn descriptor_rejects_duplicate_unknown_trailing_and_wrong_version() {
    let raw = String::from_utf8(bytes(&descriptor())).unwrap();
    for malformed in [
        format!("{{\"schema_version\":\"1\",{}", &raw[1..]),
        format!("{{\"unknown\":true,{}", &raw[1..]),
        format!("{raw} {{}}"),
        raw.replace("\"schema_version\":\"1\"", "\"schema_version\":1"),
        raw.replace("\"schema_version\":\"1\"", "\"schema_version\":\"2\""),
    ] {
        assert_eq!(
            code(decode_descriptor(
                malformed.as_bytes(),
                &pin("descriptor", malformed.as_bytes())
            )),
            ErrorCode::InvalidArgument
        );
    }
}
#[test]
fn descriptor_policy_comes_from_checked_p0_constructors() {
    for mode in 0..5 {
        let mut value = descriptor();
        match mode {
            0 => value["profile"]["precision"] = json!("fp16"),
            1 => value["input"]["sample_rate"] = json!(48000),
            2 => value["input"]["end_sample"] = json!("514"),
            3 => value["input"]["pcm"]["bytes"] = json!("2"),
            _ => {
                value["profile"]["weights"]["bytes"] =
                    json!((kinekino_vad::domain::MAX_MODEL_BYTES + 1).to_string())
            }
        }
        let raw = bytes(&value);
        assert!(decode_descriptor(raw.as_slice(), &pin("descriptor", &raw)).is_err());
    }
}
#[test]
fn complete_candidate_keeps_expected_context_and_actual_raw_provenance() {
    let descriptor = descriptor();
    let expected = expected(&descriptor);
    let value = complete(&descriptor, &expected);
    let raw = bytes(&value);
    let decoded = result(&value, &expected).unwrap();
    assert_eq!(decoded.raw(), &pin("worker-outcome", &raw));
    assert_eq!(decoded.context().request(), expected.request());
    assert_eq!(decoded.context().profile(), expected.profile());
    assert_eq!(decoded.context().input().pcm(), expected.input().pcm());
    match decoded.result() {
        WorkerResult::Complete { scores } => assert_eq!(scores, &[0.1, 0.9]),
        _ => panic!("Unexpected kind"),
    }
    assert!(decoded.actual_runtime().is_none());
}
#[test]
fn binding_profile_audio_pcm_and_range_mismatches_are_rejected() {
    let descriptor = descriptor();
    let expected = expected(&descriptor);
    for mode in 0..6 {
        let mut value = complete(&descriptor, &expected);
        match mode {
            0 => value["request_sha256"] = json!("b".repeat(64)),
            1 => value["descriptor_sha256"] = json!("b".repeat(64)),
            2 => value["context"]["profile"]["id"] = json!("other"),
            3 => value["context"]["input"]["wav"]["sha256"] = json!("b".repeat(64)),
            4 => value["context"]["input"]["pcm"]["sha256"] = json!("b".repeat(64)),
            _ => value["context"]["input"]["start_sample"] = json!("1"),
        }
        assert_eq!(code(result(&value, &expected)), ErrorCode::RevisionConflict);
    }
}
#[test]
fn partial_preserves_nonzero_range_even_when_all_selected_samples_processed() {
    let mut descriptor = descriptor();
    descriptor["input"]["start_sample"] = json!("1");
    let expected = expected(&descriptor);
    let mut value = complete(&descriptor, &expected);
    value["kind"] = json!("partial");
    value["processed_samples"] = json!("512");
    value["scores"] = json!([0.7]);
    let decoded = result(&value, &expected).unwrap();
    assert_eq!(decoded.context().profile(), expected.profile());
    assert_eq!(decoded.context().input().wav(), expected.input().wav());
    assert_eq!(decoded.context().input().pcm(), expected.input().pcm());
    assert_eq!(
        (
            decoded.context().input().start(),
            decoded.context().input().end()
        ),
        (1, 513)
    );
    match decoded.result() {
        WorkerResult::Partial {
            processed_samples,
            scores,
        } => {
            assert_eq!(*processed_samples, 512);
            assert_eq!(scores, &[0.7]);
        }
        _ => panic!("Partial promoted to complete"),
    }
}
#[test]
fn score_count_range_and_nonfinite_representations_are_rejected() {
    let descriptor = descriptor();
    let expected = expected(&descriptor);
    for scores in [
        json!(vec![0.1; MAX_SCORES + 1]),
        json!([0.1]),
        json!([0.1, 0.2, 0.3]),
        json!([-0.1, 0.2]),
        json!([1.1, 0.2]),
        json!(["NaN", 0.2]),
        json!(["Infinity", 0.2]),
    ] {
        let mut value = complete(&descriptor, &expected);
        value["scores"] = scores;
        assert!(result(&value, &expected).is_err());
    }
    let raw = String::from_utf8(bytes(&complete(&descriptor, &expected))).unwrap();
    let overflow = raw.replace("\"scores\":[0.1,0.9]", "\"scores\":[1e400,0.9]");
    assert_ne!(raw, overflow);
    assert!(decode_worker_outcome(overflow.as_bytes(), overflow.len() as u64, &expected).is_err());
}
#[test]
fn runtime_observations_are_not_filled_from_expected_profile() {
    let descriptor = descriptor();
    let expected = expected(&descriptor);
    let mut value = complete(&descriptor, &expected);
    value["actual_runtime"] = json!({"provider":"observed-other-provider","precision":"observed-fp16",
        "intra_threads":4,"inter_threads":null,"modules":[module("runtime.dll")]});
    let decoded = result(&value, &expected).unwrap();
    let actual = decoded.actual_runtime().unwrap();
    assert_eq!(decoded.context().profile().precision(), "fp32");
    assert_eq!(actual.precision(), Some("observed-fp16"));
    assert_eq!(actual.provider(), Some("observed-other-provider"));
    assert_eq!(actual.intra_threads(), Some(4));
    assert_eq!(actual.inter_threads(), None);
    assert_eq!(actual.modules().len(), 1);
    assert_eq!(actual.modules()[0].canonical_path(), None);
}
#[test]
fn module_lists_are_bounded_distinct_and_strict() {
    let descriptor = descriptor();
    let expected = expected(&descriptor);
    let many: Vec<_> = (0..MAX_MODULES + 1)
        .map(|n| module(&format!("module-{n}.dll")))
        .collect();
    for modules in [
        json!(many),
        json!([module("DirectML.dll"), module("directml.dll")]),
    ] {
        let mut value = complete(&descriptor, &expected);
        value["actual_runtime"] = json!({"modules":modules});
        assert!(result(&value, &expected).is_err());
    }
    for mode in 0..3 {
        let mut item = module("runtime.dll");
        match mode {
            0 => item["canonical_path"] = json!("x".repeat(4097)),
            1 => item["sha256"] = json!("bad"),
            _ => item["unexpected"] = json!(true),
        }
        let mut value = complete(&descriptor, &expected);
        value["actual_runtime"] = json!({"modules":[item]});
        assert!(result(&value, &expected).is_err());
    }
}
#[test]
fn outcome_json_rejects_duplicates_unknown_trailing_and_wrong_kind() {
    let descriptor = descriptor();
    let expected = expected(&descriptor);
    let raw = String::from_utf8(bytes(&complete(&descriptor, &expected))).unwrap();
    for malformed in [
        format!("{{\"kind\":\"complete\",{}", &raw[1..]),
        format!("{{\"cleanup\":true,{}", &raw[1..]),
        format!("{raw} false"),
        raw.replace("\"kind\":\"complete\"", "\"kind\":\"unknown\""),
    ] {
        assert!(
            decode_worker_outcome(malformed.as_bytes(), malformed.len() as u64, &expected).is_err()
        );
    }
}
#[test]
fn contradictory_complete_partial_and_failure_fields_are_rejected() {
    let descriptor = descriptor();
    let expected = expected(&descriptor);
    for mode in 0..4 {
        let mut value = complete(&descriptor, &expected);
        match mode {
            0 => value["failure"] = json!({"code":"engine_failed","message":"contradictory"}),
            1 => value["processed_samples"] = json!("512"),
            2 => {
                value["kind"] = json!("partial");
                value["processed_samples"] = json!("514");
            }
            _ => {
                value["kind"] = json!("failure");
                value["failure"] = json!({"code":"engine_failed","message":"bad"});
            }
        }
        assert!(result(&value, &expected).is_err());
    }
}
#[test]
fn failure_preserves_diagnostic_without_granting_cleanup_authority() {
    let descriptor = descriptor();
    let expected = expected(&descriptor);
    let mut value = complete(&descriptor, &expected);
    value["kind"] = json!("failure");
    value["processed_samples"] = Value::Null;
    value["scores"] = json!([]);
    value["failure"] =
        json!({"code":"cancelled","message":"worker observed cancel; cleanup is unknown"});
    let decoded = result(&value, &expected).unwrap();
    match decoded.result() {
        WorkerResult::Failure { code, message } => {
            assert_eq!(*code, ErrorCode::Cancelled);
            assert!(message.contains("cleanup is unknown"));
        }
        _ => panic!("Failure became success"),
    }
    assert_eq!(decoded.context().request(), expected.request());
}

#[test]
fn exact_score_and_module_limits_are_accepted_without_lowering_bounds() {
    let mut descriptor = descriptor();
    descriptor["input"]["samples"] = json!(kinekino_vad::domain::MAX_SAMPLES.to_string());
    descriptor["input"]["end_sample"] = json!(kinekino_vad::domain::MAX_SAMPLES.to_string());
    descriptor["input"]["pcm"]["bytes"] =
        json!((kinekino_vad::domain::MAX_SAMPLES * 2).to_string());
    let expected = expected(&descriptor);
    let mut value = complete(&descriptor, &expected);
    value["processed_samples"] = json!(kinekino_vad::domain::MAX_SAMPLES.to_string());
    value["scores"] = json!(vec![0.5; MAX_SCORES]);
    let modules: Vec<_> = (0..MAX_MODULES)
        .map(|n| module(&format!("module-{n}.dll")))
        .collect();
    value["actual_runtime"] = json!({"modules":modules});
    let decoded = result(&value, &expected).unwrap();
    match decoded.result() {
        WorkerResult::Complete { scores } => assert_eq!(scores.len(), MAX_SCORES),
        _ => panic!("Bounded complete rejected"),
    }
    assert_eq!(
        decoded.actual_runtime().unwrap().modules().len(),
        MAX_MODULES
    );
}

#[test]
fn nested_duplicate_fields_and_duplicate_null_observations_are_rejected() {
    let descriptor = descriptor();
    let expected = expected(&descriptor);
    let raw = String::from_utf8(bytes(&complete(&descriptor, &expected))).unwrap();
    let duplicate_nested = raw.replacen(
        "\"sample_rate\":16000",
        "\"sample_rate\":16000,\"sample_rate\":16000",
        1,
    );
    assert_ne!(raw, duplicate_nested);
    let duplicate_null = format!("{{\"actual_runtime\":null,{}", &raw[1..]);
    for invalid in [duplicate_nested, duplicate_null] {
        assert!(
            decode_worker_outcome(invalid.as_bytes(), invalid.len() as u64, &expected).is_err()
        );
    }
}

#[test]
fn unit_enum_object_representations_are_not_wire_strings() {
    let descriptor = descriptor();
    let expected = expected(&descriptor);
    let mut value = complete(&descriptor, &expected);
    value["kind"] = json!({"complete":null});
    assert_eq!(code(result(&value, &expected)), ErrorCode::InvalidArgument);
    value["kind"] = json!("failure");
    value["processed_samples"] = Value::Null;
    value["scores"] = json!([]);
    value["failure"] = json!({"code":{"cancelled":null},"message":"not a string"});
    assert_eq!(code(result(&value, &expected)), ErrorCode::InvalidArgument);
    value["failure"]["code"] = json!("unknown");
    assert_eq!(code(result(&value, &expected)), ErrorCode::InvalidArgument);
}

#[test]
fn missing_scores_do_not_become_an_observed_empty_collection() {
    let descriptor = descriptor();
    let expected = expected(&descriptor);
    let mut value = complete(&descriptor, &expected);
    value["kind"] = json!("partial");
    value["processed_samples"] = json!("0");
    value.as_object_mut().unwrap().remove("scores");
    assert_eq!(code(result(&value, &expected)), ErrorCode::InvalidArgument);
    value["kind"] = json!("failure");
    value["processed_samples"] = Value::Null;
    value["failure"] = json!({"code":"engine_failed","message":"scores absent"});
    assert_eq!(code(result(&value, &expected)), ErrorCode::InvalidArgument);
    value["scores"] = json!([]);
    assert!(result(&value, &expected).is_ok());
}
