//! Frame-local hit regions and focus observations for the shared visibility use case.
use eframe::egui::{Pos2, Rect};

#[derive(Clone, Copy)]
pub(crate) enum ControlRegion {
    Toolbar,
    Transport,
}
#[derive(Default)]
pub(crate) struct ChromeInteraction {
    toolbar: Option<Rect>,
    transport: Option<Rect>,
    focused: bool,
}
pub(crate) struct InteractionObservation {
    pub hovering_controls: bool,
    pub resumed: bool,
}
impl ChromeInteraction {
    pub(crate) fn remember(&mut self, region: ControlRegion, rect: Rect) {
        match region {
            ControlRegion::Toolbar => self.toolbar = Some(rect),
            ControlRegion::Transport => self.transport = Some(rect),
        }
    }
    pub(crate) fn observe(
        &mut self,
        focused: bool,
        pointer: Option<Pos2>,
    ) -> InteractionObservation {
        let hovering_controls = focused
            && pointer.is_some_and(|point| {
                [self.toolbar, self.transport]
                    .into_iter()
                    .flatten()
                    .any(|rect| rect.contains(point))
            });
        let resumed = focused && !self.focused;
        self.focused = focused;
        // Only regions actually rendered after this observation survive into the next frame.
        self.toolbar = None;
        self.transport = None;
        InteractionObservation {
            hovering_controls,
            resumed,
        }
    }
}

#[cfg(test)]
#[path = "interaction/tests.rs"]
mod tests;
