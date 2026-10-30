use super::*;
use meari::{
    application::{
        Language,
        host_callbacks::{CallbackBindings, LocalDispatchError},
        state_program::StateProgram,
    },
    domain::{
        Span,
        build::{ContentHash, TargetKind},
        host_callbacks::{CallbackArguments, CallbackInvocation, CallbackReference},
    },
};
fn fixture() -> (HostCallbacks, ReadActions, PluginScope, CallbackBindings) {
    let component = meari::parse(
        "type Props={};export function Root(props:Props){return <main/>;}",
        Language::Tsx,
    )
    .unwrap();
    let program = StateProgram::mount(component, "local-read-actions".into(), 1).unwrap();
    let scope = PluginScope {
        instance: program.snapshot().unwrap().instance,
        source_generation: 1,
    };
    let mut callbacks = HostCallbacks::new(scope.clone());
    let actions = ReadActions::bind(&mut callbacks).unwrap();
    let bindings = callbacks.export_bindings().unwrap();
    (callbacks, actions, scope, bindings)
}
fn pending(bindings: &CallbackBindings) -> PendingCallback {
    PendingCallback {
        handle: bindings
            .resolve(&CallbackReference {
                binding: "onRefreshReadProjection".into(),
                signature: CallbackSignature::Action,
            })
            .unwrap(),
        invocation: CallbackInvocation {
            prop: "onRefreshReadProjection".into(),
            arguments: CallbackArguments::Action,
            span: Span::default(),
        },
    }
}
#[test]
fn local_key_mapping_does_not_use_property_spelling() {
    let (callbacks, actions, scope, bindings) = fixture();
    let input = pending(&bindings);
    let invocation = callbacks
        .dispatch_local(&input.handle, CallbackArguments::Action, &scope)
        .unwrap();
    assert_eq!(
        actions.intent(&invocation),
        Some(ReadIntent::RefreshProjection)
    );
    let (_, foreign_actions, _, _) = fixture();
    assert_eq!(foreign_actions.intent(&invocation), None);
}
#[test]
fn local_queue_is_bounded_to_four_original_callbacks() {
    let (callbacks, mut actions, scope, bindings) = fixture();
    let input = pending(&bindings);
    for _ in 0..4 {
        actions
            .enqueue(&input, ReadIntent::RefreshProjection, None)
            .unwrap();
    }
    assert_eq!(
        actions
            .enqueue(&input, ReadIntent::RefreshProjection, None)
            .unwrap_err()
            .code,
        PluginErrorCode::Busy
    );
    let returned = actions
        .restore(QueuedReadAction {
            callback: pending(&bindings),
            intent: ReadIntent::RefreshProjection,
            claim: None,
        })
        .unwrap_err();
    assert!(
        callbacks
            .dispatch_local(
                &returned.callback.handle,
                returned.callback.invocation.arguments,
                &scope
            )
            .is_ok()
    );
    let first = actions.pop().unwrap();
    actions.restore(first).unwrap();
    for _ in 0..4 {
        assert!(actions.pop().is_some());
    }
    assert!(actions.is_empty());
}
#[test]
fn invalid_signature_never_becomes_not_local() {
    let (callbacks, _, scope, bindings) = fixture();
    assert!(matches!(
        callbacks.dispatch_local(
            &pending(&bindings).handle,
            CallbackArguments::Boolean(true),
            &scope
        ),
        Err(LocalDispatchError::Invalid(_))
    ));
}
#[test]
fn changed_scope_is_revalidated_after_queueing() {
    let (callbacks, mut actions, mut scope, bindings) = fixture();
    actions
        .enqueue(&pending(&bindings), ReadIntent::RefreshProjection, None)
        .unwrap();
    scope.source_generation += 1;
    let input = actions.pop().unwrap();
    assert!(matches!(
        callbacks.dispatch_local(
            &input.callback.handle,
            input.callback.invocation.arguments,
            &scope
        ),
        Err(LocalDispatchError::Invalid(_))
    ));
}
#[test]
fn reexported_lease_does_not_revive_old_queue_handles() {
    let (callbacks, mut actions, scope, bindings) = fixture();
    actions
        .enqueue(&pending(&bindings), ReadIntent::RefreshProjection, None)
        .unwrap();
    bindings.invalidate();
    let fresh = callbacks.export_bindings().unwrap();
    let old = actions.pop().unwrap();
    assert!(matches!(
        callbacks.dispatch_local(&old.callback.handle, CallbackArguments::Action, &scope),
        Err(LocalDispatchError::Invalid(_))
    ));
    assert!(
        callbacks
            .dispatch_local(&pending(&fresh).handle, CallbackArguments::Action, &scope)
            .is_ok()
    );
}
#[test]
fn local_target_cannot_dispatch_to_sdk_registry() {
    let (callbacks, _, scope, bindings) = fixture();
    let mut registry = PluginRegistry::new(
        ContentHash::of(b"local-sdk-zero"),
        TargetKind::Native,
        scope,
        8,
    )
    .unwrap();
    assert_eq!(
        callbacks
            .dispatch(
                &pending(&bindings).handle,
                CallbackArguments::Action,
                &mut registry
            )
            .unwrap_err()
            .code,
        PluginErrorCode::InvalidPayload
    );
    assert_eq!(registry.pending_count(), 0);
}
