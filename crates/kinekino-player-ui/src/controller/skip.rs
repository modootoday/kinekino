//! Manual skip (ADR 20261105111300): the user marks a span of the current item; inside it a
//! skip is offered and seeks to its end. Kept for the current item only, never saved.
use super::{Controller, Engine};
use crate::session::clock;
use kinekino_core::playback::SkipRange;
use std::time::Instant;

const NO_START: &str = "먼저 건너뛸 구간의 시작을 표시하세요";
const END_BEFORE_START: &str = "구간 끝은 시작보다 뒤여야 합니다";
const SKIPPED: &str = "구간을 건너뛰었습니다";

#[derive(Debug, Default)]
pub(super) struct Skip {
    /// A start marked and not yet closed by an end.
    start: Option<f64>,
    range: Option<SkipRange>,
}

impl<E: Engine> Controller<E> {
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
        self.skip = Skip {
            start: None,
            range: Some(range),
        };
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

    /// Another item: its span and marked start belonged to the one before.
    pub(super) fn forget_skip(&mut self) {
        self.skip = Skip::default();
        self.view.skip_offer = false;
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
