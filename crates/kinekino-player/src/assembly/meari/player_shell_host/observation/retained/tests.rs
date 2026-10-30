//! Real handle/runtime lineage and synthetic None rejection, never checked-frame success.
use super::*;
use crate::{
    assembly::meari::viewer_cleanup::ViewerNoChild,
    presentation::web_ui_adapter::player_shell::{NavigationGuards, RouteCatalog, ShellPhase},
};
use meari::application::host_callbacks::PendingCallback;
use meari_native::{
    application::window_lifecycle::{LifecycleLimits, MonotonicClock, ReadyKind, WindowLifecycle},
    domain::{Viewport, widgets::Action},
    infrastructure::{
        document::InteractiveDocument, native_host::*, window::NativeFrameSubmission,
    },
};
use std::{io, task::Waker, time::Duration};

struct Clock;
impl MonotonicClock for Clock {
    fn now(&self) -> Duration {
        Duration::ZERO
    }
}
struct Idle;
impl NativeWindowHost for Idle {
    type Error = io::Error;
    type CloseCheckpoint = ();
    fn admit_callback(
        &mut self,
        _: &HostStamp,
        _: &PendingCallback,
    ) -> IntentAdmission<Self::Error> {
        IntentAdmission::Rejected(io::Error::other(
            "Unexpected callback in policy-only document",
        ))
    }
    fn after_dispatch(&mut self, _: HostTurn) -> Result<HostEffects, Self::Error> {
        Ok(HostEffects::default())
    }
    fn close(&mut self, _: CloseRequest<()>) -> CloseDecision<(), Self::Error> {
        CloseDecision::KeepOpen
    }
}
struct Fixture {
    observer: StartRouteObservation,
    document: NativeDocumentHandle,
    window: WindowStamp,
    ticket: RouteTicket,
    proof: RetainedCommitProof,
}
fn document() -> NativeDocumentHandle {
    let source = meari::parse(
        "export function Root(){return <button>go</button>}",
        meari::application::Language::Tsx,
    )
    .unwrap();
    NativeDocumentHandle::new(
        InteractiveDocument::new(
            source,
            "",
            Viewport {
                width: 160,
                height: 80,
                dark: false,
            },
        )
        .unwrap(),
        1,
    )
    .unwrap()
}
fn notice(revision: u64, stamp: &HostStamp, window: WindowStamp, scene: u64) -> NativeWindowNotice {
    NativeWindowNotice {
        revision,
        host_id: stamp.host_id,
        kind: NativeWindowNoticeKind::FrameSubmitted,
        window: Some(window),
        geometry: None,
        frame: Some(NativeFrameObservation {
            document: stamp.clone(),
            retained: None,
            submission: NativeFrameSubmission {
                ticket: FrameTicket {
                    context_id: "synthetic-retained-policy".into(),
                    context_generation: 1,
                    scene_generation: scene,
                },
                width: 160,
                height: 80,
                commands: 0,
            },
        }),
    }
}
fn fixture() -> Fixture {
    let document = document();
    let window = WindowLifecycle::<ViewerNoChild>::new(
        LifecycleLimits::new(1, Duration::from_secs(1)).unwrap(),
        Clock,
        Waker::noop().clone(),
    )
    .unwrap()
    .ready(ReadyKind::NewWindow)
    .unwrap();
    let catalog = RouteCatalog::bundled().unwrap();
    let mut state = ShellState::new();
    let initial = state
        .request_route(&catalog, "view.start", NavigationGuards::default())
        .unwrap();
    let mut observer = StartRouteObservation::new(state, initial);
    let stamp = document.stamp().unwrap();
    observer.observe(&notice(1, &stamp, window, 1), &stamp, window);
    let jobs = observer
        .state
        .request_route(&catalog, "studio.jobs", NavigationGuards::default())
        .unwrap();
    let (_, receipt) = document
        .try_reproject(&document.stamp().unwrap(), |_| Ok::<_, String>(()))
        .unwrap();
    let stamp = document.stamp().unwrap();
    observer.arm_jobs_route(jobs, &receipt, &stamp).unwrap();
    observer.observe(&notice(2, &stamp, window, 2), &stamp, window);
    assert_eq!(
        observer.state.observed_route().unwrap().as_str(),
        "studio.jobs"
    );
    let ticket = observer
        .state
        .request_route(&catalog, "view.start", NavigationGuards::default())
        .unwrap();
    let (_, receipt) = document
        .try_reproject(&document.stamp().unwrap(), |_| Ok::<_, String>(()))
        .unwrap();
    let proof = document.retain_commit(&receipt).unwrap();
    Fixture {
        observer,
        document,
        window,
        ticket,
        proof,
    }
}
impl Fixture {
    fn arm(&mut self) -> Result<(), NavigationError> {
        self.observer.arm_retained_viewing_route(
            self.ticket.clone(),
            self.document.clone(),
            self.proof.clone(),
        )
    }
}

