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
        } else {
            let keep_on_player = url::Url::parse(url).ok().is_some_and(|u| {
                u.scheme() == "https"
                    && u.host_str().is_some_and(|host| {
                        crate::domain::admitted_hosts::active_hosts().admits_player(host)
                    })
            });
            if !keep_on_player {
                *self = Self::default();
            }
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

    #[test]
    fn keeps_queue_on_admitted_player_host_and_clears_otherwise() {
        use crate::domain::admitted_hosts::{AdmittedHosts, with_admitted_hosts};
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
        let hosts = AdmittedHosts::new(["catalog.example.test", "cdn.other.test"])
            .with_player_hosts(["cdn.other.test"]);
        with_admitted_hosts(hosts, || {
            let mut q = EpisodeQueue::default();
            assert!(q.observe(page, "https://catalog.example.test/e/a1"));
            assert_eq!(q.entries.len(), 2);
            q.navigate("https://cdn.other.test/video/0123456789abcdef0123456789abcdef");
            assert_eq!(q.entries.len(), 2, "player host must keep the queue");
            q.navigate("https://media.example.test/player/index.php");
            assert!(
                q.entries.is_empty(),
                "non-admitted host must clear the queue"
            );
        });
    }
}
