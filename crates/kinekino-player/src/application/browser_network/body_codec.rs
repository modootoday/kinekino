use super::MAX_RESPONSE_BYTES;
use crate::application::api::{ApiError, ErrorCode};
use base64::Engine;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Envelope {
    body: String,
    base64_encoded: bool,
}

/// Browser-decoded response bytes, not an assertion about compressed HTTP wire bytes.
pub fn decode_body(json: &str) -> Result<Vec<u8>, ApiError> {
    if json.len() > MAX_RESPONSE_BYTES * 2 + 1024 {
        return Err(limit());
    }
    let envelope: Envelope = serde_json::from_str(json).map_err(|_| invalid())?;
    let bytes = if envelope.base64_encoded {
        base64::engine::general_purpose::STANDARD
            .decode(envelope.body)
            .map_err(|_| invalid())?
    } else {
        envelope.body.into_bytes()
    };
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err(limit());
    }
    Ok(bytes)
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid owned protocol body envelope",
    )
}
fn limit() -> ApiError {
    ApiError::new(
        ErrorCode::ResourceExhausted,
        "Owned protocol response body exceeds limit",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn text_and_base64_preserve_returned_body_bytes() {
        assert_eq!(
            decode_body(r#"{"body":"원문\n","base64Encoded":false}"#).unwrap(),
            "원문\n".as_bytes()
        );
        assert_eq!(
            decode_body(r#"{"body":"AP8=","base64Encoded":true}"#).unwrap(),
            [0, 255]
        );
    }
    #[test]
    fn missing_flags_invalid_base64_and_excessive_decoded_body_fail_without_echoing_input() {
        assert!(decode_body(r#"{"body":"fixture-secret"}"#).is_err());
        let error = decode_body(r#"{"body":"fixture-secret","base64Encoded":true}"#)
            .err()
            .unwrap();
        assert!(!error.message.contains("fixture-secret"));
        let json =
            serde_json::json!({"body":"x".repeat(MAX_RESPONSE_BYTES + 1),"base64Encoded":false})
                .to_string();
        assert!(decode_body(&json).is_err());
    }
}
