use super::super::{ObservationFailureStage, record_delivery};
use super::*;
use crate::presentation::web_ui_adapter::player_shell::{
    NavigationGuards, RouteCatalog, ShellPhase,
};
use std::cell::Cell;

fn observation() -> StartRouteObservation {
    let mut state = ShellState::new();
    let catalog = RouteCatalog::bundled().unwrap();
    let ticket = state
        .request_route(&catalog, "view.start", NavigationGuards::default())
        .unwrap();
    StartRouteObservation::new(state, ticket)
}
fn ticket(scene: u64) -> FrameTicket {
    FrameTicket {
        context_id: "synthetic-context".into(),
        context_generation: 1,
        scene_generation: scene,
    }
}

#[test]
fn preparation_has_no_route_observation() {
    let observed = observation();
    assert_eq!(observed.state.phase(), ShellPhase::AwaitingHost);
    assert!(observed.state.observed_route().is_none());
    assert!(!observed.committed);
}

#[test]
fn synthetic_ticket_validation_rejects_empty_zero_duplicate_and_foreign_contexts() {
    let mut observed = observation();
    assert!(!observed.accept_ticket(&ticket(0)));
    let mut empty = ticket(1);
    empty.context_id.clear();
    assert!(!observed.accept_ticket(&empty));
    assert!(observed.accept_ticket(&ticket(1)));
    assert!(!observed.accept_ticket(&ticket(1)));
    let mut foreign = ticket(2);
    foreign.context_generation = 2;
    assert!(!observed.accept_ticket(&foreign));
    foreign.context_generation = 1;
    foreign.context_id = "other".into();
    assert!(!observed.accept_ticket(&foreign));
    assert!(observed.accept_ticket(&ticket(2)));
    // Pure metadata checks do not commit a route or prove a real renderer submission.
    assert!(observed.state.observed_route().is_none());
}

#[test]
fn cancelled_pending_route_remains_unobserved() {
    let mut observed = observation();
    observed.state.cancel_route(&observed.ticket).unwrap();
    assert!(observed.accept_ticket(&ticket(1)));
    assert!(observed.state.desired_route().is_none());
    assert!(observed.state.observed_route().is_none());
    assert_eq!(observed.state.phase(), ShellPhase::NoSelection);
}

#[test]
fn unavailable_delivery_returns_err_and_keeps_exact_notice_identity_visible() {
    let failure = Cell::new(None);
    let notice = NativeWindowNotice {
        revision: 7,
        host_id: 12,
        kind: NativeWindowNoticeKind::FrameSubmitted,
        window: None,
        geometry: None,
        frame: None,
    };
    for stage in [
        ObservationFailureStage::DocumentStamp,
        ObservationFailureStage::ChildrenBorrow,
        ObservationFailureStage::ChildrenMissing,
        ObservationFailureStage::WindowStamp,
        ObservationFailureStage::RouteBorrow,
    ] {
        assert!(record_delivery(&failure, &notice, Err(stage)).is_err());
        let observed = failure.get().unwrap();
        assert_eq!(observed.stage, stage);
        assert_eq!(observed.host_id, 12);
        assert_eq!(observed.notice_revision, 7);
    }
    // The production ParentBinding retains notice on Err; this helper test does not run that queue.
    assert!(record_delivery(&failure, &notice, Ok(())).is_ok());
    assert!(failure.get().is_none());
}
