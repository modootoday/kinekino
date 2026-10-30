//! Bounded, UI-thread-only requests. Authority is checked by the existing callback registration.
use super::inspector_state::{InspectorLayout, InspectorViewState};
use std::collections::VecDeque;

enum InspectorIntent {
    Open(bool),
    Layout(InspectorLayout),
}

#[derive(Default)]
pub struct InspectorIntents {
    entries: VecDeque<InspectorIntent>,
}
impl InspectorIntents {
    pub const CAPACITY: usize = 16;
    pub fn enqueue_open(&mut self, open: bool) -> Result<(), &'static str> {
        self.enqueue(InspectorIntent::Open(open))
    }
    pub fn enqueue_layout(&mut self, layout: InspectorLayout) -> Result<(), &'static str> {
        self.enqueue(InspectorIntent::Layout(layout))
    }
    fn enqueue(&mut self, intent: InspectorIntent) -> Result<(), &'static str> {
        if self.entries.len() >= Self::CAPACITY {
            return Err("Inspector request queue is full");
        }
        self.entries.push_back(intent);
        Ok(())
    }
    pub fn pending(&self) -> bool {
        !self.entries.is_empty()
    }
    /// Consumes requests into a copy. Only the successful renderer commit promotes that copy.
    pub fn stage(&mut self, mut state: InspectorViewState) -> InspectorViewState {
        for intent in self.entries.drain(..) {
            match intent {
                InspectorIntent::Open(open) => {
                    let _ = state.apply_open_change(open);
                }
                InspectorIntent::Layout(layout) => state.observe_layout(layout),
            }
        }
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_fifo_stages_without_mutating_committed_state() {
        let committed = InspectorViewState::new(InspectorLayout::Drawer, false);
        let mut queue = InspectorIntents::default();
        for _ in 0..InspectorIntents::CAPACITY {
            queue.enqueue_open(true).unwrap();
        }
        assert!(queue.enqueue_open(false).is_err());
        let staged = queue.stage(committed);
        assert!(staged.projection().drawer_open);
        assert!(!committed.projection().drawer_open);
        assert!(!queue.pending());
        // Rejecting the staged value leaves committed state and does not replay the request.
        assert_eq!(queue.stage(committed), committed);
        queue.enqueue_open(true).unwrap();
        queue.enqueue_open(false).unwrap();
        assert_eq!(queue.stage(committed), committed);
    }
}
