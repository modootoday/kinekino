//! Synthetic source plugin built only on `kinekino-plugin-sdk`, covering every `source/1`
//! method: categories → series → episodes → a stream on a local test server. No real site.
//!
//! Install as `<plugin dir>/fake/plugin.json` with `"id": "com.example.fake"`, capabilities
//! `browse, search, series, episode, resolve`, `"media_hosts": ["127.0.0.1"]` and
//! `"executable"` pointing at this binary. Streams go to `$KINEKINO_FAKE_MEDIA`
//! (default `http://127.0.0.1:8765`) as `/<episode>.mp4`, with a `Referer` header.
use kinekino_plugin_sdk::wire::source::{Item, ItemKind, Ref, Response, Stream};
use kinekino_plugin_sdk::{SourcePlugin, serve};

const PROVIDER: &str = "com.example.fake";
const CATEGORIES: [(&str, &str); 2] = [("airing", "방영 중"), ("movies", "극장판")];
/// Series per category; more than one page so paging and scrolling have work to do.
const SERIES: u32 = 30;
const PAGE: u32 = 12;
const EPISODES: u32 = 3;

fn item(id: String, title: String, kind: ItemKind) -> Item {
    Item {
        reference: Ref {
            provider: PROVIDER.into(),
            id,
        },
        title,
        kind: Some(kind),
    }
}

fn series_title(category: &str, n: u32) -> String {
    format!("Fake {category} {n:02}")
}

fn paged(all: Vec<Item>, page: u32) -> Response {
    let from = ((page.max(1) - 1) * PAGE) as usize;
    let more = all.len() > from + PAGE as usize;
    Response::Page {
        items: all.into_iter().skip(from).take(PAGE as usize).collect(),
        next_page: more.then_some(page + 1),
    }
}

fn not_found() -> Response {
    Response::Error {
        kind: "not_found".into(),
        message: "no such item".into(),
    }
}

/// `<category>~<series>` and `<category>~<series>~<episode>`.
fn parts(id: &str) -> Vec<&str> {
    id.split('~').collect()
}

struct FakeSource;

impl SourcePlugin for FakeSource {
    fn browse(&mut self, category: &Ref, page: u32) -> Response {
        delay();
        if category.id.is_empty() {
            let all = CATEGORIES
                .iter()
                .map(|(id, t)| item((*id).into(), (*t).into(), ItemKind::Container))
                .collect();
            return paged(all, page);
        }
        // A series is just a container whose items are playable: the plugin decides.
        if parts(&category.id).len() == 2 {
            return self.series(category);
        }
        if !CATEGORIES.iter().any(|(id, _)| *id == category.id) {
            return not_found();
        }
        let all = (1..=SERIES)
            .map(|n| {
                item(
                    format!("{}~{n}", category.id),
                    series_title(&category.id, n),
                    ItemKind::Container,
                )
            })
            .collect();
        paged(all, page)
    }

    fn search(&mut self, query: &str, page: u32) -> Response {
        let q = query.to_lowercase();
        let all = CATEGORIES
            .iter()
            .flat_map(|(c, _)| (1..=SERIES).map(move |n| (c, n)))
            .filter(|(c, n)| series_title(c, *n).to_lowercase().contains(&q))
            .map(|(c, n)| item(format!("{c}~{n}"), series_title(c, n), ItemKind::Container))
            .collect();
        paged(all, page)
    }

    fn series(&mut self, series: &Ref) -> Response {
        let [c, n] = parts(&series.id)[..] else {
            return not_found();
        };
        let items = (1..=EPISODES)
            .map(|e| {
                item(
                    format!("{c}~{n}~{e}"),
                    format!("{} {e}화", series_title(c, n.parse().unwrap_or(0))),
                    ItemKind::Playable,
                )
            })
            .collect();
        Response::Page {
            items,
            next_page: None,
        }
    }

    fn episode(&mut self, episode: &Ref) -> Response {
        let [c, n, e] = parts(&episode.id)[..] else {
            return not_found();
        };
        let e: u32 = e.parse().unwrap_or(0);
        let at = |e: u32| Ref {
            provider: PROVIDER.into(),
            id: format!("{c}~{n}~{e}"),
        };
        Response::Adjacency {
            previous: (e > 1).then(|| at(e - 1)),
            next: (e < EPISODES).then(|| at(e + 1)),
        }
    }

    fn resolve(&mut self, episode: &Ref) -> Response {
        if parts(&episode.id).len() != 3 {
            return not_found();
        }
        let base =
            std::env::var("KINEKINO_FAKE_MEDIA").unwrap_or_else(|_| "http://127.0.0.1:8765".into());
        Response::Streams {
            streams: vec![Stream {
                url: format!("{base}/{}.mp4", episode.id.replace('~', "-")),
                headers: vec![("Referer".into(), format!("{base}/watch"))],
            }],
        }
    }
}

/// `KINEKINO_FAKE_DELAY_MS` makes listings slow, to watch the player stay responsive.
fn delay() {
    if let Some(ms) = std::env::var("KINEKINO_FAKE_DELAY_MS")
        .ok()
        .and_then(|v| v.parse().ok())
    {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

fn main() -> std::io::Result<()> {
    serve(&mut FakeSource)
}
