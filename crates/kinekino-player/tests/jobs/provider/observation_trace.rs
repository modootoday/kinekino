//! Bounded read-only observation metadata; no requests, endpoints, URLs or payload bytes.
use super::*;
use kinekino_player::application::api::{ApiError, ErrorCode};
use serde::Serialize;
use std::io::{self, Write};

const MAX_EVENTS: usize = 512;
#[derive(Serialize)]
struct Event {
    sequence: u64,
    elapsed_us: u128,
    phase: &'static str,
    state: Option<JobState>,
    code: Option<ErrorCode>,
    delivery: Option<&'static str>,
    operation: Option<&'static str>,
    diagnostic_phase: Option<&'static str>,
}
pub(super) struct ObservationTrace {
    started: Instant,
    sequence: u64,
    events: Vec<Event>,
    truncated: bool,
}
impl ObservationTrace {
    pub(super) fn new() -> Self {
        Self {
            started: Instant::now(),
            sequence: 0,
            events: Vec::new(),
            truncated: false,
        }
    }
    fn push(&mut self, phase: &'static str, state: Option<JobState>, error: Option<&ApiError>) {
        if self.events.len() == MAX_EVENTS {
            self.truncated = true;
            return;
        }
        let delivery = error.and_then(|error| match error.details["delivery"].as_str() {
            Some("execution_unknown") => Some("execution_unknown"),
            Some("not_dispatched") => Some("not_dispatched"),
            _ => None,
        });
        let operation =
            error.and_then(
                |error| match error.details["diagnostic"]["operation"].as_str() {
                    Some("read_response") => Some("read_response"),
                    Some("write_request") => Some("write_request"),
                    Some("open_pipe") => Some("open_pipe"),
                    Some("verify_server") => Some("verify_server"),
                    _ => None,
                },
            );
        let diagnostic_phase =
            error.and_then(
                |error| match error.details["diagnostic"]["phase"].as_str() {
                    Some("client_deadline") => Some("client_deadline"),
                    Some("client") => Some("client"),
                    Some("peer_identity") => Some("peer_identity"),
                    _ => None,
                },
            );
        self.events.push(Event {
            sequence: self.sequence,
            elapsed_us: self.started.elapsed().as_micros(),
            phase,
            state,
            code: error.map(|error| error.code),
            delivery,
            operation,
            diagnostic_phase,
        });
    }
    pub(super) fn begin_read(&mut self) {
        self.sequence = self.sequence.saturating_add(1);
        self.push("get_begin", None, None);
    }
    pub(super) fn observed(&mut self, state: JobState) {
        self.push("get_returned", Some(state), None);
    }
    pub(super) fn failed(&mut self, error: &ApiError) {
        self.push("get_failed", None, Some(error));
    }
    fn bytes(&self, returned_record: bool) -> io::Result<Vec<u8>> {
        let bytes = serde_json::to_vec(
            &json!({"schema_version":1,"elapsed_us":self.started.elapsed().as_micros(),"truncated":self.truncated,"observation_returned_record":returned_record,"events":self.events}),
        )?;
        if bytes.len() > 128 * 1024 {
            return Err(io::Error::other("Job observation trace exceeds byte limit"));
        }
        Ok(bytes)
    }
    pub(super) fn finish(
        self,
        result: Result<JobRecord, Box<super::super::kinekino_process::CliFailure>>,
        root: &std::path::Path,
        id: &JobId,
    ) -> Result<JobRecord, Box<super::super::kinekino_process::CliFailure>> {
        let written = self.bytes(result.is_ok()).and_then(|bytes| {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join(format!("logs/job-observation-{}.json", id.as_str())))?
                .write_all(&bytes)
        });
        match (result, written) {
            (Err(mut original), Err(error)) => {
                original.secondary.push(format!(
                    "Job observation diagnostic failed: {:?}",
                    error.kind()
                ));
                Err(original)
            }
            (original, Ok(())) => original,
            (Ok(_), Err(error)) => Err(super::super::kinekino_process::CliFailure::not_spawned(
                "job observation diagnostic",
                error,
                root,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostic_is_bounded_and_never_records_error_payload_or_url() {
        kinekino_platform::install();
        let mut trace = ObservationTrace::new();
        let mut error = ApiError::new(ErrorCode::Timeout, "PRIVATE_TOKEN https://private.invalid");
        error.details = json!({"delivery":"execution_unknown","diagnostic":{"operation":"read_response","phase":"client_deadline","payload":"PRIVATE_TOKEN"}});
        trace.begin_read();
        trace.observed(JobState::Running);
        trace.begin_read();
        trace.failed(&error);
        assert_eq!(
            trace
                .events
                .iter()
                .map(|event| (event.sequence, event.phase))
                .collect::<Vec<_>>(),
            vec![
                (1, "get_begin"),
                (1, "get_returned"),
                (2, "get_begin"),
                (2, "get_failed")
            ]
        );
        for _ in 0..MAX_EVENTS {
            trace.begin_read();
            trace.failed(&error);
        }
        assert_eq!(trace.events.len(), MAX_EVENTS);
        assert!(trace.truncated);
        let bytes = trace.bytes(false).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains("PRIVATE_TOKEN"));
        assert!(!text.contains("private.invalid"));
        assert!(text.contains("execution_unknown"));
        assert!(text.contains("read_response"));
    }
    #[test]
    fn diagnostic_write_failure_preserves_original_failure_and_adds_secondary() {
        kinekino_platform::install();
        let trace = ObservationTrace::new();
        let original = super::super::super::kinekino_process::CliFailure::not_spawned(
            "job observation",
            io::Error::new(io::ErrorKind::TimedOut, "original SDK timeout"),
            std::path::Path::new("."),
        );
        let address = std::ptr::from_ref(&*original);
        let id = "job-00000000000000000000000000000001"
            .to_owned()
            .try_into()
            .unwrap();
        let root = std::env::temp_dir().join(format!(
            "absent-observation-trace-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let failure = trace.finish(Err(original), &root, &id).unwrap_err();
        assert_eq!(std::ptr::from_ref(&*failure), address);
        assert_eq!(failure.stage, "job observation");
        assert_eq!(failure.error.kind(), io::ErrorKind::TimedOut);
        assert_eq!(failure.error.to_string(), "original SDK timeout");
        assert_eq!(failure.secondary.len(), 1);
    }
}
