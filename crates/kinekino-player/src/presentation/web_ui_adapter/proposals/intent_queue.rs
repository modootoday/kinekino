use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use super::document_controller::{InteractionController, InteractionError};
use super::{ReviewController, ReviewHost, ReviewScope, ReviewSession, TextInput, ViewError};
#[path = "output_review_intents.rs"]
pub mod output_review_intents;
use output_review_intents::{OutputReviewAction, OutputReviewField, OutputReviewIntent};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewAction {
    Save,
    Accept,
    Refresh,
    BeginDocumentRepair,
    BeginManualRepair,
}

/// Host-held node lifetime. The renderer must still enforce Meari's real node/event guard.
#[derive(Clone, Debug)]
pub struct NodeLease {
    owner: Arc<()>,
    alive: Arc<AtomicBool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntentError {
    InvalidBudget,
    Capacity,
    TextBudget,
    /// Each text fits, but queued texts must drain before this text can fit.
    TextCapacity,
    Stale,
    DuplicateAction,
    NoDraft,
}

#[derive(PartialEq, Eq)]
enum TextTarget {
    RawReview(super::raw_review_state::RawReviewField),
    Draft(TextInput),
    DocumentChoice,
    OutputReview(OutputReviewField),
}

enum Intent {
    RawReview(super::raw_review_state::RawReviewButton),
    Text { target: TextTarget, text: String },
    Action(ReviewAction),
    OutputReviewAction(OutputReviewAction),
}
struct Entry {
    node: NodeLease,
    intent: Intent,
}

#[derive(Debug)]
pub struct DrainReport {
    pub processed: usize,
    pub discarded: usize,
    pub view_error: Option<ViewError>,
    /// The canonical error remains in ReviewSession; processed does not mean SDK success.
    pub sdk_error: bool,
    pub transport_error: Option<meari::application::plugins::PluginError>,
}

/// Local presentation mailbox, not an SDK job queue. Drain only after callback dispatch returns.
pub struct ReviewIntents {
    owner: Arc<()>,
    session_owner: Arc<()>,
    scope: ReviewScope,
    capacity: usize,
    max_text_bytes: usize,
    nodes: usize,
    closed: Arc<AtomicBool>,
    entries: VecDeque<Entry>,
}

impl ReviewIntents {
    pub(super) fn close_signal(&self) -> Arc<AtomicBool> {
        self.closed.clone()
    }
    pub(super) fn belongs_to(&self, session: &ReviewSession) -> bool {
        !self.closed.load(Ordering::Acquire)
            && self.scope == *session.scope()
            && Arc::ptr_eq(&self.session_owner, session.input_owner())
    }
    pub(super) fn scope(&self) -> &ReviewScope {
        &self.scope
    }
    /// Bounds are presentation transport memory budgets, not product text validation policies.
    pub fn new(
        session: &ReviewSession,
        capacity: usize,
        max_text_bytes: usize,
    ) -> Result<Self, IntentError> {
        if !(1..=64).contains(&capacity) || !(1..=1_048_576).contains(&max_text_bytes) {
            return Err(IntentError::InvalidBudget);
        }
        Ok(Self {
            owner: Arc::new(()),
            session_owner: session.input_owner().clone(),
            scope: session.scope().clone(),
            capacity,
            max_text_bytes,
            nodes: 0,
            closed: Arc::new(AtomicBool::new(false)),
            entries: VecDeque::new(),
        })
    }

    pub fn bind_node(&mut self) -> Result<NodeLease, IntentError> {
        if self.closed.load(Ordering::Acquire) {
            return Err(IntentError::Stale);
        }
        if self.nodes >= self.capacity {
            return Err(IntentError::Capacity);
        }
        self.nodes += 1;
        Ok(NodeLease {
            owner: self.owner.clone(),
            alive: Arc::new(AtomicBool::new(true)),
        })
    }

    pub(super) fn check(&self, node: &NodeLease) -> Result<(), IntentError> {
        if self.closed.load(Ordering::Acquire)
            || !Arc::ptr_eq(&node.owner, &self.owner)
            || !node.alive.load(Ordering::Relaxed)
        {
            Err(IntentError::Stale)
        } else {
            Ok(())
        }
    }

