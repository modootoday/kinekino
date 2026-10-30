//! A checked presentation conversion; snapshot membership remains project_initial policy.
use super::bootstrap::{BootstrapSelection, BootstrapTarget};
use kinekino_player::application::{
    api::{ApiError, ErrorCode},
    launch::native_proposal::{
        DocumentRepairSelection, NativeProposalSelection, NativeProposalTarget,
    },
};

impl TryFrom<NativeProposalSelection> for BootstrapSelection {
    type Error = ApiError;
    fn try_from(value: NativeProposalSelection) -> Result<Self, Self::Error> {
        let index = value.target.checked_item_index()?;
        let required_index = || {
            index.ok_or_else(|| {
                ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Explicit item index is required",
                )
            })
        };
        let target = match value.target {
            NativeProposalTarget::ManualRepair { task } => BootstrapTarget::ManualRepair { task },
            NativeProposalTarget::Card { .. } => BootstrapTarget::Card {
                index: required_index()?,
            },
            NativeProposalTarget::Highlight { .. } => BootstrapTarget::Highlight {
                index: required_index()?,
            },
            NativeProposalTarget::CueCorrection { cue_id, .. } => BootstrapTarget::CueCorrection {
                index: required_index()?,
                cue_id,
            },
            NativeProposalTarget::DocumentCorrection { source_ref, .. } => {
                BootstrapTarget::DocumentCorrection {
                    index: required_index()?,
                    source_ref,
                }
            }
            NativeProposalTarget::DocumentRepair { selection } => BootstrapTarget::DocumentRepair {
                source_ref: match selection {
                    DocumentRepairSelection::Unselected {} => None,
                    DocumentRepairSelection::Selected { source_ref } => Some(source_ref),
                },
            },
        };
        Ok(Self {
            proposal_id: value.proposal_id,
            document_revision: value.document_revision,
            source_id: value.source_id,
            source_revision: value.source_revision,
            source_sha256: value.source_sha256,
            target,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use kinekino_player::domain::{
        api_values::DecimalU64,
        proposals::{DocumentTextRef, SourceExtraction},
    };
    fn selection(target: NativeProposalTarget) -> NativeProposalSelection {
        NativeProposalSelection {
            proposal_id: "selected".into(),
            document_revision: DecimalU64(9_007_199_254_740_993),
            source_id: "source".into(),
            source_revision: DecimalU64(u64::MAX),
            source_sha256: "a".repeat(64),
            target,
        }
    }
    fn reference() -> DocumentTextRef {
        DocumentTextRef {
            source_asset_sha256: "a".repeat(64),
            outer_pointer: "/messages/0/content".into(),
            extraction: SourceExtraction::AsrRequestAfterNoThinkLfV1,
            inner_pointer: "/1".into(),
        }
    }
    #[test]
    fn checked_item_conversion_preserves_precision_and_explicit_second_item() {
        let converted =
            BootstrapSelection::try_from(selection(NativeProposalTarget::Card { index: 1 }))
                .unwrap();
        assert_eq!(converted.document_revision.0, 9_007_199_254_740_993);
        assert_eq!(converted.source_revision.0, u64::MAX);
        assert!(matches!(
            converted.target,
            BootstrapTarget::Card { index: 1 }
        ));
        let target = NativeProposalTarget::Highlight { index: u32::MAX };
        let expected = target.checked_item_index().unwrap().unwrap();
        assert!(
            matches!(BootstrapSelection::try_from(selection(target)).unwrap().target, BootstrapTarget::Highlight { index } if index == expected)
        );
    }
    #[test]
    fn selected_and_unselected_repair_never_substitute_for_each_other() {
        let reference = reference();
        let selected = selection(NativeProposalTarget::DocumentRepair {
            selection: DocumentRepairSelection::Selected {
                source_ref: reference.clone(),
            },
        });
        assert!(
            matches!(BootstrapSelection::try_from(selected).unwrap().target, BootstrapTarget::DocumentRepair { source_ref: Some(actual) } if actual == reference)
        );
        let unselected = selection(NativeProposalTarget::DocumentRepair {
            selection: DocumentRepairSelection::Unselected {},
        });
        assert!(matches!(
            BootstrapSelection::try_from(unselected).unwrap().target,
            BootstrapTarget::DocumentRepair { source_ref: None }
        ));
    }
    #[test]
    fn both_correction_targets_keep_the_full_explicit_identity() {
        let reference = reference();
        let document = selection(NativeProposalTarget::DocumentCorrection {
            index: 1,
            source_ref: reference.clone(),
        });
        assert!(
            matches!(BootstrapSelection::try_from(document).unwrap().target, BootstrapTarget::DocumentCorrection { index: 1, source_ref } if source_ref == reference)
        );
        let cue = selection(NativeProposalTarget::CueCorrection {
            index: 1,
            cue_id: "second-cue".into(),
        });
        assert!(
            matches!(BootstrapSelection::try_from(cue).unwrap().target, BootstrapTarget::CueCorrection { index: 1, cue_id } if cue_id == "second-cue")
        );
    }
}
