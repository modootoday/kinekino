//! Pure initial props from explicitly selected canonical snapshots.
use super::panel_model::PanelLabels;
use kinekino_player::domain::{api_values::DecimalU64, proposals::*};
use meari::{
    domain::{
        Value,
        host_callbacks::{CallbackReference, CallbackSignature},
    },
    runtime::IntoValue,
};
use std::collections::BTreeMap;

pub enum BootstrapTarget {
    Card {
        index: usize,
    },
    Highlight {
        index: usize,
    },
    CueCorrection {
        index: usize,
        cue_id: String,
    },
    DocumentCorrection {
        index: usize,
        source_ref: DocumentTextRef,
    },
    /// No selection is explicit, not an instruction to select the first source text.
    DocumentRepair {
        source_ref: Option<DocumentTextRef>,
    },
    ManualRepair {
        task: ProposalTask,
    },
}
pub struct BootstrapSelection {
    pub proposal_id: String,
    pub document_revision: DecimalU64,
    pub source_id: String,
    pub source_revision: DecimalU64,
    pub source_sha256: String,
    pub target: BootstrapTarget,
}
pub struct BootstrapLabels {
    pub panel: PanelLabels,
    pub source_count_label: String,
    pub unselected_source_text: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootstrapPanel {
    Card,
    Highlight,
    Correction,
    DocumentRepair,
    ManualRepair,
}
impl BootstrapPanel {
    pub fn entry(self) -> &'static str {
        match self {
            Self::Card => "studio/cards/proposal-panel.tsx",
            Self::Highlight => "studio/clips/highlight-proposal-panel.tsx",
            Self::Correction => "verification/proposal-review-intent.tsx",
            Self::DocumentRepair => "studio/subtitles/document-repair.tsx",
            Self::ManualRepair => "studio/manual-repair.tsx",
        }
    }
}
pub struct BootstrapProjection<'a> {
    pub panel: BootstrapPanel,
    pub props: BTreeMap<String, Value>,
    /// Entire canonical metadata stays available, including future optional persisted fields.
    pub stored: &'a StoredProposal,
    pub selection: &'a BootstrapSelection,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootstrapError {
    InvalidSnapshot(&'static str),
    StaleSelection,
    NoTypedDraft,
    TargetMismatch,
    ItemOutOfRange,
    SourceReferenceMismatch,
}

