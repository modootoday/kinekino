use super::*;
use crate::presentation::web_ui_adapter::proposals::{
    panel_model::{PanelBudget, PanelLabels, PanelSelection, prepare_panel},
    product_window::product_intent_pins,
    scoped_panel::{ScopedCardPanel, bind_card_panel},
};
use kinekino_player::assembly::meari::proposal_registration::ProposalSdkRegistration;
use meari::application::state_program::StateProgram;

struct Lease(ProposalSdkRegistration);
impl crate::presentation::web_ui_adapter::proposals::document_factory::ParentProposalLease
    for Lease
{
    fn registration(&self) -> Result<&RegistrationHandle, PluginError> {
        self.0.handle()
    }
    fn close(&mut self) -> Result<(), PluginError> {
        self.0.close()
    }
}
fn fixture() -> (StateProgram, ScopedCardPanel<Lease>, PanelLabels) {
    let (frame, mut input, stored) =
        crate::presentation::web_ui_adapter::proposals::scoped_panel::tests::fixture();
    let labels = input.labels.panel.clone();
    input.local_signatures = BTreeMap::from([
        ("onReturnToViewing".into(), CallbackSignature::Action),
        ("onSelectedProposalChange".into(), CallbackSignature::Text),
    ]);
    let panel = bind_card_panel(input, stored, |client, registry, scope, context| {
        ProposalSdkRegistration::register(client, registry, scope, context).map(Lease)
    })
    .unwrap_or_else(|_| panic!("Distinct root and editor callbacks must compose"));
    (frame, panel, labels)
}
fn fresh_bindings(panel: &ScopedCardPanel<Lease>, labels: PanelLabels) -> ReviewBindings {
    let review = panel.session.review();
    let selected = PanelSelection::at(review, review.scope(), 1).unwrap();
    prepare_panel(
        review,
        review.scope(),
        &selected,
        labels,
        PanelBudget {
            entries: 8,
            text_bytes: 8192,
        },
        product_intent_pins(),
    )
    .unwrap()
    .bindings
}
fn reference() -> CallbackReference {
    CallbackReference {
        binding: "onReturnToViewing".into(),
        signature: CallbackSignature::Action,
    }
}

#[test]
fn same_export_resolves_root_local_and_editor_plugin_with_scope_and_signature_guards() {
    let (_frame, mut panel, _) = fixture();
    let export = panel.session.export_callbacks().unwrap();
    let scope = panel.session.review().scope().view.clone();
    let root = export.resolve(&reference()).unwrap();
    let invocation = panel
        .session
        .dispatch_local_after_renderer_guard(&root, CallbackArguments::Action, &scope)
        .unwrap();
    assert!(invocation.matches_key(panel.session.local_action_key("onReturnToViewing").unwrap()));
    let editor = export
        .resolve(&CallbackReference {
            binding: "onSave".into(),
            signature: CallbackSignature::Action,
        })
        .unwrap();
    assert!(matches!(
        panel.session.dispatch_local_after_renderer_guard(
            &editor,
            CallbackArguments::Action,
            &scope
        ),
        Err(LocalDispatchError::NotLocal)
    ));
    assert!(matches!(
        panel.session.dispatch_local_after_renderer_guard(
            &root,
            CallbackArguments::Text("wrong".into()),
            &scope
        ),
        Err(LocalDispatchError::Invalid(_))
    ));
    let mut foreign = scope.clone();
    foreign.source_generation += 1;
    assert!(matches!(
        panel.session.dispatch_local_after_renderer_guard(
            &root,
            CallbackArguments::Action,
            &foreign
        ),
        Err(LocalDispatchError::Invalid(_))
    ));
    assert!(
        panel
            .session
            .references()
            .unwrap()
            .contains_key("onSelectedProposalChange")
    );
    panel.unbind().unwrap();
}

#[test]
fn rejected_projection_retains_old_local_key_and_success_replaces_only_after_commit() {
    let (_frame, mut panel, labels) = fixture();
    let scope = panel.session.review().scope().view.clone();
    let old_export = panel.session.export_callbacks().unwrap();
    let old_handle = old_export.resolve(&reference()).unwrap();
    let old_key = panel
        .session
        .local_action_key("onReturnToViewing")
        .unwrap()
        .clone();
    let mut rejected_handle = None;
    let bindings = fresh_bindings(&panel, labels.clone());
    let failure = panel.session.reproject_with(bindings, |export| {
        rejected_handle = Some(export.resolve(&reference()).unwrap());
        Err("Injected original renderer rejection".into())
    });
    assert!(matches!(failure, Err(ReprojectionError::Renderer(_))));
    assert_eq!(
        panel.session.local_action_key("onReturnToViewing").unwrap(),
        &old_key
    );
    assert!(
        panel
            .session
            .dispatch_local_after_renderer_guard(&old_handle, CallbackArguments::Action, &scope)
            .is_ok()
    );
    assert!(matches!(
        panel.session.dispatch_local_after_renderer_guard(
            &rejected_handle.unwrap(),
            CallbackArguments::Action,
            &scope
        ),
        Err(LocalDispatchError::Invalid(_))
    ));
    let bindings = fresh_bindings(&panel, labels);
    let mut committed = None;
    panel
        .session
        .reproject_with(bindings, |export| {
            committed = Some(export);
            Ok(())
        })
        .unwrap();
    assert_ne!(
        panel.session.local_action_key("onReturnToViewing").unwrap(),
        &old_key
    );
    assert!(matches!(
        panel.session.dispatch_local_after_renderer_guard(
            &old_handle,
            CallbackArguments::Action,
            &scope
        ),
        Err(LocalDispatchError::Invalid(_))
    ));
    let next_handle = committed.as_ref().unwrap().resolve(&reference()).unwrap();
    assert!(
        panel
            .session
            .dispatch_local_after_renderer_guard(&next_handle, CallbackArguments::Action, &scope)
            .is_ok()
    );
    panel.unbind().unwrap();
}

#[test]
fn root_local_name_cannot_shadow_existing_sdk_editor_binding() {
    let (_frame, mut input, stored) =
        crate::presentation::web_ui_adapter::proposals::scoped_panel::tests::fixture();
    input.local_signatures = BTreeMap::from([("onSave".into(), CallbackSignature::Action)]);
    let mut failure = match bind_card_panel(input, stored, |client, registry, scope, context| {
        ProposalSdkRegistration::register(client, registry, scope, context).map(Lease)
    }) {
        Err(failure) => failure,
        Ok(_) => panic!("Root local callbacks must not shadow editor capabilities"),
    };
    assert!(failure.parent.is_some());
    assert!(failure.session.is_some());
    failure.retry_unbind().unwrap();
    assert!(failure.parent.as_ref().unwrap().0.handle().is_err());
}