    pub fn retire_node(&mut self, node: &NodeLease) -> Result<(), IntentError> {
        self.check(node)?;
        node.alive.store(false, Ordering::Relaxed);
        self.nodes -= 1;
        // Retain queued entries so drain reports a stale event and cancels following actions.
        Ok(())
    }

    pub fn enqueue_text(
        &mut self,
        node: &NodeLease,
        field: TextInput,
        text: String,
    ) -> Result<(), IntentError> {
        self.enqueue_target(node, TextTarget::Draft(field), text)
    }
    pub fn enqueue_document_choice(
        &mut self,
        node: &NodeLease,
        text: String,
    ) -> Result<(), IntentError> {
        self.enqueue_target(node, TextTarget::DocumentChoice, text)
    }
    pub fn enqueue_raw_text(
        &mut self,
        node: &NodeLease,
        field: super::raw_review_state::RawReviewField,
        text: String,
    ) -> Result<(), IntentError> {
        self.enqueue_target(node, TextTarget::RawReview(field), text)
    }
    pub fn enqueue_raw_action(
        &mut self,
        node: &NodeLease,
        action: super::raw_review_state::RawReviewButton,
    ) -> Result<(), IntentError> {
        self.check(node)?;
        if self.entries.iter().any(
            |entry| matches!(&entry.intent, Intent::RawReview(previous) if *previous == action),
        ) {
            return Err(IntentError::DuplicateAction);
        }
        if self.entries.len() >= self.capacity {
            return Err(IntentError::Capacity);
        }
        self.entries.push_back(Entry {
            node: node.clone(),
            intent: Intent::RawReview(action),
        });
        Ok(())
    }
    pub fn reject_unbound_text(&self, node: &NodeLease) -> Result<(), IntentError> {
        self.check(node)?;
        Err(IntentError::NoDraft)
    }
    pub fn enqueue_output_review_text(
        &mut self,
        node: &NodeLease,
        field: OutputReviewField,
        text: String,
    ) -> Result<(), IntentError> {
        self.enqueue_target(node, TextTarget::OutputReview(field), text)
    }
    pub fn enqueue_output_review_action(
        &mut self,
        node: &NodeLease,
        action: OutputReviewAction,
    ) -> Result<(), IntentError> {
        self.check(node)?;
        if self.entries.iter().any(|entry| matches!(&entry.intent, Intent::OutputReviewAction(queued) if *queued == action)) { return Err(IntentError::DuplicateAction); }
        if self.entries.len() >= self.capacity {
            return Err(IntentError::Capacity);
        }
        self.entries.push_back(Entry {
            node: node.clone(),
            intent: Intent::OutputReviewAction(action),
        });
        Ok(())
    }
    pub(super) fn validate_single_text(&self, text: &str) -> Result<(), IntentError> {
        if text.len() > self.max_text_bytes {
            Err(IntentError::TextBudget)
        } else {
            Ok(())
        }
    }
    fn enqueue_target(
        &mut self,
        node: &NodeLease,
        target: TextTarget,
        text: String,
    ) -> Result<(), IntentError> {
        self.check(node)?;
        if let TextTarget::Draft(field) = &target
            && !Arc::ptr_eq(&field.owner, &self.session_owner)
        {
            return Err(IntentError::Stale);
        }
        self.validate_single_text(&text)?;
        let replace = self.entries.back().is_some_and(|entry| {
            Arc::ptr_eq(&entry.node.alive, &node.alive)
                && matches!(&entry.intent, Intent::Text { target: previous, .. } if previous == &target)
        });
        let mut bytes = self
            .entries
            .iter()
            .map(|entry| match &entry.intent {
                Intent::Text { text, .. } => text.len(),
                _ => 0,
            })
            .sum::<usize>();
        if replace
            && let Some(Entry {
                intent: Intent::Text { text, .. },
                ..
            }) = self.entries.back()
        {
            bytes -= text.len();
        }
        if text.len() > self.max_text_bytes.saturating_sub(bytes) {
            return Err(IntentError::TextCapacity);
        }
        if replace {
            self.entries.pop_back();
        } else if self.entries.len() >= self.capacity {
            return Err(IntentError::Capacity);
        }
        self.entries.push_back(Entry {
            node: node.clone(),
            intent: Intent::Text { target, text },
        });
        Ok(())
    }

