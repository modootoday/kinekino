//! Receipt/frame-matching policy only. Frame notices/context are synthetic, no OS rendering.
use super::*;
use crate::{
    assembly::meari::viewer_cleanup::ViewerNoChild,
    presentation::web_ui_adapter::player_shell::{NavigationGuards, RouteCatalog, ShellPhase},
};
use meari_native::{
    application::window_lifecycle::{LifecycleLimits, MonotonicClock, ReadyKind, WindowLifecycle},
    domain::Viewport,
    infrastructure::{
        document::InteractiveDocument,
        native_host::NativeDocumentHandle,
        window::{NativeFrameObservation, NativeFrameSubmission},
    },
};
use std::{task::Waker, time::Duration};
struct Clock;
impl MonotonicClock for Clock {
    fn now(&self) -> Duration {
        Duration::ZERO
    }
}
fn document() -> NativeDocumentHandle {
    let source = meari::parse(
        "type Props={};export function Frame(props:Props){return <main/>;}",
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
                dark: true,
            },
        )
        .unwrap(),
        1,
    )
    .unwrap()
}
fn window() -> WindowStamp {
    WindowLifecycle::<ViewerNoChild>::new(
        LifecycleLimits::new(1, Duration::from_secs(1)).unwrap(),
        Clock,
        Waker::noop().clone(),
    )
    .unwrap()
    .ready(ReadyKind::NewWindow)
    .unwrap()
}
fn notice(
    revision: u64,
    document: &HostStamp,
    window: WindowStamp,
    scene: u64,
) -> NativeWindowNotice {
    NativeWindowNotice {
        revision,
        host_id: document.host_id,
        kind: NativeWindowNoticeKind::FrameSubmitted,
        window: Some(window),
        geometry: None,
        frame: Some(NativeFrameObservation {
            retained: None,
            document: document.clone(),
            submission: NativeFrameSubmission {
                ticket: FrameTicket {
                    context_id: "synthetic-card-frame-policy".into(),
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
fn started() -> (StartRouteObservation, NativeDocumentHandle, WindowStamp) {
    let mut state = ShellState::new();
    let catalog = RouteCatalog::bundled().unwrap();
    let ticket = state
        .request_route(&catalog, "view.start", NavigationGuards::default())
        .unwrap();
    let mut observed = StartRouteObservation::new(state, ticket);
    let document = document();
    let window = window();
    let stamp = document.stamp().unwrap();
    observed.observe(&notice(1, &stamp, window, 1), &stamp, window);
    assert_eq!(
        observed.state.observed_route().unwrap().as_str(),
        "view.start"
    );
    (observed, document, window)
}
fn request(observed: &mut StartRouteObservation) -> RouteTicket {
    observed
        .state
        .request_route(
            &RouteCatalog::bundled().unwrap(),
            "studio.cards",
            NavigationGuards::default(),
        )
        .unwrap()
}
fn receipt(document: &NativeDocumentHandle) -> HostCommitReceipt {
    // An actual handle receipt with a no-op closure validates identity/version policy only;
    // this helper does not install a Card source or prove a Card frame was rendered.
    document
        .try_reproject(&document.stamp().unwrap(), |_| Ok::<(), String>(()))
        .unwrap()
        .1
}

#[test]
fn viewing_return_requires_its_own_new_receipt_and_fresh_frame_after_card() {
    let (mut observed, document, window) = started();
    let ticket = request(&mut observed);
    let card = receipt(&document);
    let card_stamp = document.stamp().unwrap();
    observed.arm_card_route(ticket, &card, &card_stamp).unwrap();
    observed.observe(&notice(2, &card_stamp, window, 2), &card_stamp, window);
    let returning = observed
        .state
        .request_route(
            &RouteCatalog::bundled().unwrap(),
            "view.start",
            NavigationGuards::default(),
        )
        .unwrap();
    assert_eq!(
        observed.arm_viewing_route(returning.clone(), &card, &card_stamp),
        Err(NavigationError::StaleObservation)
    );
    let viewing = receipt(&document);
    let viewing_stamp = document.stamp().unwrap();
    observed
        .arm_viewing_route(returning, &viewing, &viewing_stamp)
        .unwrap();
    observed.observe(&notice(3, &card_stamp, window, 2), &viewing_stamp, window);
    assert_eq!(observed.state.phase(), ShellPhase::AwaitingHost);
    assert_eq!(
        observed.state.observed_route().unwrap().as_str(),
        "studio.cards"
    );
    observed.observe(
        &notice(4, &viewing_stamp, window, 3),
        &viewing_stamp,
        window,
    );
    assert_eq!(observed.state.phase(), ShellPhase::Observed);
    assert_eq!(
        observed.state.observed_route().unwrap().as_str(),
        "view.start"
    );
}

#[test]
fn card_requires_new_checked_receipt_and_matching_synthetic_frame_after_view_start() {
    let (mut observed, document, window) = started();
    let old = document.stamp().unwrap();
    let ticket = request(&mut observed);
    let commit = receipt(&document);
    let current = document.stamp().unwrap();
    observed.arm_card_route(ticket, &commit, &current).unwrap();
    assert_eq!(observed.state.phase(), ShellPhase::AwaitingHost);
    assert_eq!(
        observed.state.observed_route().unwrap().as_str(),
        "view.start"
    );
    observed.observe(&notice(2, &old, window, 1), &current, window);
    assert_eq!(observed.state.phase(), ShellPhase::AwaitingHost);
    observed.observe(&notice(3, &current, window, 2), &current, window);
    assert_eq!(observed.state.phase(), ShellPhase::Observed);
    assert_eq!(
        observed.state.observed_route().unwrap().as_str(),
        "studio.cards"
    );
}

#[test]
fn wrong_window_context_scope_or_zero_frame_does_not_report_card() {
    let (mut observed, document, window) = started();
    let ticket = request(&mut observed);
    let commit = receipt(&document);
    let current = document.stamp().unwrap();
    observed.arm_card_route(ticket, &commit, &current).unwrap();
    let mut bad = notice(2, &current, window, 2);
    bad.window = None;
    observed.observe(&bad, &current, window);
    let mut bad = notice(3, &current, window, 2);
    bad.frame.as_mut().unwrap().submission.ticket.context_id = "foreign".into();
    observed.observe(&bad, &current, window);
    let mut bad = notice(4, &current, window, 2);
    bad.frame.as_mut().unwrap().document.owner.source_generation += 1;
    observed.observe(&bad, &current, window);
    let mut bad = notice(5, &current, window, 2);
    bad.frame.as_mut().unwrap().submission.width = 0;
    observed.observe(&bad, &current, window);
    assert_eq!(observed.state.phase(), ShellPhase::AwaitingHost);
    observed.observe(&notice(6, &current, window, 2), &current, window);
    assert_eq!(
        observed.state.observed_route().unwrap().as_str(),
        "studio.cards"
    );
}

#[test]
fn cancelled_ticket_and_reused_props_receipt_cannot_observe_a_later_card_request() {
    let (mut observed, document, window) = started();
    let ticket = request(&mut observed);
    let commit = receipt(&document);
    let current = document.stamp().unwrap();
    observed
        .arm_card_route(ticket.clone(), &commit, &current)
        .unwrap();
    observed.state.cancel_route(&ticket).unwrap();
    observed.observe(&notice(2, &current, window, 2), &current, window);
    let next = request(&mut observed);
    assert_eq!(
        observed.arm_card_route(next.clone(), &commit, &current),
        Err(NavigationError::StaleObservation)
    );
    let next_commit = receipt(&document);
    let current = document.stamp().unwrap();
    observed
        .arm_card_route(next, &next_commit, &current)
        .unwrap();
    observed.observe(&notice(3, &current, window, 2), &current, window);
    assert_eq!(
        observed.state.observed_route().unwrap().as_str(),
        "studio.cards"
    );
}

#[test]
fn confirmed_window_close_revokes_pending_card_frame_expectation() {
    let (mut observed, document, window) = started();
    let ticket = request(&mut observed);
    let commit = receipt(&document);
    let current = document.stamp().unwrap();
    observed.arm_card_route(ticket, &commit, &current).unwrap();
    let mut closed = notice(2, &current, window, 2);
    closed.kind = NativeWindowNoticeKind::Closed;
    closed.frame = None;
    observed.observe(&closed, &current, window);
    observed.observe(&notice(3, &current, window, 2), &current, window);
    assert_eq!(observed.state.phase(), ShellPhase::AwaitingHost);
    assert_eq!(
        observed.state.observed_route().unwrap().as_str(),
        "view.start"
    );
    assert!(observed.terminal_window);
    assert!(observed.pending_card.is_none());
}
