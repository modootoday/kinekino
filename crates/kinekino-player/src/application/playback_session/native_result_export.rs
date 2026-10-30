//! Rust-only export preparation. Provenance never grants native playback authority.
use super::{GuardedSeek, GuardedSeekSubmission, NativeSeekTicket};
use crate::application::api::{ApiError, ErrorCode};
use std::{any::Any, sync::Arc};
mod codec;
mod metadata;
mod projection;

/// Versioned lookup/read contracts. This namespace cannot mint a native guard.
pub mod v1 {
    pub use super::codec::{
        NATIVE_ISSUER_SCOPE_V1_BYTES, NATIVE_RESULT_REF_V1_BYTES, NativeExportValueError,
        NativeIssuerScopeV1, NativeResultRefV1,
    };
    pub use super::metadata::{
        NativeExportAnnotationInputV1, NativeExportAnnotationV1, NativeExportMetadataInputV1,
        NativeExportMetadataV1, NativeExportOriginInputV1, NativeExportOriginV1,
    };
    pub use super::projection::{
        NATIVE_EXPORT_ITEM_BYTES, NATIVE_EXPORT_JSON_BYTES, NATIVE_EXPORT_MAX_ROWS,
        NATIVE_EXPORT_MESSAGE_BYTES, NATIVE_EXPORT_WIRE_BYTES, NativeExportCoreErrorV1,
        NativeExportFailureParts, NativeExportFailureStageV1, NativeExportFailureV1,
        NativeExportItemV1, NativeExportListV1, NativeExportReadModeV1, NativeExportResultV1,
        NativeExportStateV1, encode_native_export_v1,
    };
    pub use super::{
        ExportGuardedSeekV1, ExportedGuardedSeekV1, NativeExportIssuer,
        PreparedNativeResultExportV1,
    };
}

/// P1 adapters must retain only the original Weak owner; no current-route lookup.
pub(crate) trait NativeExportIssuerSourceV1: std::fmt::Debug + Send + Sync {
    fn matches_issuer(&self, _issuer: &dyn Any) -> bool {
        false
    }
    fn scope(&self) -> codec::NativeIssuerScopeV1;
    fn list(
        &self,
        origin: &metadata::NativeExportOriginV1,
    ) -> Result<projection::NativeExportListV1, ApiError>;
    fn read(
        &self,
        reference: codec::NativeResultRefV1,
        origin: &metadata::NativeExportOriginV1,
        consume: bool,
    ) -> Result<projection::NativeExportResultV1, ApiError>;
}
#[derive(Clone, Debug)]
pub struct NativeExportIssuer {
    scope: codec::NativeIssuerScopeV1,
    source: Arc<dyn super::NativeExportIssuerSourceV1>,
}
impl NativeExportIssuer {
    pub(crate) fn require_issuer(&self, issuer: &dyn Any) -> Result<(), ApiError> {
        if self.source.matches_issuer(issuer) {
            Ok(())
        } else {
            Err(issuer_conflict())
        }
    }
    #[allow(
        dead_code,
        reason = "P1 captures the original Weak issuer before exposing v1"
    )]
    pub(crate) fn from_original_source(source: Arc<dyn super::NativeExportIssuerSourceV1>) -> Self {
        Self {
            scope: source.scope(),
            source,
        }
    }
    pub fn scope(&self) -> codec::NativeIssuerScopeV1 {
        self.scope
    }
    pub fn list(
        &self,
        origin: &metadata::NativeExportOriginV1,
    ) -> Result<projection::NativeExportListV1, ApiError> {
        let result = self.source.list(origin)?;
        if result.scope() != self.scope
            || result
                .items()
                .iter()
                .any(|item| item.metadata().origin() != origin)
        {
            return Err(issuer_conflict());
        }
        Ok(result)
    }
    pub fn peek(
        &self,
        reference: codec::NativeResultRefV1,
        origin: &metadata::NativeExportOriginV1,
    ) -> Result<projection::NativeExportResultV1, ApiError> {
        self.read(reference, origin, false)
    }
    pub fn consume(
        &self,
        reference: codec::NativeResultRefV1,
        origin: &metadata::NativeExportOriginV1,
    ) -> Result<projection::NativeExportResultV1, ApiError> {
        self.read(reference, origin, true)
    }
    fn read(
        &self,
        reference: codec::NativeResultRefV1,
        origin: &metadata::NativeExportOriginV1,
        consume: bool,
    ) -> Result<projection::NativeExportResultV1, ApiError> {
        if reference.scope() != self.scope {
            return Err(issuer_conflict());
        }
        let result = self.source.read(reference, origin, consume)?;
        if result.item.reference() != reference || result.item.metadata().origin() != origin {
            return Err(issuer_conflict());
        }
        Ok(result)
    }
}
fn issuer_conflict() -> ApiError {
    ApiError::new(
        ErrorCode::RevisionConflict,
        "Native export differs from the original issuer or origin",
    )
}

