//! Manual skip (ADR 20261105111300): the user marks a span of the current item, the player
//! offers to skip it while playback is inside, and a skip seeks to its end.
use super::folder;
use super::support::FakeEngine;
use kinekino_player_ui::{Command, Controller};

fn at(c: &mut Controller<FakeEngine>, secs: f64) {
    c.engine.time = secs;
    c.poll();
}

fn seeks(e: &FakeEngine) -> Vec<&str> {
    e.commands
        .iter()
        .filter(|c| c.starts_with("seek"))
        .map(String::as_str)
        .collect()
}

#[test]
fn marked_span_is_offered_inside_and_skipped_to_its_end() {
    let (dir, files) = folder("skip-mark");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    at(&mut c, 90.0);
    c.apply(Command::MarkSkipStart);
    assert_eq!(c.view.notice, "건너뛸 구간 시작 1:30");
    assert!(!c.view.skip_offer, "no span until its end is marked");
    at(&mut c, 180.0);
    c.apply(Command::MarkSkipEnd);
    assert_eq!(c.view.notice, "건너뛸 구간 1:30–3:00");
    assert!(!c.view.skip_offer, "the end itself is outside the span");

    at(&mut c, 89.0);
    assert!(!c.view.skip_offer);
    c.apply(Command::SkipSegment);
    assert!(
        seeks(&c.engine).is_empty(),
        "outside the span nothing is skipped"
    );

    at(&mut c, 95.0);
    assert!(c.view.skip_offer, "inside the span the skip is offered");
    c.apply(Command::SkipSegment);
    assert_eq!(seeks(&c.engine), ["seek 180.000 absolute"]);
    assert_eq!(c.view.position, 180.0);
    assert!(!c.view.skip_offer);
    assert_eq!(c.view.notice, "구간을 건너뛰었습니다");
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn an_end_needs_a_start_before_it() {
    let (dir, files) = folder("skip-order");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    at(&mut c, 60.0);
    c.apply(Command::MarkSkipEnd);
    assert_eq!(c.view.notice, "먼저 건너뛸 구간의 시작을 표시하세요");
    c.apply(Command::MarkSkipStart);
    at(&mut c, 30.0);
    c.apply(Command::MarkSkipEnd);
    assert_eq!(c.view.notice, "구간 끝은 시작보다 뒤여야 합니다");
    at(&mut c, 45.0);
    assert!(!c.view.skip_offer, "a refused end makes no span");
    at(&mut c, 120.0);
    c.apply(Command::MarkSkipEnd);
    assert_eq!(
        c.view.notice, "건너뛸 구간 1:00–2:00",
        "the start is still kept"
    );
    at(&mut c, 61.0);
    assert!(c.view.skip_offer);
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn another_item_forgets_the_span_and_the_marked_start() {
    let (dir, files) = folder("skip-item");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    at(&mut c, 10.0);
    c.apply(Command::MarkSkipStart);
    at(&mut c, 20.0);
    c.apply(Command::MarkSkipEnd);
    at(&mut c, 15.0);
    assert!(c.view.skip_offer);
    c.apply(Command::MarkSkipStart);
    c.apply(Command::Next);
    at(&mut c, 15.0);
    assert!(!c.view.skip_offer, "the span belonged to the previous item");
    c.apply(Command::MarkSkipEnd);
    assert_eq!(
        c.view.notice, "먼저 건너뛸 구간의 시작을 표시하세요",
        "the marked start belonged to the previous item"
    );
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn a_newly_opened_file_forgets_the_span_even_at_the_same_queue_row() {
    let (first, files) = folder("skip-open-a");
    let (second, others) = folder("skip-open-b");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    at(&mut c, 10.0);
    c.apply(Command::MarkSkipStart);
    at(&mut c, 20.0);
    c.apply(Command::MarkSkipEnd);
    c.open(&others[0]);
    at(&mut c, 15.0);
    assert!(
        !c.view.skip_offer,
        "the span belonged to the other folder's item"
    );
    std::fs::remove_dir_all(first).ok();
    std::fs::remove_dir_all(second).ok();
}

#[test]
fn marking_and_skipping_need_media() {
    let mut c = Controller::new(FakeEngine::default());
    c.apply(Command::MarkSkipStart);
    c.apply(Command::MarkSkipEnd);
    c.apply(Command::SkipSegment);
    assert!(c.view.notice.is_empty() && c.view.status.is_empty());
    assert!(c.engine.commands.is_empty());
}
