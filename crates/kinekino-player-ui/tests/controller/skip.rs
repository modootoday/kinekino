//! Manual skip (ADR 20261105111300): the user marks a span of the current item, the player
//! offers to skip it while playback is inside, and a skip seeks to its end.
use super::folder;
use super::support::FakeEngine;
use kinekino_core::playback::{MemorySkipRangeStore, ResumeKey, SkipRange, SkipRangeStore};
use kinekino_core::port::error::PortResult;
use kinekino_player_ui::{Command, Controller};
use std::sync::{Arc, Mutex};

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

#[test]
fn clear_skip_forgets_the_span_and_tells_the_user() {
    let (dir, files) = folder("skip-clear");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    at(&mut c, 10.0);
    c.apply(Command::MarkSkipStart);
    at(&mut c, 20.0);
    c.apply(Command::MarkSkipEnd);
    at(&mut c, 15.0);
    assert!(c.view.skip_offer);
    c.apply(Command::ClearSkip);
    assert!(!c.view.skip_offer);
    assert_eq!(c.view.notice, "건너뛸 구간 지움");
    at(&mut c, 15.0);
    assert!(!c.view.skip_offer, "the span stayed gone");
    c.apply(Command::SkipSegment);
    assert!(seeks(&c.engine).is_empty());
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn clear_skip_with_nothing_marked_is_safe() {
    let (dir, files) = folder("skip-clear-empty");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    c.engine.commands.clear();
    c.apply(Command::ClearSkip);
    assert_eq!(c.view.notice, "지울 구간이 없습니다");
    assert!(!c.view.skip_offer);
    assert!(
        seeks(&c.engine).is_empty(),
        "clearing nothing must not seek"
    );
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn clear_skip_also_drops_an_open_start() {
    let (dir, files) = folder("skip-clear-start");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    at(&mut c, 10.0);
    c.apply(Command::MarkSkipStart);
    c.apply(Command::ClearSkip);
    assert_eq!(c.view.notice, "건너뛸 구간 지움");
    at(&mut c, 20.0);
    c.apply(Command::MarkSkipEnd);
    assert_eq!(
        c.view.notice, "먼저 건너뛸 구간의 시작을 표시하세요",
        "the start was cleared"
    );
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn skip_shortcuts_match_physical_keys() {
    use kinekino_player_ui::shortcut;
    use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
    let key = |k| PhysicalKey::Code(k);
    let ctrl = ModifiersState::CONTROL;
    let none = ModifiersState::empty();
    assert_eq!(
        shortcut(ctrl, key(KeyCode::BracketLeft), false),
        Some(Command::MarkSkipStart)
    );
    assert_eq!(
        shortcut(ctrl, key(KeyCode::BracketRight), false),
        Some(Command::MarkSkipEnd)
    );
    assert_eq!(
        shortcut(none, key(KeyCode::KeyS), false),
        Some(Command::SkipSegment)
    );
    assert_eq!(
        shortcut(ctrl, key(KeyCode::Backslash), false),
        Some(Command::ClearSkip)
    );
    assert_eq!(
        shortcut(none, key(KeyCode::BracketLeft), false),
        None,
        "plain [ is not a skip shortcut"
    );
    assert_eq!(
        shortcut(none, key(KeyCode::KeyS), true),
        None,
        "auto-repeat does not skip"
    );
    assert_eq!(
        shortcut(ctrl, key(KeyCode::Backslash), true),
        None,
        "auto-repeat does not clear"
    );
    assert_eq!(
        shortcut(ctrl, key(KeyCode::KeyS), false),
        None,
        "Ctrl+S is not SkipSegment"
    );
}

#[test]
fn skip_offer_props_and_buttons_are_in_the_document() {
    use kinekino_player_ui::{Session, View, meari_props};
    let view = View {
        has_media: true,
        skip_offer: false,
        ..View::default()
    };
    let hidden = format!("{:?}", meari_props(&view)["noSkipOffer"]);
    assert!(hidden.contains("true"), "no offer → hidden: {hidden}");
    let offering = View {
        skip_offer: true,
        ..view.clone()
    };
    let shown = format!("{:?}", meari_props(&offering)["noSkipOffer"]);
    assert!(shown.contains("false"), "offer → visible: {shown}");
    let (_, doc) = Session::mount(&offering).unwrap();
    let base = &*doc.renderer.document;
    assert!(
        base.query_selector("#skipSegment").unwrap().is_some(),
        "skipSegment button exists"
    );
    assert!(
        base.query_selector("#clearSkip").unwrap().is_some(),
        "clearSkip button exists"
    );
}

/// A memory store the test can still read after handing it to the controller.
#[derive(Clone, Default)]
struct SharedSkip {
    store: Arc<Mutex<MemorySkipRangeStore>>,
}

impl SharedSkip {
    fn saved(&self, key: &str) -> Option<SkipRange> {
        self.store.lock().unwrap().get(key).unwrap()
    }
}

impl SkipRangeStore for SharedSkip {
    fn get(&self, key: &str) -> PortResult<Option<SkipRange>> {
        self.store.lock().unwrap().get(key)
    }
    fn put(&mut self, key: &str, range: SkipRange) -> PortResult<()> {
        self.store.lock().unwrap().put(key, range)
    }
    fn clear(&mut self, key: &str) -> PortResult<bool> {
        self.store.lock().unwrap().clear(key)
    }
}

fn skip_key(path: &std::path::Path) -> String {
    ResumeKey::for_source(&path.to_string_lossy())
        .expect("a local file has a key")
        .as_str()
        .to_owned()
}

#[test]
fn marked_span_is_restored_when_the_same_file_opens_again() {
    let (dir, files) = folder("skip-persist-same");
    let store = SharedSkip::default();
    let mut c = Controller::new(FakeEngine::default()).with_skip_store(Box::new(store.clone()));
    c.open(&files[0]);
    at(&mut c, 90.0);
    c.apply(Command::MarkSkipStart);
    at(&mut c, 180.0);
    c.apply(Command::MarkSkipEnd);
    assert!(
        store.saved(&skip_key(&files[0])).is_some(),
        "end writes the store"
    );

    // Re-open the same file: memory was forgotten with the queue rebuild, then restored.
    c.open(&files[0]);
    at(&mut c, 95.0);
    assert!(
        c.view.skip_offer,
        "the saved span comes back for the same key"
    );
    c.apply(Command::SkipSegment);
    assert_eq!(seeks(&c.engine), ["seek 180.000 absolute"]);
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn a_sibling_file_does_not_receive_the_saved_span() {
    let (dir, files) = folder("skip-persist-sib");
    let store = SharedSkip::default();
    let mut c = Controller::new(FakeEngine::default()).with_skip_store(Box::new(store.clone()));
    // folder() writes 02,01,03 videos → sorted playlist 01,02,03; files[0] is 02.mkv.
    c.open(&files[0]);
    at(&mut c, 10.0);
    c.apply(Command::MarkSkipStart);
    at(&mut c, 20.0);
    c.apply(Command::MarkSkipEnd);
    assert!(store.saved(&skip_key(&files[0])).is_some());

    c.apply(Command::Next); // 03.mkv, a sibling
    at(&mut c, 15.0);
    assert!(!c.view.skip_offer, "siblings never receive a copied span");
    assert!(
        store.saved(&skip_key(&files[0])).is_some(),
        "the previous item's store entry stays"
    );
    assert!(
        store.saved(&skip_key(&files[2])).is_none(),
        "the sibling got no store entry"
    );
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn clear_skip_removes_the_store_entry_so_reopen_has_none() {
    let (dir, files) = folder("skip-persist-clear");
    let store = SharedSkip::default();
    let mut c = Controller::new(FakeEngine::default()).with_skip_store(Box::new(store.clone()));
    c.open(&files[0]);
    at(&mut c, 10.0);
    c.apply(Command::MarkSkipStart);
    at(&mut c, 20.0);
    c.apply(Command::MarkSkipEnd);
    c.apply(Command::ClearSkip);
    assert!(
        store.saved(&skip_key(&files[0])).is_none(),
        "clear drops the store"
    );

    c.open(&files[0]);
    at(&mut c, 15.0);
    assert!(!c.view.skip_offer, "reopen finds nothing saved");
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn without_a_store_skip_still_works_for_the_current_item() {
    let (dir, files) = folder("skip-persist-nostore");
    let mut c = Controller::new(FakeEngine::default());
    c.open(&files[0]);
    at(&mut c, 10.0);
    c.apply(Command::MarkSkipStart);
    at(&mut c, 20.0);
    c.apply(Command::MarkSkipEnd);
    at(&mut c, 15.0);
    assert!(c.view.skip_offer);
    c.apply(Command::SkipSegment);
    assert_eq!(seeks(&c.engine), ["seek 20.000 absolute"]);
    std::fs::remove_dir_all(dir).ok();
}
