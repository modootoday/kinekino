//! Original claims and bounded reply cells. Dropping a receiver never blocks an OS callback.
use super::{
    barrier::Sample,
    queue::{Envelope, SubmitInput},
    results::{FrozenObservation, GrantKey},
    scope::{ConnectionFence, HostScope},
};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        playback_session::{
            GuardedSeekSubmission, NativeGuardAuthority, NativeGuardSnapshot,
            NativeObservationGuard, native_guard_from_authority,
        },
    },
    domain::{api_values::DecimalU64, skip_segments::Seekability},
};
use std::{
    sync::{
        Arc, Condvar, Mutex, Weak,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};
use tokio::sync::mpsc::Sender;

pub(in crate::infrastructure::mpv) struct ObservationReply {
    published: AtomicBool,
    value: Mutex<Option<Result<NativeObservationGuard, ApiError>>>,
    ready: Condvar,
}
impl ObservationReply {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            published: AtomicBool::new(false),
            value: Mutex::new(None),
            ready: Condvar::new(),
        })
    }
    pub fn finish(&self, value: Result<NativeObservationGuard, ApiError>) -> Result<(), ApiError> {
        if self.published.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        let mut slot = self.value.lock().map_err(|_| poisoned())?;
        if slot.is_none() {
            *slot = Some(value);
            self.ready.notify_all();
        }
        Ok(())
    }
}
pub(in crate::infrastructure::mpv) struct ObservationWaiter {
    pub(super) key: Option<GrantKey>,
    pub(super) reply: Arc<ObservationReply>,
    pub(super) deadline: Instant,
}
impl ObservationWaiter {
    /// SDK execution worker only. No Active/registry mutex is held across this wait.
    pub fn wait(mut self) -> Result<NativeObservationGuard, ApiError> {
        let mut slot = self.reply.value.lock().map_err(|_| poisoned())?;
        loop {
            if Instant::now() >= self.deadline {
                if let Some(key) = &self.key {
                    key.expire();
                }
                return Err(ApiError::new(
                    ErrorCode::NeedsInput,
                    "Native observation deadline elapsed",
                ));
            }
            if let Some(result) = slot.take() {
                if result.is_ok() {
                    self.key.take();
                }
                return result;
            }
            let remaining = self.deadline.saturating_duration_since(Instant::now());
            slot = self
                .reply
                .ready
                .wait_timeout(slot, remaining)
                .map_err(|_| poisoned())?
                .0;
        }
    }
}
impl Drop for ObservationWaiter {
    fn drop(&mut self) {
        if let Some(key) = &self.key {
            key.abandon();
        }
    }
}

