//! The browse view headless: sources, pages, series and episodes through a fake catalog.
mod support;
use kinekino_core::source::{
    SourceError, SourceItem, SourceItemKind, SourcePage, SourceRef, StreamCandidate,
};
use kinekino_player_ui::{BROWSE_ROWS, Catalog, Command, Controller, Session, meari_props};
use support::{FakeEngine, go};

fn item(provider: &str, id: &str, title: &str) -> SourceItem {
    SourceItem {
        reference: SourceRef {
            provider: provider.into(),
            id: id.into(),
        },
        title: title.into(),
        kind: SourceItemKind::Container,
    }
}

fn kind(mut i: SourceItem, kind: SourceItemKind) -> SourceItem {
    i.kind = kind;
    i
}

/// Synthetic catalog: one source; its root has 20 items in pages of `BROWSE_ROWS`.
struct Fake;
impl Catalog for Fake {
    fn sources(&self) -> Vec<SourceItem> {
        vec![item("com.example.fixture", "", "com.example.fixture")]
    }
    fn browse(&self, at: &SourceRef, page: u32) -> Result<SourcePage, SourceError> {
        match at.id.as_str() {
            "" => {
                let from = (page as usize - 1) * BROWSE_ROWS;
                let items = (from..20.min(from + BROWSE_ROWS))
                    .map(|i| item(&at.provider, &format!("w{i}"), &format!("Work {i}")))
                    .collect();
                Ok(SourcePage {
                    items,
                    next_page: (from + BROWSE_ROWS < 20).then_some(page + 1),
                })
            }
            "series" => Ok(SourcePage {
                items: vec![kind(
                    item(&at.provider, "s1", "Show One"),
                    SourceItemKind::Container,
                )],
                next_page: None,
            }),
            id if id.starts_with('s') => Ok(playlist(at)),
            _ => Err(SourceError::Unsupported(
                kinekino_core::source::SourceCapability::Browse,
            )),
        }
    }
    fn resolve(&self, at: &SourceRef) -> Result<Vec<StreamCandidate>, SourceError> {
        if at.id.ends_with("e3") {
            return Err(SourceError::Unavailable("gone".into()));
        }
        Ok(vec![StreamCandidate {
            url: format!("http://127.0.0.1:9/{}.mp4", at.id),
            headers: vec![
                ("Referer".into(), "http://127.0.0.1:9/watch".into()),
                ("Cookie".into(), "secret".into()),
            ],
        }])
    }
}

/// A container's playables: what a plugin hands over as the play queue.
fn playlist(at: &SourceRef) -> SourcePage {
    let items = (1..=3)
        .map(|e| {
            kind(
                item(
                    &at.provider,
                    &format!("{}-e{e}", at.id),
                    &format!("Show One {e}"),
                ),
                SourceItemKind::Playable,
            )
        })
        .collect();
    SourcePage {
        items,
        next_page: None,
    }
}

/// Root → a container listing → its playables (the queue) → playing a resolved stream with
/// only allowed
/// headers; "next" resolves the following entry, a failing resolve says why.
#[test]
fn playables_become_the_queue_and_resolve() {
    // Open the category through a source whose root lists it.
    struct Root;
    impl Catalog for Root {
        fn sources(&self) -> Vec<SourceItem> {
            Fake.sources()
        }
        fn browse(&self, at: &SourceRef, page: u32) -> Result<SourcePage, SourceError> {
            if at.id.is_empty() {
                return Ok(SourcePage {
                    items: vec![kind(
                        item(&at.provider, "series", "Series"),
                        SourceItemKind::Container,
                    )],
                    next_page: None,
                });
            }
            Fake.browse(at, page)
        }
        fn resolve(&self, at: &SourceRef) -> Result<Vec<StreamCandidate>, SourceError> {
            Fake.resolve(at)
        }
    }
    let mut c = Controller::new(FakeEngine::default()).with_catalog(Box::new(Root));
    go(&mut c, Command::Browse);
    go(&mut c, Command::BrowseItem(0)); // source root
    go(&mut c, Command::BrowseItem(0)); // category "Series"
    assert_eq!(c.view.browse_rows, vec!["Show One"]);
    go(&mut c, Command::BrowseItem(0)); // series → episodes
    assert_eq!(c.view.browse_heading, "Show One");
    assert_eq!(c.view.browse_rows.len(), 3);
    go(&mut c, Command::BrowseItem(1)); // episode 2
    assert!(c.view.has_media && !c.view.browsing);
    assert_eq!(c.view.title, "Show One 2");
    assert_eq!(c.view.work, "Show One");
    assert_eq!(c.view.episodes.len(), 3);
    assert_eq!(c.engine.loaded, vec!["http://127.0.0.1:9/s1-e2.mp4"]);
    assert!(
        c.engine
            .commands
            .contains(&"set referrer http://127.0.0.1:9/watch".to_string())
    );
    assert!(!c.engine.commands.iter().any(|x| x.contains("secret")));
    go(&mut c, Command::Next);
    // Episode 3 does not resolve: the current one keeps playing, a notice says why.
    assert_eq!(c.view.title, "Show One 2");
    assert!(c.view.notice.contains("재생할 수 없습니다"));
    go(&mut c, Command::Episode(0));
    assert_eq!(
        c.engine.loaded.last().unwrap(),
        "http://127.0.0.1:9/s1-e1.mp4"
    );
}

