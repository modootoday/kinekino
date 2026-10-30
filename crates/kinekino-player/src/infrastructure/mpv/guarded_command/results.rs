use super::{
    barrier::Sample,
    issuer::ObservationReply,
    scope::{ConnectionFence, HostScope},
};
use crate::application::{
    api::{ApiError, ErrorCode},
    playback_session::GuardedSeekState,
};
use crate::application::{
    native_playback::NativePlayback, playback_control::ControlReceipt,
    playback_status::PlaybackStatus,
};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU8, Ordering};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, Weak},
    time::Instant,
};
pub(super) const CAPACITY: usize = 64;
pub(super) const ROW_PAYLOAD_BYTES: usize = 8192;
const PREFIX_BYTES: usize = ROW_PAYLOAD_BYTES - 512;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::infrastructure::mpv) struct Identity {
    pub engine: u64,
    pub resolution: [u8; 32],
    pub connection: u64,
    pub lifecycle: u64,
    pub entry: Option<i64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::infrastructure::mpv) enum FailureStage {
    BeforeWrite,
    Write,
    Acknowledgement,
    Connection,
    Deadline,
}
/// No native error/JSON object is retained. UTF-8 storage is bounded before insertion.
#[derive(Clone, Debug)]
pub(in crate::infrastructure::mpv) struct Failure {
    pub code: ErrorCode,
    pub stage: FailureStage,
    pub kind: Option<std::io::ErrorKind>,
    pub raw_os_code: Option<i32>,
    prefix: String,
    pub original_bytes: usize,
    pub sha256: [u8; 32],
    pub truncated: bool,
}
impl Failure {
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
    pub fn text(stage: FailureStage, text: &str) -> Self {
        let mut end = text.len().min(PREFIX_BYTES);
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        Self {
            code: if stage == FailureStage::BeforeWrite {
                ErrorCode::NeedsInput
            } else {
                ErrorCode::EngineFailed
            },
            stage,
            kind: None,
            raw_os_code: None,
            prefix: text[..end].into(),
            original_bytes: text.len(),
            sha256: Sha256::digest(text.as_bytes()).into(),
            truncated: end != text.len(),
        }
    }
    pub fn io(stage: FailureStage, error: &std::io::Error) -> Self {
        // OS errors use a bounded message; custom Error payloads are never formatted/cloned.
        let message = match error.raw_os_error() {
            Some(code) => format!("OS error {code}"),
            None => format!("I/O error {:?}", error.kind()),
        };
        let mut failure = Self::text(stage, &message);
        failure.kind = Some(error.kind());
        failure.raw_os_code = error.raw_os_error();
        failure
    }
    pub fn api(stage: FailureStage, error: &ApiError) -> Self {
        let mut failure = Self::text(stage, &error.message);
        failure.code = error.code;
        failure
    }
}
#[derive(Clone, Debug)]
pub(in crate::infrastructure::mpv) struct Snapshot {
    export: Option<super::exports::ExportRecord>,
    sample_revision: Option<u64>,
    fence_generation: Option<u64>,
    deadline: Option<Instant>,
    pub identity: Identity,
    pub state: GuardedSeekState,
    pub committed: bool,
    pub failure: Option<Failure>,
}
#[derive(Clone)]
pub(in crate::infrastructure::mpv) struct Ticket {
    host: Option<Weak<HostScope>>,
    owner: Weak<Mutex<Rows>>,
    id: u64,
    identity: Identity,
}
impl std::fmt::Debug for Ticket {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativeReadTicket")
            .field("id", &self.id)
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}
impl Ticket {
    fn check_host(&self) -> Result<(), ApiError> {
        if self
            .host
            .as_ref()
            .is_some_and(|host| host.upgrade().is_none())
        {
            return Err(super::issuer::expired());
        }
        Ok(())
    }
    pub(super) fn identity(&self) -> &Identity {
        &self.identity
    }
    pub fn submission(
        &self,
        receipt: crate::application::playback_control::ControlReceipt,
    ) -> Result<crate::application::playback_session::GuardedSeekSubmission, ApiError> {
        crate::application::playback_session::native_seek_submission(
            receipt,
            Arc::new(self.clone()),
        )
    }
}
impl crate::application::playback_session::NativeSeekResultSource for Ticket {
    fn matches_issuer(&self, issuer: &dyn std::any::Any) -> bool {
        self.host.as_ref().is_some_and(|host| {
            issuer
                .downcast_ref::<Weak<HostScope>>()
                .is_some_and(|expected| Weak::ptr_eq(host, expected))
        })
    }
    fn identity(&self) -> (crate::domain::api_values::DecimalU64, String) {
        (
            crate::domain::api_values::DecimalU64(self.identity.engine),
            self.identity
                .resolution
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        )
    }
    fn read(
        &self,
        consume: bool,
    ) -> Result<crate::application::playback_session::NativeSeekResultParts, ApiError> {
        use crate::application::playback_session::{
            GuardedSeekFailureStage as Stage, NativeSeekResultParts,
        };
        self.check_host()?;
        let owner = self.owner.upgrade().ok_or_else(|| {
            ApiError::new(
                ErrorCode::SourceExpired,
                "Original native result host expired",
            )
        })?;
        let snapshot = Registry(owner).read(self, consume)?;
        let failure=snapshot.failure.map(|failure| {
            let stage=match failure.stage {FailureStage::BeforeWrite=>Stage::BeforeWrite,FailureStage::Write=>Stage::Write,FailureStage::Acknowledgement=>Stage::Acknowledgement,FailureStage::Connection=>Stage::Connection,FailureStage::Deadline=>Stage::Deadline};
            // JSON escaping can expand bytes sixfold. A 1KiB display prefix plus fixed
            // metadata stays below the 8KiB response payload budget as well.
            let mut end=failure.prefix.len().min(1024);while !failure.prefix.is_char_boundary(end){end-=1;}
            let mut error=ApiError::new(failure.code,&failure.prefix[..end]);
            let hash:String=failure.sha256.iter().map(|byte|format!("{byte:02x}")).collect();
            error.details=serde_json::json!({"raw_os_code":failure.raw_os_code,"kind":failure.kind.map(|kind|format!("{kind:?}")),"original_bytes":failure.original_bytes,"sha256":hash,"truncated":failure.truncated || end<failure.prefix.len()});
            (stage,error)
        });
        Ok(NativeSeekResultParts {
            state: snapshot.state,
            committed: snapshot.committed,
            failure,
        })
    }
}
#[derive(Default)]
struct Rows {
    #[cfg(feature = "validation-export-delivery-loss")]
    validation: ValidationCounters,
    revision: u64,
    exhausted: bool,
    next: u64,
    rows: BTreeMap<u64, Snapshot>,
    grants: BTreeMap<u64, ObservationGrant>,
}
#[cfg(feature = "validation-export-delivery-loss")]
#[derive(Default)]
struct ValidationCounters {
    attempts: u64,
    enqueued: u64,
    overflowed: bool,
}
#[cfg(feature = "validation-export-delivery-loss")]
impl ValidationCounters {
    fn increment(value: &mut u64, overflowed: &mut bool) {
        match value.checked_add(1) {
            Some(next) => *value = next,
            None => *overflowed = true,
        }
    }
    fn snapshot(&self) -> Result<(u64, u64), ApiError> {
        if self.overflowed {
            Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Validation admission counters overflowed",
            ))
        } else {
            Ok((self.attempts, self.enqueued))
        }
    }
}
const LIVE: u8 = 0;
const ABANDONED: u8 = 1;
const STALE: u8 = 2;
const EXPIRED: u8 = 3;
const CLOSED: u8 = 4;
const CONSUMED: u8 = 5;
#[derive(Clone)]
pub(in crate::infrastructure::mpv) struct GrantKey {
    owner: Weak<Mutex<Rows>>,
    host: Weak<HostScope>,
    id: u64,
    state: Arc<AtomicU8>,
}
impl GrantKey {
    pub fn is_live(&self) -> bool {
        self.state.load(Ordering::Acquire) == LIVE
    }
    pub fn abandon(&self) {
        let _ = self
            .state
            .compare_exchange(LIVE, ABANDONED, Ordering::AcqRel, Ordering::Acquire);
    }
    pub fn expire(&self) {
        let _ = self
            .state
            .compare_exchange(LIVE, EXPIRED, Ordering::AcqRel, Ordering::Acquire);
    }
    fn error(&self) -> Option<ApiError> {
        match self.state.load(Ordering::Acquire) {
            LIVE => None,
            STALE => Some(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original native sample changed",
            )),
            EXPIRED => Some(ApiError::new(
                ErrorCode::NeedsInput,
                "Native observation deadline elapsed",
            )),
            CONSUMED => Some(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native observation already consumed",
            )),
            _ => Some(ApiError::new(
                ErrorCode::Cancelled,
                "Native observation abandoned or closed",
            )),
        }
    }
}
#[derive(Clone)]
pub(in crate::infrastructure::mpv) struct FrozenObservation {
    pub identity: Identity,
    pub generation: u64,
    pub deadline: Instant,
    pub playback: PlaybackStatus,
    pub revision: u64,
}
struct ObservationGrant {
    key: GrantKey,
    identity: Identity,
    generation: u64,
    deadline: Instant,
    reply: Arc<ObservationReply>,
    frozen: Option<FrozenObservation>,
}
#[derive(Clone, Default)]
pub(in crate::infrastructure::mpv) struct Registry(Arc<Mutex<Rows>>);
struct AdmissionOutput {
    submission: crate::application::playback_session::GuardedSeekSubmission,
    reference: Option<crate::application::playback_session::NativeResultReference>,
    wire: Option<crate::application::playback_session::v1::ExportedGuardedSeekV1>,
}
pub(super) struct WireSubmitInput {
    pub export: crate::application::playback_session::v1::PreparedNativeResultExportV1,
    pub target: crate::domain::api_values::DecimalU64,
    pub read_owner: Option<crate::application::playback_session::PreparedNativeExportReadOwner>,
}
impl Registry {
    #[cfg(feature = "validation-export-delivery-loss")]
    pub(in crate::infrastructure::mpv) fn validation_export_counts(
        &self,
    ) -> Result<(u64, u64), ApiError> {
        self.0.lock().map_err(|_| poisoned())?.validation.snapshot()
    }
    #[cfg(test)]
    pub(super) fn set_deadline_for_test(&self, ticket: &Ticket, deadline: Instant) {
        self.0
            .lock()
            .unwrap()
            .rows
            .get_mut(&ticket.id)
            .unwrap()
            .deadline = Some(deadline);
    }
    #[cfg(test)]
    pub(in crate::infrastructure::mpv) fn with_row_lock_for_test<T>(
        &self,
        body: impl FnOnce() -> T,
    ) -> T {
        let _rows = self.0.lock().unwrap();
        body()
    }
    #[cfg(test)]
    pub(super) fn exhaust_nonce_for_test(&self) {
        self.0.lock().unwrap().next = u64::MAX;
    }
    #[cfg(test)]
    pub(super) fn exhaust_revision_for_test(&self) {
        self.0.lock().unwrap().revision = u64::MAX;
    }
    pub fn reserve_observation(
        &self,
        host: &Arc<HostScope>,
        link: &Arc<ConnectionFence>,
        identity: Identity,
        generation: u64,
        deadline: Instant,
    ) -> Result<(GrantKey, Arc<ObservationReply>), ApiError> {
        host.require_enabled()?;
        link.validate(generation)?;
        if !Weak::ptr_eq(&link.host, &Arc::downgrade(host))
            || !Arc::ptr_eq(&self.0, &host.registry.0)
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Foreign logical host scope",
            ));
        }
        if identity.engine != link.instance {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Observation does not belong to this connection",
            ));
        }
        let mut rows = self.0.lock().map_err(|_| poisoned())?;
        if rows.exhausted {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Native sample revision exhausted",
            ));
        }
        if rows.rows.len() + rows.grants.len() >= CAPACITY {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Native observation/result slots full",
            ));
        }
        let id = rows.next.checked_add(1).ok_or_else(|| {
            ApiError::new(
                ErrorCode::ResourceExhausted,
                "Native claim identifiers exhausted",
            )
        })?;
        rows.next = id;
        let key = GrantKey {
            owner: Arc::downgrade(&self.0),
            host: Arc::downgrade(host),
            id,
            state: Arc::new(AtomicU8::new(LIVE)),
        };
        let reply = ObservationReply::new();
        rows.grants.insert(
            id,
            ObservationGrant {
                key: key.clone(),
                identity,
                generation,
                deadline,
                reply: reply.clone(),
                frozen: None,
            },
        );
        Ok((key, reply))
    }
    fn validate_key(&self, key: &GrantKey) -> Result<(), ApiError> {
        if key.host.upgrade().is_none() {
            return Err(super::issuer::expired());
        }
        if !Weak::ptr_eq(&key.owner, &Arc::downgrade(&self.0)) {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Foreign observation issuer",
            ));
        }
        if let Some(error) = key.error() {
            return Err(error);
        }
        Ok(())
    }
    pub fn complete_observation(
        &self,
        key: &GrantKey,
        link: &ConnectionFence,
        sample: Sample,
    ) -> Result<FrozenObservation, ApiError> {
        self.validate_key(key)?;
        if !Weak::ptr_eq(&key.host, &link.host) {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Foreign original connection scope",
            ));
        }
        let mut rows = self.0.lock().map_err(|_| poisoned())?;
        let revision = rows.revision;
        if rows.exhausted {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Native sample revision exhausted",
            ));
        }
        let grant = rows.grants.get_mut(&key.id).ok_or_else(missing)?;
        link.validate(grant.generation)?;
        if Instant::now() >= grant.deadline {
            key.expire();
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Observation deadline elapsed",
            ));
        }
        if grant.frozen.is_some() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Observation already completed",
            ));
        }
        if grant
            .identity
            .entry
            .is_some_and(|entry| entry != sample.entry)
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Observation entry changed",
            ));
        }
        let playback =
            crate::application::playback_status::project_native_status(&NativePlayback {
                connected: true,
                time: Some(sample.position),
                duration: Some(sample.duration),
                paused: sample.paused,
                pause_observed: true,
                idle: sample.idle,
                idle_observed: true,
                ..Default::default()
            });
        if playback.position_us.is_none() || playback.duration_us.is_none() {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Native sample is outside the shared status range",
            ));
        }
        let mut identity = grant.identity.clone();
        identity.entry = Some(sample.entry);
        let frozen = FrozenObservation {
            identity,
            generation: grant.generation,
            deadline: grant.deadline,
            playback,
            revision,
        };
        grant.frozen = Some(frozen.clone());
        Ok(frozen)
    }
    pub fn admit_observation(
        &self,
        key: &GrantKey,
        link: &ConnectionFence,
        send: impl FnOnce(super::queue::SubmitInput) -> Result<(), ApiError>,
    ) -> Result<crate::application::playback_session::GuardedSeekSubmission, ApiError> {
        self.admit_prepared(key, link, None, None, None, send)
            .map(|output| output.submission)
    }
    #[cfg(test)]
    pub(super) fn admit_exported_observation(
        &self,
        key: &GrantKey,
        link: &ConnectionFence,
        export: crate::application::playback_session::PreparedNativeResultExport,
        send: impl FnOnce(super::queue::SubmitInput) -> Result<(), ApiError>,
    ) -> Result<crate::application::playback_session::ExportedGuardedSeekSubmission, ApiError> {
        let output = self.admit_prepared(
            key,
            link,
            Some(super::exports::ExportPreparation::Legacy(export)),
            None,
            None,
            send,
        )?;
        Ok(
            crate::application::playback_session::ExportedGuardedSeekSubmission {
                submission: output.submission,
                reference: output
                    .reference
                    .expect("export was prepared before admission"),
            },
        )
    }
    pub(super) fn admit_standalone_export(
        &self,
        key: &GrantKey,
        link: &ConnectionFence,
        export: crate::application::playback_session::PreparedNativeResultExport,
        owner: super::scope::StandaloneExportOwner,
        send: impl FnOnce(super::queue::SubmitInput) -> Result<(), ApiError>,
    ) -> Result<crate::application::playback_session::ExportedGuardedSeekSubmission, ApiError> {
        if !Weak::ptr_eq(&owner.host, &key.host) {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Foreign standalone export owner",
            ));
        }
        // Keep the original scope outside the row-lock lifetime. Dropping its
        // final strong owner may close the registry and must never run under it.
        let host = owner.host.upgrade().ok_or_else(super::issuer::expired)?;
        host.require_standalone_export()?;
        let output = self.admit_prepared(
            key,
            link,
            Some(super::exports::ExportPreparation::Legacy(export)),
            Some(owner),
            None,
            send,
        )?;
        Ok(
            crate::application::playback_session::ExportedGuardedSeekSubmission {
                submission: output.submission,
                reference: output
                    .reference
                    .expect("standalone export prepared before enqueue"),
            },
        )
    }
    pub(super) fn admit_routed_export(
        &self,
        key: &GrantKey,
        link: &ConnectionFence,
        export: crate::application::playback_session::PreparedNativeResultExport,
        owner: crate::application::playback_session::PreparedNativeExportReadOwner,
        send: impl FnOnce(super::queue::SubmitInput) -> Result<(), ApiError>,
    ) -> Result<crate::application::playback_session::ExportedGuardedSeekSubmission, ApiError> {
        let issuer = owner
            .issuer()
            .downcast_ref::<Weak<HostScope>>()
            .ok_or_else(|| {
                ApiError::new(ErrorCode::RevisionConflict, "Invalid routed export issuer")
            })?;
        if !Weak::ptr_eq(issuer, &key.host) {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Foreign routed export issuer",
            ));
        }
        let _read_owners = owner.hold_for_admission()?;
        let output = self.admit_prepared(
            key,
            link,
            Some(super::exports::ExportPreparation::Legacy(export)),
            None,
            Some(owner),
            send,
        )?;
        Ok(
            crate::application::playback_session::ExportedGuardedSeekSubmission {
                submission: output.submission,
                reference: output
                    .reference
                    .expect("routed export prepared before enqueue"),
            },
        )
    }
    pub(super) fn admit_wire_v1(
        &self,
        host: &Arc<HostScope>,
        key: &GrantKey,
        link: &ConnectionFence,
        input: WireSubmitInput,
        send: impl FnOnce(super::queue::SubmitInput) -> Result<(), ApiError>,
    ) -> Result<crate::application::playback_session::v1::ExportedGuardedSeekV1, ApiError> {
        #[cfg(feature = "validation-export-delivery-loss")]
        if let Ok(mut rows) = self.0.lock() {
            let counts = &mut rows.validation;
            ValidationCounters::increment(&mut counts.attempts, &mut counts.overflowed);
        }
        if !Weak::ptr_eq(&key.host, &Arc::downgrade(host))
            || !Arc::ptr_eq(&self.0, &host.registry.0)
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Foreign original wire issuer",
            ));
        }
        let scope = host.issuer_scope()?;
        let (standalone, routed, lease) = match input.read_owner {
            Some(owner) => {
                let issuer = owner
                    .issuer()
                    .downcast_ref::<Weak<HostScope>>()
                    .ok_or_else(|| {
                        ApiError::new(
                            ErrorCode::RevisionConflict,
                            "Invalid original wire read owner",
                        )
                    })?;
                if !Weak::ptr_eq(issuer, &key.host) {
                    return Err(ApiError::new(
                        ErrorCode::RevisionConflict,
                        "Foreign original wire read owner",
                    ));
                }
                let lease = owner.hold_for_admission()?;
                (None, Some(owner), Some(lease))
            }
            None => (Some(host.prepare_export_owner()?), None, None),
        };
        let preparation =
            super::exports::ExportPreparation::Wire(Box::new(super::exports::WirePreparation {
                export: input.export,
                scope,
                target: input.target,
            }));
        let output = self.admit_prepared(key, link, Some(preparation), standalone, routed, send)?;
        drop(lease);
        Ok(output.wire.expect("wire result prepared before enqueue"))
    }
    fn admit_prepared(
        &self,
        key: &GrantKey,
        link: &ConnectionFence,
        export: Option<super::exports::ExportPreparation>,
        owner: Option<super::scope::StandaloneExportOwner>,
        routed_owner: Option<crate::application::playback_session::PreparedNativeExportReadOwner>,
        send: impl FnOnce(super::queue::SubmitInput) -> Result<(), ApiError>,
    ) -> Result<AdmissionOutput, ApiError> {
        self.validate_key(key)?;
        if !Weak::ptr_eq(&key.host, &link.host) {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Foreign original connection scope",
            ));
        }
        let mut rows = self.0.lock().map_err(|_| poisoned())?;
        if rows.exhausted {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Native sample revision exhausted",
            ));
        }
        let grant = rows.grants.get(&key.id).ok_or_else(missing)?;
        let frozen = grant
            .frozen
            .as_ref()
            .ok_or_else(|| {
                ApiError::new(ErrorCode::NeedsInput, "Native observation is not complete")
            })?
            .clone();
        if rows.revision != frozen.revision {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native sample changed before queue admission",
            ));
        }
        if Instant::now() >= frozen.deadline {
            key.expire();
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Native observation expired before submission",
            ));
        }
        if frozen.identity.engine != link.instance {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Observation belongs to another connection",
            ));
        }
        link.validate(frozen.generation)?;
        let ticket = Ticket {
            owner: Arc::downgrade(&self.0),
            host: Some(key.host.clone()),
            id: key.id,
            identity: frozen.identity.clone(),
        };
        let submission = ticket.submission(ControlReceipt::enqueued(frozen.identity.engine))?;
        let prepared = export
            .map(|prepared| {
                super::exports::PreparedExport::new(
                    prepared,
                    &submission,
                    key.id,
                    owner,
                    routed_owner,
                )
            })
            .transpose()?;
        let (export, position_us, wire) = match prepared {
            Some(prepared) => (Some(prepared.record), prepared.position_us, prepared.wire),
            None => (None, 0, None),
        };
        let reference = export.as_ref().map(super::exports::ExportRecord::reference);
        let packet = super::queue::SubmitInput {
            ticket: ticket.clone(),
            identity: frozen.identity.clone(),
            generation: frozen.generation,
            position_us,
            deadline: frozen.deadline,
        };
        // Allocate the same command row before enqueue, while the original row lock
        // prevents workers/readers from observing this tentative conversion.
        rows.rows.insert(
            key.id,
            Snapshot {
                export,
                sample_revision: Some(frozen.revision),
                fence_generation: Some(frozen.generation),
                deadline: Some(frozen.deadline),
                identity: frozen.identity.clone(),
                state: GuardedSeekState::Queued,
                committed: false,
                failure: None,
            },
        );
        if let Err(error) = send(packet) {
            rows.rows.remove(&key.id);
            rows.grants.remove(&key.id);
            key.abandon();
            return Err(error);
        }
        rows.grants.remove(&key.id);
        key.state.store(CONSUMED, Ordering::Release);
        #[cfg(feature = "validation-export-delivery-loss")]
        if wire.is_some() {
            let counts = &mut rows.validation;
            ValidationCounters::increment(&mut counts.enqueued, &mut counts.overflowed);
        }
        Ok(AdmissionOutput {
            submission,
            reference,
            wire,
        })
    }
    pub fn invalidate_observations(&self, all: bool) -> Result<(), ApiError> {
        let mut rows = self.0.lock().map_err(|_| poisoned())?;
        let Some(revision) = rows.revision.checked_add(1) else {
            rows.exhausted = true;
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Native sample revision exhausted",
            ));
        };
        rows.revision = revision;
        for grant in rows
            .grants
            .values()
            .filter(|grant| all || grant.frozen.is_some())
        {
            let _ =
                grant
                    .key
                    .state
                    .compare_exchange(LIVE, STALE, Ordering::AcqRel, Ordering::Acquire);
        }
        Ok(())
    }
    pub fn reap_observations(&self, now: Instant) -> Result<(), ApiError> {
        let mut rows = self.0.lock().map_err(|_| poisoned())?;
        let expired: Vec<_> = rows
            .grants
            .iter()
            .filter(|(_, grant)| grant.key.error().is_some() || now >= grant.deadline)
            .map(|(id, _)| *id)
            .collect();
        for id in expired {
            if let Some(grant) = rows.grants.remove(&id) {
                grant.key.expire();
                let error = grant
                    .key
                    .error()
                    .unwrap_or_else(|| ApiError::new(ErrorCode::Cancelled, "Observation ended"));
                grant.reply.finish(Err(error))?;
            }
        }
        Ok(())
    }
    pub fn reject_observation(&self, key: &GrantKey, error: ApiError) -> Result<(), ApiError> {
        let mut rows = self.0.lock().map_err(|_| poisoned())?;
        if let Some(grant) = rows.grants.remove(&key.id) {
            grant.key.abandon();
            grant.reply.finish(Err(error))?;
        }
        Ok(())
    }
    pub fn close_connection(&self, instance: u64, failure: Failure) -> Result<(), ApiError> {
        self.close_matching(Some(instance), failure)
    }
    pub fn commit(
        &self,
        ticket: &Ticket,
        fence: &Arc<super::fence::Fence>,
        generation: u64,
    ) -> Result<super::fence::WritePermit, ApiError> {
        self.validate(ticket)?;
        let mut rows = self.0.lock().map_err(|_| poisoned())?;
        if rows.exhausted {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Native sample revision exhausted",
            ));
        }
        let revision = rows.revision;
        let row = rows.rows.get_mut(&ticket.id).ok_or_else(missing)?;
        if row
            .sample_revision
            .is_some_and(|original| original != revision)
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native sample changed before write commit",
            ));
        }
        if row.state != GuardedSeekState::Queued {
            return Err(ApiError::new(
                ErrorCode::Cancelled,
                "Native request is no longer pending admission",
            ));
        }
        if row
            .fence_generation
            .is_some_and(|original| original != generation)
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Command changed its original fence generation",
            ));
        }
        // close first closes the same fence, then takes this lock. A winning CAS
        // is therefore recorded before close can classify this original row.
        // The dequeue timestamp may precede a wait for this row lock. Use the
        // current clock at the final admission boundary, not the caller's now.
        if row
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            let error = ApiError::new(
                ErrorCode::NeedsInput,
                "Original sample deadline elapsed before write commit",
            );
            row.state = GuardedSeekState::RejectedBeforeWrite;
            row.failure = Some(Failure::api(FailureStage::BeforeWrite, &error));
            return Err(error);
        }
        let permit = fence.commit(generation)?;
        row.committed = true;
        row.state = GuardedSeekState::WriteCommitted;
        Ok(permit)
    }
    #[cfg(test)]
    pub fn reserve(&self, identity: Identity) -> Result<Ticket, ApiError> {
        self.reserve_deadline(identity, None)
    }
    pub fn pending_deadline(&self) -> Result<Option<Instant>, ApiError> {
        let rows = self.0.lock().map_err(|_| poisoned())?;
        Ok(rows
            .rows
            .values()
            .filter(|row| !row.state.is_terminal())
            .filter_map(|row| row.deadline)
            .chain(rows.grants.values().map(|grant| grant.deadline))
            .min())
    }
    #[cfg(test)]
    pub fn reserve_deadline(
        &self,
        identity: Identity,
        deadline: Option<Instant>,
    ) -> Result<Ticket, ApiError> {
        let mut rows = self.0.lock().map_err(|_| poisoned())?;
        if rows.rows.len() + rows.grants.len() >= CAPACITY {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Native result slots full",
            ));
        }
        let id = rows.next.checked_add(1).ok_or_else(|| {
            ApiError::new(
                ErrorCode::ResourceExhausted,
                "Native result identifiers exhausted",
            )
        })?;
        rows.next = id;
        rows.rows.insert(
            id,
            Snapshot {
                export: None,
                sample_revision: None,
                fence_generation: None,
                deadline,
                identity: identity.clone(),
                state: GuardedSeekState::Queued,
                committed: false,
                failure: None,
            },
        );
        Ok(Ticket {
            host: None,
            owner: Arc::downgrade(&self.0),
            id,
            identity,
        })
    }
    fn validate(&self, ticket: &Ticket) -> Result<(), ApiError> {
        ticket.check_host()?;
        if ticket.owner.strong_count() == 0 {
            return Err(ApiError::new(
                ErrorCode::SourceExpired,
                "Original native result host expired",
            ));
        }
        if !Weak::ptr_eq(&ticket.owner, &Arc::downgrade(&self.0)) {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Foreign native result issuer",
            ));
        }
        Ok(())
    }
    pub(in crate::infrastructure::mpv) fn exported_references(
        &self,
        provenance: crate::application::playback_session::NativeResultProvenance,
    ) -> Result<Vec<crate::application::playback_session::NativeResultReference>, ApiError> {
        let rows = self.0.lock().map_err(|_| poisoned())?;
        Ok(rows
            .rows
            .values()
            .filter_map(|row| row.export.as_ref())
            .filter(|record| record.provenance() == provenance)
            .map(super::exports::ExportRecord::reference)
            .collect())
    }
    #[cfg(test)]
    pub(in crate::infrastructure::mpv) fn resolve_export(
        &self,
        reference: &crate::application::playback_session::NativeResultReference,
    ) -> Result<crate::application::playback_session::NativeSeekTicket, ApiError> {
        let record = {
            let rows = self.0.lock().map_err(|_| poisoned())?;
            rows.rows
                .values()
                .filter_map(|row| row.export.as_ref())
                .find(|record| record.matches(reference))
                .cloned()
                .ok_or_else(missing)?
        };
        record.require_read()?;
        Ok(record.reference().ticket)
    }
    pub(in crate::infrastructure::mpv) fn read_export(
        &self,
        reference: &crate::application::playback_session::NativeResultReference,
        consume: bool,
    ) -> Result<crate::application::playback_session::GuardedSeekResult, ApiError> {
        let record = {
            let rows = self.0.lock().map_err(|_| poisoned())?;
            rows.rows
                .values()
                .filter_map(|row| row.export.as_ref())
                .find(|record| record.matches(reference))
                .cloned()
                .ok_or_else(missing)?
        };
        record.read(consume)
    }
    pub(super) fn list_wire_v1(
        &self,
        scope: crate::application::playback_session::v1::NativeIssuerScopeV1,
        origin: &crate::application::playback_session::v1::NativeExportOriginV1,
    ) -> Result<crate::application::playback_session::v1::NativeExportListV1, ApiError> {
        let items = {
            let rows = self.0.lock().map_err(|_| poisoned())?;
            rows.rows
                .iter()
                .filter_map(|(id, row)| {
                    row.export
                        .as_ref()
                        .filter(|record| record.wire_origin_matches(origin))
                        .map(|record| record.wire_item(scope, *id, row.state))
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        crate::application::playback_session::v1::NativeExportListV1::new(scope, items)
    }
    pub(super) fn read_wire_v1(
        &self,
        reference: crate::application::playback_session::v1::NativeResultRefV1,
        origin: &crate::application::playback_session::v1::NativeExportOriginV1,
        consume: bool,
    ) -> Result<crate::application::playback_session::v1::NativeExportResultV1, ApiError> {
        let id = reference.original_row().get();
        let record = {
            let rows = self.0.lock().map_err(|_| poisoned())?;
            let record = rows
                .rows
                .get(&id)
                .and_then(|row| row.export.as_ref())
                .filter(|record| record.has_wire())
                .ok_or_else(missing)?;
            if !record.wire_origin_matches(origin) {
                return Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Original export origin differs",
                ));
            }
            record.clone()
        };
        // Original row ids are never reused and saved origin is immutable. Owner
        // validation and any last strong owner drop run outside the row lock.
        let observed = record.read(false)?;
        if consume && !observed.state().is_terminal() {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Pending native result cannot be consumed",
            ));
        }
        let item = record.wire_item(reference.scope(), id, observed.state())?;
        let output = super::exports::project_wire_result(item, &observed)?;
        if consume {
            record.read(true)?;
        }
        Ok(output)
    }
    pub fn read(&self, ticket: &Ticket, consume: bool) -> Result<Snapshot, ApiError> {
        self.validate(ticket)?;
        let mut rows = self.0.lock().map_err(|_| poisoned())?;
        let row = rows.rows.get(&ticket.id).ok_or_else(missing)?;
        if row.identity != ticket.identity {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native result identity changed",
            ));
        }
        if consume && !row.state.is_terminal() {
            return Err(ApiError::new(
                ErrorCode::NeedsInput,
                "Pending native result cannot be consumed",
            ));
        }
        if consume {
            rows.rows.remove(&ticket.id).ok_or_else(missing)
        } else {
            Ok(row.clone())
        }
    }
    pub fn rollback(&self, ticket: &Ticket) -> Result<(), ApiError> {
        self.validate(ticket)?;
        let mut rows = self.0.lock().map_err(|_| poisoned())?;
        if rows
            .rows
            .get(&ticket.id)
            .is_some_and(|row| row.state == GuardedSeekState::Queued)
        {
            rows.rows.remove(&ticket.id);
            Ok(())
        } else {
            Err(missing())
        }
    }
    pub fn transition(
        &self,
        ticket: &Ticket,
        state: GuardedSeekState,
        failure: Option<Failure>,
    ) -> Result<bool, ApiError> {
        self.validate(ticket)?;
        let mut rows = self.0.lock().map_err(|_| poisoned())?;
        let Some(row) = rows.rows.get_mut(&ticket.id) else {
            return Ok(false);
        };
        if row.state.is_terminal() {
            return Ok(false);
        }
        let valid = matches!(
            (row.state, state),
            (
                GuardedSeekState::Queued,
                GuardedSeekState::WriteCommitted | GuardedSeekState::RejectedBeforeWrite
            ) | (
                GuardedSeekState::WriteCommitted,
                GuardedSeekState::WrittenAwaitingAck
                    | GuardedSeekState::OutcomeUnobserved
                    | GuardedSeekState::Failed
            ) | (
                GuardedSeekState::WrittenAwaitingAck,
                GuardedSeekState::CommandAccepted
                    | GuardedSeekState::Failed
                    | GuardedSeekState::OutcomeUnobserved
            )
        );
        if !valid {
            return Err(ApiError::new(
                ErrorCode::Internal,
                "Invalid native result transition",
            ));
        }
        row.committed |= state == GuardedSeekState::WriteCommitted;
        row.state = state;
        row.failure = failure.map(|failure| bound_export_failure(row.export.is_some(), failure));
        Ok(true)
    }
    pub fn close(&self, failure: Failure) -> Result<(), ApiError> {
        self.close_matching(None, failure)
    }
    fn close_matching(&self, instance: Option<u64>, failure: Failure) -> Result<(), ApiError> {
        let mut rows = self.0.lock().map_err(|_| poisoned())?;
        for row in rows.rows.values_mut().filter(|row| {
            !row.state.is_terminal()
                && instance.is_none_or(|instance| row.identity.engine == instance)
        }) {
            row.state = if row.committed {
                GuardedSeekState::OutcomeUnobserved
            } else {
                GuardedSeekState::RejectedBeforeWrite
            };
            row.failure = Some(bound_export_failure(row.export.is_some(), failure.clone()));
            if !row.committed {
                row.failure.as_mut().expect("failure just stored").code = ErrorCode::Cancelled;
            }
        }
        let grants: Vec<_> = rows
            .grants
            .iter()
            .filter(|(_, grant)| instance.is_none_or(|instance| grant.identity.engine == instance))
            .map(|(id, _)| *id)
            .collect();
        for id in grants {
            if let Some(grant) = rows.grants.remove(&id) {
                let _ = grant.key.state.compare_exchange(
                    LIVE,
                    CLOSED,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );
                grant
                    .reply
                    .finish(Err(grant.key.error().unwrap_or_else(|| {
                        ApiError::new(ErrorCode::Cancelled, "Native observation closed")
                    })))?;
            }
        }
        Ok(())
    }
}
fn poisoned() -> ApiError {
    ApiError::new(ErrorCode::Internal, "Native result registry poisoned")
}
fn bound_export_failure(exported: bool, mut failure: Failure) -> Failure {
    if exported {
        let mut end = failure
            .prefix
            .len()
            .min(super::exports::EXPORT_FAILURE_PREFIX_BYTES);
        while !failure.prefix.is_char_boundary(end) {
            end -= 1;
        }
        failure.truncated |= end < failure.prefix.len();
        // Release the old oversized allocation rather than retaining its capacity.
        failure.prefix = failure.prefix[..end].into();
    }
    failure
}
fn missing() -> ApiError {
    ApiError::new(ErrorCode::NotFound, "Native result consumed or absent")
}

