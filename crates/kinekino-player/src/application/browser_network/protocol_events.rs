//! Bounded CDP observations. Identifiers remain in the protocol namespace, not native frame IDs.
use crate::application::api::{ApiError, ErrorCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NetworkEvent {
    RequestExtraInfo {
        request_id: String,
        referer: crate::domain::request_referer::RequestReferer,
        credentials: super::request_headers::CredentialObservation,
    },
    FrameTree {
        frame_id: String,
        parent_frame_id: Option<String>,
        loader_id: String,
        url: String,
    },
    TargetAttached {
        session_id: String,
        target_id: String,
        target_type: String,
        waiting_for_debugger: bool,
    },
    TargetDetached {
        session_id: String,
        target_id: Option<String>,
    },
    FrameAttached {
        frame_id: String,
        parent_frame_id: String,
    },
    FrameNavigated {
        frame_id: String,
        parent_frame_id: Option<String>,
        loader_id: String,
        url: String,
    },
    FrameDetached {
        frame_id: String,
        reason: String,
    },
    Request {
        request_id: String,
        frame_id: Option<String>,
        loader_id: String,
        document_url: String,
        url: String,
        method: String,
        resource_type: Option<String>,
        referer: Option<String>,
        redirect: Option<NetworkResponse>,
    },
    Response {
        request_id: String,
        frame_id: Option<String>,
        loader_id: String,
        resource_type: String,
        response: NetworkResponse,
    },
    Finished {
        request_id: String,
        encoded_bytes: u64,
    },
    Failed {
        request_id: String,
        cancelled: Option<bool>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkResponse {
    pub url: String,
    pub status: u16,
    pub mime_type: String,
    pub encoded_bytes: Option<u64>,
    pub from_disk_cache: Option<bool>,
    pub from_service_worker: Option<bool>,
}

pub fn decode(event: &str, json: &str) -> Result<NetworkEvent, ApiError> {
    if json.len() > 128 * 1024 {
        return Err(invalid());
    }
    let value: Value = serde_json::from_str(json).map_err(|_| invalid())?;
    let output = match event {
        "Network.requestWillBeSentExtraInfo" => super::extra_info::request(&value)?,
        "Page.getFrameTree" => {
            let frame = &value["frameTree"]["frame"];
            NetworkEvent::FrameTree {
                frame_id: id(frame, "id")?,
                parent_frame_id: optional_id(frame, "parentId")?,
                loader_id: text(frame, "loaderId", 128)?,
                url: text(frame, "url", 8192)?,
            }
        }
        "Target.attachedToTarget" => NetworkEvent::TargetAttached {
            session_id: id(&value, "sessionId")?,
            target_id: id(&value["targetInfo"], "targetId")?,
            target_type: text(&value["targetInfo"], "type", 64)?,
            waiting_for_debugger: value["waitingForDebugger"].as_bool().ok_or_else(invalid)?,
        },
        "Target.detachedFromTarget" => NetworkEvent::TargetDetached {
            session_id: id(&value, "sessionId")?,
            target_id: optional_id(&value, "targetId")?,
        },
        "Page.frameAttached" => NetworkEvent::FrameAttached {
            frame_id: id(&value, "frameId")?,
            parent_frame_id: id(&value, "parentFrameId")?,
        },
        "Page.frameNavigated" => {
            let frame = &value["frame"];
            NetworkEvent::FrameNavigated {
                frame_id: id(frame, "id")?,
                parent_frame_id: optional_id(frame, "parentId")?,
                loader_id: id(frame, "loaderId")?,
                url: text(frame, "url", 8192)?,
            }
        }
        "Page.frameDetached" => NetworkEvent::FrameDetached {
            frame_id: id(&value, "frameId")?,
            reason: text(&value, "reason", 64)?,
        },
        "Network.requestWillBeSent" => {
            let request = &value["request"];
            let headers = request["headers"].as_object().ok_or_else(invalid)?;
            let referer = headers
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("referer"))
                .map(|(_, value)| bounded(value, 8192))
                .transpose()?;
            NetworkEvent::Request {
                request_id: id(&value, "requestId")?,
                frame_id: optional_id(&value, "frameId")?,
                loader_id: text(&value, "loaderId", 128)?,
                document_url: text(&value, "documentURL", 8192)?,
                url: text(request, "url", 8192)?,
                method: text(request, "method", 16)?,
                resource_type: value
                    .get("type")
                    .map(|value| bounded(value, 64))
                    .transpose()?,
                referer,
                redirect: value.get("redirectResponse").map(response).transpose()?,
            }
        }
        "Network.responseReceived" => NetworkEvent::Response {
            request_id: id(&value, "requestId")?,
            frame_id: optional_id(&value, "frameId")?,
            loader_id: text(&value, "loaderId", 128)?,
            resource_type: text(&value, "type", 64)?,
            response: response(&value["response"])?,
        },
        "Network.loadingFinished" => NetworkEvent::Finished {
            request_id: id(&value, "requestId")?,
            encoded_bytes: count(&value["encodedDataLength"])?,
        },
        "Network.loadingFailed" => NetworkEvent::Failed {
            request_id: id(&value, "requestId")?,
            cancelled: optional_bool(&value, "canceled")?,
        },
        _ => {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Unsupported owned network event",
            ));
        }
    };
    Ok(output)
}
fn response(value: &Value) -> Result<NetworkResponse, ApiError> {
    let status = count(&value["status"])?;
    if status > 599 || status != 0 && status < 100 {
        return Err(invalid());
    }
    Ok(NetworkResponse {
        url: text(value, "url", 8192)?,
        status: status as u16,
        mime_type: text(value, "mimeType", 1024)?,
        encoded_bytes: value.get("encodedDataLength").map(count).transpose()?,
        from_disk_cache: optional_bool(value, "fromDiskCache")?,
        from_service_worker: optional_bool(value, "fromServiceWorker")?,
    })
}
fn count(value: &Value) -> Result<u64, ApiError> {
    let number = value.as_f64().ok_or_else(invalid)?;
    if !number.is_finite()
        || !(0.0..=9_007_199_254_740_991.0).contains(&number)
        || number.fract() != 0.0
    {
        return Err(invalid());
    }
    Ok(number as u64)
}
pub(super) fn bounded(value: &Value, limit: usize) -> Result<String, ApiError> {
    let value = value.as_str().ok_or_else(invalid)?;
    if value.len() > limit || value.chars().any(char::is_control) {
        return Err(invalid());
    }
    Ok(value.into())
}
fn text(value: &Value, name: &str, limit: usize) -> Result<String, ApiError> {
    bounded(&value[name], limit)
}
pub(super) fn id(value: &Value, name: &str) -> Result<String, ApiError> {
    let value = text(value, name, 128)?;
    if value.is_empty() {
        return Err(invalid());
    }
    Ok(value)
}
fn optional_id(value: &Value, name: &str) -> Result<Option<String>, ApiError> {
    value.get(name).map(|_| id(value, name)).transpose()
}
fn optional_bool(value: &Value, name: &str) -> Result<Option<bool>, ApiError> {
    value
        .get(name)
        .map(|value| value.as_bool().ok_or_else(invalid))
        .transpose()
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Invalid bounded owned network observation",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn target_projection_keeps_session_distinct_from_target_and_frame() {
        let event = decode(
            "Target.attachedToTarget",
            r#"{"sessionId":"session-a","targetInfo":{"targetId":"target-b","type":"iframe","url":"https://example.invalid/private"},"waitingForDebugger":true}"#,
        )
        .unwrap();
        let value = serde_json::to_value(event).unwrap();
        assert_eq!(value["session_id"], "session-a");
        assert_eq!(value["target_id"], "target-b");
        assert_eq!(value["waiting_for_debugger"], true);
        assert!(value.get("frame_id").is_none());
        assert!(!value.to_string().contains("private"));
        let detached = decode("Target.detachedFromTarget", r#"{"sessionId":"session-a"}"#).unwrap();
        assert!(serde_json::to_value(detached).unwrap()["target_id"].is_null());
    }

    #[test]
    fn frame_attachment_and_swap_reason_preserve_protocol_lineage() {
        let attached = decode(
            "Page.frameAttached",
            r#"{"frameId":"child","parentFrameId":"parent"}"#,
        )
        .unwrap();
        let value = serde_json::to_value(attached).unwrap();
        assert_eq!(value["frame_id"], "child");
        assert_eq!(value["parent_frame_id"], "parent");
        let detached = decode(
            "Page.frameDetached",
            r#"{"frameId":"child","reason":"swap"}"#,
        )
        .unwrap();
        assert_eq!(serde_json::to_value(detached).unwrap()["reason"], "swap");
        assert!(decode("Page.frameDetached", r#"{"frameId":"child"}"#).is_err());
    }

    #[test]
    fn request_projection_keeps_ids_and_actual_referer_without_sensitive_headers() {
        let raw = json!({"requestId":"request-1","frameId":"protocol-frame","loaderId":"loader-1","documentURL":"https://media.example.test/video/fixture","type":"Script","request":{"url":"https://media.example.test/player/assets/scripts.php?v=fixture","method":"GET","headers":{"Referer":"https://media.example.test/video/fixture","Cookie":"fixture-cookie-secret","Authorization":"fixture-auth-secret"}}});
        let event = decode("Network.requestWillBeSent", &raw.to_string()).unwrap();
        let projected = serde_json::to_value(event).unwrap();
        assert_eq!(projected["frame_id"], "protocol-frame");
        assert_eq!(projected["loader_id"], "loader-1");
        assert!(projected["referer"].is_string());
        assert!(!projected.to_string().contains("fixture-cookie-secret"));
        assert!(!projected.to_string().contains("fixture-auth-secret"));
    }
    #[test]
    fn absent_frame_and_empty_worker_loader_are_preserved_without_inference() {
        let raw = json!({"requestId":"worker-request","loaderId":"","documentURL":"","type":"Fetch","request":{"url":"https://cdn.example/a.txt","method":"GET","headers":{}}});
        let event =
            serde_json::to_value(decode("Network.requestWillBeSent", &raw.to_string()).unwrap())
                .unwrap();
        assert!(event["frame_id"].is_null());
        assert!(event["referer"].is_null());
        assert_eq!(event["loader_id"], "");
    }
    #[test]
    fn raw_status_zero_stays_zero_and_fractional_lengths_are_rejected() {
        let raw = json!({"requestId":"request","frameId":"frame","loaderId":"loader","type":"Fetch","response":{"url":"https://cdn.example/a.txt","status":0,"mimeType":"text/plain"}});
        let event =
            serde_json::to_value(decode("Network.responseReceived", &raw.to_string()).unwrap())
                .unwrap();
        assert_eq!(event["response"]["status"], 0);
        assert!(event["response"]["from_disk_cache"].is_null());
        assert!(
            decode(
                "Network.loadingFinished",
                r#"{"requestId":"request","encodedDataLength":1.5}"#
            )
            .is_err()
        );
    }
    #[test]
    fn malformed_and_excessive_events_do_not_copy_raw_data_into_errors() {
        let error = decode(
            "Network.loadingFinished",
            r#"{"requestId":"fixture-secret","encodedDataLength":-1}"#,
        )
        .err()
        .unwrap();
        assert!(!error.message.contains("fixture-secret"));
        assert!(decode("Page.frameNavigated", &"x".repeat(128 * 1024 + 1)).is_err());
        assert!(decode("Runtime.evaluate", "{}").is_err());
    }
}
