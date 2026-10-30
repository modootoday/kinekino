use super::*;
use meari::{
    application::{Language, host_callbacks::HostCallbacks, state_program::StateProgram},
    domain::{
        host_callbacks::{CallbackReference, CallbackSignature},
        plugins::PluginScope,
    },
};

struct Source {
    callbacks: HostCallbacks,
    key: LocalActionKey,
    stamp: HostStamp,
}
impl Source {
    fn new(epoch: u64) -> Self {
        let component = meari::parse(
            "type Props={};export function Root(props:Props){return <main/>;}",
            Language::Tsx,
        )
        .unwrap();
        let program = StateProgram::mount(component, "navigation-unit".into(), epoch).unwrap();
        let scope = PluginScope {
            instance: program.snapshot().unwrap().instance,
            source_generation: 1,
        };
        let mut callbacks = HostCallbacks::new(scope.clone());
        let key = callbacks
            .bind_local(NAVIGATION_BINDING.into(), CallbackSignature::Text)
            .unwrap();
        Self {
            callbacks,
            key,
            stamp: HostStamp {
                host_id: 1,
                owner: scope,
                definition_revision: 1,
                document_revision: 1,
                turn_sequence: 1,
            },
        }
    }
    fn click(
        &mut self,
        mailbox: &mut NavigationIntents,
        value: &str,
        task: StudioTask,
    ) -> IntentAdmission<PanelBridgeError> {
        let export = self.callbacks.export_bindings().unwrap();
        let reference = CallbackReference {
            binding: NAVIGATION_BINDING.into(),
            signature: CallbackSignature::Text,
        };
        let handle = export.resolve(&reference).unwrap();
        let invocation = self
            .callbacks
            .dispatch_local(
                &handle,
                CallbackArguments::Text(value.into()),
                &self.stamp.owner,
            )
            .unwrap();
        mailbox.admit(invocation, &self.key, self.stamp.clone(), task)
    }
}
#[test]
fn authentic_view_is_one_intent_and_same_click_coalesces() {
    let mut source = Source::new(1);
    let mut mailbox = NavigationIntents::default();
    assert!(matches!(
        source.click(&mut mailbox, "VIEW", StudioTask::Card),
        IntentAdmission::Accepted
    ));
    assert!(matches!(
        source.click(&mut mailbox, "VIEW", StudioTask::Card),
        IntentAdmission::Accepted
    ));
    assert_eq!(mailbox.pending.as_ref().unwrap().task, StudioTask::Card);
    assert!(!mailbox.blocked);
}
#[test]
fn non_view_and_another_owner_do_not_replace_original_intent() {
    let mut original = Source::new(1);
    let mut foreign = Source::new(2);
    let mut mailbox = NavigationIntents::default();
    assert!(matches!(
        original.click(&mut mailbox, "JOB", StudioTask::Card),
        IntentAdmission::Rejected(_)
    ));
    assert!(mailbox.pending.is_none());
    assert!(matches!(
        original.click(&mut mailbox, "VIEW", StudioTask::Card),
        IntentAdmission::Accepted
    ));
    assert!(matches!(
        foreign.click(&mut mailbox, "VIEW", StudioTask::Highlight),
        IntentAdmission::Rejected(_)
    ));
    assert_eq!(
        mailbox.pending.as_ref().unwrap().stamp.owner,
        original.stamp.owner
    );
}
#[test]
fn old_export_and_wrong_signature_are_rejected_before_mailbox_admission() {
    let source = Source::new(1);
    let first = source.callbacks.export_bindings().unwrap();
    let reference = CallbackReference {
        binding: NAVIGATION_BINDING.into(),
        signature: CallbackSignature::Text,
    };
    let handle = first.resolve(&reference).unwrap();
    first.invalidate();
    let replacement = source.callbacks.export_bindings().unwrap();
    assert!(
        source
            .callbacks
            .dispatch_local(
                &handle,
                CallbackArguments::Text("VIEW".into()),
                &source.stamp.owner
            )
            .is_err()
    );
    let current = replacement.resolve(&reference).unwrap();
    assert!(
        source
            .callbacks
            .dispatch_local(&current, CallbackArguments::Action, &source.stamp.owner)
            .is_err()
    );
    let foreign_scope = Source::new(2).stamp.owner;
    assert!(
        source
            .callbacks
            .dispatch_local(
                &current,
                CallbackArguments::Text("VIEW".into()),
                &foreign_scope
            )
            .is_err()
    );
}
#[test]
fn route_failure_keeps_intent_and_first_error_without_automatic_retry() {
    use crate::presentation::web_ui_adapter::player_shell::{
        NavigationGuards, RouteCatalog, ShellState,
    };
    let mut source = Source::new(1);
    let mut mailbox = NavigationIntents::default();
    assert!(matches!(
        source.click(&mut mailbox, "VIEW", StudioTask::Card),
        IntentAdmission::Accepted
    ));
    let mut state = ShellState::new();
    let catalog = RouteCatalog::bundled().unwrap();
    let error = mailbox.request(|| {
        state
            .request_route(
                &catalog,
                "view.start",
                NavigationGuards {
                    dirty: true,
                    ..NavigationGuards::default()
                },
            )
            .map(|_| ())
            .map_err(ViewerHostError::Navigation)
    });
    assert!(matches!(
        error,
        Err(ViewerHostError::Navigation(
            crate::presentation::web_ui_adapter::player_shell::NavigationError::UnsavedEdits
        ))
    ));
    assert!(mailbox.pending.is_some() && mailbox.blocked && mailbox.first_error.is_some());
    assert!(matches!(
        source.click(&mut mailbox, "VIEW", StudioTask::Card),
        IntentAdmission::Accepted
    ));
    assert!(!mailbox.blocked && mailbox.first_error.is_some());
    let mut ticket = None;
    mailbox
        .request(|| {
            ticket = Some(
                state
                    .request_route(&catalog, "view.start", NavigationGuards::default())
                    .map_err(ViewerHostError::Navigation)?,
            );
            Ok(())
        })
        .unwrap();
    assert!(ticket.is_some() && mailbox.pending.is_none());
    let called = Cell::new(false);
    assert!(
        mailbox
            .request(|| {
                called.set(true);
                Ok(())
            })
            .is_err()
    );
    assert!(!called.get());
}
#[test]
fn projection_change_is_not_the_same_accepted_intent() {
    let mut source = Source::new(1);
    let mut mailbox = NavigationIntents::default();
    assert!(matches!(
        source.click(&mut mailbox, "VIEW", StudioTask::Card),
        IntentAdmission::Accepted
    ));
    source.stamp.document_revision += 1;
    assert!(matches!(
        source.click(&mut mailbox, "VIEW", StudioTask::Card),
        IntentAdmission::Rejected(_)
    ));
    assert_eq!(mailbox.pending.as_ref().unwrap().stamp.document_revision, 1);
}
#[test]
fn close_retires_pending_but_keeps_original_failure_and_rejects_late_click() {
    let mut source = Source::new(1);
    let mut mailbox = NavigationIntents::default();
    assert!(matches!(
        source.click(&mut mailbox, "VIEW", StudioTask::Card),
        IntentAdmission::Accepted
    ));
    let _error = mailbox.failed(ViewerHostError::ForeignCheckpoint);
    mailbox.close();
    assert!(mailbox.pending.is_none() && mailbox.first_error.is_some());
    assert!(matches!(
        source.click(&mut mailbox, "VIEW", StudioTask::Card),
        IntentAdmission::Rejected(_)
    ));
}
