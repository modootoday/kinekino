//! Original JOB active/recovery exports and current product graph, no replacement owner.
use super::*;
use meari::application::host_callbacks::CallbackBindings;
use meari::domain::host_callbacks::{CallbackInvocation, CallbackReference, CallbackSignature};

fn pending(bindings: &CallbackBindings, name: &str) -> PendingCallback {
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
}

pub(in crate::assembly::meari::viewer_host::jobs) fn assert_original_document(
    document: &mut JobsDocument,
) {
    let caption = document.caption_view();
    document.document.try_with(|native| {
        let props = composition::props(native, JobsFrameInput {
            caption: &caption, product_name: &document.product_name, context_label: &document.context_label,
            route: document.generation, list: &document.list, selection: &document.selection,
            detail: document.detail.as_ref(), inspector: document.inspector, blocked: false,
            retry_list: false, retry_detail: false, connection_retry: false, diagnostic: None, status: None,
        }).unwrap();
        crate::presentation::web_ui_adapter::player_shell::window_caption::contract_fixture::assert_required_caption_fields(
            &props, |invalid| document.frame.workspace_props(native, WorkspaceRoute::Jobs,
                document.generation, invalid).is_ok());
    }).unwrap();
    let source = document.caption_source.as_ref().unwrap().upgrade().unwrap();
    let originals = document.active.as_ref().unwrap().owner.bindings().unwrap();
    // Capture queued handles while the export is live, then exercise those
    // same handles after replacement. Resolving a revoked export must fail.
    let original_pending: BTreeMap<_, _> =
        crate::presentation::web_ui_adapter::player_shell::window_caption::BINDINGS
            .into_iter()
            .map(|(name, _)| (name, pending(&originals, name)))
            .collect();
    let stamp = document.document.stamp().unwrap();
    let foreign_owner = RecoveryExport::new(stamp.owner.clone()).unwrap();
    let foreign_bindings = foreign_owner.bindings().unwrap();
    for (name, intent) in
        crate::presentation::web_ui_adapter::player_shell::window_caption::BINDINGS
    {
        let original = &original_pending[name];
        let foreign = pending(&foreign_bindings, name);
        assert!(document.admit(&stamp, &foreign).is_err());
        let mut stale = stamp.clone();
        stale.document_revision += 1;
        assert!(document.admit(&stale, original).is_err());
        assert!(document.admit(&stamp, original).is_ok());
        assert!(document.admit(&stamp, original).is_err());
        assert!(!document.has_queued());
        assert_eq!(source.borrow_mut().take(), Some(intent));
    }
    // The production recovery producer replaces the document export, without issuing a read.
    document.paint_recovery().unwrap();
    let current = document.document.stamp().unwrap();
    let recovery_bindings = document.recovery.as_ref().unwrap().bindings().unwrap();
    let recovery_pending: BTreeMap<_, _> =
        crate::presentation::web_ui_adapter::player_shell::window_caption::BINDINGS
            .into_iter()
            .map(|(name, _)| (name, pending(&recovery_bindings, name)))
            .collect();
    for (name, intent) in
        crate::presentation::web_ui_adapter::player_shell::window_caption::BINDINGS
    {
        let original = &original_pending[name];
        let foreign = pending(&foreign_bindings, name);
        assert!(document.admit(&current, original).is_err());
        let recovery = &recovery_pending[name];
        assert!(document.admit_route_recovery(&current, &foreign).is_err());
        let mut stale = current.clone();
        stale.owner.source_generation += 1;
        assert!(document.admit_route_recovery(&stale, recovery).is_err());
        assert!(document.admit_route_recovery(&current, recovery).is_ok());
        assert!(document.admit_route_recovery(&current, recovery).is_err());
        assert!(!document.has_queued());
        assert_eq!(source.borrow_mut().take(), Some(intent));
    }
    document.recovery.as_mut().unwrap().invalidate().unwrap();
    for (name, _) in crate::presentation::web_ui_adapter::player_shell::window_caption::BINDINGS {
        let recovery = &recovery_pending[name];
        assert!(
            recovery_bindings
                .resolve(&CallbackReference {
                    binding: name.into(),
                    signature: CallbackSignature::Action,
                })
                .is_err()
        );
        assert!(document.admit_route_recovery(&current, recovery).is_err());
    }
    source.borrow_mut().close();
    let _ = document.close().unwrap();
    let current = document.document.stamp().unwrap();
    for (name, _) in crate::presentation::web_ui_adapter::player_shell::window_caption::BINDINGS {
        let original = &original_pending[name];
        let recovery = &recovery_pending[name];
        assert!(document.admit(&current, original).is_err());
        assert!(document.admit_route_recovery(&current, recovery).is_err());
        assert!(!document.has_queued());
    }
}
