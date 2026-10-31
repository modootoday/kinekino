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
    assert_eq!(height(".kk-browse-search"), 0.0, "no search over the source list");
}

#[test]
fn leaving_browse_for_an_episode_needs_a_remount() {
    let mut view = View {
        browsing: true,
        browse_heading: "Fake airing 01".into(),
        browse_rows: vec!["Fake airing 01 1화".into(), "Fake airing 01 2화".into()],
        ..View::default()
    };
    let (session, _) = Session::mount(&view).unwrap();
    view.browsing = false;
    view.has_media = true;
    view.playing = true;
    view.title = "Fake airing 01 2화".into();
    view.work = "Fake airing 01".into();
    view.episodes = vec![
        kinekino_player_ui::Episode {
            number: "01".into(),
            title: "Fake airing 01 1화".into(),
        },
        kinekino_player_ui::Episode {
            number: "02".into(),
            title: "Fake airing 01 2화".into(),
        },
    ];
    view.current = Some(1);
    assert!(!session.fits(&view));
    let (session, mut doc) = Session::mount(&view).unwrap();
    assert!(session.fits(&view));
    let viewport = doc.viewport.clone();
    doc.renderer.rasterize(&viewport).unwrap();
}

#[test]
fn the_search_row_is_drawn_inside_a_source() {
    let view = View {
        browsing: true,
        browse_heading: "com.example.fixture".into(),
        browse_rows: vec!["Work 0".into()],
        browse_search: true,
        ..View::default()
    };
    let (_, mut doc) = Session::mount(&view).unwrap();
    doc.update_module_props(meari_props(&view)).unwrap();
    let viewport = doc.viewport.clone();
    doc.renderer.rasterize(&viewport).unwrap();
    let base = &*doc.renderer.document;
    let size = |sel: &str| {
        let id = base.query_selector(sel).unwrap().unwrap();
        base.get_node(id).unwrap().final_layout().size
    };
    let field = size("#browseQuery");
    assert!(field.height > 0.0 && field.width > 100.0);
    assert!(size("#browseSearch").height > 0.0);
}
