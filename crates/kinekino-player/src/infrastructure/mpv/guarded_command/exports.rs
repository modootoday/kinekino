//! Optional metadata in the original result row, never an additional registry.
use crate::application::{
    api::{ApiError, ErrorCode},
    playback_session::{
        GuardedSeekSubmission, NativeResultProvenance, NativeResultReference,
        PreparedNativeResultExport,
    },
};
use crate::{
    application::playback_session::{GuardedSeekFailureStage, GuardedSeekResult, v1::*},
    domain::api_values::DecimalU64,
};

pub(super) enum ExportPreparation {
    Legacy(PreparedNativeResultExport),
    Wire(Box<WirePreparation>),
}
pub(super) struct WirePreparation {
    pub export: PreparedNativeResultExportV1,
    pub scope: NativeIssuerScopeV1,
    pub target: DecimalU64,
}
#[derive(Clone, Copy, Debug)]
struct SavedWire {
    annotation: NativeExportAnnotationV1,
    target: DecimalU64,
}
pub(super) struct PreparedExport {
    pub record: ExportRecord,
    pub position_us: u64,
    pub wire: Option<ExportedGuardedSeekV1>,
}
impl PreparedExport {
    pub fn new(
        preparation: ExportPreparation,
        submission: &GuardedSeekSubmission,
        row: u64,
        standalone: Option<super::scope::StandaloneExportOwner>,
        routed: Option<crate::application::playback_session::PreparedNativeExportReadOwner>,
    ) -> Result<Self, ApiError> {
        let (legacy, saved, wire, position_us) = match preparation {
            ExportPreparation::Legacy(legacy) => (legacy, None, None, 0),
            ExportPreparation::Wire(prepared) => {
                let metadata = NativeExportMetadataV1::stamp(
                    prepared.export.origin().clone(),
                    prepared.export.annotation(),
                    prepared.target,
                    submission.ticket.identity(),
                )?;
                let origin = metadata.origin();
                let legacy = PreparedNativeResultExport {
                    provenance: NativeResultProvenance {
                        registry_generation: origin.registry_generation().into(),
                        session: origin.session().into(),
                        pass: origin.pass().0,
                        segment: origin.segment().into(),
                        expected_revision: origin.request_revision().0,
                    },
                };
                let reference = NativeResultRefV1::from_original_row(
                    prepared.scope,
                    std::num::NonZeroU64::new(row).ok_or_else(inconsistent)?,
                );
                let output = ExportedGuardedSeekV1::from_original(
                    GuardedSeekSubmission {
                        ticket: submission.ticket.clone(),
                    },
                    reference,
                    metadata,
                    prepared.scope,
                )?;
                (
                    legacy,
                    Some(SavedWire {
                        annotation: prepared.export.annotation(),
                        target: prepared.target,
                    }),
                    Some(output),
                    prepared.target.0,
                )
            }
        };
        let mut record = match (standalone, routed) {
            (Some(owner), None) => ExportRecord::prepare_standalone(legacy, submission, owner)?,
            (None, Some(owner)) => ExportRecord::prepare_routed(legacy, submission, owner)?,
            (None, None) => ExportRecord::prepare(legacy, submission)?,
            (Some(_), Some(_)) => {
                return Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Conflicting original export owners",
                ));
            }
        };
        record.wire = saved;
        Ok(Self {
            record,
            position_us,
            wire,
        })
    }
}

pub(super) const EXPORT_METADATA_BYTES: usize = 2048;
pub(super) const EXPORT_FAILURE_PREFIX_BYTES: usize = 4096;
const _: () = assert!(
    std::mem::size_of::<ExportRecord>()
        + std::mem::size_of::<super::results::Snapshot>()
        + std::mem::size_of::<super::results::Ticket>()
        + crate::application::playback_session::NativeSeekTicket::READ_METADATA_BYTES
        + 200
        + 128
        + crate::application::playback_session::PreparedNativeExportReadOwner::MAX_METADATA_BYTES
        <= EXPORT_METADATA_BYTES
);

#[derive(Clone, Debug)]
pub(super) enum ReadAvailability {
    Unavailable,
    Standalone(std::sync::Weak<super::scope::HostScope>),
    Routed(std::sync::Arc<dyn crate::application::playback_session::NativeExportReadOwner>),
}

