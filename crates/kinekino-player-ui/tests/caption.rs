//! The caption row on the headless raster: a short file title is shown whole, not "lo…".
use kinekino_player_ui::{Session, View, meari_props};
use meari_native::application::Renderer;

/// Mounts with `first` as the title, then (like the player after opening a file) updates
/// the props to `title` and lays out again.
fn widths(first: &str, title: &str) -> Vec<(String, f32, f32)> {
    let mut view = View {
        title: first.into(),
        has_media: true,
        ..View::default()
    };
    let (_, mut doc) = Session::mount(&view).unwrap();
    let viewport = doc.viewport.clone();
    doc.renderer.rasterize(&viewport).unwrap();
    view.title = title.into();
    doc.update_module_props(meari_props(&view)).unwrap();
    doc.renderer.rasterize(&viewport).unwrap();
    let base = &*doc.renderer.document;
    [
        ".aw-window-brand",
        ".aw-product-name",
        "#context",
        ".aw-window-captions",
    ]
    .iter()
    .map(|sel| {
        let id = base.query_selector(sel).unwrap().unwrap();
        let l = *base.get_node(id).unwrap().final_layout();
        (
            sel.to_string(),
            l.size.width,
            l.padding.left + l.border.left,
        )
    })
    .collect()
}

fn assert_whole(w: Vec<(String, f32, f32)>) {
    eprintln!("{w:?}");
    // The title box is rounded to whole pixels while the ellipsis check uses the unrounded
    // text advance, so a box that merely hugs "long" (27px for a 27.25px advance) paints
    // "lo…". On a 1280px window the title must own the rest of the row instead.
    let (_, width, inset) = w[2].clone();
    assert!(
        width - inset >= 400.0,
        "title text box {} for \"long\"",
        width - inset
    );
}

#[test]
fn a_short_title_gets_its_full_width() {
    assert_whole(widths("long", "long"));
}

#[test]
fn a_title_set_after_mount_gets_its_full_width() {
    assert_whole(widths("", "long"));
    assert_whole(widths("x", "long"));
}