fn studio_fixture(task: StudioTask) -> Fixture {
    let mut fixture = fixture();
    fixture
        .observer
        .state
        .cancel_route(&fixture.ticket)
        .unwrap();
    fixture.ticket = fixture
        .observer
        .state
        .request_route(
            &RouteCatalog::bundled().unwrap(),
            task.route_id(),
            NavigationGuards::default(),
        )
        .unwrap();
    fixture
}

#[test]
fn only_the_matching_typed_studio_entry_can_arm_and_none_never_observes_it() {
    for task in [
        StudioTask::Card,
        StudioTask::Highlight,
        StudioTask::Correction,
    ] {
        let mut fixture = studio_fixture(task);
        assert_eq!(fixture.arm(), Err(NavigationError::RouteMismatch));
        fixture
            .observer
            .arm_retained_studio_route(
                fixture.ticket.clone(),
                fixture.document.clone(),
                fixture.proof.clone(),
                task,
            )
            .unwrap();
        let current = fixture.document.stamp().unwrap();
        fixture.observer.observe(
            &notice(3, &current, fixture.window, 3),
            &current,
            fixture.window,
        );
        assert_eq!(fixture.observer.state.phase(), ShellPhase::AwaitingHost);
        assert_eq!(
            fixture.observer.state.observed_route().unwrap().as_str(),
            "studio.jobs"
        );
        assert!(fixture.observer.pending_card.is_none());
    }
    let mut fixture = studio_fixture(StudioTask::Correction);
    assert_eq!(
        fixture.observer.arm_retained_studio_route(
            fixture.ticket.clone(),
            fixture.document.clone(),
            fixture.proof.clone(),
            StudioTask::Highlight
        ),
        Err(NavigationError::RouteMismatch)
    );
    assert!(fixture.observer.pending_retained.is_none());
}

#[test]
fn typed_retained_entry_rearms_after_a_real_turn_without_replaying_a_strict_receipt() {
    let mut fixture = studio_fixture(StudioTask::Highlight);
    let original = fixture.proof.receipt().clone();
    let mut runtime = NativeHostRuntime::new(fixture.document.clone(), Idle);
    runtime.after_dispatch(HostTurnReason::Completion).unwrap();
    for _ in 0..2 {
        fixture
            .observer
            .arm_retained_studio_route(
                fixture.ticket.clone(),
                fixture.document.clone(),
                fixture.proof.clone(),
                StudioTask::Highlight,
            )
            .unwrap();
    }
    assert_eq!(
        fixture
            .observer
            .pending_retained
            .as_ref()
            .unwrap()
            .proof
            .receipt(),
        &original
    );
    assert_eq!(
        fixture.observer.arm_card_route(
            fixture.ticket.clone(),
            &original,
            &fixture.document.stamp().unwrap()
        ),
        Err(NavigationError::Busy)
    );
}

#[test]
fn receipt_without_immediate_proof_cannot_be_issued_as_history_after_a_turn() {
    let document = document();
    let (_, receipt) = document
        .try_reproject(&document.stamp().unwrap(), |_| Ok::<_, String>(()))
        .unwrap();
    let mut runtime = NativeHostRuntime::new(document.clone(), Idle);
    runtime.after_dispatch(HostTurnReason::Completion).unwrap();
    assert!(matches!(
        document.retain_commit(&receipt),
        Err(HostAccessError::Stale)
    ));
    assert_ne!(document.stamp().unwrap(), *receipt.after());
}