#[derive(Clone, Debug)]
pub(super) struct ExportRecord {
    reference: NativeResultReference,
    availability: ReadAvailability,
    wire: Option<SavedWire>,
}
impl ExportRecord {
    pub(super) fn prepare_routed(
        export: PreparedNativeResultExport,
        submission: &GuardedSeekSubmission,
        owner: crate::application::playback_session::PreparedNativeExportReadOwner,
    ) -> Result<Self, ApiError> {
        let mut record = Self::prepare(export, submission)?;
        record.availability = ReadAvailability::Routed(owner.bind(&submission.ticket)?);
        Ok(record)
    }
    pub(super) fn read(
        &self,
        consume: bool,
    ) -> Result<crate::application::playback_session::GuardedSeekResult, ApiError> {
        match &self.availability {
            ReadAvailability::Routed(owner) => owner.read(&self.reference.ticket, consume),
            _ => {
                self.require_read()?;
                self.reference.ticket.read_original(consume)
            }
        }
    }
    pub(super) fn prepare_standalone(
        export: PreparedNativeResultExport,
        submission: &GuardedSeekSubmission,
        owner: super::scope::StandaloneExportOwner,
    ) -> Result<Self, ApiError> {
        submission.ticket.require_issuer(&owner.host)?;
        let mut record = Self::prepare(export, submission)?;
        record.availability = ReadAvailability::Standalone(owner.host);
        Ok(record)
    }
    pub(super) fn prepare(
        export: PreparedNativeResultExport,
        submission: &GuardedSeekSubmission,
    ) -> Result<Self, ApiError> {
        let mut p = export.provenance;
        if p.registry_generation.len() > 40
            || !crate::domain::identifiers::valid_id(&p.registry_generation)
            || !crate::domain::identifiers::valid_id(&p.session)
            || !crate::domain::identifiers::valid_id(&p.segment)
            || p.pass == 0
            || p.expected_revision == 0
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid export provenance",
            ));
        }
        // Caller capacity is not retained as uncharged storage.
        p.registry_generation = p.registry_generation.into_boxed_str().into_string();
        p.session = p.session.into_boxed_str().into_string();
        p.segment = p.segment.into_boxed_str().into_string();
        Ok(Self {
            wire: None,
            reference: NativeResultReference {
                ticket: submission.ticket.clone(),
                provenance: p,
            },
            availability: ReadAvailability::Unavailable,
        })
    }
    pub(super) fn reference(&self) -> NativeResultReference {
        self.reference.clone()
    }
    pub(super) fn wire_origin_matches(&self, origin: &NativeExportOriginV1) -> bool {
        let p = &self.reference.provenance;
        let identity = self.reference.ticket.identity();
        self.wire.is_some()
            && p.registry_generation == origin.registry_generation()
            && p.session == origin.session()
            && p.pass == origin.pass().0
            && p.segment == origin.segment()
            && p.expected_revision == origin.request_revision().0
            && identity.engine_instance_id() == origin.engine_instance_id()
            && identity.resolution_artifact_id() == origin.resolution_artifact_id()
    }
    pub(super) fn has_wire(&self) -> bool {
        self.wire.is_some()
    }
    pub(super) fn wire_item(
        &self,
        scope: NativeIssuerScopeV1,
        row: u64,
        state: crate::application::playback_session::GuardedSeekState,
    ) -> Result<NativeExportItemV1, ApiError> {
        let wire = self
            .wire
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Original v1 export row absent"))?;
        let p = &self.reference.provenance;
        let identity = self.reference.ticket.identity();
        let origin = NativeExportOriginV1::try_from(NativeExportOriginInputV1 {
            registry_generation: p.registry_generation.clone(),
            session: p.session.clone(),
            pass: DecimalU64(p.pass),
            segment: p.segment.clone(),
            request_revision: DecimalU64(p.expected_revision),
            engine_instance_id: identity.engine_instance_id(),
            resolution_artifact_id: identity.resolution_artifact_id().into(),
        })?;
        let metadata =
            NativeExportMetadataV1::stamp(origin, wire.annotation, wire.target, identity)?;
        let reference = NativeResultRefV1::from_original_row(
            scope,
            std::num::NonZeroU64::new(row).ok_or_else(inconsistent)?,
        );
        let mode = match &self.availability {
            ReadAvailability::Unavailable => NativeExportReadModeV1::Unavailable,
            ReadAvailability::Standalone(_) => NativeExportReadModeV1::Standalone,
            ReadAvailability::Routed(_) => NativeExportReadModeV1::Routed,
        };
        NativeExportItemV1::new(reference, metadata, state.into(), mode)
    }
    pub(super) fn matches(&self, reference: &NativeResultReference) -> bool {
        self.reference.same_request(reference)
            && self.reference.provenance() == reference.provenance()
    }
    pub(super) fn provenance(&self) -> NativeResultProvenance {
        self.reference.provenance()
    }
    pub(super) fn require_read(&self) -> Result<(), ApiError> {
        match &self.availability {
            ReadAvailability::Routed(_) => Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original routed export read context is required",
            )),
            ReadAvailability::Unavailable => Err(ApiError::new(
                ErrorCode::Unsupported,
                "Original exported read owner is unavailable",
            )),
            ReadAvailability::Standalone(owner) => {
                let host = owner.upgrade().ok_or_else(super::issuer::expired)?;
                host.require_standalone_export()?;
                self.reference.ticket.require_issuer(owner)
            }
        }
    }
}
pub(super) fn project_wire_result(
    item: NativeExportItemV1,
    result: &GuardedSeekResult,
) -> Result<NativeExportResultV1, ApiError> {
    let failure = result
        .failure()
        .map(|failure| {
            let error = failure.api_error();
            let details = &error.details;
            let hash = details
                .get("sha256")
                .and_then(serde_json::Value::as_str)
                .filter(|value| crate::domain::identifiers::valid_sha256(value))
                .ok_or_else(inconsistent)?;
            let mut sha256 = [0; 32];
            for (index, pair) in hash.as_bytes().as_chunks::<2>().0.iter().enumerate() {
                sha256[index] =
                    u8::from_str_radix(std::str::from_utf8(pair).map_err(|_| inconsistent())?, 16)
                        .map_err(|_| inconsistent())?;
            }
            let original_message_bytes = details
                .get("original_bytes")
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(inconsistent)?;
            let truncated = details
                .get("truncated")
                .and_then(serde_json::Value::as_bool)
                .ok_or_else(inconsistent)?;
            let raw_os_code = details
                .get("raw_os_code")
                .and_then(serde_json::Value::as_i64)
                .map(i32::try_from)
                .transpose()
                .map_err(|_| inconsistent())?;
            let stage = match failure.stage() {
                GuardedSeekFailureStage::BeforeWrite => NativeExportFailureStageV1::BeforeWrite,
                GuardedSeekFailureStage::Write => NativeExportFailureStageV1::Write,
                GuardedSeekFailureStage::Acknowledgement => {
                    NativeExportFailureStageV1::Acknowledgement
                }
                GuardedSeekFailureStage::Connection => NativeExportFailureStageV1::Connection,
                GuardedSeekFailureStage::Deadline => NativeExportFailureStageV1::Deadline,
            };
            NativeExportFailureV1::from_native(NativeExportFailureParts {
                code: error.code,
                stage,
                message: &error.message,
                original_message_bytes,
                sha256,
                truncated,
                raw_os_code,
                kind: details.get("kind").and_then(serde_json::Value::as_str),
            })
        })
        .transpose()?;
    Ok(NativeExportResultV1 {
        item,
        write_committed: result.write_commit(),
        failure,
    })
}
fn inconsistent() -> ApiError {
    ApiError::new(
        ErrorCode::Internal,
        "Original native export projection invariant failed",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::{
        playback_control::ControlReceipt,
        playback_session::{NativeSeekResultParts, NativeSeekResultSource, native_seek_submission},
    };
    use crate::domain::api_values::DecimalU64;
    use std::sync::Arc;

    #[derive(Debug)]
    struct Source;
    impl NativeSeekResultSource for Source {
        fn identity(&self) -> (DecimalU64, String) {
            (DecimalU64(7), "03".repeat(32))
        }
        fn read(&self, _: bool) -> Result<NativeSeekResultParts, ApiError> {
            panic!("unavailable export must not read the source")
        }
    }
    fn provenance() -> NativeResultProvenance {
        NativeResultProvenance {
            registry_generation: "generation-1".into(),
            session: "session-2".into(),
            pass: 3,
            segment: "segment-4".into(),
            expected_revision: 5,
        }
    }
    fn submission() -> GuardedSeekSubmission {
        native_seek_submission(ControlReceipt::enqueued(7), Arc::new(Source)).unwrap()
    }
    #[test]
    fn prepared_record_and_caller_share_original_cell_and_late_context() {
        let submission = submission();
        let record = ExportRecord::prepare(
            PreparedNativeResultExport {
                provenance: provenance(),
            },
            &submission,
        )
        .unwrap();
        assert!(record.reference.ticket.shares_read_cell(&submission.ticket));
        submission
            .ticket
            .bind_read_context(Arc::new(7_u64))
            .unwrap();
        assert!(record.reference.ticket.read_context().is_some());
        assert!(
            record
                .reference
                .ticket
                .bind_read_context(Arc::new(7_u64))
                .is_err()
        );
        assert_eq!(
            record.require_read().unwrap_err().code,
            ErrorCode::Unsupported
        );
    }
    #[test]
    fn unbound_context_is_not_standalone_read_authority() {
        let submission = submission();
        assert!(submission.ticket.read_context().is_none());
        let record = ExportRecord::prepare(
            PreparedNativeResultExport {
                provenance: provenance(),
            },
            &submission,
        )
        .unwrap();
        assert_eq!(
            record.require_read().unwrap_err().code,
            ErrorCode::Unsupported
        );
    }
    #[test]
    fn invalid_provenance_and_different_original_cell_are_rejected() {
        let mut p = provenance();
        p.pass = 0;
        assert_eq!(
            ExportRecord::prepare(PreparedNativeResultExport { provenance: p }, &submission())
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        let record = ExportRecord::prepare(
            PreparedNativeResultExport {
                provenance: provenance(),
            },
            &submission(),
        )
        .unwrap();
        let other = ExportRecord::prepare(
            PreparedNativeResultExport {
                provenance: provenance(),
            },
            &submission(),
        )
        .unwrap();
        assert!(!record.matches(&other.reference()));
    }
    #[test]
    fn export_storage_and_escaped_summary_fit_original_budget() {
        assert!(std::mem::size_of::<ExportRecord>() + 512 <= EXPORT_METADATA_BYTES);
        let charged =
            std::hint::black_box(EXPORT_FAILURE_PREFIX_BYTES + EXPORT_METADATA_BYTES + 512);
        assert!(charged <= super::super::results::ROW_PAYLOAD_BYTES);
        let payload = serde_json::json!({"message": "\u{0001}".repeat(1024), "provenance": {"generation":"g".repeat(40),"session":"s".repeat(80),"segment":"t".repeat(80),"pass":u64::MAX,"revision":u64::MAX}, "sha256":"f".repeat(64), "original_bytes":u64::MAX,"truncated":true});
        assert!(
            serde_json::to_vec(&payload).unwrap().len() <= super::super::results::ROW_PAYLOAD_BYTES
        );
    }
    #[test]
    fn overcapacity_provenance_is_normalized_before_row_storage() {
        let mut p = provenance();
        let mut oversized = String::with_capacity(1_048_576);
        oversized.push_str("session-2");
        p.session = oversized;
        let record =
            ExportRecord::prepare(PreparedNativeResultExport { provenance: p }, &submission())
                .unwrap();
        assert_eq!(
            record.reference.provenance.session.capacity(),
            record.reference.provenance.session.len()
        );
        assert!(record.reference.provenance.session.capacity() <= 80);
    }
    #[test]
    fn retained_metadata_and_transient_clone_charges_are_separate() {
        let record = ExportRecord::prepare(
            PreparedNativeResultExport {
                provenance: provenance(),
            },
            &submission(),
        )
        .unwrap();
        let p = &record.reference.provenance;
        let strings =
            p.registry_generation.capacity() + p.session.capacity() + p.segment.capacity();
        let retained = std::mem::size_of::<ExportRecord>()
            + std::mem::size_of::<super::super::results::Snapshot>()
            + std::mem::size_of::<super::super::results::Ticket>()
            + crate::application::playback_session::NativeSeekTicket::READ_METADATA_BYTES
            + strings
            + 128;
        assert!(retained <= EXPORT_METADATA_BYTES);
        // Resolver/list clones share the read cell, but own separate bounded strings.
        // This charge is not folded into the persistent row's 8KiB promise.
        let clone = record.reference();
        assert!(clone.ticket.shares_read_cell(&record.reference.ticket));
        let transient = std::mem::size_of::<NativeResultReference>()
            + clone.provenance.registry_generation.capacity()
            + clone.provenance.session.capacity()
            + clone.provenance.segment.capacity();
        assert!(transient <= EXPORT_METADATA_BYTES);
        assert!(retained + transient > retained);
    }
}
