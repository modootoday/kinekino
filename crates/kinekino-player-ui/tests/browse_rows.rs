//! 작품 탐색 on the headless raster: empty row slots take no space, filled rows do.
use kinekino_player_ui::{Session, View, meari_props};
use meari_native::application::Renderer;

#[test]
fn empty_browse_slots_are_not_drawn() {
    let view = View {
        browsing: true,
        browse_heading: "작품 탐색".into(),
        browse_rows: vec!["com.example.fixture".into()],
        ..View::default()
    };
    let (_, mut doc) = Session::mount(&view).unwrap();
    doc.update_module_props(meari_props(&view)).unwrap();
    let viewport = doc.viewport.clone();
    doc.renderer.rasterize(&viewport).unwrap();
    let base = &*doc.renderer.document;
    let height = |sel: &str| {
        let id = base.query_selector(sel).unwrap().unwrap();
        base.get_node(id).unwrap().final_layout().size.height
    };
    assert!(height("#br0") > 0.0);
    assert_eq!(height("#br1"), 0.0);
    assert_eq!(height("#browseMore"), 0.0);
    assert_eq!(height("#start"), 0.0);
}
