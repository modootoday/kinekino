//! Minimal synthetic source plugin built only on `kinekino-plugin-sdk`. Pair it with a
//! `plugin.json` such as `tests/golden/manifest.json` (capabilities `browse`, `search`).
use kinekino_plugin_sdk::wire::source::{Item, Ref, Response};
use kinekino_plugin_sdk::{SourcePlugin, serve};

struct FakeSource;

impl SourcePlugin for FakeSource {
    fn browse(&mut self, category: &Ref, page: u32) -> Response {
        self.search(&category.id, page)
    }

    fn search(&mut self, query: &str, page: u32) -> Response {
        let items = (0..2)
            .map(|i| Item {
                reference: Ref {
                    provider: "com.example.fake".into(),
                    id: format!("{query}-{page}-{i}"),
                },
                title: format!("Fake {query} #{i}"),
            })
            .collect();
        Response::Page {
            items,
            next_page: (page < 3).then_some(page + 1),
        }
    }
}

fn main() -> std::io::Result<()> {
    serve(&mut FakeSource)
}
