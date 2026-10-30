use crate::domain::episodes::{Episode, EpisodePage};
#[derive(Default)]
pub struct EpisodeQueue {
    pub title: String,
    pub entries: Vec<Episode>,
    pub current: Option<usize>,
    pub series: String,
}
impl EpisodeQueue {
    pub fn observe(&mut self, page: EpisodePage, current_url: &str) -> bool {
        if !page.valid() || page.page != current_url {
            return false;
        }
        self.title = page.title;
        self.series = page.series;
        self.entries = page.entries;
        let mut seen = std::collections::HashSet::new();
        self.entries.retain(|e| seen.insert(e.url.clone()));
        self.current = self.entries.iter().position(|e| e.url == page.page);
        true
    }
    pub fn adjacent(&self, delta: isize) -> Option<String> {
        self.current?
            .checked_add_signed(delta)
            .and_then(|i| self.entries.get(i))
            .map(|e| e.url.clone())
    }
    pub fn navigate(&mut self, url: &str) {
        if let Some(i) = self.entries.iter().position(|e| e.url == url) {
            self.current = Some(i);
        } else if !url.starts_with("https://frame.example.test/")
            && !url.starts_with("https://media.example.test/")
        {
            *self = Self::default();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_stale_pages_and_keeps_boundaries_through_player() {
        let page = EpisodePage {
            page: "https://catalog.example.test/e/a1".into(),
            series: "https://catalog.example.test/c/a".into(),
            title: "A".into(),
            entries: vec![
                Episode {
                    title: "1".into(),
                    url: "https://catalog.example.test/e/a1".into(),
                },
                Episode {
                    title: "2".into(),
                    url: "https://catalog.example.test/e/a2".into(),
                },
            ],
        };
        let mut q = EpisodeQueue::default();
        assert!(!q.observe(page.clone(), "https://catalog.example.test/e/b1"));
        assert!(q.observe(page, "https://catalog.example.test/e/a1"));
        assert!(q.adjacent(-1).is_none());
        q.navigate("https://frame.example.test/tv-show.php?v=abc");
        assert!(q.adjacent(1).unwrap().ends_with("a2"));
        q.navigate("https://catalog.example.test/e/a2");
        assert!(q.adjacent(1).is_none());
        q.navigate("https://catalog.example.test/종영");
        assert!(q.entries.is_empty());
    }
}
