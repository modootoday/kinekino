use crate::domain::api_values::DecimalU64;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Transport-neutral event envelope. Unknown event kinds retain their payload.
/// No runtime event stream is advertised until a host adapter is available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_id: String,
    pub stream_id: String,
    pub sequence: DecimalU64,
    pub emitted_at: String,
    pub request_id: Option<String>,
    pub job_id: Option<String>,
    pub session_id: Option<String>,
    pub kind: String,
    pub payload: Value,
}
