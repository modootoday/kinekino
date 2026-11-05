//! Manual skip (ADR 20261105111300, ADR 20261105131646): the user marks a span of the
//! current item; inside it a skip is offered and seeks to its end. Confirmed spans are
//! saved under the same local-file key as 이어보기 and restored when that key opens again.
//! Sibling files never receive a copy.
use super::{Controller, Engine, Media};
use crate::session::clock;
use kinekino_core::playback::{ResumeKey, SkipRange, SkipRangeStore};
use kinekino_core::port::error::{PortError, PortResult};
use std::time::Instant;

const NO_START: &str = "먼저 건너뛸 구간의 시작을 표시하세요";
const END_BEFORE_START: &str = "구간 끝은 시작보다 뒤여야 합니다";
const SKIPPED: &str = "구간을 건너뛰었습니다";

#[derive(Default)]
pub(super) struct Skip {
    /// A start marked and not yet closed by an end.
    start: Option<f64>,
    range: Option<SkipRange>,
    store: Option<Box<dyn SkipRangeStore>>,
    /// The opened item's key, when it has one; cleared when memory is forgotten.
    key: Option<ResumeKey>,
    /// A store error was already reported in this run.
    warned: bool,
}

impl Skip {
    fn with<T>(
        &mut self,
        call: impl FnOnce(&mut dyn SkipRangeStore) -> PortResult<T>,
    ) -> Option<T> {
        let result = call(self.store.as_deref_mut()?);
        result.map_err(|e| self.warn(&e)).ok()
    }

    fn warn(&mut self, e: &PortError) {
        if !std::mem::replace(&mut self.warned, true) {
            eprintln!(
                "{{\"warning\":\"skip range not saved or read\",\"code\":\"{}\"}}",
                e.code.name()
            );
        }
    }
}

impl<E: Engine> Controller<E> {
    /// Saves and restores skip spans of local files in `store`.
    pub fn with_skip_store(mut self, store: Box<dyn SkipRangeStore>) -> Self {
        self.skip.store = Some(store);
        self
    }

    pub(super) fn mark_skip_start(&mut self) {
        let at = self.view.position;
        self.skip.start = Some(at);
        self.skip_notice(format!("건너뛸 구간 시작 {}", clock(at)));
    }

    /// A refused end keeps the marked start, so the user can mark the end again.
    pub(super) fn mark_skip_end(&mut self) {
        let Some(start) = self.skip.start else {
            return self.skip_notice(NO_START.into());
        };
        let Some(range) = SkipRange::new(start, self.view.position) else {
            return self.skip_notice(END_BEFORE_START.into());
        };
        self.skip.start = None;
        self.skip.range = Some(range);
        if let Some(key) = self.skip.key.clone() {
            self.skip.with(|s| s.put(key.as_str(), range));
        }
        let (from, to) = (clock(range.start()), clock(range.end()));
        self.skip_notice(format!("건너뛸 구간 {from}–{to}"));
        self.refresh_skip_offer();
    }

    /// Outside the span nothing happens; a skip there would be a surprise jump.
    pub(super) fn skip_segment(&mut self) {
        let position = self.view.position;
        let Some(range) = self.skip.range.filter(|r| r.contains(position)) else {
            return;
        };
        let end = range.end();
        let r = self
            .engine
            .command(&["seek", &format!("{end:.3}"), "absolute"]);
        if r.is_err() {
            return self.report(r);
        }
        self.view.position = end;
        self.resume_rewound();
        self.refresh_skip_offer();
        self.skip_notice(SKIPPED.into());
    }

    /// User asked to drop the span (and any open start). Always says what happened.
    /// The opened item's key stays so a new span marked later can be saved again.
    pub(super) fn clear_skip(&mut self) {
        let had = self.skip.start.is_some() || self.skip.range.is_some();
        if let Some(key) = self.skip.key.clone() {
            self.skip.with(|s| s.clear(key.as_str()).map(drop));
        }
        self.skip.start = None;
        self.skip.range = None;
        self.view.skip_offer = false;
        self.skip_notice(if had {
            "건너뛸 구간 지움".into()
        } else {
            "지울 구간이 없습니다".into()
        });
    }

    /// Another item: its span and marked start belonged to the one before.
    /// The store entry for that key is kept so reopening the same key restores it.
    pub(super) fn forget_skip(&mut self) {
        self.skip.start = None;
        self.skip.range = None;
        self.skip.key = None;
        self.view.skip_offer = false;
    }

    /// The open of queue item `index` finished: restore a saved span for its key.
    pub(super) fn skip_opened(&mut self, index: usize) {
        self.skip.key = None;
        self.skip.start = None;
        self.skip.range = None;
        self.view.skip_offer = false;
        if self.skip.store.is_none() {
            return;
        }
        let Some(Media::Local(path)) = self.files.get(index) else {
            return;
        };
        let path = std::path::absolute(path).unwrap_or_else(|_| path.clone());
        let Some(key) = ResumeKey::for_source(&path.to_string_lossy()) else {
            return;
        };
        let saved = self.skip.with(|s| s.get(key.as_str())).flatten();
        self.skip.key = Some(key);
        if let Some(range) = saved {
            self.skip.range = Some(range);
            self.refresh_skip_offer();
        }
    }

    /// True when the offer changed.
    pub(super) fn refresh_skip_offer(&mut self) -> bool {
        let position = self.view.position;
        let offer = self.skip.range.is_some_and(|r| r.contains(position));
        std::mem::replace(&mut self.view.skip_offer, offer) != offer
    }

    /// A short notice, unless an open is showing its own line.
    fn skip_notice(&mut self, message: String) {
        if !self.view.opening {
            self.view.notice = message;
            self.notice_since = Some(Instant::now());
        }
    }
}