    pub fn enqueue_action(
        &mut self,
        node: &NodeLease,
        action: ReviewAction,
    ) -> Result<(), IntentError> {
        self.check(node)?;
        if self
            .entries
            .iter()
            .any(|entry| matches!(&entry.intent, Intent::Action(queued) if *queued == action))
        {
            return Err(IntentError::DuplicateAction);
        }
        if self.entries.len() >= self.capacity {
            return Err(IntentError::Capacity);
        }
        self.entries.push_back(Entry {
            node: node.clone(),
            intent: Intent::Action(action),
        });
        Ok(())
    }

    pub fn close(&mut self) -> usize {
        self.closed.store(true, Ordering::Release);
        let discarded = self.entries.len();
        self.entries.clear();
        discarded
    }

    pub fn drain<H: ReviewHost>(&mut self, controller: &mut ReviewController<H>) -> DrainReport {
        self.drain_interactions(controller)
    }
    pub fn drain_interactions(
        &mut self,
        controller: &mut impl InteractionController,
    ) -> DrainReport {
        let mut report = DrainReport {
            processed: 0,
            discarded: 0,
            view_error: None,
            sdk_error: false,
            transport_error: None,
        };
        if self.closed.load(Ordering::Acquire)
            || self.scope != *controller.session().scope()
            || !Arc::ptr_eq(&self.session_owner, controller.session().input_owner())
        {
            report.discarded = self.close();
            report.view_error = Some(ViewError::Stale);
            return report;
        }
        while let Some(entry) = self.entries.pop_front() {
            let result = if self.check(&entry.node).is_err() {
                Err(InteractionError::View(ViewError::Stale))
            } else {
                match entry.intent {
                    Intent::RawReview(action) => {
                        controller.raw_review(&self.scope, action.action())
                    }
                    Intent::Text {
                        target: TextTarget::RawReview(field),
                        text,
                    } => controller.raw_review(&self.scope, field.action(text)),
                    Intent::Text {
                        target: TextTarget::OutputReview(field),
                        text,
                    } => controller
                        .output_review(&self.scope, OutputReviewIntent::Text { field, text }),
                    Intent::OutputReviewAction(action) => {
                        controller.output_review(&self.scope, OutputReviewIntent::Action(action))
                    }
                    Intent::Text {
                        target: TextTarget::Draft(field),
                        text,
                    } => controller
                        .session_mut()
                        .change_text(&self.scope, &field, text)
                        .map_err(InteractionError::View),
                    Intent::Text {
                        target: TextTarget::DocumentChoice,
                        text,
                    } => controller
                        .session_mut()
                        .choose_manual_source(&self.scope, text)
                        .map_err(InteractionError::View),
                    Intent::Action(ReviewAction::Save) => controller.save(&self.scope),
                    Intent::Action(ReviewAction::Accept) => controller.accept(&self.scope),
                    Intent::Action(ReviewAction::Refresh) => controller.refresh(),
                    Intent::Action(ReviewAction::BeginDocumentRepair) => controller
                        .session_mut()
                        .begin_document_repair(&self.scope)
                        .map_err(InteractionError::View),
                    Intent::Action(ReviewAction::BeginManualRepair) => controller
                        .session_mut()
                        .begin_manual_repair(&self.scope)
                        .map_err(InteractionError::View),
                }
            };
            if let Err(error) = result {
                match error {
                    InteractionError::View(error) => report.view_error = Some(error),
                    InteractionError::Plugin(error) => report.transport_error = Some(error),
                }
                report.discarded = 1 + self.entries.len();
                self.entries.clear();
                break;
            }
            report.processed += 1;
            if controller.session().error().is_some() || controller.output_review_error() {
                report.sdk_error = true;
                report.discarded = self.entries.len();
                self.entries.clear();
                break;
            }
        }
        report
    }
}
