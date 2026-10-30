//! Additive Rust-only diagnostics. No file attribution or seek authority is minted in phase one.
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::api_values::DecimalU64,
};
use crate::{
    application::{playback_control::ControlReceipt, playback_status::PlaybackStatus},
    domain::skip_segments::Seekability,
};
use std::{sync::Arc, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileAttribution {
    Unverified,
}
#[derive(Clone, Debug)]
pub struct NativePropertySample<T> {
    pub value: Option<T>,
    pub receive_sequence: Option<u64>,
    pub received_age: Option<Duration>,
    pub lifecycle_generation_at_receive: Option<u64>,
    pub file_attribution: FileAttribution,
}
#[derive(Clone, Debug)]
pub struct NativeReceiveObservation {
    pub connected: bool,
    pub counter_exhausted: bool,
    pub connection_generation: u64,
    pub lifecycle_generation: u64,
    pub connection_receive_sequence: u64,
    pub position_seconds: NativePropertySample<f64>,
    pub duration_seconds: NativePropertySample<f64>,
    pub paused: NativePropertySample<bool>,
    pub idle: NativePropertySample<bool>,
    pub seekable: NativePropertySample<bool>,
    pub partially_seekable: NativePropertySample<bool>,
}
#[derive(Debug)]
pub struct NativeObservationGuard {
    export_read_owner: Option<super::PreparedNativeExportReadOwner>,
    view: NativeGuardView,
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "C owner integration follows this source-only contract"
        )
    )]
    authority: Option<Box<dyn NativeGuardAuthority>>,
}
/// Infrastructure supplies an original opaque claim, never a public diagnostic reconstruction.
pub(crate) trait NativeGuardAuthority: std::fmt::Debug + Send + Sync {
    fn submit_exported_v1(
        self: Box<Self>,
        _position_us: DecimalU64,
        _export: super::v1::PreparedNativeResultExportV1,
        _read_owner: Option<super::PreparedNativeExportReadOwner>,
    ) -> Result<super::v1::ExportedGuardedSeekV1, ApiError> {
        Err(unsupported())
    }
    fn submit_exported(
        self: Box<Self>,
        _position_us: DecimalU64,
        _export: super::PreparedNativeResultExport,
        _read_owner: Option<super::PreparedNativeExportReadOwner>,
    ) -> Result<super::ExportedGuardedSeekSubmission, ApiError> {
        Err(unsupported())
    }
    fn matches_issuer(&self, _issuer: &dyn std::any::Any) -> bool {
        false
    }
    fn snapshot(&self) -> NativeGuardSnapshot;
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "C owner integration follows this source-only contract"
        )
    )]
    fn submit(self: Box<Self>, position_us: DecimalU64) -> Result<GuardedSeekSubmission, ApiError>;
}
pub(crate) struct NativeGuardSnapshot {
    pub engine_instance_id: DecimalU64,
    pub resolution_artifact_id: String,
    pub playback: PlaybackStatus,
    pub seekability: Seekability,
}
pub(crate) fn native_guard_from_authority(
    authority: Box<dyn NativeGuardAuthority>,
) -> NativeObservationGuard {
    let snapshot = authority.snapshot();
    NativeObservationGuard {
        export_read_owner: None,
        view: NativeGuardView {
            identity: NativeGuardIdentity {
                engine_instance_id: snapshot.engine_instance_id,
                resolution_artifact_id: snapshot.resolution_artifact_id,
            },
            playback: snapshot.playback,
            seekability: snapshot.seekability,
            authorized: true,
        },
        authority: Some(authority),
    }
}
impl NativeObservationGuard {
    pub(crate) fn submit_exported_native_v1(
        mut self,
        position_us: DecimalU64,
        export: super::v1::PreparedNativeResultExportV1,
    ) -> Result<super::v1::ExportedGuardedSeekV1, ApiError> {
        self.authority
            .take()
            .ok_or_else(unsupported)?
            .submit_exported_v1(position_us, export, self.export_read_owner.take())
    }
    pub(crate) fn prepare_routed_export(
        mut self,
        owner: super::PreparedNativeExportReadOwner,
    ) -> Result<Self, ApiError> {
        if self.export_read_owner.is_some() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original export owner already prepared",
            ));
        }
        self.require_issuer(owner.issuer())?;
        self.export_read_owner = Some(owner);
        Ok(self)
    }
    pub(crate) fn has_routed_export_owner(&self) -> bool {
        self.export_read_owner.is_some()
    }
    pub(crate) fn submit_exported_native(
        mut self,
        position_us: DecimalU64,
        export: super::PreparedNativeResultExport,
    ) -> Result<super::ExportedGuardedSeekSubmission, ApiError> {
        self.authority
            .take()
            .ok_or_else(unsupported)?
            .submit_exported(position_us, export, self.export_read_owner.take())
    }
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Original host admission is connected in B7")
    )]
    pub(crate) fn require_issuer(&self, issuer: &dyn std::any::Any) -> Result<(), ApiError> {
        if self
            .authority
            .as_ref()
            .is_some_and(|authority| authority.matches_issuer(issuer))
        {
            Ok(())
        } else {
            Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Observation belongs to another logical host",
            ))
        }
    }
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "C owner integration follows this source-only contract"
        )
    )]
    pub(crate) fn submit_native(
        mut self,
        position_us: DecimalU64,
    ) -> Result<GuardedSeekSubmission, ApiError> {
        self.authority
            .take()
            .ok_or_else(unsupported)?
            .submit(position_us)
    }
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "C application integration follows this source-only contract"
        )
    )]
    pub(crate) fn frozen_view(&self) -> &NativeGuardView {
        &self.view
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeGuardIdentity {
    engine_instance_id: DecimalU64,
    resolution_artifact_id: String,
}
impl NativeGuardIdentity {
    pub fn engine_instance_id(&self) -> DecimalU64 {
        self.engine_instance_id
    }
    pub fn resolution_artifact_id(&self) -> &str {
        &self.resolution_artifact_id
    }
}
/// Read-only view stored by an issuer, never rebuilt from mutable public diagnostic fields.
#[derive(Clone, Debug)]
pub struct NativeGuardView {
    identity: NativeGuardIdentity,
    playback: PlaybackStatus,
    seekability: Seekability,
    authorized: bool,
}
impl NativeGuardView {
    pub fn identity(&self) -> &NativeGuardIdentity {
        &self.identity
    }
    pub fn playback(&self) -> &PlaybackStatus {
        &self.playback
    }
    pub fn seekability(&self) -> Seekability {
        self.seekability
    }
    pub fn observation_authorized(&self) -> bool {
        self.authorized
    }
}
#[derive(Debug)]
struct AdmissionRecord {
    identity: NativeGuardIdentity,
    receipt: ControlReceipt,
    source: Option<Arc<dyn NativeSeekResultSource>>,
}
/// Original read capability only. This contract cannot mint an observation/control guard.
pub(crate) trait NativeSeekResultSource: std::fmt::Debug + Send + Sync {
    fn matches_issuer(&self, _issuer: &dyn std::any::Any) -> bool {
        false
    }
    fn identity(&self) -> (DecimalU64, String);
    fn read(&self, consume_terminal: bool) -> Result<NativeSeekResultParts, ApiError>;
}
pub(crate) struct NativeSeekResultParts {
    pub state: GuardedSeekState,
    pub committed: bool,
    pub failure: Option<(GuardedSeekFailureStage, ApiError)>,
}
pub(crate) fn native_seek_submission(
    receipt: ControlReceipt,
    source: Arc<dyn NativeSeekResultSource>,
) -> Result<GuardedSeekSubmission, ApiError> {
    let (engine_instance_id, resolution_artifact_id) = source.identity();
    if engine_instance_id != receipt.engine_instance_id {
        return Err(ApiError::new(
            ErrorCode::RevisionConflict,
            "Native receipt differs from original result issuer",
        ));
    }
    Ok(GuardedSeekSubmission {
        ticket: NativeSeekTicket {
            read_context: Arc::new(std::sync::OnceLock::new()),
            admission: Arc::new(AdmissionRecord {
                identity: NativeGuardIdentity {
                    engine_instance_id,
                    resolution_artifact_id,
                },
                receipt,
                source: Some(source),
            }),
        },
    })
}
type OpaqueReadContext = Arc<dyn std::any::Any + Send + Sync>;
/// Clone shares only immutable read/admission metadata, never an observation/control guard.
#[derive(Clone)]
pub struct NativeSeekTicket {
    admission: Arc<AdmissionRecord>,
    read_context: Arc<std::sync::OnceLock<OpaqueReadContext>>,
}
impl std::fmt::Debug for NativeSeekTicket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeSeekTicket")
            .field("admission", &self.admission)
            .field("has_read_context", &self.read_context.get().is_some())
            .finish()
    }
}
impl NativeSeekTicket {
    pub(crate) const READ_METADATA_BYTES: usize = std::mem::size_of::<Self>()
        + std::mem::size_of::<AdmissionRecord>()
        + std::mem::size_of::<std::sync::OnceLock<OpaqueReadContext>>()
        + 8 * std::mem::size_of::<usize>();
    #[cfg(test)]
    pub(crate) fn shares_read_cell(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.admission, &other.admission)
            && Arc::ptr_eq(&self.read_context, &other.read_context)
    }
    /// Trusted routing decorates the original read metadata once across all clones.
    /// The admission/source/receipt and original command slot are never replaced.
    pub(crate) fn bind_read_context(
        &self,
        context: Arc<dyn std::any::Any + Send + Sync>,
    ) -> Result<(), ApiError> {
        self.read_context.set(context).map_err(|_| {
            ApiError::new(
                ErrorCode::RevisionConflict,
                "Original read context is already bound",
            )
        })
    }
    pub(crate) fn read_context(&self) -> Option<&(dyn std::any::Any + Send + Sync)> {
        self.read_context.get().map(Arc::as_ref)
    }
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "Original host admission is connected in B7")
    )]
    pub(crate) fn require_issuer(&self, issuer: &dyn std::any::Any) -> Result<(), ApiError> {
        if self
            .admission
            .source
            .as_ref()
            .is_some_and(|source| source.matches_issuer(issuer))
        {
            Ok(())
        } else {
            Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Read ticket belongs to another logical host",
            ))
        }
    }
    pub(crate) fn read_original(
        &self,
        consume_terminal: bool,
    ) -> Result<GuardedSeekResult, ApiError> {
        if self.read_context().is_some() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original route read context is required",
            ));
        }
        self.read_admitted_source(consume_terminal)
    }
    pub(crate) fn read_in_context(
        &self,
        context: &(dyn std::any::Any + Send + Sync),
        consume_terminal: bool,
    ) -> Result<GuardedSeekResult, ApiError> {
        if !self
            .read_context()
            .is_some_and(|original| std::ptr::eq(original, context))
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Foreign route read context",
            ));
        }
        self.read_admitted_source(consume_terminal)
    }
    fn read_admitted_source(&self, consume_terminal: bool) -> Result<GuardedSeekResult, ApiError> {
        let source = self.admission.source.as_ref().ok_or_else(unsupported)?;
        let parts = source.read(consume_terminal)?;
        Ok(GuardedSeekResult {
            state: parts.state,
            identity: self.admission.identity.clone(),
            write_committed: parts.committed,
            failure: parts
                .failure
                .map(|(stage, error)| GuardedSeekFailure { stage, error }),
        })
    }
    pub fn same_request(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.admission, &other.admission)
    }
    pub fn identity(&self) -> &NativeGuardIdentity {
        &self.admission.identity
    }
}
#[derive(Debug)]
pub struct NativeGuardObservation {
    pub engine_instance_id: DecimalU64,
    pub resolution_artifact_id: String,
    pub native: NativeReceiveObservation,
    pub observation_authorized: bool,
    pub guard: Option<NativeObservationGuard>,
}
impl NativeGuardObservation {
    /// The public binding/native/authorized fields are diagnostics, not authority inputs.
    pub fn into_guarded_parts(self) -> Result<(NativeGuardView, NativeObservationGuard), ApiError> {
        let guard = self.guard.ok_or_else(|| {
            ApiError::new(
                ErrorCode::NeedsInput,
                "No issuer-owned native observation guard",
            )
        })?;
        Ok((guard.view.clone(), guard))
    }
}
pub struct GuardedSeek {
    pub guard: NativeObservationGuard,
    pub position_us: DecimalU64,
}
#[derive(Debug)]
pub struct GuardedSeekSubmission {
    pub ticket: NativeSeekTicket,
}
impl GuardedSeekSubmission {
    pub fn receipt(&self) -> &ControlReceipt {
        &self.ticket.admission.receipt
    }
    pub fn into_parts(self) -> (ControlReceipt, NativeSeekTicket) {
        (self.ticket.admission.receipt.clone(), self.ticket)
    }
}
pub struct GuardedSeekResultQuery {
    pub ticket: NativeSeekTicket,
    pub consume_terminal: bool,
}
#[derive(Debug)]
pub struct GuardedSeekResult {
    state: GuardedSeekState,
    identity: NativeGuardIdentity,
    write_committed: bool,
    failure: Option<GuardedSeekFailure>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuardedSeekState {
    Queued,
    WriteCommitted,
    WrittenAwaitingAck,
    CommandAccepted,
    RejectedBeforeWrite,
    Failed,
    OutcomeUnobserved,
}
impl GuardedSeekState {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::CommandAccepted
                | Self::RejectedBeforeWrite
                | Self::Failed
                | Self::OutcomeUnobserved
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuardedSeekFailureStage {
    BeforeWrite,
    Write,
    Acknowledgement,
    Connection,
    Deadline,
}
/// The producer must apply the B-stage byte budget before retaining a production failure.
#[derive(Debug)]
pub struct GuardedSeekFailure {
    stage: GuardedSeekFailureStage,
    error: ApiError,
}
impl GuardedSeekFailure {
    pub fn stage(&self) -> GuardedSeekFailureStage {
        self.stage
    }
    pub fn api_error(&self) -> &ApiError {
        &self.error
    }
}
impl GuardedSeekResult {
    pub fn state(&self) -> GuardedSeekState {
        self.state
    }
    pub fn original_identity(&self) -> &NativeGuardIdentity {
        &self.identity
    }
    pub fn write_commit(&self) -> bool {
        self.write_committed
    }
    pub fn failure(&self) -> Option<&GuardedSeekFailure> {
        self.failure.as_ref()
    }
}
pub(super) fn unsupported() -> ApiError {
    ApiError::new(ErrorCode::Unsupported, "Native guarded seek unavailable")
}
#[cfg(test)]
pub(super) fn test_inputs() -> (GuardedSeek, GuardedSeekResultQuery) {
    let (_, guard) = test_observation(Default::default())
        .into_guarded_parts()
        .unwrap();
    let ticket = test_submission(ControlReceipt::enqueued(7)).ticket;
    (
        GuardedSeek {
            guard,
            position_us: DecimalU64(1),
        },
        GuardedSeekResultQuery {
            ticket,
            consume_terminal: false,
        },
    )
}
// Synthetic contract fixtures only. There is deliberately no production issuer in A7.
#[cfg(test)]
pub(crate) fn test_observation(
    state: crate::application::native_playback::NativePlayback,
) -> NativeGuardObservation {
    fn unknown<T>() -> NativePropertySample<T> {
        NativePropertySample {
            value: None,
            receive_sequence: None,
            received_age: None,
            lifecycle_generation_at_receive: None,
            file_attribution: FileAttribution::Unverified,
        }
    }
    NativeGuardObservation {
        engine_instance_id: DecimalU64(7),
        resolution_artifact_id: "a".repeat(64),
        native: NativeReceiveObservation {
            connected: false,
            counter_exhausted: false,
            connection_generation: 0,
            lifecycle_generation: 0,
            connection_receive_sequence: 0,
            position_seconds: unknown(),
            duration_seconds: unknown(),
            paused: unknown(),
            idle: unknown(),
            seekable: unknown(),
            partially_seekable: unknown(),
        },
        observation_authorized: false,
        guard: Some(NativeObservationGuard {
            export_read_owner: None,
            authority: None,
            view: NativeGuardView {
                identity: NativeGuardIdentity {
                    engine_instance_id: DecimalU64(7),
                    resolution_artifact_id: "a".repeat(64),
                },
                playback: crate::application::playback_status::project_native_status(&state),
                seekability: Seekability::Yes,
                authorized: true,
            },
        }),
    }
}
#[cfg(test)]
pub(super) fn test_submission(receipt: ControlReceipt) -> GuardedSeekSubmission {
    GuardedSeekSubmission {
        ticket: NativeSeekTicket {
            read_context: Arc::new(std::sync::OnceLock::new()),
            admission: Arc::new(AdmissionRecord {
                identity: NativeGuardIdentity {
                    engine_instance_id: receipt.engine_instance_id,
                    resolution_artifact_id: "a".repeat(64),
                },
                receipt,
                source: None,
            }),
        },
    }
}
#[cfg(test)]
pub(super) fn test_result(
    state: GuardedSeekState,
    write_committed: bool,
    failure: Option<ApiError>,
) -> GuardedSeekResult {
    GuardedSeekResult {
        state,
        identity: NativeGuardIdentity {
            engine_instance_id: DecimalU64(7),
            resolution_artifact_id: "a".repeat(64),
        },
        write_committed,
        failure: failure.map(|error| GuardedSeekFailure {
            stage: GuardedSeekFailureStage::Write,
            error,
        }),
    }
}
