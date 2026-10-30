//! Original queue and opaque field tokens, without typed editor component props.
use super::super::{
    IntentPins, QueueBinding, ReviewAction, ReviewBindings, ReviewIntents, ReviewSession,
    TextField, TextFieldKind, ViewError,
    panel_model::{PanelBudget, PanelPreparationError},
};
use kinekino_player::domain::proposals::ProposalTask;
use meari::domain::{Value, build::TargetKind};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

type BoundField = (&'static str, &'static str, TextField);
pub(super) fn fields(review: &ReviewSession) -> Result<Vec<BoundField>, ViewError> {
    let all = review.text_fields();
    if all.is_empty() {
        return Ok(Vec::new());
    }
    let expected: &[(&'static str, &'static str, TextFieldKind)] =
        match review.stored().document.generated.task {
            ProposalTask::Cards => &[
                ("front", "onFrontChange", TextFieldKind::CardFront),
                ("back", "onBackChange", TextFieldKind::CardBack),
            ],
            ProposalTask::Highlights => {
                &[("title", "onTitleChange", TextFieldKind::HighlightTitle)]
            }
            ProposalTask::Correction => &[(
                "workingText",
                "onDraftChange",
                if review.stored().document.source.as_document().is_some() {
                    TextFieldKind::DocumentCorrection
                } else {
                    TextFieldKind::CueCorrection
                },
            )],
        };
    if all.len() < expected.len() {
        return Err(ViewError::InvalidSelection);
    }
    expected
        .iter()
        .zip(all)
        .map(|((prop, callback, kind), field)| {
            if *kind != field.kind {
                return Err(ViewError::InvalidSelection);
            }
            Ok((*prop, *callback, field))
        })
        .collect()
}
pub(super) fn prepare(
    review: &ReviewSession,
    budget: PanelBudget,
    pins: IntentPins,
    props: &mut BTreeMap<String, Value>,
) -> Result<ReviewBindings, PanelPreparationError> {
    let queue = Rc::new(RefCell::new(
        ReviewIntents::new(review, budget.entries, budget.text_bytes)
            .map_err(PanelPreparationError::Queue)?,
    ));
    let node = queue
        .borrow_mut()
        .bind_node()
        .map_err(PanelPreparationError::Queue)?;
    let mut bindings = BTreeMap::from([(
        "onSourceChoice".into(),
        QueueBinding::DocumentChoice { node: node.clone() },
    )]);
    for name in [
        "onFrontChange",
        "onBackChange",
        "onTitleChange",
        "onDraftChange",
    ] {
        bindings.insert(
            name.into(),
            QueueBinding::UnboundText { node: node.clone() },
        );
    }
    for (_, callback, field) in fields(review).map_err(PanelPreparationError::View)? {
        bindings.insert(
            callback.into(),
            QueueBinding::Text {
                node: node.clone(),
                field: field.input,
            },
        );
    }
    for (name, action) in [
        ("onStartManual", ReviewAction::BeginManualRepair),
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
    super::super::output_review_panel::bind_component_callbacks(&node, &mut bindings);
    super::super::raw_review_panel::bind_callbacks(&node, &mut bindings);
    // Every reference comes from the same explicit base contract and original binding.
    for (name, binding) in &bindings {
        props.insert(
            name.clone(),
            Value::CallbackReference(meari::domain::host_callbacks::CallbackReference {
                binding: name.clone(),
                signature: binding.signature(),
            }),
        );
    }
    Ok(ReviewBindings {
        queue,
        bindings,
        pins,
        targets: BTreeSet::from([TargetKind::Native]),
    })
}
