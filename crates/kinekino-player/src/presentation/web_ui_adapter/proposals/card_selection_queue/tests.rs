use super::*;
use kinekino_player::domain::{
    api_values::DecimalU64,
    projects::{ProjectProposalSnapshot, ProjectWorkState, WorkingState, canonical_sha256},
    proposals::ProposalPayload,
};
use meari::{
    application::{host_callbacks::HostCallbacks, state_program::StateProgram},
    domain::host_callbacks::{CallbackReference, CallbackSignature},
};
use std::num::NonZeroU64;

struct Fixture {
    _frame: StateProgram,
    queue: CardSelectionQueue,
    callbacks: HostCallbacks,
    key: LocalActionKey,
    first: String,
    second: String,
    scope: PluginScope,
}
fn fixture(capacity: usize) -> Fixture {
    let (frame, input, stored) = super::super::scoped_panel::tests::fixture();
    let mut working = stored.document.draft.clone().unwrap();
    let ProposalPayload::Cards { cards } = &mut working else {
        panic!("Typed Cards required")
    };
    cards[0].front = "Saved project working edit".into();
    let mut project = ProjectRevision {
        schema_version: "1".into(),
        project_id: "selected-project".into(),
        revision: DecimalU64(1),
        parent_revision: None,
        title: "Selected project".into(),
        watching: None,
        proposals: vec![ProjectProposalSnapshot {
            capture_sha256: canonical_sha256(&stored).unwrap(),
            captured: stored,
            working_draft: Some(working),
            working_state: WorkingState::Valid,
            proposal_origin: None,
        }],
        work_state: ProjectWorkState::default(),
    };
    project.work_state = project.derived_work_state();
    let choices = ProjectCardChoices::new(&project, NonZeroU64::new(3).unwrap()).unwrap();
    let scope = input.scope;
    let queue = CardSelectionQueue::new(
        scope.clone(),
        &choices,
        NonZeroUsize::new(capacity).unwrap(),
    );
    let first = choices.rows()[0].key().into();
    let second = choices.rows()[1].key().into();
    let mut callbacks = HostCallbacks::new(scope.clone());
    let key = callbacks
        .bind_local(CARD_SELECTION_BINDING.into(), CallbackSignature::Text)
        .unwrap();
    Fixture {
        _frame: frame,
        queue,
        callbacks,
        key,
        first,
        second,
        scope,
    }
}
fn invocation(fixture: &Fixture, key: &str) -> LocalActionInvocation {
    let export = fixture.callbacks.export_bindings().unwrap();
    let handle = export
        .resolve(&CallbackReference {
            binding: CARD_SELECTION_BINDING.into(),
            signature: CallbackSignature::Text,
        })
        .unwrap();
    fixture
        .callbacks
        .dispatch_local(&handle, CallbackArguments::Text(key.into()), &fixture.scope)
        .unwrap()
}

#[test]
fn capacity_retains_first_explicit_choice_and_retry_admits_original_next_target() {
    let mut fixture = fixture(1);
    let second = invocation(&fixture, &fixture.second);
    assert!(matches!(
        fixture.queue.admit(second, &fixture.key),
        IntentAdmission::Accepted
    ));
    let first = invocation(&fixture, &fixture.first);
    assert!(matches!(
        fixture.queue.admit(first, &fixture.key),
        IntentAdmission::Capacity
    ));
    assert_eq!(fixture.queue.len(), 1);
    assert!(matches!(
        fixture.queue.front().unwrap().selection.target,
        kinekino_player::application::launch::native_proposal::NativeProposalTarget::Card {
            index: 1
        }
    ));
    let selected = fixture.queue.pop().unwrap();
    assert!(matches!(
        selected.selection.target,
        kinekino_player::application::launch::native_proposal::NativeProposalTarget::Card {
            index: 1
        }
    ));
    let first = invocation(&fixture, &fixture.first);
    assert!(matches!(
        fixture.queue.admit(first, &fixture.key),
        IntentAdmission::Accepted
    ));
    assert!(matches!(
        fixture.queue.pop().unwrap().selection.target,
        kinekino_player::application::launch::native_proposal::NativeProposalTarget::Card {
            index: 0
        }
    ));
    assert!(fixture.queue.is_empty());
}

