use super::*;
use meari::domain::host_callbacks::{
    CallbackArguments, CallbackInvocation, CallbackReference, CallbackSignature,
};

pub(in crate::assembly::meari::viewer_host) fn assert_original_surface(host: &ViewerHost) {
    let handle = host.document.as_ref().unwrap();
    let stamp = handle.stamp().unwrap();
    let caption = host.window_ui.captions.clone();
    let before_tickets = host.callbacks.borrow().tickets().unwrap().len();
    let mut cards = host.cards.borrow_mut();
    let capture = cards.capture.as_ref().unwrap();
    let surface = cards.capture_surface.as_ref().unwrap();
    let mut editor = surface
        .model
        .props(capture, surface.progress.unwrap())
        .unwrap();
    let guards = surface.navigation_guards();
    editor.insert(
        "captureCanReturn".into(),
        meari::domain::Value::Bool(!guards.dirty && !guards.pending && !guards.recovery_blocked),
    );
    editor.extend(surface.inputs.props());
    handle.try_with(|document| {
        let editor = surface.projection.compose_editor_slot(editor, |key, reference, props|
            document.node_slot(key, reference, props)).unwrap();
        let root = surface.projection.root_props(editor);
        let frame = host.product_frame.as_ref().unwrap();
        crate::presentation::web_ui_adapter::player_shell::window_caption::contract_fixture::assert_required_caption_fields(
            &root, |invalid| frame.workspace_props(document, WorkspaceRoute::Cards,
                NonZeroU64::new(1).unwrap(), invalid).is_ok());
    }).unwrap();
    let pending = |bindings: &meari::application::host_callbacks::CallbackBindings, name: &str| {
        PendingCallback {
            handle: bindings
                .resolve(&CallbackReference {
                    binding: name.into(),
                    signature: CallbackSignature::Action,
                })
                .unwrap(),
            invocation: CallbackInvocation {
                prop: name.into(),
                arguments: CallbackArguments::Action,
                span: Default::default(),
            },
        }
    };
    let originals = surface.inputs.bindings().unwrap();
    // Reuse handles issued while live when checking rejection after stop.
    let original_pending: std::collections::BTreeMap<_, _> =
        crate::presentation::web_ui_adapter::player_shell::window_caption::BINDINGS
            .into_iter()
            .map(|(name, _)| (name, pending(&originals, name)))
            .collect();
    let foreign_owner = input::Inputs::new(stamp.owner.clone()).unwrap();
    let foreign_bindings = foreign_owner.bindings().unwrap();
    let surface = cards.capture_surface.as_mut().unwrap();
    let queued = surface.inputs.queue.len();
    for (name, intent) in
        crate::presentation::web_ui_adapter::player_shell::window_caption::BINDINGS
    {
        let original = &original_pending[name];
        let foreign = pending(&foreign_bindings, name);
        assert!(matches!(
            surface.admit(&stamp, &foreign),
            IntentAdmission::Rejected(_)
        ));
        let mut stale = stamp.clone();
        stale.owner.source_generation += 1;
        assert!(matches!(
            surface.admit(&stale, original),
            IntentAdmission::Rejected(_)
        ));
        assert!(matches!(
            surface.admit(&stamp, original),
            IntentAdmission::Accepted
        ));
        assert!(matches!(
            surface.admit(&stamp, original),
            IntentAdmission::Capacity
        ));
        assert_eq!(caption.borrow_mut().take(), Some(intent));
    }
    caption.borrow_mut().close();
    for (name, _) in crate::presentation::web_ui_adapter::player_shell::window_caption::BINDINGS {
        let original = &original_pending[name];
        assert!(matches!(
            surface.admit(&stamp, original),
            IntentAdmission::Rejected(_)
        ));
    }
    surface.stop().unwrap();
    for (name, _) in crate::presentation::web_ui_adapter::player_shell::window_caption::BINDINGS {
        let original = &original_pending[name];
        assert!(!matches!(
            surface.admit(&stamp, original),
            IntentAdmission::Accepted
        ));
    }
    assert_eq!(surface.inputs.queue.len(), queued);
    drop(cards);
    assert_eq!(
        host.callbacks.borrow().tickets().unwrap().len(),
        before_tickets
    );
}