pub fn project_initial<'a>(
    stored: &'a StoredProposal,
    selected: &'a BootstrapSelection,
    labels: BootstrapLabels,
) -> Result<BootstrapProjection<'a>, BootstrapError> {
    stored.validate().map_err(BootstrapError::InvalidSnapshot)?;
    let document = &stored.document;
    let source = &document.source;
    if selected.proposal_id != document.proposal_id
        || selected.document_revision != document.draft_revision
        || selected.source_id != source.source_id()
        || selected.source_revision != source.source_revision()
        || selected.source_sha256 != source.source_sha256()
    {
        return Err(BootstrapError::StaleSelection);
    }

    let feedback_labels = labels.panel.clone();
    let (panel, mut props) = match &selected.target {
        BootstrapTarget::ManualRepair { task } => {
            if document.generated.task != *task
                || document.draft.is_some()
                || document.state != DraftState::NeedsRepair
            {
                return Err(BootstrapError::TargetMismatch);
            }
            (
                BootstrapPanel::ManualRepair,
                super::repair_panel::initial_repair_props(stored)
                    .map_err(|_| BootstrapError::InvalidSnapshot("Invalid repair snapshot"))?,
            )
        }
        BootstrapTarget::Card { index } => {
            let ProposalPayload::Cards { cards } = document
                .draft
                .as_ref()
                .ok_or(BootstrapError::NoTypedDraft)?
            else {
                return Err(BootstrapError::TargetMismatch);
            };
            let card = cards.get(*index).ok_or(BootstrapError::ItemOutOfRange)?;
            let selected_label = labels.panel.selected_item.clone();
            let mut props = panel_props(stored, &card.source_refs, labels.panel)?;
            props.insert("selectedCardLabel".into(), Value::String(selected_label));
            props.insert("front".into(), Value::String(card.front.clone()));
            props.insert("back".into(), Value::String(card.back.clone()));
            callback(&mut props, "onFrontChange", CallbackSignature::Text);
            callback(&mut props, "onBackChange", CallbackSignature::Text);
            (BootstrapPanel::Card, props)
        }
        BootstrapTarget::Highlight { index } => {
            let ProposalPayload::Highlights { highlights } = document
                .draft
                .as_ref()
                .ok_or(BootstrapError::NoTypedDraft)?
            else {
                return Err(BootstrapError::TargetMismatch);
            };
            let highlight = highlights
                .get(*index)
                .ok_or(BootstrapError::ItemOutOfRange)?;
            let range = labels.panel.range.clone();
            let selected_label = labels.panel.selected_item.clone();
            let mut props = panel_props(stored, &highlight.source_refs, labels.panel)?;
            props.insert(
                "selectedCandidateLabel".into(),
                Value::String(selected_label),
            );
            props.insert("title".into(), Value::String(highlight.title.clone()));
            props.insert("rangeLabel".into(), Value::String(range));
            callback(&mut props, "onTitleChange", CallbackSignature::Text);
            (BootstrapPanel::Highlight, props)
        }
        BootstrapTarget::CueCorrection { index, cue_id } => {
            let ProposalPayload::Correction { edits } = document
                .draft
                .as_ref()
                .ok_or(BootstrapError::NoTypedDraft)?
            else {
                return Err(BootstrapError::TargetMismatch);
            };
            let edit = edits.get(*index).ok_or(BootstrapError::ItemOutOfRange)?;
            if &edit.cue_id != cue_id {
                return Err(BootstrapError::SourceReferenceMismatch);
            }
            (
                BootstrapPanel::Correction,
                correction_props(stored, &edit.text, true, labels.panel.result),
            )
        }
        BootstrapTarget::DocumentCorrection { index, source_ref } => {
            let ProposalPayload::DocumentCorrection { edits } = document
                .draft
                .as_ref()
                .ok_or(BootstrapError::NoTypedDraft)?
            else {
                return Err(BootstrapError::TargetMismatch);
            };
            let edit = edits.get(*index).ok_or(BootstrapError::ItemOutOfRange)?;
            if &edit.source_ref != source_ref {
                return Err(BootstrapError::SourceReferenceMismatch);
            }
            (
                BootstrapPanel::Correction,
                correction_props(stored, &edit.text, true, labels.panel.result),
            )
        }
        BootstrapTarget::DocumentRepair { source_ref } => {
            let SourceSnapshot::Document(source) = source else {
                return Err(BootstrapError::TargetMismatch);
            };
            if document.draft.is_some() {
                return Err(BootstrapError::TargetMismatch);
            }
            let (choice, text) = match source_ref {
                None => (String::new(), labels.unselected_source_text),
                Some(reference) => {
                    let (index, text) = source
                        .texts
                        .iter()
                        .enumerate()
                        .find(|(_, text)| {
                            source.text_reference(&text.inner_pointer).as_ref() == Some(reference)
                        })
                        .ok_or(BootstrapError::SourceReferenceMismatch)?;
                    ((index + 1).to_string(), text.text.clone())
                }
            };
            let mut props = correction_props(stored, "", false, labels.panel.result);
            props.extend([
                (
                    "sourceCountLabel".into(),
                    Value::String(labels.source_count_label),
                ),
                ("sourceChoice".into(), Value::String(choice)),
                ("selectedSourceText".into(), Value::String(text)),
                (
                    "generatedRaw".into(),
                    Value::String(document.generated.raw_text.clone()),
                ),
                (
                    "importError".into(),
                    Value::String(document.import_error.clone().unwrap_or_default()),
                ),
                ("selectionDisabled".into(), Value::Bool(false)),
                ("startDisabled".into(), Value::Bool(source_ref.is_none())),
            ]);
            callback(&mut props, "onSourceChoice", CallbackSignature::Text);
            callback(&mut props, "onStartManual", CallbackSignature::Action);
            (BootstrapPanel::DocumentRepair, props)
        }
    };
    if panel != BootstrapPanel::ManualRepair {
        props.extend(super::historical_source_props(
            stored.historical.as_ref().map(|binding| &binding.reference),
        ));
    }
    if matches!(panel, BootstrapPanel::Card | BootstrapPanel::Highlight) {
        props.extend(super::output_review_panel::initial_component_props(
            &document.proposal_id,
        ));
    }
    let feedback = super::review_feedback::ReviewFeedback::from_stored(stored);
    if panel == BootstrapPanel::ManualRepair {
        super::repair_panel::apply_feedback(&feedback, &mut props, &feedback_labels);
    } else {
        feedback.apply(&mut props, &feedback_labels);
    }
    Ok(BootstrapProjection {
        panel,
        props,
        stored,
        selection: selected,
    })
}
fn callback(props: &mut BTreeMap<String, Value>, name: &str, signature: CallbackSignature) {
    props.insert(
        name.into(),
        Value::CallbackReference(CallbackReference {
            binding: name.into(),
            signature,
        }),
    );
}
fn panel_props(
    stored: &StoredProposal,
    refs: &[String],
    labels: PanelLabels,
) -> Result<BTreeMap<String, Value>, BootstrapError> {
    let source = &stored.document.source;
    let mut text = Vec::with_capacity(refs.len());
    for reference in refs {
        text.push(
            source
                .cues()
                .iter()
                .find(|cue| &cue.cue_id == reference)
                .ok_or(BootstrapError::SourceReferenceMismatch)?
                .text
                .as_str(),
        );
    }
    let mut props = BTreeMap::from([
        (
            "proposalId".into(),
            Value::String(stored.document.proposal_id.clone()),
        ),
        ("revisionLabel".into(), Value::String(labels.revision)),
        (
            "sourceIdentityLabel".into(),
            Value::String(labels.source_identity),
        ),
        ("sourceRefs".into(), refs.to_vec().into_value()),
        ("sourceText".into(), Value::String(text.join("\n"))),
        (
            "generatedRaw".into(),
            Value::String(stored.document.generated.raw_text.clone()),
        ),
        ("qualityLabel".into(), Value::String(labels.quality_unknown)),
        ("resultLabel".into(), Value::String(labels.result)),
        ("canEdit".into(), Value::Bool(true)),
        ("canSave".into(), Value::Bool(false)),
        ("canAccept".into(), Value::Bool(true)),
        ("busy".into(), Value::Bool(false)),
        ("refreshRequired".into(), Value::Bool(false)),
    ]);
    for name in ["onSave", "onAccept", "onRefresh"] {
        callback(&mut props, name, CallbackSignature::Action);
    }
    Ok(props)
}
fn correction_props(
    stored: &StoredProposal,
    text: &str,
    has_draft: bool,
    result: String,
) -> BTreeMap<String, Value> {
    let mut props = BTreeMap::from([
        (
            "proposalId".into(),
            Value::String(stored.document.proposal_id.clone()),
        ),
        (
            "sourceSha256".into(),
            Value::String(stored.document.source.source_sha256().into()),
        ),
        (
            "documentRevision".into(),
            Value::String(stored.document.draft_revision.0.to_string()),
        ),
        ("workingText".into(), Value::String(text.into())),
        ("errorText".into(), Value::String(result)),
        ("inputDisabled".into(), Value::Bool(!has_draft)),
        ("saveDisabled".into(), Value::Bool(true)),
        ("acceptDisabled".into(), Value::Bool(!has_draft)),
        ("refreshDisabled".into(), Value::Bool(true)),
    ]);
    callback(&mut props, "onDraftChange", CallbackSignature::Text);
    for name in ["onSave", "onAccept", "onRefresh"] {
        callback(&mut props, name, CallbackSignature::Action);
    }
    props
}

#[cfg(test)]
#[path = "bootstrap/tests.rs"]
mod tests;
