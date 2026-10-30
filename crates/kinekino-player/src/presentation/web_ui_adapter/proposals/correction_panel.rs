//! Shared pure correction projection and bounded bindings, without SDK calls.
pub use super::panel_model::PanelBudget as CorrectionBudget;
pub use super::panel_model::PanelPreparationError as CorrectionPreparationError;
use super::panel_model::{PanelBudget, PanelLabels, PanelPreparationError, PreparedPanel};
use super::{
    IntentPins, QueueBinding, ReviewAction, ReviewBindings, ReviewIntents, ReviewScope,
    ReviewSession, TextFieldKind, TextInput, ViewError,
};
use kinekino_player::domain::proposals::*;
use meari::{
    domain::{
        Value,
        build::TargetKind,
        host_callbacks::{CallbackReference, CallbackSignature},
    },
    runtime::IntoValue,
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

#[derive(Clone)]
pub struct CorrectionLabels {
    pub panel: PanelLabels,
    pub source_count: String,
    pub unselected_source: String,
}
pub struct CorrectionSelection {
    field: Option<TextInput>,
}
fn view(error: ViewError) -> PanelPreparationError {
    PanelPreparationError::View(error)
}
impl CorrectionSelection {
    pub fn at(
        review: &ReviewSession,
        scope: &ReviewScope,
        index: usize,
    ) -> Result<Self, ViewError> {
        if review.scope() != scope {
            return Err(ViewError::Stale);
        }
        let field = review
            .text_fields()
            .into_iter()
            .nth(index)
            .ok_or(ViewError::InvalidSelection)?;
        if !matches!(
            field.kind,
            TextFieldKind::CueCorrection | TextFieldKind::DocumentCorrection
        ) {
            return Err(ViewError::InvalidSelection);
        }
        Ok(Self {
            field: Some(field.input),
        })
    }
    pub fn repair(review: &ReviewSession, scope: &ReviewScope) -> Result<Self, ViewError> {
        if review.scope() != scope {
            return Err(ViewError::Stale);
        }
        let display = review.display().map_err(|_| ViewError::InvalidSnapshot)?;
        if display.working_draft.is_some()
            || display.snapshot.task != ProposalTask::Correction
            || !matches!(display.snapshot.source, SourceSnapshot::Document(_))
        {
            return Err(ViewError::InvalidSelection);
        }
        Ok(Self { field: None })
    }
}
struct CorrectionState<'a> {
    stored: &'a StoredProposal,
    draft: Option<&'a ProposalPayload>,
    index: Option<usize>,
    choice: &'a str,
    selected_document: Option<&'a DocumentTextRef>,
    error: Option<&'a str>,
    dirty: bool,
    busy: bool,
    refresh: bool,
}
/// Startup caller supplies an explicit item, or None for unselected document repair.
pub fn initial_correction_props(
    stored: &StoredProposal,
    index: Option<usize>,
    labels: CorrectionLabels,
) -> Result<BTreeMap<String, Value>, PanelPreparationError> {
    initial_correction_props_with_selection(stored, index, None, labels)
}
pub fn initial_correction_props_with_selection(
    stored: &StoredProposal,
    index: Option<usize>,
    reference: Option<&DocumentTextRef>,
    labels: CorrectionLabels,
) -> Result<BTreeMap<String, Value>, PanelPreparationError> {
    let feedback_labels = labels.panel.clone();
    stored
        .validate()
        .map_err(|_| view(ViewError::InvalidSnapshot))?;
    let choice = match reference {
        Some(reference) => {
            if index.is_some() || stored.document.draft.is_some() {
                return Err(view(ViewError::InvalidSelection));
            }
            let source = stored
                .document
                .source
                .as_document()
                .ok_or_else(|| view(ViewError::InvalidSelection))?;
            let position = source
                .texts
                .iter()
                .position(|text| {
                    source.text_reference(&text.inner_pointer).as_ref() == Some(reference)
                })
                .ok_or_else(|| view(ViewError::InvalidSelection))?;
            (position + 1).to_string()
        }
        None => String::new(),
    };
    let mut props = project_props(
        CorrectionState {
            stored,
            draft: stored.document.draft.as_ref(),
            index,
            choice: &choice,
            selected_document: reference,
            error: None,
            dirty: false,
            busy: false,
            refresh: false,
        },
        labels,
    )?;
    super::review_feedback::ReviewFeedback::from_stored(stored).apply(&mut props, &feedback_labels);
    Ok(props)
}
fn project_props(
    state: CorrectionState<'_>,
    labels: CorrectionLabels,
) -> Result<BTreeMap<String, Value>, PanelPreparationError> {
    let document = &state.stored.document;
    if document.generated.task != ProposalTask::Correction {
        return Err(view(ViewError::InvalidSelection));
    }
    let (source_text, working_text, references) = match (state.draft, state.index) {
        (Some(ProposalPayload::Correction { edits }), Some(index)) => {
            let edit = edits
                .get(index)
                .ok_or_else(|| view(ViewError::InvalidSelection))?;
            let cue = document
                .source
                .cues()
                .iter()
                .find(|cue| cue.cue_id == edit.cue_id)
                .ok_or_else(|| view(ViewError::InvalidSnapshot))?;
            (
                cue.text.clone(),
                edit.text.clone(),
                vec![edit.cue_id.clone()],
            )
        }
        (Some(ProposalPayload::DocumentCorrection { edits }), Some(index)) => {
            let edit = edits
                .get(index)
                .ok_or_else(|| view(ViewError::InvalidSelection))?;
            let source = document
                .source
                .as_document()
                .ok_or_else(|| view(ViewError::InvalidSnapshot))?;
            let text = source
                .texts
                .iter()
                .find(|text| {
                    source.text_reference(&text.inner_pointer).as_ref() == Some(&edit.source_ref)
                })
                .ok_or_else(|| view(ViewError::InvalidSnapshot))?;
            (
                text.text.clone(),
                edit.text.clone(),
                vec![
                    serde_json::to_string(&edit.source_ref)
                        .map_err(|_| view(ViewError::InvalidSnapshot))?,
                ],
            )
        }
        (None, None) => {
            let source = document
                .source
                .as_document()
                .ok_or_else(|| view(ViewError::InvalidSelection))?;
            match state.selected_document {
                Some(reference) => {
                    let text = source
                        .texts
                        .iter()
                        .find(|text| {
                            source.text_reference(&text.inner_pointer).as_ref() == Some(reference)
                        })
                        .ok_or_else(|| view(ViewError::InvalidSelection))?;
                    (
                        text.text.clone(),
                        String::new(),
                        vec![
                            serde_json::to_string(reference)
                                .map_err(|_| view(ViewError::InvalidSnapshot))?,
                        ],
                    )
                }
                None => (labels.unselected_source, String::new(), Vec::new()),
            }
        }
        _ => return Err(view(ViewError::InvalidSelection)),
    };
    let has_field = state.index.is_some();
    let disabled = state.busy || state.refresh || !has_field;
    let result = state.error.unwrap_or(&labels.panel.result).to_owned();
    let mut props = BTreeMap::from([
        ("repairMode".into(), Value::Bool(state.draft.is_none())),
        (
            "proposalId".into(),
            Value::String(document.proposal_id.clone()),
        ),
        (
            "sourceSha256".into(),
            Value::String(document.source.source_sha256().into()),
        ),
        (
            "documentRevision".into(),
            Value::String(document.draft_revision.0.to_string()),
        ),
        ("workingText".into(), Value::String(working_text)),
        ("errorText".into(), Value::String(result.clone())),
        ("resultLabel".into(), Value::String(result)),
        ("revisionLabel".into(), Value::String(labels.panel.revision)),
        (
            "sourceIdentityLabel".into(),
            Value::String(labels.panel.source_identity),
        ),
        (
            "qualityLabel".into(),
            Value::String(labels.panel.quality_unknown),
        ),
        (
            "selectedItemLabel".into(),
            Value::String(labels.panel.selected_item),
        ),
        ("sourceRefs".into(), references.into_value()),
        ("sourceText".into(), Value::String(source_text.clone())),
        ("canEdit".into(), Value::Bool(!disabled)),
        ("canSave".into(), Value::Bool(!disabled && state.dirty)),
        ("canAccept".into(), Value::Bool(!disabled && !state.dirty)),
        ("busy".into(), Value::Bool(state.busy)),
        ("refreshRequired".into(), Value::Bool(state.refresh)),
        ("inputDisabled".into(), Value::Bool(disabled)),
        ("saveDisabled".into(), Value::Bool(disabled || !state.dirty)),
        (
            "acceptDisabled".into(),
            Value::Bool(disabled || state.dirty),
        ),
        ("refreshDisabled".into(), Value::Bool(!state.refresh)),
        (
            "sourceCountLabel".into(),
            Value::String(labels.source_count),
        ),
        ("sourceChoice".into(), Value::String(state.choice.into())),
        ("selectedSourceText".into(), Value::String(source_text)),
        (
            "generatedRaw".into(),
            Value::String(document.generated.raw_text.clone()),
        ),
        (
            "importError".into(),
            Value::String(document.import_error.clone().unwrap_or_default()),
        ),
        (
            "selectionDisabled".into(),
            Value::Bool(has_field || state.busy || state.refresh),
        ),
        (
            "startDisabled".into(),
            Value::Bool(
                has_field || state.busy || state.refresh || state.selected_document.is_none(),
            ),
        ),
    ]);
    for name in [
        "onDraftChange",
        "onSourceChoice",
        "onStartManual",
        "onSave",
        "onAccept",
        "onRefresh",
    ] {
        let signature = if matches!(name, "onDraftChange" | "onSourceChoice") {
            CallbackSignature::Text
        } else {
            CallbackSignature::Action
        };
        props.insert(
            name.into(),
            Value::CallbackReference(CallbackReference {
                binding: name.into(),
                signature,
            }),
        );
    }
    props.extend(super::historical_source_props(
        state
            .stored
            .historical
            .as_ref()
            .map(|binding| &binding.reference),
    ));
    props.extend(super::output_review_panel::initial_component_props(
        &document.proposal_id,
    ));
    Ok(props)
}
pub fn prepare_correction(
    review: &ReviewSession,
    scope: &ReviewScope,
    selected: &CorrectionSelection,
    labels: CorrectionLabels,
    budget: PanelBudget,
    pins: IntentPins,
) -> Result<PreparedPanel, PanelPreparationError> {
    let feedback_labels = labels.panel.clone();
    if review.scope() != scope {
        return Err(view(ViewError::Stale));
    }
    let fields = review.text_fields();
    let index = match &selected.field {
        Some(input) => Some(
            fields
                .iter()
                .position(|field| &field.input == input)
                .ok_or_else(|| view(ViewError::Stale))?,
        ),
        None => {
            CorrectionSelection::repair(review, scope).map_err(view)?;
            None
        }
    };
    let display = review
        .display()
        .map_err(|_| view(ViewError::InvalidSnapshot))?;
    let mut props = project_props(
        CorrectionState {
            stored: review.stored(),
            draft: display.working_draft,
            index,
            choice: review.document_choice(),
            selected_document: review.document_selection(),
            error: review.error().map(|error| error.message.as_str()),
            dirty: review.dirty(),
            busy: review.pending().is_some() && !display.refresh_required,
            refresh: display.refresh_required,
        },
        labels,
    )?;
    let queue = Rc::new(RefCell::new(
        ReviewIntents::new(review, budget.entries, budget.text_bytes)
            .map_err(PanelPreparationError::Queue)?,
    ));
    let node = queue
        .borrow_mut()
        .bind_node()
        .map_err(PanelPreparationError::Queue)?;
    let text = match &selected.field {
        Some(input) => QueueBinding::Text {
            node: node.clone(),
            field: input.clone(),
        },
        None => QueueBinding::UnboundText { node: node.clone() },
    };
    let mut bindings = BTreeMap::from([
        ("onDraftChange".into(), text),
        (
            "onSourceChoice".into(),
            QueueBinding::DocumentChoice { node: node.clone() },
        ),
    ]);
    for (name, action) in [
        ("onStartManual", ReviewAction::BeginDocumentRepair),
        ("onSave", ReviewAction::Save),
        ("onAccept", ReviewAction::Accept),
        ("onRefresh", ReviewAction::Refresh),
    ] {
        bindings.insert(
            name.into(),
            QueueBinding::Action {
                node: node.clone(),
                action,
            },
        );
    }
    super::output_review_panel::bind_component_callbacks(&node, &mut bindings);
    super::review_feedback::ReviewFeedback::from_review(review).apply(&mut props, &feedback_labels);
    Ok(PreparedPanel {
        props,
        bindings: ReviewBindings {
            queue,
            bindings,
            pins,
            targets: BTreeSet::from([TargetKind::Native]),
        },
    })
}
#[cfg(test)]
#[path = "correction_panel/tests.rs"]
mod tests;