#[cfg(test)]
mod export_tests {
    use super::*;
    use crate::application::playback_session::{
        NativeResultProvenance, PreparedNativeResultExport,
    };
    use std::{cell::RefCell, time::Duration};

    fn prepared() -> PreparedNativeResultExport {
        PreparedNativeResultExport {
            provenance: NativeResultProvenance {
                registry_generation: "generation-31".into(),
                session: "session-2".into(),
                pass: 3,
                segment: "segment-4".into(),
                expected_revision: 5,
            },
        }
    }
    fn grant(host: &Arc<HostScope>, link: &Arc<ConnectionFence>) -> GrantKey {
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
        host.registry
            .complete_observation(
                &key,
                link,
                Sample {
                    entry: 9,
                    position: 3.0,
                    duration: 60.0,
                    paused: false,
                    idle: false,
                },
            )
            .unwrap();
        key
    }
    fn admitted(
        host: &Arc<HostScope>,
        link: &Arc<ConnectionFence>,
    ) -> (
        crate::application::playback_session::ExportedGuardedSeekSubmission,
        Ticket,
    ) {
        let key = grant(host, link);
        let captured = RefCell::new(None);
        let output = host
            .registry
            .admit_exported_observation(&key, link, prepared(), |packet| {
                *captured.borrow_mut() = Some(packet.ticket);
                Ok(())
            })
            .unwrap();
        (output, captured.into_inner().unwrap())
    }
    #[test]
    fn original_row_and_caller_keep_one_cell_and_recovery_after_lost_reply() {
        let host = HostScope::testing();
        let link = host.connect(7).unwrap();
        let (output, ticket) = admitted(&host, &link);
        assert_eq!(ticket.id, 1);
        let recovered = host
            .registry
            .exported_references(prepared().provenance)
            .unwrap();
        assert_eq!(recovered.len(), 1);
        assert!(
            recovered[0]
                .ticket
                .shares_read_cell(&output.submission.ticket)
        );
        output
            .submission
            .ticket
            .bind_read_context(Arc::new(19_u64))
            .unwrap();
        assert!(recovered[0].ticket.read_context().is_some());
        assert_eq!(
            host.registry
                .resolve_export(&recovered[0])
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
        host.registry.invalidate_observations(true).unwrap();
        drop(output);
        assert_eq!(
            host.registry
                .exported_references(prepared().provenance)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(host.registry.0.lock().unwrap().grants.len(), 0);
    }
    #[test]
    fn failed_send_releases_original_grant_and_tentative_export_row() {
        let host = HostScope::testing();
        let link = host.connect(7).unwrap();
        let key = grant(&host, &link);
        let error = host
            .registry
            .admit_exported_observation(&key, &link, prepared(), |_| {
                Err(ApiError::new(
                    ErrorCode::EngineFailed,
                    "original send failure",
                ))
            })
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::EngineFailed);
        let rows = host.registry.0.lock().unwrap();
        assert!(rows.rows.is_empty());
        assert!(rows.grants.is_empty());
    }
    #[test]
    fn full64_pending_consume_and_terminal_once_use_only_original_slots() {
        let host = HostScope::testing();
        let link = host.connect(7).unwrap();
        let mut tickets = Vec::new();
        for _ in 0..CAPACITY {
            tickets.push(admitted(&host, &link));
        }
        assert_eq!(host.registry.0.lock().unwrap().rows.len(), CAPACITY);
        let identity = tickets[0].1.identity.clone();
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
        let ticket = &tickets[0].1;
        assert_eq!(
            host.registry.read(ticket, true).unwrap_err().code,
            ErrorCode::NeedsInput
        );
        assert_eq!(
            host.registry
                .exported_references(prepared().provenance)
                .unwrap()
                .len(),
            CAPACITY
        );
        host.registry
            .transition(ticket, GuardedSeekState::RejectedBeforeWrite, None)
            .unwrap();
        host.registry.read(ticket, true).unwrap();
        assert_eq!(
            host.registry.read(ticket, true).unwrap_err().code,
            ErrorCode::NotFound
        );
        assert_eq!(
            host.registry
                .exported_references(prepared().provenance)
                .unwrap()
                .len(),
            CAPACITY - 1
        );
        assert_eq!(
            host.registry
                .resolve_export(&tickets[0].0.reference)
                .unwrap_err()
                .code,
            ErrorCode::NotFound
        );
    }
    #[test]
    fn weak_original_capability_does_not_keep_registry_or_host_alive() {
        let host = HostScope::testing();
        let link = host.connect(7).unwrap();
        let (output, ticket) = admitted(&host, &link);
        let owner = Arc::downgrade(&host.registry.0);
        let weak_host = Arc::downgrade(&host);
        let foreign = Registry::default();
        assert_eq!(
            foreign.resolve_export(&output.reference).unwrap_err().code,
            ErrorCode::NotFound
        );
        drop(link);
        drop(host);
        assert!(owner.upgrade().is_none());
        assert!(weak_host.upgrade().is_none());
        assert_eq!(
            ticket.check_host().unwrap_err().code,
            ErrorCode::SourceExpired
        );
        assert!(
            output
                .reference
                .ticket
                .shares_read_cell(&output.submission.ticket)
        );
    }
    #[test]
    fn bounded_export_error_preserves_hash_length_and_utf8() {
        let original = Failure::text(FailureStage::Acknowledgement, &"한".repeat(4000));
        let failure = bound_export_failure(true, original.clone());
        assert!(failure.prefix.len() <= super::super::exports::EXPORT_FAILURE_PREFIX_BYTES);
        assert_eq!(failure.sha256, original.sha256);
        assert_eq!(failure.original_bytes, 12000);
        assert!(failure.truncated);
        assert_eq!(
            bound_export_failure(false, original.clone()).prefix,
            original.prefix
        );
    }
}
#[cfg(all(test, feature = "validation-export-delivery-loss"))]
mod validation_counter_tests {
    use super::*;
    #[test]
    fn overflow_is_unknown_and_never_wraps_to_zero() {
        let mut counts = ValidationCounters {
            attempts: u64::MAX,
            enqueued: 0,
            overflowed: false,
        };
        ValidationCounters::increment(&mut counts.attempts, &mut counts.overflowed);
        assert_eq!(counts.attempts, u64::MAX);
        assert_eq!(
            counts.snapshot().unwrap_err().code,
            ErrorCode::ResourceExhausted
        );
    }
    #[test]
    fn poisoned_registry_counters_are_not_reported_as_zero() {
        let registry = Registry::default();
        let original = registry.clone();
        assert!(
            std::thread::spawn(move || {
                let _held = original.0.lock().unwrap();
                panic!("synthetic counter poison");
            })
            .join()
            .is_err()
        );
        assert!(registry.validation_export_counts().is_err());
    }
}
