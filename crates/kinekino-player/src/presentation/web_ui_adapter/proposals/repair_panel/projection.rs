use super::super::ViewError;
use super::super::{ReviewSession, SourceReference, panel_model::PanelLabels};
use kinekino_player::domain::proposals::ProposalPayload;
use kinekino_player::domain::proposals::ProposalTask;
use meari::domain::{
    Value,
    host_callbacks::{CallbackReference, CallbackSignature},
};
use meari::runtime::IntoValue;
use std::collections::BTreeMap;

pub(super) fn project(
    review: &ReviewSession,
    labels: &PanelLabels,
) -> Result<BTreeMap<String, Value>, ViewError> {
    let display = review.display().map_err(|_| ViewError::InvalidSnapshot)?;
    let task = display.snapshot.task;
    let repairing = display.working_draft.is_none();
    let mut props = initial_repair_props(review.stored())?;
    props.insert(
        "sourceChoice".into(),
        Value::String(review.document_choice().into()),
    );
    let source_text = match display.working_draft {
        None => review
            .manual_repair_entry()?
            .selected_source
            .unwrap_or_default()
            .to_owned(),
        Some(draft) => working_source(draft, &display.snapshot.original)?,
    };
    props.insert("sourceText".into(), Value::String(source_text));
    for (name, _, field) in super::bindings::fields(review)? {
        props.insert(name.into(), Value::String(field.text));
    }
    let editable = !repairing && !display.refresh_required && review.pending().is_none();
    let selected = if repairing {
        review.manual_repair_entry()?.selected_source.is_some()
    } else {
        false
    };
    for (name, value) in [
        ("selectionDisabled", !repairing),
        ("startDisabled", !repairing || !selected),
        ("cardDisabled", !editable || task != ProposalTask::Cards),
        (
            "highlightDisabled",
            !editable || task != ProposalTask::Highlights,
        ),
        (
            "correctionDisabled",
            !editable || task != ProposalTask::Correction,
        ),
        ("canSave", editable && review.dirty()),
        ("canAccept", editable && !review.dirty()),
        (
            "busy",
            review.pending().is_some() && !display.refresh_required,
        ),
        ("refreshRequired", display.refresh_required),
    ] {
        props.insert(name.into(), Value::Bool(value));
    }
    super::apply_feedback(
        &super::super::review_feedback::ReviewFeedback::from_review(review),
        &mut props,
        labels,
    );
    Ok(props)
}

fn working_source(
    draft: &ProposalPayload,
    original: &[super::super::OriginalText],
) -> Result<String, ViewError> {
    let cue_refs: Vec<&str> = match draft {
        ProposalPayload::Cards { cards } => cards
            .first()
            .ok_or(ViewError::InvalidSelection)?
            .source_refs
            .iter()
            .map(String::as_str)
            .collect(),
        ProposalPayload::Highlights { highlights } => highlights
            .first()
            .ok_or(ViewError::InvalidSelection)?
            .source_refs
            .iter()
            .map(String::as_str)
            .collect(),
        ProposalPayload::Correction { edits } => vec![
            edits
                .first()
                .ok_or(ViewError::InvalidSelection)?
                .cue_id
                .as_str(),
        ],
        ProposalPayload::DocumentCorrection { edits } => {
            let reference = &edits.first().ok_or(ViewError::InvalidSelection)?.source_ref;
            return original.iter().find(|text| matches!(&text.reference, SourceReference::Document { source_ref } if source_ref == reference)).map(|text| text.text.clone()).ok_or(ViewError::InvalidSnapshot);
        }
    };
    cue_refs.iter().map(|reference| original.iter().find(|text| matches!(&text.reference, SourceReference::Cue { cue_id } if cue_id.as_str() == *reference)).map(|text| text.text.as_str()).ok_or(ViewError::InvalidSnapshot)).collect::<Result<Vec<_>, _>>().map(|texts| texts.join("\n"))
}
pub fn initial_repair_props(
    stored: &kinekino_player::domain::proposals::StoredProposal,
) -> Result<BTreeMap<String, Value>, ViewError> {
    let task = stored.document.generated.task;
    let source_options = super::super::project(stored)
        .map_err(|_| ViewError::InvalidSnapshot)?
        .original
        .into_iter()
        .enumerate()
        .map(|(index, text)| format!("{}: {}", index + 1, text.text))
        .collect::<Vec<_>>();
    let mut props = BTreeMap::from([
        (
            "proposalId".into(),
            Value::String(stored.document.proposal_id.clone()),
        ),
        (
            "generatedRaw".into(),
            Value::String(stored.document.generated.raw_text.clone()),
        ),
        (
            "importError".into(),
            Value::String(stored.document.import_error.clone().unwrap_or_default()),
        ),
        ("sourceOptions".into(), source_options.into_value()),
        ("startDisabled".into(), Value::Bool(true)),
        ("selectionDisabled".into(), Value::Bool(false)),
    ]);
    for name in [
        "sourceChoice",
        "sourceText",
        "front",
        "back",
        "title",
        "workingText",
        "resultLabel",
    ] {
        props.insert(name.into(), Value::String(String::new()));
    }
    for name in ["canSave", "canAccept", "busy", "refreshRequired"] {
        props.insert(name.into(), Value::Bool(false));
    }
    for (name, value) in [
        ("isCard", task == ProposalTask::Cards),
        ("isHighlight", task == ProposalTask::Highlights),
        ("isCorrection", task == ProposalTask::Correction),
        ("cardDisabled", true),
        ("highlightDisabled", true),
        ("correctionDisabled", true),
    ] {
        props.insert(name.into(), Value::Bool(value));
    }
    for name in [
        "onSourceChoice",
        "onFrontChange",
        "onBackChange",
        "onTitleChange",
        "onDraftChange",
    ] {
        props.insert(
            name.into(),
            Value::CallbackReference(CallbackReference {
                binding: name.into(),
                signature: CallbackSignature::Text,
            }),
        );
    }
    for name in ["onStartManual", "onSave", "onAccept", "onRefresh"] {
        props.insert(
            name.into(),
            Value::CallbackReference(CallbackReference {
                binding: name.into(),
                signature: CallbackSignature::Action,
            }),
        );
    }
    props.extend(super::super::output_review_panel::initial_component_props(
        &stored.document.proposal_id,
    ));
    super::super::raw_review_panel::project_component_props(&mut props, None, false, None, None);
    Ok(props)
}
