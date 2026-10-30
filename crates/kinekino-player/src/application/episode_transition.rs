use std::time::Duration;
#[derive(Default)]
pub struct EpisodeTransition {
    pending: Option<Pending>,
}
struct Pending {
    page: Option<String>,
    native: bool,
    native_pending: bool,
    paused: bool,
    autoplay: bool,
    started: Duration,
}
impl EpisodeTransition {
    pub fn select(&mut self, url: String, native: bool, paused: bool, now: Duration) {
        let (native, paused) = self
            .pending
            .as_ref()
            .map(|p| (p.native, p.paused))
            .unwrap_or((native, paused));
        self.pending = Some(Pending {
            page: Some(url),
            native,
            native_pending: native,
            paused,
            autoplay: true,
            started: now,
        });
    }
    pub fn paused_intent(&self) -> Option<bool> {
        self.pending.as_ref().map(|p| p.paused)
    }
    pub fn native_intent(&self) -> bool {
        self.pending.as_ref().is_some_and(|p| p.native)
    }
    pub fn page_ready(&mut self, url: &str) -> bool {
        if let Some(p) = &mut self.pending
            && p.page.as_deref() == Some(url)
        {
            p.page = None;
            return true;
        }
        false
    }
    pub fn ready(&self) -> bool {
        self.pending.as_ref().is_some_and(|p| p.page.is_none())
    }
    pub fn take_native(&mut self) -> Option<bool> {
        let p = self.pending.as_mut()?;
        if p.page.is_some() || !p.native_pending {
            return None;
        }
        p.native_pending = false;
        let paused = p.paused;
        if !p.autoplay {
            self.pending = None;
        }
        Some(paused)
    }
    pub fn take_autoplay(&mut self) -> Option<bool> {
        let p = self.pending.as_mut()?;
        if p.page.is_some() || !p.autoplay {
            return None;
        }
        p.autoplay = false;
        let play = !p.paused;
        if !p.native_pending {
            self.pending = None;
        }
        Some(play)
    }
    pub fn cancel(&mut self) {
        self.pending = None;
    }
    pub fn expire(&mut self, now: Duration) -> bool {
        if self
            .pending
            .as_ref()
            .is_some_and(|p| now.saturating_sub(p.started) >= Duration::from_secs(30))
        {
            self.cancel();
            return true;
        }
        false
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn browser_autoplay_before_manifest_does_not_leave_completed_transition_pending() {
        let mut t = EpisodeTransition::default();
        t.select("one".into(), true, false, Duration::ZERO);
        t.page_ready("one");
        assert_eq!(t.take_autoplay(), Some(true));
        assert_eq!(t.take_native(), Some(false));
        assert!(!t.expire(Duration::from_secs(60)));
    }
    #[test]
    fn superseded_selection_keeps_intent_and_rejects_old_page() {
        let mut t = EpisodeTransition::default();
        t.select("one".into(), true, true, Duration::ZERO);
        t.select("two".into(), false, false, Duration::from_secs(1));
        assert!(!t.page_ready("one"));
        assert!(!t.ready());
        assert!(t.page_ready("two"));
        assert_eq!(t.take_native(), Some(true));
        assert_eq!(t.take_autoplay(), Some(false));
        assert!(!t.ready());
    }
    #[test]
    fn cancellation_and_timeout_prevent_late_autoplay() {
        let mut t = EpisodeTransition::default();
        t.select("one".into(), true, false, Duration::ZERO);
        t.cancel();
        assert!(!t.page_ready("one"));
        assert_eq!(t.take_native(), None);
        assert_eq!(t.take_autoplay(), None);
        t.select("two".into(), true, false, Duration::from_secs(1));
        assert!(!t.expire(Duration::from_secs(30)));
        assert!(t.expire(Duration::from_secs(31)));
        assert!(!t.page_ready("two"));
    }
}