struct GrantAuthority {
    scope: Weak<HostScope>,
    link: Weak<ConnectionFence>,
    sender: Sender<Envelope>,
    key: GrantKey,
    frozen: FrozenObservation,
}
impl std::fmt::Debug for GrantAuthority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeObservationAuthority")
            .field("identity", &self.frozen.identity)
            .finish_non_exhaustive()
    }
}
impl Drop for GrantAuthority {
    fn drop(&mut self) {
        self.key.abandon();
    }
}
impl NativeGuardAuthority for GrantAuthority {
    fn submit_exported_v1(
        self: Box<Self>,
        position_us: DecimalU64,
        export: crate::application::playback_session::v1::PreparedNativeResultExportV1,
        read_owner: Option<crate::application::playback_session::PreparedNativeExportReadOwner>,
    ) -> Result<crate::application::playback_session::v1::ExportedGuardedSeekV1, ApiError> {
        crate::domain::playback_control::PlaybackControl::Seek { position_us }
            .validate()
            .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
        let scope = self.scope.upgrade().ok_or_else(expired)?;
        scope.require_enabled()?;
        let link = self.link.upgrade().ok_or_else(expired)?;
        scope.registry.admit_wire_v1(
            &scope,
            &self.key,
            &link,
            super::results::WireSubmitInput {
                export,
                target: position_us,
                read_owner,
            },
            |input| {
                self.sender
                    .try_send(Envelope::Submit(input))
                    .map_err(super::queue::send_error)
            },
        )
    }
    fn submit_exported(
        self: Box<Self>,
        position_us: DecimalU64,
        export: crate::application::playback_session::PreparedNativeResultExport,
        read_owner: Option<crate::application::playback_session::PreparedNativeExportReadOwner>,
    ) -> Result<crate::application::playback_session::ExportedGuardedSeekSubmission, ApiError> {
        crate::domain::playback_control::PlaybackControl::Seek { position_us }
            .validate()
            .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
        let scope = self.scope.upgrade().ok_or_else(expired)?;
        scope.require_enabled()?;
        if let Some(owner) = read_owner {
            let link = self.link.upgrade().ok_or_else(expired)?;
            return scope
                .registry
                .admit_routed_export(&self.key, &link, export, owner, |input| {
                    self.sender
                        .try_send(Envelope::Submit(SubmitInput {
                            position_us: position_us.0,
                            ..input
                        }))
                        .map_err(super::queue::send_error)
                });
        }
        let owner = scope.prepare_export_owner()?;
        let link = self.link.upgrade().ok_or_else(expired)?;
        scope
            .registry
            .admit_standalone_export(&self.key, &link, export, owner, |input| {
                self.sender
                    .try_send(Envelope::Submit(SubmitInput {
                        position_us: position_us.0,
                        ..input
                    }))
                    .map_err(super::queue::send_error)
            })
    }
    fn matches_issuer(&self, issuer: &dyn std::any::Any) -> bool {
        issuer
            .downcast_ref::<Weak<HostScope>>()
            .is_some_and(|expected| Weak::ptr_eq(&self.scope, expected))
    }
    fn snapshot(&self) -> NativeGuardSnapshot {
        NativeGuardSnapshot {
            engine_instance_id: DecimalU64(self.frozen.identity.engine),
            resolution_artifact_id: self
                .frozen
                .identity
                .resolution
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
            playback: self.frozen.playback.clone(),
            seekability: Seekability::Yes,
        }
    }
    fn submit(self: Box<Self>, position_us: DecimalU64) -> Result<GuardedSeekSubmission, ApiError> {
        crate::domain::playback_control::PlaybackControl::Seek { position_us }
            .validate()
            .map_err(|message| ApiError::new(ErrorCode::InvalidArgument, message))?;
        let scope = self.scope.upgrade().ok_or_else(expired)?;
        scope.require_enabled()?;
        let link = self.link.upgrade().ok_or_else(expired)?;
        scope.registry.admit_observation(&self.key, &link, |input| {
            self.sender
                .try_send(Envelope::Submit(SubmitInput {
                    position_us: position_us.0,
                    ..input
                }))
                .map_err(super::queue::send_error)
        })
    }
}
pub(super) fn issue(
    scope: &Arc<HostScope>,
    link: &Arc<ConnectionFence>,
    sender: Sender<Envelope>,
    key: &GrantKey,
    sample: Sample,
) -> Result<NativeObservationGuard, ApiError> {
    scope.require_enabled()?;
    let frozen = scope.registry.complete_observation(key, link, sample)?;
    Ok(native_guard_from_authority(Box::new(GrantAuthority {
        scope: Arc::downgrade(scope),
        link: Arc::downgrade(link),
        sender,
        key: key.clone(),
        frozen,
    })))
}
pub(super) fn expired() -> ApiError {
    ApiError::new(
        ErrorCode::SourceExpired,
        "Original native observation issuer expired",
    )
}
fn poisoned() -> ApiError {
    ApiError::new(ErrorCode::Internal, "Native observation reply poisoned")
}

