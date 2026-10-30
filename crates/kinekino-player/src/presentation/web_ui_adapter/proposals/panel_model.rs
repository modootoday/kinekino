//! Product panel preparation candidate. Not registered until its consumer checkpoint.
use super::{
    IntentError, IntentPins, QueueBinding, ReviewAction, ReviewBindings, ReviewIntents, ReviewItem,
    ReviewScope, ReviewSession, TextInput, ViewError,
};
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

/// Host-issued selection belongs to one session/draft generation, not a stable domain item ID.
pub struct PanelSelection {
    anchor: TextInput,
}
impl PanelSelection {
    pub fn at(
        review: &ReviewSession,
        scope: &ReviewScope,
        index: usize,
    ) -> Result<Self, ViewError> {
        if review.scope() != scope {
            return Err(ViewError::Stale);
        }
        let item = review
            .review_items()
            .into_iter()
            .nth(index)
            .ok_or(ViewError::InvalidSelection)?;
        Ok(Self {
            anchor: anchor(&item).clone(),
        })
    }
    fn resolve(&self, review: &ReviewSession) -> Result<ReviewItem, ViewError> {
        review
            .review_items()
            .into_iter()
            .find(|item| anchor(item) == &self.anchor)
            .ok_or(ViewError::Stale)
    }
}
fn anchor(item: &ReviewItem) -> &TextInput {
    match item {
        ReviewItem::Card { front, .. } => &front.input,
        ReviewItem::Highlight { title, .. } => &title.input,
    }
}
/// Localized display text is supplied by presentation; no fixture labels or hidden locale policy.
#[derive(Clone)]
pub struct PanelLabels {
    pub feedback: super::review_feedback::FeedbackLabels,
    pub revision: String,
    pub source_identity: String,
    pub quality_unknown: String,
    pub selected_item: String,
    pub range: String,
    pub result: String,
}
pub struct PanelBudget {
    pub entries: usize,
    pub text_bytes: usize,
}
pub struct PreparedPanel {
    pub props: BTreeMap<String, Value>,
    pub bindings: ReviewBindings,
}
impl PreparedPanel {
    /// Transport activity includes the async follow-up inspect after a confirmed mutation.
    pub fn with_transport_pending(mut self, pending: bool) -> Self {
        if pending {
            for name in ["canEdit", "canSave", "canAccept"] {
                if self.props.contains_key(name) {
                    self.props.insert(name.into(), Value::Bool(false));
                }
            }
            if self.props.contains_key("busy") {
                self.props.insert("busy".into(), Value::Bool(true));
            }
            for name in [
                "inputDisabled",
                "saveDisabled",
                "acceptDisabled",
                "refreshDisabled",
                "selectionDisabled",
                "startDisabled",
                "cardDisabled",
                "highlightDisabled",
                "correctionDisabled",
            ] {
                if self.props.contains_key(name) {
                    self.props.insert(name.into(), Value::Bool(true));
                }
            }
        }
        self
    }
}
#[derive(Debug)]
pub enum PanelPreparationError {
    View(ViewError),
    Queue(IntentError),
}

/// Pure projection and bounded callback preparation; no registry, SDK, store or window creation.
pub fn prepare_panel(
    review: &ReviewSession,
    scope: &ReviewScope,
    selection: &PanelSelection,
    labels: PanelLabels,
    budget: PanelBudget,
    pins: IntentPins,
) -> Result<PreparedPanel, PanelPreparationError> {
    let feedback_labels = labels.clone();
    if review.scope() != scope {
        return Err(PanelPreparationError::View(ViewError::Stale));
    }
    let item = selection
        .resolve(review)
        .map_err(PanelPreparationError::View)?;
    let display = review
        .display()
        .map_err(|_| PanelPreparationError::View(ViewError::InvalidSnapshot))?;
    let refresh = display.refresh_required;
    let busy = review.pending().is_some() && !refresh;
    let (refs, text_fields, item_props) = match item {
        ReviewItem::Card {
            source_refs,
            front,
            back,
            ..
        } => (
            source_refs,
            vec![("onFrontChange", front.input), ("onBackChange", back.input)],
            vec![
                ("selectedCardLabel", labels.selected_item),
                ("front", front.text),
                ("back", back.text),
            ],
        ),
        ReviewItem::Highlight {
            source_refs, title, ..
        } => (
            source_refs,
            vec![("onTitleChange", title.input)],
            vec![
                ("selectedCandidateLabel", labels.selected_item),
                ("title", title.text),
                ("rangeLabel", labels.range),
            ],
        ),
    };
    let mut source_texts = Vec::with_capacity(refs.len());
    for reference in &refs {
        let original = display.snapshot.original.iter().find(|original| {
            matches!(&original.reference, super::SourceReference::Cue { cue_id } if cue_id == reference)
        }).ok_or(PanelPreparationError::View(ViewError::InvalidSelection))?;
        source_texts.push(original.text.as_str());
    }
    let mut props = BTreeMap::from([
        (
            "proposalId".into(),
            Value::String(display.snapshot.proposal_id.clone()),
        ),
        ("revisionLabel".into(), Value::String(labels.revision)),
        (
            "sourceIdentityLabel".into(),
            Value::String(labels.source_identity),
        ),
        ("sourceRefs".into(), refs.into_value()),
        ("sourceText".into(), Value::String(source_texts.join("\n"))),
        (
            "generatedRaw".into(),
            Value::String(display.snapshot.generated_raw.clone()),
        ),
        ("qualityLabel".into(), Value::String(labels.quality_unknown)),
        (
            "resultLabel".into(),
            Value::String(
                review
                    .error()
                    .map_or(labels.result, |error| error.message.clone()),
            ),
        ),
        ("canEdit".into(), Value::Bool(true)),
        ("canSave".into(), Value::Bool(review.dirty())),
        ("canAccept".into(), Value::Bool(!review.dirty())),
        ("busy".into(), Value::Bool(busy)),
        ("refreshRequired".into(), Value::Bool(refresh)),
    ]);
    for (name, text) in item_props {
        props.insert(name.into(), Value::String(text));
    }
    let queue = Rc::new(RefCell::new(
        ReviewIntents::new(review, budget.entries, budget.text_bytes)
            .map_err(PanelPreparationError::Queue)?,
    ));
    let node = queue
        .borrow_mut()
        .bind_node()
        .map_err(PanelPreparationError::Queue)?;
    let mut bindings = BTreeMap::new();
    for (name, field) in text_fields {
        props.insert(
            name.into(),
            Value::CallbackReference(CallbackReference {
                binding: name.into(),
                signature: CallbackSignature::Text,
            }),
        );
        bindings.insert(
            name.into(),
            QueueBinding::Text {
                node: node.clone(),
                field,
            },
        );
    }
    for (name, action) in [
        ("onSave", ReviewAction::Save),
        ("onAccept", ReviewAction::Accept),
        ("onRefresh", ReviewAction::Refresh),
    ] {
        props.insert(
            name.into(),
            Value::CallbackReference(CallbackReference {
                binding: name.into(),
                signature: CallbackSignature::Action,
            }),
        );
        bindings.insert(
            name.into(),
            QueueBinding::Action {
                node: node.clone(),
                action,
            },
        );
    }
    props.extend(super::historical_source_props(
        display.snapshot.historical_reference.as_ref(),
    ));
    props.extend(super::output_review_panel::initial_component_props(
        &display.snapshot.proposal_id,
    ));
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
