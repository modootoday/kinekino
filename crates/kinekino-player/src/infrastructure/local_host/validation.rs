//! Opt-in validation instrumentation; never a request capability or alternate SDK.
#[cfg(not(debug_assertions))]
compile_error!("validation-export-delivery-loss is forbidden in standard release builds");

use crate::application::{
    api::{ApiError, ErrorCode, Response},
    local_host::encode_bounded,
    playback_session::v1,
    skip_segments::host::{SKIP_EXPORT, SkipExportAction, SkipExportResult},
};
use serde::{Deserialize, Serialize};
use std::{
    io,
    sync::{Arc, Mutex},
    time::Instant,
};
pub const EVENT_BYTES: usize = 2048;
pub const MAX_EVENTS: usize = 16;
pub const TRACE_BYTES: usize = EVENT_BYTES * MAX_EVENTS;
pub type CounterProbe = Arc<dyn Fn() -> Result<(u64, u64), ApiError> + Send + Sync>;
pub type ProofSink = Arc<dyn Fn(&[u8]) -> io::Result<()> + Send + Sync>;
pub fn validate_request_id(value: &str) -> Result<(), ApiError> {
    if value.len() > 128 || !crate::application::api::commands::valid_request_id(value) {
        Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Invalid explicit validation request id",
        ))
    } else {
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum CounterObservation {
    Known { attempts: u64, enqueued: u64 },
    Unknown { code: ErrorCode },
}
fn counts(probe: &CounterProbe) -> CounterObservation {
    match probe() {
        Ok((attempts, enqueued)) => CounterObservation::Known { attempts, enqueued },
        Err(error) => CounterObservation::Unknown { code: error.code },
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryPhase {
    Unadmitted,
    ProofUnavailable,
    InjectedBeforeFrame,
    Final,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeliveryEvent {
    pub phase: DeliveryPhase,
    pub request_id: String,
    pub command: String,
    pub matched_dispatches: u64,
    pub matched_overflow: bool,
    pub injected: bool,
    pub frame_bytes_sent: u64,
    pub elapsed_us: u64,
    pub before: CounterObservation,
    pub after: CounterObservation,
    pub reference: Option<v1::NativeResultRefV1>,
    pub metadata: Option<v1::NativeExportMetadataV1>,
    pub post_admission_error: bool,
}
#[derive(Default)]
struct State {
    claimed: bool,
    injected: bool,
    matched: u64,
    matched_overflow: bool,
    events: usize,
    proof_failed: bool,
    reference: Option<v1::NativeResultRefV1>,
    metadata: Option<v1::NativeExportMetadataV1>,
    post_error: bool,
}
pub struct ValidationDeliveryLoss {
    request_id: String,
    probe: CounterProbe,
    sink: ProofSink,
    before: CounterObservation,
    start: Instant,
    state: Mutex<State>,
}
impl ValidationDeliveryLoss {
    pub fn new(request_id: String, probe: CounterProbe, sink: ProofSink) -> Result<Self, ApiError> {
        validate_request_id(&request_id)?;
        let before = counts(&probe);
        Ok(Self {
            request_id: request_id.into_boxed_str().into_string(),
            probe,
            sink,
            before,
            start: Instant::now(),
            state: Mutex::new(State::default()),
        })
    }
    fn event(
        &self,
        phase: DeliveryPhase,
        after: CounterObservation,
        state: &State,
    ) -> DeliveryEvent {
        DeliveryEvent {
            phase,
            request_id: self.request_id.clone(),
            command: SKIP_EXPORT.into(),
            matched_dispatches: state.matched,
            matched_overflow: state.matched_overflow,
            injected: state.injected,
            frame_bytes_sent: 0,
            elapsed_us: u64::try_from(self.start.elapsed().as_micros()).unwrap_or(u64::MAX),
            before: self.before,
            after,
            reference: state.reference,
            metadata: state.metadata.clone(),
            post_admission_error: state.post_error,
        }
    }
    fn emit(&self, event: &DeliveryEvent) -> io::Result<()> {
        let mut bytes = encode_bounded(event, EVENT_BYTES - 1)
            .map_err(|_| io::Error::other("Validation event budget exceeded"))?;
        bytes.push(b'\n');
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| io::Error::other("Validation state poisoned"))?;
            if state.events >= MAX_EVENTS {
                state.proof_failed = true;
                return Err(io::Error::other("Validation trace budget exhausted"));
            }
            state.events += 1;
        }
        (self.sink)(&bytes)
    }
    /// Called only after the existing SDK execution, before any response frame write.
    pub(super) fn after_response(
        &self,
        request_id: &str,
        command: &str,
        response: &Response,
    ) -> bool {
        if request_id != self.request_id || command != SKIP_EXPORT {
            return false;
        }
        {
            let Ok(mut state) = self.state.lock() else {
                return false;
            };
            match state.matched.checked_add(1) {
                Some(next) => state.matched = next,
                None => state.matched_overflow = true,
            };
            if state.claimed {
                return false;
            }
            state.claimed = true;
        }
        let after = counts(&self.probe);
        let admitted = if response.ok
            && response.error.is_none()
            && response.request_id == request_id
            && response.api_version == crate::application::api::API_VERSION
        {
            response
                .data
                .as_ref()
                .and_then(|data| v1::encode_native_export_v1(data).ok())
                .and_then(|bytes| serde_json::from_slice::<SkipExportResult>(&bytes).ok())
                .and_then(|result| match result.result {
                    SkipExportAction::Admitted {
                        reference,
                        metadata,
                        ..
                    } => Some((reference, metadata, false)),
                    SkipExportAction::AdmittedWithError {
                        reference,
                        metadata,
                        ..
                    } => Some((reference, metadata, true)),
                    _ => None,
                })
        } else {
            None
        };
        let event = {
            let Ok(mut state) = self.state.lock() else {
                return false;
            };
            if let Some((reference, metadata, post_error)) = admitted {
                state.reference = Some(reference);
                state.metadata = Some(metadata);
                state.post_error = post_error;
            } else {
                let event = self.event(DeliveryPhase::Unadmitted, after, &state);
                drop(state);
                let _ = self.emit(&event);
                return false;
            }
            let eligible = self.before
                == CounterObservation::Known {
                    attempts: 0,
                    enqueued: 0,
                }
                && after
                    == CounterObservation::Known {
                        attempts: 1,
                        enqueued: 1,
                    }
                && !state.matched_overflow;
            self.event(
                if eligible {
                    DeliveryPhase::InjectedBeforeFrame
                } else {
                    DeliveryPhase::ProofUnavailable
                },
                after,
                &state,
            )
        };
        let eligible = event.phase == DeliveryPhase::InjectedBeforeFrame;
        if self.emit(&event).is_err() {
            if let Ok(mut state) = self.state.lock() {
                state.proof_failed = true;
            }
            return false;
        }
        if eligible && let Ok(mut state) = self.state.lock() {
            state.injected = true;
            return true;
        }
        false
    }
    pub fn finish(&self) -> io::Result<()> {
        let after = counts(&self.probe);
        let event = {
            let state = self
                .state
                .lock()
                .map_err(|_| io::Error::other("Validation state poisoned"))?;
            if state.proof_failed {
                return Err(io::Error::other(
                    "Original validation proof was unavailable",
                ));
            }
            self.event(DeliveryPhase::Final, after, &state)
        };
        self.emit(&event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    fn response() -> Response {
        let origin = serde_json::json!({"registry_generation":"g","session":"s","pass":"1","segment":"op","request_revision":"3","engine_instance_id":"7","resolution_artifact_id":"a".repeat(64)});
        Response {
            api_version: crate::application::api::API_VERSION,
            request_id: "validation-request".into(),
            ok: true,
            error: None,
            data: Some(
                serde_json::json!({"session_state":{"session":{"skip_session_id":"s","binding":{"engine_instance_id":"7","resolution_artifact_id":"a".repeat(64)}},"revision":"4","pass_id":"1"},"result":{"state":"admitted","receipt":{"acceptance":"accepted","completion":"unobserved","engine_instance_id":"7"},"reference":"nsr1.abcdefabcdefabcdefabcdefabcdefab.0000000000000001","metadata":{"origin":origin,"annotation":{"annotation_revision":"1","verified_revision":"1","timebase":"engine_playback"},"actual_target_us":"10000"}}}),
            ),
        }
    }
    #[test]
    fn one_original_admission_injects_once_and_records_final_count() {
        let counter = Arc::new(AtomicU64::new(0));
        let sample = counter.clone();
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let captured = bytes.clone();
        let gate = ValidationDeliveryLoss::new(
            "validation-request".into(),
            Arc::new(move || {
                let value = sample.load(Ordering::SeqCst);
                Ok((value, value))
            }),
            Arc::new(move |data| {
                captured.lock().unwrap().extend_from_slice(data);
                Ok(())
            }),
        )
        .unwrap();
        counter.store(1, Ordering::SeqCst);
        assert!(gate.after_response("validation-request", SKIP_EXPORT, &response()));
        assert!(!gate.after_response("validation-request", SKIP_EXPORT, &response()));
        gate.finish().unwrap();
        let bytes = bytes.lock().unwrap();
        assert!(bytes.len() <= TRACE_BYTES);
        let events: Vec<DeliveryEvent> = bytes
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice(line).unwrap())
            .collect();
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].matched_dispatches, 2);
        assert!(events[1].injected);
        assert_eq!(
            events[1].after,
            CounterObservation::Known {
                attempts: 1,
                enqueued: 1
            }
        );
    }
    #[test]
    fn no_admission_unknown_counts_and_sink_failure_never_forge_injection() {
        let failed = Response::failure(
            "validation-request".into(),
            ApiError::new(ErrorCode::Unsupported, "original"),
        );
        let gate = ValidationDeliveryLoss::new(
            "validation-request".into(),
            Arc::new(|| Ok((0, 0))),
            Arc::new(|_| Ok(())),
        )
        .unwrap();
        assert!(!gate.after_response("validation-request", SKIP_EXPORT, &failed));
        assert!(!gate.after_response("validation-request", SKIP_EXPORT, &response()));
        let unknown = ValidationDeliveryLoss::new(
            "validation-request".into(),
            Arc::new(|| Err(ApiError::new(ErrorCode::SourceExpired, "expired"))),
            Arc::new(|_| Ok(())),
        )
        .unwrap();
        assert!(!unknown.after_response("validation-request", SKIP_EXPORT, &response()));
        assert!(matches!(
            unknown.before,
            CounterObservation::Unknown {
                code: ErrorCode::SourceExpired
            }
        ));
        let sink = ValidationDeliveryLoss::new(
            "validation-request".into(),
            Arc::new(|| Ok((0, 0))),
            Arc::new(|_| Err(io::Error::other("original sink failure"))),
        )
        .unwrap();
        assert!(!sink.after_response("validation-request", SKIP_EXPORT, &response()));
        assert!(sink.finish().is_err());
    }
    #[test]
    fn selector_and_event_budget_are_bounded() {
        assert!(
            ValidationDeliveryLoss::new(
                "x".repeat(129),
                Arc::new(|| Ok((0, 0))),
                Arc::new(|_| Ok(()))
            )
            .is_err()
        );
        let gate = ValidationDeliveryLoss::new(
            "validation-request".into(),
            Arc::new(|| Ok((0, 0))),
            Arc::new(|_| Ok(())),
        )
        .unwrap();
        assert!(!gate.after_response("different", SKIP_EXPORT, &response()));
        assert!(!gate.after_response("validation-request", "playback.engine", &response()));
        let mut event = gate.event(DeliveryPhase::Final, gate.before, &State::default());
        event.request_id = "x".repeat(EVENT_BYTES);
        assert!(gate.emit(&event).is_err());
        for _ in 0..MAX_EVENTS {
            gate.finish().unwrap();
        }
        assert!(gate.finish().is_err());
    }
}