#[test]
fn actual_input_and_turn_allow_original_proof_rearm_but_none_cannot_complete_route() {
    let mut fixture = fixture();
    let original = fixture.proof.receipt().clone();
    let mut runtime = NativeHostRuntime::new(fixture.document.clone(), Idle);
    fixture
        .document
        .dispatch_action(Action::Activate { path: "0".into() })
        .unwrap();
    runtime.after_dispatch(HostTurnReason::Input).unwrap();
    let current = fixture.document.stamp().unwrap();
    assert!(current.document_revision > original.after().document_revision);
    assert!(current.turn_sequence > original.after().turn_sequence);
    assert_eq!(
        fixture
            .observer
            .arm_viewing_route(fixture.ticket.clone(), &original, &current),
        Err(NavigationError::StaleObservation)
    );
    fixture.arm().unwrap();
    runtime.after_dispatch(HostTurnReason::Completion).unwrap();
    fixture.arm().unwrap();
    assert_eq!(fixture.proof.receipt(), &original);
    let delivered = fixture.document.stamp().unwrap();
    // Notice describes the original stamp; real delivery reads a newer current stamp.
    fixture.observer.observe(
        &notice(3, original.after(), fixture.window, 3),
        &delivered,
        fixture.window,
    );
    assert!(fixture.observer.pending_retained.is_some());
    assert_eq!(fixture.observer.state.phase(), ShellPhase::AwaitingHost);
    assert_eq!(
        fixture.observer.state.observed_route().unwrap().as_str(),
        "studio.jobs"
    );
}

#[test]
fn new_projection_rejects_original_proof_and_keeps_accepted_expectation() {
    let mut fixture = fixture();
    fixture.arm().unwrap();
    let original = fixture.proof.receipt().clone();
    fixture
        .document
        .try_reproject(&fixture.document.stamp().unwrap(), |_| Ok::<_, String>(()))
        .unwrap();
    assert_eq!(fixture.arm(), Err(NavigationError::StaleObservation));
    assert_eq!(
        fixture
            .observer
            .pending_retained
            .as_ref()
            .unwrap()
            .proof
            .receipt(),
        &original
    );
    assert_eq!(
        fixture.observer.state.observed_route().unwrap().as_str(),
        "studio.jobs"
    );
}

#[test]
fn failed_mutable_entry_retires_proof_without_claiming_revision_or_rollback() {
    let mut fixture = fixture();
    let before = fixture.document.stamp().unwrap();
    assert!(
        fixture
            .document
            .try_reproject(&before, |_| Err::<(), _>("Mutable entry failed"))
            .is_err()
    );
    assert_eq!(fixture.document.stamp().unwrap(), before);
    assert_eq!(fixture.arm(), Err(NavigationError::StaleObservation));
    assert!(fixture.observer.pending_retained.is_none());
}

#[test]
fn foreign_handle_and_foreign_ticket_do_not_register_observation() {
    let mut fixture = fixture();
    assert_eq!(
        fixture.observer.arm_retained_viewing_route(
            fixture.ticket.clone(),
            document(),
            fixture.proof.clone()
        ),
        Err(NavigationError::StaleObservation)
    );
    let mut foreign = ShellState::new();
    let ticket = foreign
        .request_route(
            &RouteCatalog::bundled().unwrap(),
            "view.start",
            NavigationGuards::default(),
        )
        .unwrap();
    assert!(
        fixture
            .observer
            .arm_retained_viewing_route(ticket, fixture.document.clone(), fixture.proof.clone())
            .is_err()
    );
    assert!(fixture.observer.pending_retained.is_none());
}

#[test]
fn existing_strict_arm_cannot_replace_a_retained_expectation() {
    let mut fixture = fixture();
    fixture.arm().unwrap();
    assert_eq!(
        fixture.observer.arm_viewing_route(
            fixture.ticket.clone(),
            fixture.proof.receipt(),
            &fixture.document.stamp().unwrap()
        ),
        Err(NavigationError::Busy)
    );
    assert!(fixture.observer.pending_card.is_none());
    assert!(fixture.observer.pending_retained.is_some());
}

#[test]
fn cancelled_original_ticket_drops_only_its_observation_on_delivery() {
    let mut fixture = fixture();
    fixture.arm().unwrap();
    fixture
        .observer
        .state
        .cancel_route(&fixture.ticket)
        .unwrap();
    let current = fixture.document.stamp().unwrap();
    fixture.observer.observe(
        &notice(3, &current, fixture.window, 3),
        &current,
        fixture.window,
    );
    assert!(fixture.observer.pending_retained.is_none());
    assert!(fixture.arm().is_err());
    fixture
        .document
        .validate_retained_commit(&fixture.proof)
        .unwrap();
}

#[test]
fn terminal_notice_releases_pending_observation_and_prevents_rearm() {
    let mut fixture = fixture();
    fixture.arm().unwrap();
    let current = fixture.document.stamp().unwrap();
    let mut terminal = notice(3, &current, fixture.window, 3);
    terminal.kind = NativeWindowNoticeKind::Closed;
    terminal.frame = None;
    fixture
        .observer
        .observe(&terminal, &current, fixture.window);
    assert!(fixture.observer.pending_retained.is_none());
    assert_eq!(fixture.arm(), Err(NavigationError::RecoveryBlocked));
}
