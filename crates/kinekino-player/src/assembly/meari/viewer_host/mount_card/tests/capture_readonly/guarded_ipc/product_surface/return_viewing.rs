//! Actual product callbacks/SDK/frame commits; frame notices below are synthetic policy inputs.
use super::*;
use crate::{
    assembly::meari::viewer_cleanup::ViewerNoChild,
    presentation::web_ui_adapter::player_shell::{NavigationGuards, RouteCatalog, ShellState},
};
use meari_native::{
    application::window_lifecycle::{
        LifecycleLimits, MonotonicClock, ReadyKind, WindowLifecycle, WindowStamp,
    },
    domain::graphics::FrameTicket,
    infrastructure::window::{
        NativeFrameObservation, NativeFrameSubmission, NativeWindowNotice, NativeWindowNoticeKind,
    },
};
use std::task::Waker;

struct Clock;
impl MonotonicClock for Clock {
    fn now(&self) -> Duration {
        Duration::ZERO
    }
}
struct SyntheticFrames {
    observer: Rc<RefCell<StartRouteObservation>>,
    window: WindowStamp,
    sequence: u64,
}
impl SyntheticFrames {
    fn start(host: &mut ViewerHost, rig: &Rig, project: StoredProjectRevision) -> Self {
        let artifacts =
            crate::infrastructure::meari::capture_evidence::open(&rig.root.join("projects"))
                .unwrap();
        host.install_capture_evidence(artifacts).unwrap();
        bind_product(host, rig, project);
        let mut state = ShellState::new();
        let ticket = state
            .request_route(
                &RouteCatalog::bundled().unwrap(),
                "view.start",
                NavigationGuards::default(),
            )
            .unwrap();
        let observer = Rc::new(RefCell::new(StartRouteObservation::new(state, ticket)));
        host.bind_route_observation(observer.clone());
        let window = WindowLifecycle::<ViewerNoChild>::new(
            LifecycleLimits::new(1, Duration::from_secs(1)).unwrap(),
            Clock,
            Waker::noop().clone(),
        )
        .unwrap()
        .ready(ReadyKind::NewWindow)
        .unwrap();
        let mut frames = Self {
            observer,
            window,
            sequence: 0,
        };
        frames.submit_policy_notice(host);
        frames
    }
    fn submit_policy_notice(&mut self, host: &ViewerHost) {
        self.sequence = self.sequence.checked_add(1).unwrap();
        let stamp = host.document.as_ref().unwrap().stamp().unwrap();
        let notice = NativeWindowNotice {
            revision: self.sequence,
            host_id: stamp.host_id,
            kind: NativeWindowNoticeKind::FrameSubmitted,
            window: Some(self.window),
            geometry: None,
            frame: Some(NativeFrameObservation {
                retained: None,
                document: stamp.clone(),
                submission: NativeFrameSubmission {
                    ticket: FrameTicket {
                        context_id: "synthetic-capture-return-policy".into(),
                        context_generation: 1,
                        scene_generation: self.sequence,
                    },
                    width: 1280,
                    height: 720,
                    commands: 0,
                },
            }),
        };
        self.observer
            .borrow_mut()
            .observe_synthetic_notice_for_contract_test(&notice, &stamp, self.window);
    }
    fn open(&mut self, host: &mut ViewerHost) {
        open(host);
        self.submit_policy_notice(host);
        assert_eq!(
            self.observer
                .borrow()
                .state
                .observed_route()
                .unwrap()
                .as_str(),
            "studio.cards"
        );
    }
    fn finish_return(&mut self, host: &mut ViewerHost) {
        let end = Instant::now() + Duration::from_secs(3);
        while host.viewing_return_receipt().is_none() {
            host.after_dispatch(turn(host, HostTurnReason::Completion))
                .unwrap();
            assert!(Instant::now() < end);
        }
        let receipt = host.viewing_return_receipt().unwrap();
        assert_eq!(
            receipt.after(),
            &host.document.as_ref().unwrap().stamp().unwrap()
        );
        assert!(host.viewing_return_error().is_none());
        assert!(host.cards.borrow().capture.is_none());
        assert!(host.cards.borrow().capture_surface.is_none());
        assert!(host.cards.borrow().capture_entry.is_none());
        // Product source commit is real; no frame has been rendered by this test.
        assert_eq!(
            self.observer
                .borrow()
                .state
                .desired_route()
                .unwrap()
                .as_str(),
            "view.start"
        );
        assert_eq!(
            self.observer
                .borrow()
                .state
                .observed_route()
                .unwrap()
                .as_str(),
            "studio.cards"
        );
        assert!(!host.callbacks().borrow().is_released());
        let handle = host.document.as_ref().unwrap();
        let _read = callback_path(handle, "onReadStatus");
        let _choice = captured_radio_path(handle);
        self.submit_policy_notice(host);
        host.after_dispatch(turn(host, HostTurnReason::Completion))
            .unwrap();
        assert!(host.cards.borrow().viewing_return.is_none());
        assert_eq!(
            self.observer
                .borrow()
                .state
                .observed_route()
                .unwrap()
                .as_str(),
            "view.start"
        );
    }
}

