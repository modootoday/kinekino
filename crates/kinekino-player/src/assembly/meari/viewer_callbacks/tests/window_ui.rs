//! Original exported local callback ownership, not native window-effect acknowledgement.
use super::*;
#[test]
fn original_caption_exports_queue_only_local_intents_and_reject_closed_owner() {
    use crate::assembly::meari::viewer_host::window_ui::CaptionIntent;
    for (name, expected) in [
        ("onWindowMinimize", CaptionIntent::Minimize),
        ("onWindowToggleMaximize", CaptionIntent::ToggleMaximize),
        ("onWindowUserClose", CaptionIntent::UserClose),
    ] {
        let fixture = route_fixture();
        let before = fixture.owner.borrow().tickets().unwrap().len();
        let (_bindings, input) = local_input(&fixture.owner.borrow(), name);
        fixture
            .owner
            .borrow_mut()
            .queue_local(&fixture.scope, &input)
            .unwrap();
        assert_eq!(
            fixture
                .owner
                .borrow()
                .projection
                .borrow()
                .window_ui
                .caption_requested,
            None
        );
        fixture
            .owner
            .borrow_mut()
            .drain_local(&fixture.scope)
            .unwrap();
        assert_eq!(
            fixture
                .owner
                .borrow()
                .projection
                .borrow()
                .window_ui
                .caption_requested,
            Some(expected)
        );
        assert_eq!(fixture.owner.borrow().tickets().unwrap().len(), before);
        assert!(
            fixture
                .owner
                .borrow_mut()
                .prepare_route_release(&fixture.scope)
                .unwrap()
                .is_none()
        );
        assert!(
            fixture
                .owner
                .borrow_mut()
                .queue_local(&fixture.scope, &input)
                .is_err()
        );
        close(&fixture.execution);
    }
}
#[test]
fn diagnostics_expansion_is_local_and_closing_rejects_its_original_callback() {
    let fixture = route_fixture();
    let before = fixture.owner.borrow().tickets().unwrap().len();
    let (_bindings, input) = local_input(&fixture.owner.borrow(), "onToggleDiagnostics");
    assert!(
        !fixture
            .owner
            .borrow()
            .projection
            .borrow()
            .diagnostics_expanded
    );
    fixture
        .owner
        .borrow_mut()
        .queue_local(&fixture.scope, &input)
        .unwrap();
    fixture
        .owner
        .borrow_mut()
        .drain_local(&fixture.scope)
        .unwrap();
    assert!(
        fixture
            .owner
            .borrow()
            .projection
            .borrow()
            .diagnostics_expanded
    );
    assert_eq!(fixture.owner.borrow().tickets().unwrap().len(), before);
    fixture
        .owner
        .borrow_mut()
        .queue_local(&fixture.scope, &input)
        .unwrap();
    fixture
        .owner
        .borrow_mut()
        .drain_local(&fixture.scope)
        .unwrap();
    assert!(
        !fixture
            .owner
            .borrow()
            .projection
            .borrow()
            .diagnostics_expanded
    );
    assert!(
        fixture
            .owner
            .borrow_mut()
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .is_none()
    );
    assert!(
        fixture
            .owner
            .borrow_mut()
            .queue_local(&fixture.scope, &input)
            .is_err()
    );
    close(&fixture.execution);
}
#[test]
fn original_window_intents_preserve_sdk_tickets_and_reject_input_after_close() {
    let fixture = route_fixture();
    let before = fixture.owner.borrow().tickets().unwrap().len();
    let (_panel_bindings, panel) = local_input(&fixture.owner.borrow(), "onToggleEpisodes");
    fixture
        .owner
        .borrow_mut()
        .queue_local(&fixture.scope, &panel)
        .unwrap();
    assert!(
        fixture
            .owner
            .borrow()
            .projection
            .borrow()
            .window_ui
            .episodes_open
    );
    let props = fixture.owner.borrow().projection.borrow().window_ui.props();
    assert_eq!(props["episodePanelClosed"], Value::Bool(false));
    assert_eq!(
        props["viewingRootClass"],
        Value::String("aw-viewing-read-root".into())
    );
    fixture
        .owner
        .borrow_mut()
        .drain_local(&fixture.scope)
        .unwrap();
    assert!(
        !fixture
            .owner
            .borrow()
            .projection
            .borrow()
            .window_ui
            .episodes_open
    );
    let props = fixture.owner.borrow().projection.borrow().window_ui.props();
    assert_eq!(props["episodePanelClosed"], Value::Bool(true));
    assert_eq!(
        props["viewingRootClass"],
        Value::String("aw-viewing-read-root aw-viewing-panel-closed".into())
    );
    let (_fullscreen_bindings, fullscreen) =
        local_input(&fixture.owner.borrow(), "onToggleFullscreen");
    fixture
        .owner
        .borrow_mut()
        .queue_local(&fixture.scope, &fullscreen)
        .unwrap();
    fixture
        .owner
        .borrow_mut()
        .drain_local(&fixture.scope)
        .unwrap();
    assert!(
        fixture
            .owner
            .borrow()
            .projection
            .borrow()
            .window_ui
            .toggle_requested
    );
    assert_eq!(fixture.owner.borrow().tickets().unwrap().len(), before);
    assert!(
        fixture
            .owner
            .borrow_mut()
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .is_none()
    );
    assert!(
        fixture
            .owner
            .borrow_mut()
            .queue_local(&fixture.scope, &fullscreen)
            .is_err()
    );
    close(&fixture.execution);
}
