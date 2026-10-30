/// User-selected playback route; a failed automatic attempt must not loop each UI frame.
#[derive(Default)]
pub struct PlaybackRoute {
    web_selected: bool,
    attempted_generation: Option<u64>,
}
impl PlaybackRoute {
    pub fn native_preferred(&self) -> bool {
        !self.web_selected
    }
    pub fn select_web(&mut self) {
        self.web_selected = true;
    }
    pub fn select_native(&mut self) {
        self.web_selected = false;
    }
    pub fn can_start(&self, generation: u64) -> bool {
        self.native_preferred() && self.attempted_generation != Some(generation)
    }
    pub fn attempted(&mut self, generation: u64) {
        self.attempted_generation = Some(generation);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_native_without_retry_loops_and_honors_explicit_web() {
        let mut route = PlaybackRoute::default();
        assert!(route.can_start(1));
        route.attempted(1);
        assert!(!route.can_start(1));
        assert!(route.can_start(2));
        route.select_web();
        assert!(!route.can_start(2));
        route.select_native();
        assert!(route.can_start(2));
    }
}
