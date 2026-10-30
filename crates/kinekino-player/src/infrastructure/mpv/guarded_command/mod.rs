//! Inert protocol machinery. Owner admission and application capability issuance belong to C.
mod barrier;
mod exports;
mod fence;
#[cfg(test)]
mod fence_tests;
mod issuer;
mod queue;
mod results;
mod scope;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(super) fn standalone_export_guard_for_test(
    host: &std::sync::Arc<HostScope>,
) -> (
    crate::application::playback_session::NativeObservationGuard,
    std::sync::Arc<scope::ConnectionFence>,
    tokio::sync::mpsc::Receiver<queue::Envelope>,
) {
    let link = host.connect(7).unwrap();
    let (sender, receiver) = tokio::sync::mpsc::channel(64);
    let guard = standalone_export_tests::guard(host, &link, sender);
    (guard, link, receiver)
}

#[cfg(test)]
mod standalone_export_tests {
    use super::*;
    use crate::application::playback_session::{
        NativeObservationGuard, NativeResultProvenance, PreparedNativeResultExport,
    };
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };

    fn provenance() -> NativeResultProvenance {
        NativeResultProvenance {
            registry_generation: "skip-generation".into(),
            session: "session-1".into(),
            pass: 1,
            segment: "segment-1".into(),
            expected_revision: 1,
        }
    }
    fn export() -> PreparedNativeResultExport {
        PreparedNativeResultExport {
            provenance: provenance(),
        }
    }
    pub(super) fn guard(
        host: &Arc<HostScope>,
        link: &Arc<scope::ConnectionFence>,
        sender: tokio::sync::mpsc::Sender<queue::Envelope>,
    ) -> NativeObservationGuard {
        let identity = Identity {
            engine: 7,
            resolution: [3; 32],
            connection: 1,
            lifecycle: 2,
            entry: Some(9),
        };
        let (key, _) = host
            .registry
            .reserve_observation(
                host,
                link,
                identity,
                link.generation(),
                Instant::now() + Duration::from_secs(30),
            )
            .unwrap();
        issuer::issue(
            host,
            link,
            sender,
            &key,
            barrier::Sample {
                entry: 9,
                position: 3.0,
                duration: 60.0,
                paused: false,
                idle: false,
            },
        )
        .unwrap()
    }
    #[test]
    fn original_standalone_authority_exports_same_cell_and_reads_after_close_once() {
        let host = HostScope::testing_export_owner();
        let link = host.connect(7).unwrap();
        let (sender, mut receiver) = tokio::sync::mpsc::channel(64);
        let output = guard(&host, &link, sender)
            .submit_exported_native(crate::domain::api_values::DecimalU64(8_000_000), export())
            .unwrap();
        assert!(matches!(
            receiver.try_recv().unwrap(),
            queue::Envelope::Submit(_)
        ));
        let recovered = host.registry.exported_references(provenance()).unwrap();
        assert_eq!(recovered.len(), 1);
        assert!(
            recovered[0]
                .ticket
                .shares_read_cell(&output.submission.ticket)
        );
        let ticket = host.registry.resolve_export(&recovered[0]).unwrap();
        host.registry.with_row_lock_for_test(|| ());
        assert_eq!(
            ticket.read_original(true).unwrap_err().code,
            ErrorCode::NeedsInput
        );
        host.close(Failure::text(FailureStage::Connection, "synthetic close"))
            .unwrap();
        let original = host.registry.resolve_export(&recovered[0]).unwrap();
        assert!(original.read_original(true).unwrap().state().is_terminal());
        assert_eq!(
            original.read_original(true).unwrap_err().code,
            ErrorCode::NotFound
        );
        assert!(
            host.registry
                .exported_references(provenance())
                .unwrap()
                .is_empty()
        );
    }
    #[test]
    fn enabled_legacy_scope_is_not_standalone_export_authority() {
        let host = HostScope::testing();
        let link = host.connect(7).unwrap();
        let (sender, mut receiver) = tokio::sync::mpsc::channel(64);
        assert_eq!(
            guard(&host, &link, sender.clone())
                .submit_exported_native(crate::domain::api_values::DecimalU64(8_000_000), export())
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
        assert!(receiver.try_recv().is_err());
        guard(&host, &link, sender)
            .submit_native(crate::domain::api_values::DecimalU64(8_000_000))
            .unwrap();
        assert!(matches!(
            receiver.try_recv().unwrap(),
            queue::Envelope::Submit(_)
        ));
    }
    #[test]
    fn foreign_owner_is_rejected_before_enqueue_and_none_cannot_grant_read() {
        let host = HostScope::testing_export_owner();
        let other = HostScope::testing_export_owner();
        let link = host.connect(7).unwrap();
        let identity = Identity {
            engine: 7,
            resolution: [3; 32],
            connection: 1,
            lifecycle: 2,
            entry: Some(9),
        };
        let (key, _) = host
            .registry
            .reserve_observation(
                &host,
                &link,
                identity,
                link.generation(),
                Instant::now() + Duration::from_secs(30),
            )
            .unwrap();
        assert_eq!(
            host.registry
                .admit_standalone_export(
                    &key,
                    &link,
                    export(),
                    other.prepare_export_owner().unwrap(),
                    |_| panic!("foreign owner enqueue")
                )
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
        assert!(HostScope::disabled().prepare_export_owner().is_err());
    }
    #[test]
    fn exported_reference_retains_neither_original_host_nor_rows() {
        let host = HostScope::testing_export_owner();
        let link = host.connect(7).unwrap();
        let (sender, _receiver) = tokio::sync::mpsc::channel(64);
        let output = guard(&host, &link, sender)
            .submit_exported_native(crate::domain::api_values::DecimalU64(8_000_000), export())
            .unwrap();
        let weak = Arc::downgrade(&host);
        drop(link);
        drop(host);
        assert!(weak.upgrade().is_none());
        assert_eq!(
            output
                .reference
                .ticket
                .read_original(false)
                .unwrap_err()
                .code,
            ErrorCode::SourceExpired
        );
    }
    #[test]
    fn lost_response_metadata_keeps_original_after_revision_change() {
        let host = HostScope::testing_export_owner();
        let link = host.connect(7).unwrap();
        let (sender, _receiver) = tokio::sync::mpsc::channel(64);
        let output = guard(&host, &link, sender)
            .submit_exported_native(crate::domain::api_values::DecimalU64(8_000_000), export())
            .unwrap();
        let reference = output.reference.clone();
        drop(output);
        host.registry.invalidate_observations(true).unwrap();
        let recovered = host.registry.exported_references(provenance()).unwrap();
        assert!(reference.same_request(&recovered[0]));
        assert!(
            host.registry
                .resolve_export(&recovered[0])
                .unwrap()
                .read_original(false)
                .is_ok()
        );
    }
    #[test]
    fn trusted_exports_keep_full64_and_only_one_terminal_consumer_wins() {
        let host = HostScope::testing_export_owner();
        let link = host.connect(7).unwrap();
        let (sender, _receiver) = tokio::sync::mpsc::channel(64);
        let mut outputs = Vec::new();
        for _ in 0..64 {
            outputs.push(
                guard(&host, &link, sender.clone())
                    .submit_exported_native(
                        crate::domain::api_values::DecimalU64(8_000_000),
                        export(),
                    )
                    .unwrap(),
            );
        }
        let identity = Identity {
            engine: 7,
            resolution: [3; 32],
            connection: 1,
            lifecycle: 2,
            entry: Some(9),
        };
        assert_eq!(
            host.registry
                .reserve_observation(
                    &host,
                    &link,
                    identity,
                    link.generation(),
                    Instant::now() + Duration::from_secs(30)
                )
                .err()
                .unwrap()
                .code,
            ErrorCode::ResourceExhausted
        );
        let first = host.registry.resolve_export(&outputs[0].reference).unwrap();
        assert_eq!(
            first.read_original(true).unwrap_err().code,
            ErrorCode::NeedsInput
        );
        assert_eq!(
            host.registry
                .exported_references(provenance())
                .unwrap()
                .len(),
            64
        );
        host.close(Failure::text(
            FailureStage::Connection,
            "terminalize synthetic rows",
        ))
        .unwrap();
        let mut workers = Vec::new();
        for _ in 0..2 {
            let registry = host.registry.clone();
            let reference = outputs[0].reference.clone();
            workers.push(std::thread::spawn(move || {
                registry
                    .resolve_export(&reference)
                    .and_then(|ticket| ticket.read_original(true))
            }));
        }
        let outcomes: Vec<_> = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect();
        assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            outcomes
                .iter()
                .filter(|result| matches!(result,Err(error) if error.code==ErrorCode::NotFound))
                .count(),
            1
        );
        assert_eq!(
            host.registry
                .exported_references(provenance())
                .unwrap()
                .len(),
            63
        );
    }
}

