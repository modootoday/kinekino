//! Sanitized network-stack header observations. Credential values are discarded.
use super::{
    NetworkEvent,
    protocol_events::{bounded, id},
};
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::request_referer::RequestReferer,
};
use serde_json::Value;

pub(super) fn request(value: &Value) -> Result<NetworkEvent, ApiError> {
    let request_id = id(value, "requestId")?;
    let headers = value["headers"]
        .as_object()
        .ok_or_else(|| invalid("ExtraInfo headers were not observed"))?;
    let mut matches = headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("referer"));
    let referer = matches
        .next()
        .map(|(_, value)| bounded(value, 8192))
        .transpose()?;
    if matches.next().is_some() {
        return Err(invalid("Ambiguous ExtraInfo Referer headers"));
    }
    Ok(NetworkEvent::RequestExtraInfo {
        request_id,
        referer: RequestReferer::observed(referer),
        credentials: credentials(value, headers),
    })
}
fn credentials(
    value: &Value,
    headers: &serde_json::Map<String, Value>,
) -> super::request_headers::CredentialObservation {
    use super::request_headers::CredentialObservation;
    if headers.keys().any(|name| {
        ["cookie", "authorization", "proxy-authorization"]
            .iter()
            .any(|credential| name.eq_ignore_ascii_case(credential))
    }) {
        return CredentialObservation::Present;
    }
    match value.get("associatedCookies").and_then(Value::as_array) {
        Some(cookies) if !cookies.is_empty() => CredentialObservation::Present,
        Some(_) if headers.values().all(Value::is_string) => CredentialObservation::NoneObserved,
        _ => CredentialObservation::Unknown,
    }
}
fn invalid(message: &'static str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn credential_projection_preserves_unknown_and_presence_without_values() {
        for (raw, expected) in [
            (json!({"requestId":"r","headers":{}}), "unknown"),
            (
                json!({"requestId":"r","headers":{},"associatedCookies":[]}),
                "none_observed",
            ),
            (
                json!({"requestId":"r","headers":{},"associatedCookies":null}),
                "unknown",
            ),
            (
                json!({"requestId":"r","headers":{"other":false},"associatedCookies":[]}),
                "unknown",
            ),
            (
                json!({"requestId":"r","headers":{"cOoKiE":"secret-marker"},"associatedCookies":[]}),
                "present",
            ),
            (
                json!({"requestId":"r","headers":{"Authorization":"secret-marker"}}),
                "present",
            ),
            (
                json!({"requestId":"r","headers":{"Proxy-Authorization":""}}),
                "present",
            ),
            (
                json!({"requestId":"r","headers":{},"associatedCookies":[{"cookie":{"value":"secret-marker"},"blockedReasons":["blocked"]}]}),
                "present",
            ),
        ] {
            let projected = serde_json::to_value(request(&raw).unwrap()).unwrap();
            assert_eq!(projected["credentials"], expected);
            assert!(!projected.to_string().contains("secret-marker"));
        }
    }
    #[test]
    fn complete_header_map_distinguishes_absent_empty_and_unobserved_without_secrets() {
        for (headers, kind) in [(json!({}), "absent"), (json!({"Referer":""}), "empty")] {
            let event = request(&json!({"requestId":"request","headers":headers,"associatedCookies":[{"cookie":{"value":"fixture-secret-cookie"}}]})).unwrap();
            let projection = serde_json::to_value(event).unwrap();
            assert_eq!(projection["referer"]["kind"], kind);
            assert!(!projection.to_string().contains("fixture-secret-cookie"));
        }
        assert!(request(&json!({"requestId":"request"})).is_err());
        assert!(request(&json!({"requestId":"request","headers":{"Referer":"","referer":"https://example.invalid"}})).is_err());
    }
}