/// Trusted consumer preparation only. Actual target is deliberately absent.
#[derive(Debug)]
pub struct PreparedNativeResultExportV1 {
    origin: metadata::NativeExportOriginV1,
    annotation: metadata::NativeExportAnnotationV1,
}
impl PreparedNativeResultExportV1 {
    #[allow(
        dead_code,
        reason = "C consumer prepares saved annotation before P1 admission"
    )]
    pub(crate) fn new(
        origin: metadata::NativeExportOriginV1,
        annotation: metadata::NativeExportAnnotationV1,
    ) -> Self {
        Self { origin, annotation }
    }
    pub fn origin(&self) -> &metadata::NativeExportOriginV1 {
        &self.origin
    }
    pub fn annotation(&self) -> metadata::NativeExportAnnotationV1 {
        self.annotation
    }
}
pub struct ExportGuardedSeekV1 {
    pub seek: GuardedSeek,
    pub export: PreparedNativeResultExportV1,
}
#[derive(Debug)]
pub struct ExportedGuardedSeekV1 {
    submission: GuardedSeekSubmission,
    reference: codec::NativeResultRefV1,
    metadata: metadata::NativeExportMetadataV1,
}
impl ExportedGuardedSeekV1 {
    #[allow(dead_code, reason = "P1 validates the saved result before enqueue")]
    pub(crate) fn from_original(
        submission: GuardedSeekSubmission,
        reference: codec::NativeResultRefV1,
        metadata: metadata::NativeExportMetadataV1,
        scope: codec::NativeIssuerScopeV1,
    ) -> Result<Self, ApiError> {
        if reference.scope() != scope
            || metadata.origin().engine_instance_id()
                != submission.ticket.identity().engine_instance_id()
            || metadata.origin().resolution_artifact_id()
                != submission.ticket.identity().resolution_artifact_id()
        {
            return Err(issuer_conflict());
        }
        Ok(Self {
            submission,
            reference,
            metadata,
        })
    }
    pub fn receipt(&self) -> &crate::application::playback_control::ControlReceipt {
        self.submission.receipt()
    }
    pub fn reference(&self) -> codec::NativeResultRefV1 {
        self.reference
    }
    pub fn metadata(&self) -> &metadata::NativeExportMetadataV1 {
        &self.metadata
    }
    pub fn into_parts(
        self,
    ) -> (
        GuardedSeekSubmission,
        codec::NativeResultRefV1,
        metadata::NativeExportMetadataV1,
    ) {
        (self.submission, self.reference, self.metadata)
    }
}

pub(crate) trait NativeExportReadOwner: std::fmt::Debug + Send + Sync {
    fn issuer(&self) -> &dyn Any;
    fn matches_context(&self, context: &(dyn Any + Send + Sync)) -> bool;
    fn retained_bytes(&self) -> usize;
    fn hold_for_admission(&self) -> Result<Box<dyn Any + Send + Sync>, ApiError>;
    fn read(
        &self,
        ticket: &NativeSeekTicket,
        consume: bool,
    ) -> Result<super::GuardedSeekResult, ApiError>;
}