use crate::application::api::{ApiError, ErrorCode};
pub(super) use barrier::{Barrier, BarrierUpdate};
pub(super) use fence::Fence;
pub(super) use queue::GuardedQueue;
pub(super) use results::{Failure, FailureStage, Identity, Registry, Ticket};
pub(super) use scope::HostScope;

/// A connection's only wire allocator, shared by subscriptions, legacy and guarded RPCs.
#[derive(Default)]
pub(super) struct RequestIds(i64);
impl RequestIds {
    pub fn reserve(&mut self, count: usize) -> Result<Vec<i64>, ApiError> {
        if count == 0 || count > 8 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Native request batch must contain one to eight IDs",
            ));
        }
        let count = i64::try_from(count).map_err(|_| exhausted())?;
        let end = self.0.checked_add(count).ok_or_else(exhausted)?;
        let ids = (self.0 + 1..=end).collect();
        self.0 = end;
        Ok(ids)
    }
    pub fn next(&mut self) -> Result<i64, ApiError> {
        Ok(self.reserve(1)?[0])
    }
}
fn exhausted() -> ApiError {
    ApiError::new(
        ErrorCode::ResourceExhausted,
        "Native request identifiers exhausted",
    )
}

use crate::application::playback_session::GuardedSeekState;
use serde_json::{Value, json};
use std::{collections::BTreeMap, time::Instant};
pub(super) struct Packet {
    pub wire: Value,
    pub deadline: Instant,
    pub permit: Option<fence::WritePermit>,
    pub ticket: Option<Ticket>,
}
struct Pending {
    ticket: Ticket,
    deadline: Instant,
}
pub(super) struct Worker {
    barrier: Option<(Barrier, queue::ObserveInput)>,
    pending: BTreeMap<i64, Pending>,
    current: Option<(u64, u64)>,
}
impl Worker {
    pub fn new() -> Self {
        Self {
            barrier: None,
            pending: BTreeMap::new(),
            current: None,
        }
    }
    pub fn can_begin(&self) -> bool {
        self.barrier.is_none()
    }
    pub fn begin(
        &mut self,
        input: queue::Envelope,
        ids: &mut RequestIds,
        host: &GuardedQueue,
        now: Instant,
    ) -> Result<Vec<Packet>, ApiError> {
        match input {
            queue::Envelope::Observe(input) => {
                if self.barrier.is_some() {
                    return Err(ApiError::new(
                        ErrorCode::NeedsInput,
                        "Native observation barrier already active",
                    ));
                }
                if now >= input.deadline || host.fence.validate(input.generation).is_err() {
                    host.registry.reject_observation(
                        &input.key,
                        ApiError::new(
                            ErrorCode::Cancelled,
                            "Original observation scope expired or retired",
                        ),
                    )?;
                    return Ok(Vec::new());
                }
                let (barrier, wire) =
                    match Barrier::start(input.identity.clone(), input.deadline, ids) {
                        Ok(value) => value,
                        Err(error) => {
                            host.registry.reject_observation(&input.key, error)?;
                            return Ok(Vec::new());
                        }
                    };
                let packet = Packet {
                    wire,
                    deadline: input.deadline,
                    permit: None,
                    ticket: None,
                };
                self.barrier = Some((barrier, input));
                Ok(vec![packet])
            }
            queue::Envelope::Submit(input) => {
                let rejected = if now >= input.deadline {
                    Some(ApiError::new(
                        ErrorCode::NeedsInput,
                        "Original sample deadline elapsed",
                    ))
                } else if self.current
                    != Some((input.identity.connection, input.identity.lifecycle))
                    || input.identity.engine != host.fence.instance
                    || input.ticket.identity() != &input.identity
                {
                    Some(ApiError::new(
                        ErrorCode::RevisionConflict,
                        "Original native sample identity changed",
                    ))
                } else {
                    host.fence.validate(input.generation).err()
                };
                if let Some(error) = rejected {
                    host.registry.transition(
                        &input.ticket,
                        GuardedSeekState::RejectedBeforeWrite,
                        Some(Failure::api(FailureStage::BeforeWrite, &error)),
                    )?;
                    return Ok(Vec::new());
                }
                let id = ids.next()?;
                let permit =
                    match host
                        .registry
                        .commit(&input.ticket, host.fence.word(), input.generation)
                    {
                        Ok(permit) => permit,
                        Err(error) => {
                            host.registry.transition(
                                &input.ticket,
                                GuardedSeekState::RejectedBeforeWrite,
                                Some(Failure::api(FailureStage::BeforeWrite, &error)),
                            )?;
                            return Ok(Vec::new());
                        }
                    };
                self.pending.insert(
                    id,
                    Pending {
                        ticket: input.ticket.clone(),
                        deadline: input.deadline,
                    },
                );
                Ok(vec![Packet {
                    wire: json!({"command":["seek",input.position_us as f64/1_000_000.0,"absolute+exact"],"request_id":id}),
                    deadline: input.deadline,
                    permit: Some(permit),
                    ticket: Some(input.ticket),
                }])
            }
        }
    }
    pub fn receive(
        &mut self,
        message: &Value,
        generations: Option<(u64, u64)>,
        ids: &mut RequestIds,
        host: &GuardedQueue,
        now: Instant,
    ) -> Result<Vec<Packet>, ApiError> {
        self.current = generations;
        let lifecycle = matches!(
            message["event"].as_str(),
            Some("start-file" | "end-file" | "shutdown")
        );
        let relevant = message["event"].as_str() == Some("property-change")
            && matches!(
                message["name"].as_str(),
                Some(
                    "time-pos"
                        | "duration"
                        | "pause"
                        | "idle-active"
                        | "seekable"
                        | "partially-seekable"
                )
            );
        if lifecycle || relevant {
            host.registry.invalidate_observations(lifecycle)?;
            if let Some((mut barrier, input)) = self.barrier.take() {
                barrier.cancel();
                host.registry.reject_observation(
                    &input.key,
                    ApiError::new(
                        ErrorCode::RevisionConflict,
                        "Native receive changed during observation",
                    ),
                )?;
            }
        }
        if let Some(id) = message["request_id"].as_i64()
            && let Some(pending) = self.pending.remove(&id)
        {
            let (state, failure) = if now >= pending.deadline {
                (
                    GuardedSeekState::OutcomeUnobserved,
                    Some(Failure::text(
                        FailureStage::Deadline,
                        "Native acknowledgement deadline elapsed",
                    )),
                )
            } else if message["error"].as_str() == Some("success") {
                (GuardedSeekState::CommandAccepted, None)
            } else {
                (
                    GuardedSeekState::Failed,
                    Some(Failure::text(
                        FailureStage::Acknowledgement,
                        message["error"]
                            .as_str()
                            .unwrap_or("Missing native acknowledgement status"),
                    )),
                )
            };
            host.registry.transition(&pending.ticket, state, failure)?;
        }
        let Some((mut barrier, input)) = self.barrier.take() else {
            return Ok(Vec::new());
        };
        if !input.key.is_live() {
            barrier.cancel();
            host.registry.reap_observations(now)?;
            return Ok(Vec::new());
        }
        let mut current = input.identity.clone();
        if let Some((connection, lifecycle)) = generations {
            current.connection = connection;
            current.lifecycle = lifecycle;
        } else {
            barrier.cancel();
            host.registry.reject_observation(
                &input.key,
                ApiError::new(
                    ErrorCode::Cancelled,
                    "Original native connection unavailable",
                ),
            )?;
            return Ok(Vec::new());
        }
        match barrier.receive(message, &current, now, ids)? {
            BarrierUpdate::Ignored => {
                self.barrier = Some((barrier, input));
                Ok(Vec::new())
            }
            BarrierUpdate::Send(wires) => {
                let packets = wires
                    .into_iter()
                    .map(|wire| Packet {
                        wire,
                        deadline: input.deadline,
                        permit: None,
                        ticket: None,
                    })
                    .collect();
                self.barrier = Some((barrier, input));
                Ok(packets)
            }
            BarrierUpdate::Rejected => {
                host.registry.reject_observation(
                    &input.key,
                    ApiError::new(ErrorCode::NeedsInput, "Native sampled observation rejected"),
                )?;
                Ok(Vec::new())
            }
            BarrierUpdate::Complete(sample) => {
                host.finish_observation(&input, sample)?;
                Ok(Vec::new())
            }
        }
    }
    pub fn tick(&mut self, host: &GuardedQueue, now: Instant) -> Result<bool, ApiError> {
        host.registry.reap_observations(now)?;
        if self
            .barrier
            .as_ref()
            .is_some_and(|(barrier, input)| barrier.expired(now) || !input.key.is_live())
        {
            let (_, input) = self.barrier.take().expect("Checked observation barrier");
            host.registry.reject_observation(
                &input.key,
                ApiError::new(
                    ErrorCode::NeedsInput,
                    "Native observation expired or abandoned",
                ),
            )?;
        }
        Ok(host
            .registry
            .pending_deadline()?
            .is_some_and(|deadline| now >= deadline))
    }
}
