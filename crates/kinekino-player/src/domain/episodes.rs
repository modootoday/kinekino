use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Episode {
    pub title: String,
    pub url: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodePage {
    pub page: String,
    pub series: String,
    pub title: String,
    pub entries: Vec<Episode>,
}
fn site_path(value: &str, prefix: &str) -> bool {
    Url::parse(value).is_ok_and(|u| {
        u.scheme() == "https"
            && u.host_str() == Some("catalog.example.test")
            && u.port_or_known_default() == Some(443)
            && u.username().is_empty()
            && u.password().is_none()
            && u.query().is_none()
            && u.fragment().is_none()
            && u.path().starts_with(prefix)
    })
}
impl EpisodePage {
    pub fn valid(&self) -> bool {
        site_path(&self.series, "/c/")
            && (self.page == self.series || site_path(&self.page, "/e/"))
            && self.title.len() <= 512
            && !self.entries.is_empty()
            && self.entries.len() <= 2000
            && self
                .entries
                .iter()
                .all(|e| site_path(&e.url, "/e/") && !e.title.is_empty() && e.title.len() <= 512)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_external_episode_destinations_and_oversized_lists() {
        let mut page = EpisodePage {
            page: "https://catalog.example.test/c/a".into(),
            series: "https://catalog.example.test/c/a".into(),
            title: "A".into(),
            entries: vec![Episode {
                title: "1".into(),
                url: "https://catalog.example.test.evil.test/e/1".into(),
            }],
        };
        assert!(!page.valid());
        page.entries[0].url = "javascript:alert(1)".into();
        assert!(!page.valid());
        page.entries[0].url = "https://catalog.example.test/e/1".into();
        assert!(page.valid());
        page.entries = vec![page.entries[0].clone(); 2001];
        assert!(!page.valid());
    }
}
