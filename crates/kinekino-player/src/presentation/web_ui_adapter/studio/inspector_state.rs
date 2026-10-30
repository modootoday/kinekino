//! Controlled state for the original InspectorDock declaration.
//! Native Dialog payload delivery and focus execution still require the P1b host adapter.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectorLayout {
    Wide,
    Drawer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InspectorProjection {
    /// The original `open` prop; wide visibility is derived separately.
    pub drawer_open: bool,
    pub effective_open: bool,
    pub modal: bool,
    pub content_hidden: bool,
    pub trigger_hidden: bool,
    pub header_hidden: bool,
}

/// A request for the existing host focus port, not evidence that focus moved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectorFocusRequest {
    SettingsTrigger,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InspectorViewState {
    layout: InspectorLayout,
    drawer_open: bool,
}

impl InspectorViewState {
    pub fn new(layout: InspectorLayout, drawer_open: bool) -> Self {
        Self {
            layout,
            drawer_open,
        }
    }

    /// Viewport observation supplies layout; this module does not query the OS or copy tokens.
    pub fn observe_layout(&mut self, layout: InspectorLayout) {
        self.layout = layout;
    }

    pub fn projection(&self) -> InspectorProjection {
        let narrow = self.layout == InspectorLayout::Drawer;
        InspectorProjection {
            drawer_open: self.drawer_open,
            effective_open: !narrow || self.drawer_open,
            modal: narrow,
            content_hidden: narrow && !self.drawer_open,
            trigger_hidden: !narrow,
            header_hidden: !narrow,
        }
    }

    /// Receives a validated Boolean intent; this state never owns or edits SDK drafts.
    /// A wide nonmodal inspector cannot dismiss itself or overwrite the drawer preference.
    pub fn apply_open_change(&mut self, open: bool) -> Option<InspectorFocusRequest> {
        if self.layout != InspectorLayout::Drawer {
            return None;
        }
        let was_open = self.drawer_open;
        self.drawer_open = open;
        (was_open && !open).then_some(InspectorFocusRequest::SettingsTrigger)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_changes_preserve_drawer_preference() {
        let mut state = InspectorViewState::new(InspectorLayout::Drawer, true);
        state.observe_layout(InspectorLayout::Wide);
        assert_eq!(state.apply_open_change(false), None);
        assert_eq!(
            state.projection(),
            InspectorProjection {
                drawer_open: true,
                effective_open: true,
                modal: false,
                content_hidden: false,
                trigger_hidden: true,
                header_hidden: true,
            }
        );
        state.observe_layout(InspectorLayout::Drawer);
        assert!(state.projection().drawer_open);
        assert!(state.projection().modal);
    }

    #[test]
    fn drawer_visibility_and_focus_request_are_projected_once() {
        let mut state = InspectorViewState::new(InspectorLayout::Drawer, false);
        assert!(state.projection().content_hidden);
        assert_eq!(state.apply_open_change(true), None);
        assert!(!state.projection().content_hidden);
        assert_eq!(
            state.apply_open_change(false),
            Some(InspectorFocusRequest::SettingsTrigger)
        );
        assert_eq!(state.apply_open_change(false), None);
        state.observe_layout(InspectorLayout::Wide);
        assert!(!state.projection().content_hidden);
        assert!(!state.projection().drawer_open);
    }
}