fn assert_return_disabled(host: &ViewerHost) {
    let handle = host.document.as_ref().unwrap();
    let path = callback_path(handle, "onCaptureReturnViewing");
    assert!(
        handle
            .try_with(|document| document
                .elements()
                .iter()
                .find(|element| element.path == path)
                .unwrap()
                .attributes
                .contains_key("disabled"))
            .unwrap()
    );
    assert!(matches!(
        handle.dispatch_action(Action::Activate { path }),
        Err(meari_native::infrastructure::native_host::HostAccessError::Invalid(message))
            if message == "callback target is disabled, readOnly, hidden or inert"
    ));
    let (pending, _) = handle
        .try_reproject(&handle.stamp().unwrap(), |document| {
            Ok::<_, String>(document.drain_callbacks())
        })
        .unwrap();
    assert!(pending.is_empty());
}

#[test]
#[ignore = "Assigned original LocalHost slot; actual product callbacks/SDK, synthetic frame-policy notices only"]
fn dirty_capture_vetoes_return_then_explicit_revert_returns_and_retires_old_editor_callback()
-> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        let project = seed(&rig.client);
        let mut frames = SyntheticFrames::start(host, rig, project.clone());
        frames.open(host);
        let handle = host.document.as_ref().unwrap().clone();
        let id = handle.stamp().unwrap().host_id;
        text(host, "Local unsaved Card");
        let calls = requests(&rig.requests).len();
        assert_return_disabled(host);
        assert!(matches!(
            host.request_viewing_return(&turn(host, HostTurnReason::Input)),
            Err(ViewerHostError::Navigation(
                crate::presentation::web_ui_adapter::player_shell::NavigationError::UnsavedEdits
            ))
        ));
        assert!(host.cards.borrow().viewing_return.is_none());
        assert_eq!(requests(&rig.requests).len(), calls);
        click(host, "onCaptureRevert");
        let save = callback_path(&handle, "onSave");
        // Save is disabled after revert; retain a real enabled return callback instead.
        assert!(matches!(
            handle.dispatch_action(Action::Activate { path: save }),
            Err(meari_native::infrastructure::native_host::HostAccessError::Invalid(message))
                if message == "callback target is disabled, readOnly, hidden or inert"
        ));
        let back = callback_path(&handle, "onCaptureReturnViewing");
        assert!(
            handle
                .dispatch_action(Action::Activate { path: back })
                .unwrap()
        );
        let (mut pending, _) = handle
            .try_reproject(&handle.stamp().unwrap(), |document| {
                Ok::<_, String>(document.drain_callbacks())
            })
            .unwrap();
        assert_eq!(pending.len(), 1);
        let stale = pending.pop().unwrap();
        assert!(matches!(
            host.admit_callback(&handle.stamp().unwrap(), &stale),
            IntentAdmission::Accepted
        ));
        host.after_dispatch(turn(host, HostTurnReason::Input))
            .unwrap();
        frames.finish_return(host);
        assert_eq!(handle.stamp().unwrap().host_id, id);
        assert!(matches!(
            host.admit_callback(&handle.stamp().unwrap(), &stale),
            IntentAdmission::Rejected(_)
        ));
        assert_eq!(requests(&rig.requests).len(), calls);
        frames.open(host);
        assert!(matches!(
            host.admit_callback(&handle.stamp().unwrap(), &stale),
            IntentAdmission::Rejected(_)
        ));
        assert_immutable(rig, &project);
    })
}

#[test]
#[ignore = "Assigned original LocalHost slot; actual explicit save/accept/discard and original evidence publication"]
fn decided_capture_can_return_and_reenter_without_refreshing_saved_origin_or_replaying_mutation()
-> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        let project = seed(&rig.client);
        let mut frames = SyntheticFrames::start(host, rig, project.clone());
        frames.open(host);
        text(host, "Saved before returning");
        click(host, "onSave");
        ready(host);
        assert_return_disabled(host);
        assert!(
            host.request_viewing_return(&turn(host, HostTurnReason::Input))
                .is_err()
        );
        click(host, "onCaptureConsume");
        click(host, "onAccept");
        ready(host);
        assert_return_disabled(host);
        click(host, "onCaptureConsume");
        assert_eq!(host.captured_evidence_status().unwrap().saved, 2);
        let calls = requests(&rig.requests).len();
        click(host, "onCaptureReturnViewing");
        frames.finish_return(host);
        assert_eq!(requests(&rig.requests).len(), calls);
        frames.open(host);
        // Re-entry remains the original immutable capture, not an implicit live refresh.
        assert_eq!(
            host.capture_adapter_mut()
                .unwrap()
                .display()
                .state
                .document
                .draft_revision,
            project.project.proposals[0]
                .captured
                .document
                .draft_revision
        );
        text(host, "Stale local edit after reentry");
        click(host, "onSave");
        ready(host);
        assert_return_disabled(host);
        click(host, "onCaptureDiscard");
        // Discarding a response does not discard local edited text or undo the backend.
        assert_return_disabled(host);
        click(host, "onCaptureRevert");
        click(host, "onCaptureReturnViewing");
        frames.finish_return(host);
        assert_eq!(host.captured_evidence_status().unwrap().saved, 3);
        assert_immutable(rig, &project);
    })
}
