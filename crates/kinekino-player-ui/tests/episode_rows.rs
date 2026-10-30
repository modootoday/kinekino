//! Episode panel rows on the headless raster: a short episode name is shown whole, not
//! "01 · lo…" (same rounded-box vs. unrounded-advance ellipsis as the caption title).
use kinekino_player_ui::{Episode, Session, View, meari_props};
use meari_native::application::Renderer;

/// Width of the first row's title text box and of the row itself.
fn row(title: &str, current: bool) -> (f32, f32) {
    let mut view = View {
        title: title.into(),
        has_media: true,
        panel_open: true,
        episodes: vec![
            Episode {
                number: "01".into(),
                title: title.into(),
            },
            Episode {
                number: "02".into(),
                title: "next".into(),
            },
        ],
        current: current.then_some(0),
        ..View::default()
    };
    let (_, mut doc) = Session::mount(&view).unwrap();
    let viewport = doc.viewport.clone();
    doc.renderer.rasterize(&viewport).unwrap();
    view.status = " ".into();
    doc.update_module_props(meari_props(&view)).unwrap();
    doc.renderer.rasterize(&viewport).unwrap();
    let base = &*doc.renderer.document;
    let size = |sel: &str| {
        let id = base.query_selector(sel).unwrap().unwrap();
        let l = *base.get_node(id).unwrap().final_layout();
        (
            l.size.width - l.padding.left - l.padding.right,
            l.size.width,
        )
    };
    let (text, _) = size("#ep0 .aw-episode-row-title");
    let (_, row) = size("#ep0");
    (text, row)
}

fn assert_whole(title: &str, current: bool) {
    let (text, row) = row(title, current);
    eprintln!("{title:?} current={current}: text {text} row {row}");
    // The title must take the rest of the row (minus the marker), not hug its text.
    assert!(
        text >= row * 0.5,
        "episode title box {text} of row {row} for {title:?}"
    );
}

#[test]
fn a_short_episode_name_gets_the_row_width() {
    assert_whole("long", false);
    assert_whole("long", true);
    assert_whole("x", false);
}