#[test]
fn invalid_row_or_foreign_local_key_is_rejected_even_with_full_capacity() {
    let mut fixture = fixture(1);
    let selected = invocation(&fixture, &fixture.second);
    assert!(matches!(
        fixture.queue.admit(selected, &fixture.key),
        IntentAdmission::Accepted
    ));
    let invalid = invocation(&fixture, "project-card-2-0-1");
    assert!(
        matches!(fixture.queue.admit(invalid, &fixture.key), IntentAdmission::Rejected(error) if error.code == PluginErrorCode::Stale)
    );
    let mut foreign = HostCallbacks::new(fixture.scope.clone());
    let other_key = foreign
        .bind_local(CARD_SELECTION_BINDING.into(), CallbackSignature::Text)
        .unwrap();
    let export = foreign.export_bindings().unwrap();
    let handle = export
        .resolve(&CallbackReference {
            binding: CARD_SELECTION_BINDING.into(),
            signature: CallbackSignature::Text,
        })
        .unwrap();
    let invalid = foreign
        .dispatch_local(
            &handle,
            CallbackArguments::Text(fixture.first.clone()),
            &fixture.scope,
        )
        .unwrap();
    assert_ne!(other_key, fixture.key);
    assert!(
        matches!(fixture.queue.admit(invalid, &fixture.key), IntentAdmission::Rejected(error) if error.code == PluginErrorCode::Stale)
    );
    assert_eq!(fixture.queue.len(), 1);
}

#[test]
fn queued_targets_share_one_immutable_project_and_preserve_working_edits() {
    let mut fixture = fixture(2);
    for row in [fixture.first.clone(), fixture.second.clone()] {
        let event = invocation(&fixture, &row);
        assert!(matches!(
            fixture.queue.admit(event, &fixture.key),
            IntentAdmission::Accepted
        ));
    }
    let first = fixture.queue.pop().unwrap();
    let second = fixture.queue.pop().unwrap();
    assert!(Arc::ptr_eq(&first.project, &second.project));
    assert_eq!(second.selection.proposal_id, "scoped-card");
    let Some(ProposalPayload::Cards { cards }) = &second.project.proposals[0].working_draft else {
        panic!("Working draft must survive selection")
    };
    assert_eq!(cards[0].front, "Saved project working edit");
    let Some(ProposalPayload::Cards { cards }) =
        &second.project.proposals[0].captured.document.draft
    else {
        panic!("Capture must remain distinct")
    };
    assert_eq!(cards[0].front, "first");
}

#[test]
fn existing_shell_guards_keep_queued_selection_and_acceptance_is_not_a_mount_observation() {
    use super::super::super::player_shell::{
        NavigationError, NavigationGuards, RouteCatalog, ShellPhase, ShellState,
    };
    let mut fixture = fixture(1);
    let event = invocation(&fixture, &fixture.second);
    assert!(matches!(
        fixture.queue.admit(event, &fixture.key),
        IntentAdmission::Accepted
    ));
    let catalog = RouteCatalog::bundled().unwrap();
    let mut state = ShellState::new();
    for (guards, expected) in [
        (
            NavigationGuards {
                dirty: true,
                ..Default::default()
            },
            NavigationError::UnsavedEdits,
        ),
        (
            NavigationGuards {
                pending: true,
                ..Default::default()
            },
            NavigationError::Busy,
        ),
        (
            NavigationGuards {
                recovery_blocked: true,
                ..Default::default()
            },
            NavigationError::RecoveryBlocked,
        ),
    ] {
        assert!(
            matches!(fixture.queue.request_route(&mut state, &catalog, guards), Err(error) if error == expected)
        );
        assert_eq!(fixture.queue.len(), 1);
        assert_eq!(state.phase(), ShellPhase::NoSelection);
    }
    let request = fixture
        .queue
        .request_route(&mut state, &catalog, NavigationGuards::default())
        .unwrap()
        .unwrap();
    assert!(matches!(
        request.target.selection.target,
        kinekino_player::application::launch::native_proposal::NativeProposalTarget::Card {
            index: 1
        }
    ));
    assert!(fixture.queue.is_empty());
    assert_eq!(state.phase(), ShellPhase::AwaitingHost);
    assert!(state.observed_route().is_none());
    assert_eq!(state.desired_route().unwrap().as_str(), "studio.cards");
    state.cancel_route(&request.ticket).unwrap();
}
