//! 작품 탐색: the trail of opened listings, paging (appended, the row list scrolls) and
//! search inside the open source (ADR 20261031103033).
use super::jobs::{Job, Level, Pending, Reply, Then};
use super::{BROWSE_TITLE, Controller, Engine, LOADING, NO_SOURCES, browse_error};
use crate::catalog::BROWSE_ROWS;
use crate::session::Command;
use kinekino_core::source::{SourceError, SourceItem, SourceItemKind, SourcePage};

impl<E: Engine> Controller<E> {
    pub(super) fn show(&mut self, heading: String, items: Vec<SourceItem>, next: Option<u32>) {
        self.view.browsing = true;
        self.view.browse_heading = heading;
        self.listing = items;
        self.view.browse_listing += 1;
        self.next_page = next;
        self.view.browse_status.clear();
        self.rows();
    }

    fn rows(&mut self) {
        self.view.browse_rows = self
            .listing
            .iter()
            .take(BROWSE_ROWS)
            .map(|i| i.title.clone())
            .collect();
        self.view.browse_more = self.next_page.is_some() && self.listing.len() < BROWSE_ROWS;
        self.view.browse_search = !self.trail.is_empty();
    }

    pub(super) fn show_sources(&mut self) {
        let sources = self
            .catalog
            .as_ref()
            .map(|c| c.sources())
            .unwrap_or_default();
        if sources.is_empty() {
            self.view.browsing = false;
            self.view.status = NO_SOURCES.into();
            return;
        }
        self.trail.clear();
        self.show(BROWSE_TITLE.into(), sources, None);
    }

    /// Asks for `level` off the UI thread; `then` says how the trail changes once it
    /// arrives. A refusal keeps the shown listing and says why.
    fn list(&mut self, level: Level, then: Then) {
        let Some(catalog) = &self.catalog else {
            return;
        };
        let at = level.item.reference.clone();
        let (page, query) = (level.page, level.query.clone());
        let job = Job::List { level, then };
        self.pending = Some(Pending::start(catalog, job, move |c| {
            Reply::Page(match query {
                Some(q) => c.search(&at.provider, &q, page),
                None => c.browse(&at, page),
            })
        }));
        self.view.browse_status = LOADING.into();
    }

    /// A listing answer for `level`.
    pub(super) fn listed(
        &mut self,
        level: Level,
        then: Then,
        answer: Result<SourcePage, SourceError>,
    ) {
        let page = match answer {
            Ok(p) => p,
            Err(e) => {
                self.view.browse_status = browse_error(&e);
                return;
            }
        };
        match then {
            Then::Page => {
                self.listing.extend(page.items);
                self.next_page = page.next_page;
                if let Some(top) = self.trail.last_mut() {
                    top.page = level.page;
                }
                self.view.browse_status.clear();
                self.rows();
            }
            Then::Push | Then::Stay => {
                let heading = match &level.query {
                    Some(q) => format!("검색: {q}"),
                    None => level.item.title.clone(),
                };
                if then == Then::Push {
                    self.trail.push(level);
                }
                self.show(heading, page.items, page.next_page);
            }
        }
    }

    pub(super) fn browse(&mut self, command: &Command) {
        match command {
            Command::BrowseItem(i) => {
                let Some(item) = self.listing.get(*i).cloned() else {
                    return;
                };
                if item.kind == SourceItemKind::Playable {
                    self.open_playlist(*i);
                    return;
                }
                let level = Level {
                    item,
                    page: 1,
                    query: None,
                };
                self.list(level, Then::Push);
            }
            Command::BrowseSearch(query) => {
                let query = query.trim();
                let Some(root) = self.trail.first() else {
                    return;
                };
                if query.is_empty() {
                    return;
                }
                let level = Level {
                    item: root.item.clone(),
                    page: 1,
                    query: Some(query.to_owned()),
                };
                self.list(level, Then::Push);
            }
            Command::BrowseMore => {
                let (Some(next), Some(top)) = (self.next_page, self.trail.last().cloned()) else {
                    return;
                };
                self.list(Level { page: next, ..top }, Then::Page);
            }
            Command::BrowseBack => {
                if self.trail.pop().is_none() {
                    self.view.browsing = false;
                    return;
                }
                match self.trail.last().cloned() {
                    Some(top) => self.list(Level { page: 1, ..top }, Then::Stay),
                    None => {
                        self.pending = None;
                        self.show_sources();
                    }
                }
            }
            _ => {}
        }
    }
}
