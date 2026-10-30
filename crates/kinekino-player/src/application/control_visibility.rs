use std::time::Duration;
#[derive(Default)]
pub struct ControlVisibility {
    last_input: Duration,
}
impl ControlVisibility {
    pub fn visible(&mut self, now: Duration, fullscreen: bool, active: bool, pinned: bool) -> bool {
        if active || pinned || !fullscreen {
            self.last_input = now;
        }
        !fullscreen || pinned || now.saturating_sub(self.last_input) < Duration::from_secs(3)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hides_idle_and_reveals_input_or_pinned_controls() {
        let mut state = ControlVisibility::default();
        assert!(state.visible(Duration::ZERO, true, true, false));
        assert!(!state.visible(Duration::from_secs(3), true, false, false));
        assert!(state.visible(Duration::from_secs(4), true, true, false));
        assert!(state.visible(Duration::from_secs(8), true, false, true));
        assert!(state.visible(Duration::from_secs(20), false, false, false));
    }
}