/// Trusted assembly only. A token is moved into one original read cell before enqueue.
pub(crate) struct PreparedNativeExportReadOwner {
    owner: Arc<dyn NativeExportReadOwner>,
    context: Arc<dyn Any + Send + Sync>,
}
impl std::fmt::Debug for PreparedNativeExportReadOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparedNativeExportReadOwner")
    }
}
impl PreparedNativeExportReadOwner {
    pub(crate) const MAX_METADATA_BYTES: usize = 256;
    pub(crate) fn new(
        owner: Arc<dyn NativeExportReadOwner>,
        context: Arc<dyn Any + Send + Sync>,
    ) -> Result<Self, ApiError> {
        if !owner.matches_context(context.as_ref())
            || owner.retained_bytes() > Self::MAX_METADATA_BYTES
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Invalid original export read owner",
            ));
        }
        Ok(Self { owner, context })
    }
    pub(crate) fn issuer(&self) -> &dyn Any {
        self.owner.issuer()
    }
    pub(crate) fn hold_for_admission(&self) -> Result<Box<dyn Any + Send + Sync>, ApiError> {
        self.owner.hold_for_admission()
    }
    pub(crate) fn bind(
        self,
        ticket: &NativeSeekTicket,
    ) -> Result<Arc<dyn NativeExportReadOwner>, ApiError> {
        ticket.require_issuer(self.owner.issuer())?;
        ticket.bind_read_context(self.context)?;
        Ok(self.owner)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeResultProvenance {
    pub(crate) registry_generation: String,
    pub(crate) session: String,
    pub(crate) pass: u64,
    pub(crate) segment: String,
    pub(crate) expected_revision: u64,
}

/// Only trusted application assembly may prepare already-validated provenance.
#[derive(Debug)]
pub struct PreparedNativeResultExport {
    pub(crate) provenance: NativeResultProvenance,
}

pub struct ExportGuardedSeek {
    pub seek: GuardedSeek,
    pub export: PreparedNativeResultExport,
}

/// Read-only original capability; no public ID, serialization, or guard conversion.
#[derive(Clone, Debug)]
pub struct NativeResultReference {
    pub(crate) ticket: NativeSeekTicket,
    pub(crate) provenance: NativeResultProvenance,
}
impl NativeResultReference {
    pub fn same_request(&self, other: &Self) -> bool {
        self.ticket.same_request(&other.ticket)
    }
    pub fn provenance(&self) -> NativeResultProvenance {
        self.provenance.clone()
    }
}

#[derive(Debug)]
pub struct ExportedGuardedSeekSubmission {
    pub submission: GuardedSeekSubmission,
    pub reference: NativeResultReference,
}

pub struct NativeExportResultQuery {
    pub reference: NativeResultReference,
    pub consume_terminal: bool,
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::application::api::{ApiError, ErrorCode};
    struct LegacyOnly;
    impl EpisodePlaybackEngine for LegacyOnly {
        fn observe_native_guard(&self) -> Result<NativeGuardObservation, ApiError> {
            panic!("v1 must not fall back to legacy observation")
        }
        fn submit_guarded_seek(&self, _: GuardedSeek) -> Result<GuardedSeekSubmission, ApiError> {
            panic!("export must not fall back to legacy admission")
        }
        fn open(
            &self,
            _: EpisodePlaybackSource,
            _: bool,
            _: f64,
        ) -> Result<OpenedEpisodePlayback, ApiError> {
            panic!("legacy open")
        }
        fn close(&self, _: CloseEpisodePlayback) -> Result<ClosedEpisodePlayback, ApiError> {
            panic!("legacy close")
        }
        fn observe(&self) -> Result<EpisodePlaybackObservation, ApiError> {
            panic!("legacy observe")
        }
        fn control(
            &self,
            _: ControlEpisodePlayback,
        ) -> Result<crate::application::playback_control::ControlReceipt, ApiError> {
            panic!("legacy control")
        }
    }
    #[test]
    fn new_export_api_is_unsupported_before_legacy_admission() {
        let (seek, _) = guarded::test_inputs();
        let input = ExportGuardedSeek {
            seek,
            export: PreparedNativeResultExport {
                provenance: NativeResultProvenance {
                    registry_generation: "generation-1".into(),
                    session: "session-2".into(),
                    pass: 3,
                    segment: "segment-4".into(),
                    expected_revision: 5,
                },
            },
        };
        assert_eq!(
            LegacyOnly
                .submit_exported_guarded_seek(input)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    #[test]
    fn ordinary_wrappers_keep_export_list_and_read_unsupported() {
        let reference = NativeResultReference {
            ticket: guarded::test_submission(
                crate::application::playback_control::ControlReceipt::enqueued(7),
            )
            .ticket,
            provenance: NativeResultProvenance {
                registry_generation: "g".into(),
                session: "s".into(),
                pass: 1,
                segment: "t".into(),
                expected_revision: 1,
            },
        };
        assert_eq!(
            LegacyOnly
                .exported_seek_references(reference.provenance())
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
        assert_eq!(
            LegacyOnly
                .exported_seek_result(NativeExportResultQuery {
                    reference,
                    consume_terminal: true
                })
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    struct NeverResults;
    impl crate::application::episode_resolve::EpisodeResolutionResults for NeverResults {
        fn publish(
            &self,
            _: &crate::application::episode_resolve::StoredEpisodeResolution,
            _: &crate::application::catalog::CaptureMetadata,
        ) -> Result<crate::application::jobs::VerifiedJobOutput, ApiError> {
            panic!("export must not publish resolution")
        }
        fn read(
            &self,
            _: &str,
        ) -> Result<crate::application::episode_resolve::StoredEpisodeResolution, ApiError>
        {
            panic!("export must not reselect resolution")
        }
    }
    struct ExportEngine(std::sync::Arc<std::sync::atomic::AtomicUsize>);
    impl EpisodePlaybackEngine for ExportEngine {
        fn native_export_issuer(&self) -> Result<v1::NativeExportIssuer, ApiError> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(ApiError::new(ErrorCode::EngineFailed, "original v1 issuer"))
        }
        fn observe_native_guard_for_export(
            &self,
            _: &v1::NativeExportIssuer,
        ) -> Result<NativeGuardObservation, ApiError> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(ApiError::new(
                ErrorCode::SourceExpired,
                "original v1 observation",
            ))
        }
        fn submit_exported_guarded_seek_v1(
            &self,
            _: v1::ExportGuardedSeekV1,
        ) -> Result<v1::ExportedGuardedSeekV1, ApiError> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(ApiError::new(
                ErrorCode::EngineFailed,
                "original v1 submission",
            ))
        }
        fn submit_exported_guarded_seek(
            &self,
            _: ExportGuardedSeek,
        ) -> Result<ExportedGuardedSeekSubmission, ApiError> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(ApiError::new(
                ErrorCode::EngineFailed,
                "original export submit",
            ))
        }
        fn exported_seek_references(
            &self,
            _: NativeResultProvenance,
        ) -> Result<Vec<NativeResultReference>, ApiError> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(Vec::new())
        }
        fn exported_seek_result(
            &self,
            _: NativeExportResultQuery,
        ) -> Result<GuardedSeekResult, ApiError> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(ApiError::new(
                ErrorCode::SourceExpired,
                "original export result",
            ))
        }
        fn open(
            &self,
            a: EpisodePlaybackSource,
            b: bool,
            c: f64,
        ) -> Result<OpenedEpisodePlayback, ApiError> {
            LegacyOnly.open(a, b, c)
        }
        fn close(&self, a: CloseEpisodePlayback) -> Result<ClosedEpisodePlayback, ApiError> {
            LegacyOnly.close(a)
        }
        fn observe(&self) -> Result<EpisodePlaybackObservation, ApiError> {
            LegacyOnly.observe()
        }
        fn control(
            &self,
            a: ControlEpisodePlayback,
        ) -> Result<crate::application::playback_control::ControlReceipt, ApiError> {
            LegacyOnly.control(a)
        }
    }
    #[test]
    fn sessions_forward_export_only_to_original_engine_without_resolving_again() {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        let calls = Arc::new(AtomicUsize::new(0));
        let sessions = EpisodePlaybackSessions::new(
            Arc::new(NeverResults),
            Arc::new(ExportEngine(calls.clone())),
        );
        let provenance = NativeResultProvenance {
            registry_generation: "g".into(),
            session: "s".into(),
            pass: 1,
            segment: "t".into(),
            expected_revision: 1,
        };
        let (seek, _) = guarded::test_inputs();
        assert_eq!(
            sessions
                .submit_exported_guarded_seek(ExportGuardedSeek {
                    seek,
                    export: PreparedNativeResultExport {
                        provenance: provenance.clone()
                    }
                })
                .unwrap_err()
                .code,
            ErrorCode::EngineFailed
        );
        assert!(
            sessions
                .exported_seek_references(provenance.clone())
                .unwrap()
                .is_empty()
        );
        let reference = NativeResultReference {
            provenance,
            ticket: guarded::test_submission(
                crate::application::playback_control::ControlReceipt::enqueued(7),
            )
            .ticket,
        };
        assert_eq!(
            sessions
                .exported_seek_result(NativeExportResultQuery {
                    reference,
                    consume_terminal: false
                })
                .unwrap_err()
                .code,
            ErrorCode::SourceExpired
        );
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }
    #[derive(Debug)]
    struct V1Source {
        scope: v1::NativeIssuerScopeV1,
        calls: Arc<std::sync::atomic::AtomicUsize>,
    }
    impl NativeExportIssuerSourceV1 for V1Source {
        fn scope(&self) -> v1::NativeIssuerScopeV1 {
            self.scope
        }
        fn list(&self, _: &v1::NativeExportOriginV1) -> Result<v1::NativeExportListV1, ApiError> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            v1::NativeExportListV1::new(self.scope, Vec::new())
        }
        fn read(
            &self,
            _: v1::NativeResultRefV1,
            _: &v1::NativeExportOriginV1,
            _: bool,
        ) -> Result<v1::NativeExportResultV1, ApiError> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(ApiError::new(
                ErrorCode::SourceExpired,
                "original synthetic issuer expired",
            ))
        }
    }
    fn v1_issuer(calls: Arc<std::sync::atomic::AtomicUsize>) -> v1::NativeExportIssuer {
        v1::NativeExportIssuer::from_original_source(Arc::new(V1Source {
            scope: "nsi1.abcdefabcdefabcdefabcdefabcdefab".parse().unwrap(),
            calls,
        }))
    }
    fn v1_input() -> v1::ExportGuardedSeekV1 {
        let metadata = super::metadata::maximum_metadata();
        let (seek, _) = guarded::test_inputs();
        v1::ExportGuardedSeekV1 {
            seek,
            export: v1::PreparedNativeResultExportV1::new(
                metadata.origin().clone(),
                metadata.annotation(),
            ),
        }
    }
    #[test]
    fn v1_defaults_are_unsupported_without_legacy_or_issuer_source_fallback() {
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let issuer = v1_issuer(calls.clone());
        assert_eq!(
            LegacyOnly.native_export_issuer().unwrap_err().code,
            ErrorCode::Unsupported
        );
        assert_eq!(
            LegacyOnly
                .observe_native_guard_for_export(&issuer)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
        assert_eq!(
            LegacyOnly
                .submit_exported_guarded_seek_v1(v1_input())
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    }
    #[test]
    fn fixed_issuer_rejects_foreign_reference_before_read_and_preserves_original_error() {
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let issuer = v1_issuer(calls.clone());
        let metadata = super::metadata::maximum_metadata();
        let foreign = "nsr1.bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.0000000000000001"
            .parse()
            .unwrap();
        assert_eq!(
            issuer.peek(foreign, metadata.origin()).unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
        let original = "nsr1.abcdefabcdefabcdefabcdefabcdefab.0000000000000001"
            .parse()
            .unwrap();
        assert!(issuer.list(metadata.origin()).unwrap().items().is_empty());
        assert_eq!(
            issuer.peek(original, metadata.origin()).unwrap_err().code,
            ErrorCode::SourceExpired
        );
        assert_eq!(
            issuer
                .consume(original, metadata.origin())
                .unwrap_err()
                .code,
            ErrorCode::SourceExpired
        );
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 3);
    }
    #[test]
    fn sessions_forward_v1_to_the_same_engine_without_resolution_or_legacy_calls() {
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let sessions = EpisodePlaybackSessions::new(
            Arc::new(NeverResults),
            Arc::new(ExportEngine(calls.clone())),
        );
        let issuer = v1_issuer(Arc::new(std::sync::atomic::AtomicUsize::new(0)));
        assert_eq!(
            sessions.native_export_issuer().unwrap_err().code,
            ErrorCode::EngineFailed
        );
        assert_eq!(
            sessions
                .observe_native_guard_for_export(&issuer)
                .unwrap_err()
                .code,
            ErrorCode::SourceExpired
        );
        assert_eq!(
            sessions
                .submit_exported_guarded_seek_v1(v1_input())
                .unwrap_err()
                .code,
            ErrorCode::EngineFailed
        );
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 3);
    }
}