#[cfg(test)]
mod wire_submission_tests {
    use super::*;
    use crate::application::playback_session::{NativeResultProvenance, v1::*};
    fn prepared() -> PreparedNativeResultExportV1 {
        let origin = NativeExportOriginV1::try_from(NativeExportOriginInputV1 {
            registry_generation: "g".into(),
            session: "s".into(),
            pass: DecimalU64(1),
            segment: "t".into(),
            request_revision: DecimalU64(1),
            engine_instance_id: DecimalU64(7),
            resolution_artifact_id: "03".repeat(32),
        })
        .unwrap();
        let annotation = NativeExportAnnotationV1::try_from(NativeExportAnnotationInputV1 {
            annotation_revision: DecimalU64(4),
            verified_revision: DecimalU64(4),
            timebase: crate::domain::skip_segments::SegmentTimebase::EnginePlayback,
        })
        .unwrap();
        PreparedNativeResultExportV1::new(origin, annotation)
    }
    #[test]
    fn wire_packet_metadata_and_caller_share_the_single_original_admission() {
        let host = HostScope::testing_export_owner();
        host.set_issuer_for_test(Ok(
            NativeIssuerScopeV1::from_original_nonce([4; 16]).unwrap()
        ));
        let (guard, _link, mut receiver) = super::super::standalone_export_guard_for_test(&host);
        let output = guard
            .submit_exported_native_v1(DecimalU64(9_000_000), prepared())
            .unwrap();
        let Envelope::Submit(packet) = receiver.try_recv().unwrap() else {
            panic!("original submit packet expected")
        };
        assert_eq!(packet.position_us, output.metadata().actual_target_us().0);
        let (submission, reference, _) = output.into_parts();
        let records = host
            .registry
            .exported_references(NativeResultProvenance {
                registry_generation: "g".into(),
                session: "s".into(),
                pass: 1,
                segment: "t".into(),
                expected_revision: 1,
            })
            .unwrap();
        assert_eq!(records.len(), 1);
        assert!(records[0].ticket.shares_read_cell(&submission.ticket));
        assert!(reference.original_row().get() > 0);
        assert_eq!(
            host.registry.read(&packet.ticket, false).unwrap().state,
            crate::application::playback_session::GuardedSeekState::Queued
        );
    }
    #[test]
    fn failed_enqueue_leaves_no_wire_result_or_consumable_row() {
        let host = HostScope::testing_export_owner();
        host.set_issuer_for_test(Ok(
            NativeIssuerScopeV1::from_original_nonce([5; 16]).unwrap()
        ));
        let issuer = host.export_issuer().unwrap();
        let export = prepared();
        let origin = export.origin().clone();
        let (guard, _link, receiver) = super::super::standalone_export_guard_for_test(&host);
        drop(receiver);
        assert!(
            guard
                .submit_exported_native_v1(DecimalU64(9), export)
                .is_err()
        );
        assert!(issuer.list(&origin).unwrap().items().is_empty());
    }
    #[test]
    fn legacy_rows_stay_hidden_and_guessed_reference_cannot_consume_them() {
        use crate::application::playback_session::PreparedNativeResultExport;
        let host = HostScope::testing_export_owner();
        host.set_issuer_for_test(Ok(
            NativeIssuerScopeV1::from_original_nonce([6; 16]).unwrap()
        ));
        let issuer = host.export_issuer().unwrap();
        let (guard, _link, mut receiver) = super::super::standalone_export_guard_for_test(&host);
        let output = guard
            .submit_exported_native(
                DecimalU64(9),
                PreparedNativeResultExport {
                    provenance: NativeResultProvenance {
                        registry_generation: "g".into(),
                        session: "s".into(),
                        pass: 1,
                        segment: "t".into(),
                        expected_revision: 1,
                    },
                },
            )
            .unwrap();
        assert!(receiver.try_recv().is_ok());
        let origin = prepared().origin().clone();
        assert!(issuer.list(&origin).unwrap().items().is_empty());
        let guessed = NativeResultRefV1::from_original_row(
            issuer.scope(),
            std::num::NonZeroU64::new(1).unwrap(),
        );
        assert_eq!(
            issuer.consume(guessed, &origin).unwrap_err().code,
            ErrorCode::NotFound
        );
        host.registry.read_export(&output.reference, false).unwrap();
    }
    #[test]
    fn original_issuer_expiry_never_retargets_and_foreign_scope_wins_before_expiry() {
        let host = HostScope::testing_export_owner();
        host.set_issuer_for_test(Ok(
            NativeIssuerScopeV1::from_original_nonce([7; 16]).unwrap()
        ));
        let issuer = host.export_issuer().unwrap();
        let origin = prepared().origin().clone();
        let weak = Arc::downgrade(&host);
        drop(host);
        assert!(weak.upgrade().is_none());
        assert_eq!(
            issuer.list(&origin).unwrap_err().code,
            ErrorCode::SourceExpired
        );
        let foreign = NativeResultRefV1::from_original_row(
            NativeIssuerScopeV1::from_original_nonce([8; 16]).unwrap(),
            std::num::NonZeroU64::new(1).unwrap(),
        );
        assert_eq!(
            issuer.consume(foreign, &origin).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
    }
}