#[test]
fn without_sources_the_start_screen_says_so() {
    let mut c = Controller::new(FakeEngine::default());
    go(&mut c, Command::Browse);
    assert!(!c.view.browsing);
    assert!(c.view.status.contains("소스 플러그인"));
}

#[test]
fn browse_lists_sources_then_pages_then_goes_back() {
    let mut c = Controller::new(FakeEngine::default()).with_catalog(Box::new(Fake));
    go(&mut c, Command::Browse);
    assert!(c.view.browsing);
    assert_eq!(c.view.browse_rows, vec!["com.example.fixture"]);
    go(&mut c, Command::BrowseItem(0));
    assert_eq!(c.view.browse_heading, "com.example.fixture");
    assert_eq!(c.view.browse_rows.len(), BROWSE_ROWS);
    assert!(c.view.browse_more);
    go(&mut c, Command::BrowseMore);
    assert_eq!(c.view.browse_rows.first().unwrap(), "Work 12");
    assert!(!c.view.browse_more);
    // A refusing item keeps the listing and shows why.
    go(&mut c, Command::BrowseItem(0));
    assert_eq!(c.view.browse_rows.first().unwrap(), "Work 12");
    assert!(!c.view.browse_status.is_empty());
    go(&mut c, Command::BrowseBack);
    assert_eq!(c.view.browse_rows, vec!["com.example.fixture"]);
    // Esc (windowed) steps back out of the view.
    go(&mut c, Command::ExitFullscreen);
    assert!(!c.view.browsing);
}

#[test]
fn browse_rows_are_clickable_in_the_document() {
    let mut c = Controller::new(FakeEngine::default()).with_catalog(Box::new(Fake));
    go(&mut c, Command::Browse);
    let (mut session, mut doc) = Session::mount(&c.view).unwrap();
    doc.update_module_props(meari_props(&c.view)).unwrap();
    kinekino_player_ui::activate(&mut doc, "br0").unwrap();
    let commands = session.commands(&doc).unwrap();
    assert_eq!(commands, vec![Command::BrowseItem(0)]);
    for command in commands {
        go(&mut c, command);
    }
    doc.update_module_props(meari_props(&c.view)).unwrap();
    kinekino_player_ui::activate(&mut doc, "browseMore").unwrap();
    kinekino_player_ui::activate(&mut doc, "browseBack").unwrap();
    assert_eq!(
        session.commands(&doc).unwrap(),
        vec![Command::BrowseBack, Command::BrowseMore]
    );
}

/// A slow plugin does not hold up the window: the command returns at once with a
/// loading line, and the listing shows when the answer comes.
#[test]
fn slow_plugin_calls_run_off_the_ui_thread() {
    struct Slow;
    impl Catalog for Slow {
        fn sources(&self) -> Vec<SourceItem> {
            Fake.sources()
        }
        fn browse(&self, at: &SourceRef, page: u32) -> Result<SourcePage, SourceError> {
            std::thread::sleep(std::time::Duration::from_millis(400));
            Fake.browse(at, page)
        }
    }
    let mut c = Controller::new(FakeEngine::default()).with_catalog(Box::new(Slow));
    c.apply(Command::Browse);
    let started = std::time::Instant::now();
    c.apply(Command::BrowseItem(0));
    assert!(started.elapsed() < std::time::Duration::from_millis(200));
    assert_eq!(c.view.browse_status, "불러오는 중…");
    assert_eq!(c.view.browse_rows, vec!["com.example.fixture"]);
    c.finish();
    assert_eq!(c.view.browse_rows.len(), BROWSE_ROWS);
    assert!(c.view.browse_status.is_empty());
    // Back while a call is in flight drops its late answer.
    c.apply(Command::BrowseMore);
    c.apply(Command::BrowseBack);
    assert_eq!(c.view.browse_rows, vec!["com.example.fixture"]);
    std::thread::sleep(std::time::Duration::from_millis(500));
    c.poll();
    assert_eq!(c.view.browse_rows, vec!["com.example.fixture"]);
}
