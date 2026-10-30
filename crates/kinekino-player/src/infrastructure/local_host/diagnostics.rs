use crate::application::api::{ApiError, ErrorCode};
use serde::Serialize;
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Serialize)]
pub struct TransportEvent {
    pub elapsed_us: u128,
    pub phase: &'static str,
    pub operation: &'static str,
    pub error_kind: Option<String>,
    pub win32_error: Option<i32>,
    pub check_operation: Option<String>,
}
struct State {
    start: Instant,
    events: Mutex<Vec<TransportEvent>>,
    changed: Condvar,
    truncated: AtomicBool,
}
/// Explicit, bounded, in-memory observation. Never records endpoint, request or peer values.
#[derive(Clone)]
pub struct TransportDiagnostics(Arc<State>);
impl Default for TransportDiagnostics {
    fn default() -> Self {
        Self(Arc::new(State {
            start: Instant::now(),
            events: Mutex::new(Vec::new()),
            changed: Condvar::new(),
            truncated: AtomicBool::new(false),
        }))
    }
}
impl TransportDiagnostics {
    pub fn truncated(&self) -> bool {
        self.0.truncated.load(Ordering::Acquire)
    }
    pub fn snapshot(&self) -> Vec<TransportEvent> {
        self.0
            .events
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
    pub fn wait_for(&self, operation: &str, after: usize, timeout: Duration) -> bool {
        let events = self.0.events.lock().unwrap_or_else(|e| e.into_inner());
        let (events, _) = self
            .0
            .changed
            .wait_timeout_while(events, timeout, |events| {
                !events.iter().skip(after).any(|e| e.operation == operation)
            })
            .unwrap_or_else(|e| e.into_inner());
        events.iter().skip(after).any(|e| e.operation == operation)
    }
    pub(super) fn record(
        &self,
        phase: &'static str,
        operation: &'static str,
        error: Option<&std::io::Error>,
    ) {
        let mut events = self.0.events.lock().unwrap_or_else(|e| e.into_inner());
        if events.len() < 1024 {
            events.push(TransportEvent {
                elapsed_us: self.0.start.elapsed().as_micros(),
                phase,
                operation,
                error_kind: error.map(|e| format!("{:?}", e.kind())),
                win32_error: error.and_then(std::io::Error::raw_os_error),
                check_operation: None,
            });
            self.0.changed.notify_all();
        } else {
            self.0.truncated.store(true, Ordering::Release);
        }
    }
}
pub(super) fn emit_api(
    trace: &Option<TransportDiagnostics>,
    phase: &'static str,
    operation: &'static str,
    error: &ApiError,
) {
    if let Some(trace) = trace {
        let mut events = trace.0.events.lock().unwrap_or_else(|e| e.into_inner());
        if events.len() < 1024 {
            events.push(TransportEvent {
                elapsed_us: trace.0.start.elapsed().as_micros(),
                phase,
                operation,
                error_kind: Some(format!("{:?}", error.code)),
                win32_error: error.details["diagnostic"]["win32_error"]
                    .as_i64()
                    .and_then(|v| i32::try_from(v).ok()),
                check_operation: error.details["diagnostic"]["operation"]
                    .as_str()
                    .map(str::to_owned),
            });
            trace.0.changed.notify_all();
        } else {
            trace.0.truncated.store(true, Ordering::Release);
        }
    }
}
pub(super) fn emit(
    trace: &Option<TransportDiagnostics>,
    phase: &'static str,
    operation: &'static str,
    error: Option<&std::io::Error>,
) {
    if let Some(trace) = trace {
        trace.record(phase, operation, error);
    }
}
pub(super) fn annotate(
    mut error: ApiError,
    phase: &'static str,
    operation: &'static str,
    errno: Option<i32>,
) -> ApiError {
    if !error.details.is_object() {
        error.details = serde_json::json!({});
    }
    error.details["diagnostic"] =
        serde_json::json!({"phase":phase,"operation":operation,"win32_error":errno});
    error
}
pub(super) fn denied(operation: &'static str, errno: Option<i32>) -> ApiError {
    annotate(
        ApiError::new(
            ErrorCode::PermissionDenied,
            "Local host identity or security check failed",
        ),
        "peer_identity",
        operation,
        errno,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transport_trace_is_bounded_and_reports_overflow() {
        let trace = TransportDiagnostics::default();
        for _ in 0..1030 {
            trace.record("host", "read_response", None);
        }
        assert_eq!(trace.snapshot().len(), 1024);
        assert!(trace.truncated());
        assert!(
            trace
                .snapshot()
                .iter()
                .all(|event| event.phase == "host" && event.operation == "read_response")
        );
    }
}
