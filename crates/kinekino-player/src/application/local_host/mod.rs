use crate::application::api::{API_VERSION, ApiError, ErrorCode, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
pub const MAX_REQUEST_BYTES: usize = 1_048_576;
pub const MAX_RESPONSE_BYTES: usize = 4_194_304;
pub const DEFAULT_TIMEOUT_MS: u64 = 5_000;

/// The shared command budget. Call before frame-body allocation or response serialization.
pub fn response_limit(command: &str) -> usize {
    if crate::application::skip_segments::host::is_export_command(command) {
        crate::application::playback_session::v1::NATIVE_EXPORT_JSON_BYTES
    } else {
        MAX_RESPONSE_BYTES
    }
}
#[derive(Clone, Copy)]
pub enum ResponseFormat {
    Compact,
    Pretty,
}
pub struct EncodedResponse {
    pub json: Vec<u8>,
    pub exit_code: u8,
    pub fallback: bool,
}
/// Encodes the whole envelope. A failed partial encoding is discarded, never truncated.
pub fn encode_command_response(
    command: &str,
    response: &Response,
    format: ResponseFormat,
) -> Result<EncodedResponse, ApiError> {
    encode_response_at_limit(response, response_limit(command), format)
}
pub fn encode_response_at_limit(
    response: &Response,
    maximum: usize,
    format: ResponseFormat,
) -> Result<EncodedResponse, ApiError> {
    match encode_with_format(response, maximum, format) {
        Ok(json) => Ok(EncodedResponse {
            json,
            exit_code: response.error.as_ref().map_or(0, ApiError::exit_code),
            fallback: false,
        }),
        Err(_) => {
            // Match Response::failure canonicalization before allocating a fallback identity.
            let request_id =
                if crate::application::api::commands::valid_request_id(&response.request_id) {
                    response.request_id.clone()
                } else {
                    "cli".into()
                };
            let failure = Response::failure(
                request_id,
                transport_error(ErrorCode::ResourceExhausted, true),
            );
            let json = encode_with_format(&failure, maximum, format)
                .map_err(|_| transport_error(ErrorCode::ResourceExhausted, true))?;
            Ok(EncodedResponse {
                json,
                exit_code: failure.error.as_ref().map_or(0, ApiError::exit_code),
                fallback: true,
            })
        }
    }
}
pub fn decode_command_response(
    bytes: &[u8],
    request_id: &str,
    command: &str,
) -> Result<Response, ApiError> {
    if bytes.is_empty() || bytes.len() > response_limit(command) {
        return Err(transport_error(ErrorCode::ResourceExhausted, true));
    }
    decode_response(bytes, request_id)
}

pub fn encode_bounded(value: &impl Serialize, maximum: usize) -> Result<Vec<u8>, ApiError> {
    encode_with_format(value, maximum, ResponseFormat::Compact)
}
fn encode_with_format(
    value: &impl Serialize,
    maximum: usize,
    format: ResponseFormat,
) -> Result<Vec<u8>, ApiError> {
    struct Limited {
        bytes: Vec<u8>,
        maximum: usize,
    }
    impl std::io::Write for Limited {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.maximum.saturating_sub(self.bytes.len()) {
                return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
            }
            let needed = self.bytes.len() + bytes.len();
            if needed > self.bytes.capacity() {
                let capacity = self
                    .bytes
                    .capacity()
                    .max(256)
                    .saturating_mul(2)
                    .max(needed)
                    .min(self.maximum);
                self.bytes
                    .try_reserve_exact(capacity - self.bytes.len())
                    .map_err(|_| std::io::Error::from(std::io::ErrorKind::OutOfMemory))?;
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut writer = Limited {
        bytes: Vec::new(),
        maximum,
    };
    match format {
        ResponseFormat::Compact => serde_json::to_writer(&mut writer, value),
        ResponseFormat::Pretty => serde_json::to_writer_pretty(&mut writer, value),
    }
    .map_err(|_| transport_error(ErrorCode::ResourceExhausted, false))?;
    Ok(writer.bytes)
}

pub fn transport_error(code: ErrorCode, execution_unknown: bool) -> ApiError {
    let mut error = ApiError::new(
        code,
        if execution_unknown {
            "Local host transport failed; request may have executed"
        } else {
            "Local host transport failed before request transmission"
        },
    );
    error.details = json!({"delivery":if execution_unknown {"execution_unknown"} else {"not_sent"},"automatic_retry":false});
    error
}
/// Decode a bounded transport response without changing its SDK capability or data.
pub fn decode_response(bytes: &[u8], request_id: &str) -> Result<Response, ApiError> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct WireResponse {
        api_version: String,
        request_id: String,
        ok: bool,
        data: Value,
        error: Value,
    }
    let invalid = || transport_error(ErrorCode::EngineFailed, true);
    let wire: WireResponse = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    let error: Option<ApiError> = serde_json::from_value(wire.error).map_err(|_| invalid())?;
    let data = (!wire.data.is_null()).then_some(wire.data);
    if wire.api_version != API_VERSION
        || wire.request_id != request_id
        || (wire.ok && (error.is_some() || data.is_none()))
        || (!wire.ok
            && (data.is_some() || error.as_ref().is_none_or(|e| e.request_id != request_id)))
    {
        return Err(invalid());
    }
    Ok(Response {
        api_version: API_VERSION,
        request_id: wire.request_id,
        ok: wire.ok,
        data,
        error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn response_envelope_requires_identity_version_outcome_and_all_fields() {
        let valid = json!({"api_version":"1","request_id":"a","ok":true,"data":{},"error":null});
        assert!(decode_response(&serde_json::to_vec(&valid).unwrap(), "a").is_ok());
        for field in ["api_version", "request_id", "ok", "data", "error"] {
            let mut missing = valid.clone();
            missing.as_object_mut().unwrap().remove(field);
            assert!(
                decode_response(&serde_json::to_vec(&missing).unwrap(), "a").is_err(),
                "missing {field}"
            );
        }
        assert!(decode_response(&serde_json::to_vec(&valid).unwrap(), "wrong").is_err());
        assert!(encode_bounded(&json!({"large":"x".repeat(1000)}), 50).is_err());
    }
}

#[cfg(test)]
mod export_output_tests {
    use super::*;
    use crate::{
        application::{playback_session::v1, skip_segments::host},
        domain::api_values::DecimalU64,
    };
    fn response(data: Value) -> Response {
        Response {
            api_version: API_VERSION,
            request_id: "한\"\\😀".repeat(32),
            ok: true,
            data: Some(data),
            error: None,
        }
    }
    #[test]
    fn compact_and_pretty_envelopes_accept_exact_budget_then_fallback_at_one_byte_over() {
        for format in [ResponseFormat::Compact, ResponseFormat::Pretty] {
            let mut original = response(json!(""));
            let base = encode_with_format(&original, MAX_RESPONSE_BYTES, format)
                .unwrap()
                .len();
            let maximum = response_limit(host::SKIP_EXPORT);
            original.data = Some(json!("x".repeat(maximum - base)));
            let exact = encode_command_response(host::SKIP_EXPORT, &original, format).unwrap();
            assert_eq!(exact.json.len(), maximum);
            assert!(!exact.fallback);
            assert_eq!(exact.exit_code, 0);
            original.data = Some(json!("x".repeat(maximum - base + 1)));
            let oversized = encode_command_response(host::SKIP_EXPORT, &original, format).unwrap();
            assert!(oversized.fallback);
            assert!(oversized.json.len() + 5 <= v1::NATIVE_EXPORT_WIRE_BYTES);
            let decoded =
                decode_command_response(&oversized.json, &original.request_id, host::SKIP_EXPORT)
                    .unwrap();
            assert_eq!(
                decoded.error.as_ref().unwrap().code,
                ErrorCode::ResourceExhausted
            );
            assert_eq!(
                oversized.exit_code,
                decoded.error.as_ref().unwrap().exit_code()
            );
            assert_eq!(
                original.data.as_ref().unwrap().as_str().unwrap().len(),
                maximum - base + 1
            );
        }
    }
    fn metadata() -> v1::NativeExportMetadataV1 {
        let origin = v1::NativeExportOriginV1::try_from(v1::NativeExportOriginInputV1 {
            registry_generation: "g".repeat(40),
            session: "s".repeat(80),
            pass: DecimalU64(u64::MAX),
            segment: "t".repeat(80),
            request_revision: DecimalU64(u64::MAX),
            engine_instance_id: DecimalU64(u64::MAX),
            resolution_artifact_id: "f".repeat(64),
        })
        .unwrap();
        let annotation =
            v1::NativeExportAnnotationV1::try_from(v1::NativeExportAnnotationInputV1 {
                annotation_revision: DecimalU64(u64::MAX),
                verified_revision: DecimalU64(u64::MAX),
                timebase: crate::domain::skip_segments::SegmentTimebase::EnginePlayback,
            })
            .unwrap();
        v1::NativeExportMetadataV1::try_from(v1::NativeExportMetadataInputV1 {
            origin,
            annotation,
            actual_target_us: DecimalU64(crate::domain::playback_control::MAX_SEEK_US),
        })
        .unwrap()
    }
    #[test]
    fn maximum_64_sdk_items_and_unicode_identity_fit_whole_compact_and_pretty_envelopes() {
        let scope: v1::NativeIssuerScopeV1 =
            "nsi1.abcdefabcdefabcdefabcdefabcdefab".parse().unwrap();
        let items = (1..=64)
            .map(|row| {
                v1::NativeExportItemV1::new(
                    format!("nsr1.abcdefabcdefabcdefabcdefabcdefab.{row:016x}")
                        .parse()
                        .unwrap(),
                    metadata(),
                    v1::NativeExportStateV1::WrittenAwaitingAck,
                    v1::NativeExportReadModeV1::Unavailable,
                )
                .unwrap()
            })
            .collect();
        let list = v1::NativeExportListV1::new(scope, items).unwrap();
        let bounded = v1::encode_native_export_v1(&list).unwrap();
        let original = response(serde_json::from_slice(&bounded).unwrap());
        for format in [ResponseFormat::Compact, ResponseFormat::Pretty] {
            let encoded = encode_command_response(host::EXPORT_LIST, &original, format).unwrap();
            assert!(!encoded.fallback);
            assert!(encoded.json.len() + 5 <= v1::NATIVE_EXPORT_WIRE_BYTES);
            let decoded =
                decode_command_response(&encoded.json, &original.request_id, host::EXPORT_LIST)
                    .unwrap();
            assert_eq!(decoded.data.unwrap()["items"].as_array().unwrap().len(), 64);
        }
    }
    #[test]
    fn sdk_error_escaping_and_oversized_details_use_same_budget_without_mutating_original() {
        let mut original = Response::failure(
            "😀".repeat(128),
            ApiError::new(ErrorCode::EngineFailed, "\0\"\\".repeat(4096)),
        );
        original.error.as_mut().unwrap().details =
            json!({"large":"x".repeat(v1::NATIVE_EXPORT_JSON_BYTES)});
        for command in host::EXPORT_COMMANDS {
            for format in [ResponseFormat::Compact, ResponseFormat::Pretty] {
                let output = encode_command_response(command, &original, format).unwrap();
                assert!(output.fallback);
                assert!(output.json.len() <= v1::NATIVE_EXPORT_JSON_BYTES);
                let decoded =
                    decode_command_response(&output.json, &original.request_id, command).unwrap();
                assert_eq!(decoded.error.unwrap().code, ErrorCode::ResourceExhausted);
            }
        }
        assert_eq!(
            original.error.as_ref().unwrap().code,
            ErrorCode::EngineFailed
        );
        assert_eq!(
            original.error.as_ref().unwrap().details["large"]
                .as_str()
                .unwrap()
                .len(),
            v1::NATIVE_EXPORT_JSON_BYTES
        );
    }
    #[test]
    fn legacy_budget_and_bytes_are_unchanged_and_new_decode_checks_size_before_json() {
        let original = response(json!({"data":"x".repeat(v1::NATIVE_EXPORT_JSON_BYTES)}));
        let legacy =
            encode_command_response("version", &original, ResponseFormat::Compact).unwrap();
        assert_eq!(response_limit("version"), MAX_RESPONSE_BYTES);
        assert!(!legacy.fallback);
        assert_eq!(legacy.json, serde_json::to_vec(&original).unwrap());
        let invalid = vec![b'x'; v1::NATIVE_EXPORT_JSON_BYTES + 1];
        assert_eq!(
            decode_command_response(&invalid, "a", host::EXPORT_PEEK)
                .unwrap_err()
                .code,
            ErrorCode::ResourceExhausted
        );
        assert_eq!(
            decode_command_response(&invalid, "a", "version")
                .unwrap_err()
                .code,
            ErrorCode::EngineFailed
        );
    }
    #[test]
    fn oversized_serializer_chunk_and_zero_budget_are_rejected() {
        struct Chunk;
        impl Serialize for Chunk {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str(&"x".repeat(1025))
            }
        }
        assert!(encode_bounded(&Chunk, 1024).is_err());
        assert!(encode_bounded(&json!({}), 0).is_err());
    }
    #[test]
    fn fallback_that_cannot_fit_never_reports_that_execution_was_prevented() {
        let original = response(json!({"already_executed":true}));
        let error = match encode_response_at_limit(&original, 1, ResponseFormat::Compact) {
            Err(error) => error,
            Ok(_) => panic!("An impossible envelope must not be emitted"),
        };
        assert_eq!(error.code, ErrorCode::ResourceExhausted);
        assert_eq!(error.details["delivery"], "execution_unknown");
        assert_eq!(error.details["automatic_retry"], false);
    }
}
