//! Controlled frame port tests. No HWND, native lease or checked proof is fabricated.
use super::*;
use meari_native::{
    application::window_lifecycle::*,
    domain::Viewport,
    infrastructure::{
        document::InteractiveDocument,
        window::{NativeFrameObservation, NativeFrameSubmission},
    },
};
use std::{cell::Cell, collections::VecDeque, task::Waker, time::Duration};

struct Clock;
impl MonotonicClock for Clock {
    fn now(&self) -> Duration {
        Duration::ZERO
    }
}
struct MetadataOnly;
impl NativeChild for MetadataOnly {
    type Error = std::io::Error;
    fn parent_lost(&mut self) {
        panic!("metadata fixture has no child")
    }
    fn begin_cleanup(
        &mut self,
        _: CleanupRequest,
        _: CompletionSignal,
    ) -> Result<CleanupProgress, Self::Error> {
        panic!("metadata fixture has no child")
    }
    fn observe_cleanup(&mut self, _: CleanupRequest) -> Result<CleanupProgress, Self::Error> {
        panic!("metadata fixture has no child")
    }
}
fn window() -> WindowStamp {
    WindowLifecycle::<MetadataOnly>::new(
        LifecycleLimits::new(1, Duration::from_secs(1)).unwrap(),
        Clock,
        Waker::noop().clone(),
    )
    .unwrap()
    .ready(ReadyKind::NewWindow)
    .unwrap()
}
fn stamp() -> HostStamp {
    let source = meari::parse(
        "type Props={};export function App(props:Props){return <video/>;}",
        meari::application::Language::Tsx,
    )
    .unwrap();
    NativeDocumentHandle::new(
        InteractiveDocument::new(
            source,
            "",
            Viewport {
                width: 100,
                height: 100,
                dark: false,
            },
        )
        .unwrap(),
        1,
    )
    .unwrap()
    .stamp()
    .unwrap()
}
fn frame(stamp: &HostStamp, window: WindowStamp, sequence: u64) -> NativeWindowNotice {
    NativeWindowNotice {
        revision: sequence,
        host_id: stamp.host_id,
        kind: NativeWindowNoticeKind::FrameSubmitted,
        window: Some(window),
        geometry: None,
        frame: Some(NativeFrameObservation {
            retained: None,
            document: stamp.clone(),
            submission: NativeFrameSubmission {
                ticket: FrameTicket {
                    context_id: "controlled-port".into(),
                    context_generation: 1,
                    scene_generation: sequence,
                },
                width: 100,
                height: 100,
                commands: 1,
            },
        }),
    }
}
fn completion(stamp: &HostStamp) -> HostTurn {
    let mut stamp = stamp.clone();
    stamp.turn_sequence += 1;
    HostTurn {
        stamp,
        reason: HostTurnReason::Completion,
        accepted_callbacks: 0,
        rejected_callbacks: 0,
        remaining_callbacks: 0,
    }
}
struct Port {
    prepares: usize,
    consumes: usize,
    steps: VecDeque<Progress>,
    _resource: Rc<()>,
    contexts: Cell<usize>,
    context_failure: Cell<bool>,
}
impl FrameBackend for Port {
    fn context(&self, notice: &NativeWindowNotice) -> Result<FrameContext, FrameFailure> {
        self.contexts.set(self.contexts.get() + 1);
        if self.context_failure.get() {
            return Err(FrameFailure::Rejected(FrameRejection::ForeignFrame));
        }
        Ok(FrameContext {
            stamp: notice.frame.as_ref().unwrap().document.clone(),
            window: notice.window.unwrap(),
            ticket: notice.frame.as_ref().unwrap().submission.ticket.clone(),
        })
    }
    fn prepare(&mut self, _: &str, _: &HostStamp) -> Result<(), FrameFailure> {
        self.prepares += 1;
        Ok(())
    }
    fn consume(&mut self, _: Instant, _: &dyn Fn() -> Instant) -> Result<Progress, FrameFailure> {
        self.consumes += 1;
        Ok(self
            .steps
            .pop_front()
            .expect("unexpected additional native work"))
    }
}
fn flow(
    budget: usize,
    steps: Vec<Progress>,
) -> (
    Flow<Port>,
    FrameReceiver,
    HostStamp,
    Instant,
    std::rc::Weak<()>,
) {
    let now = Instant::now();
    let stamp = stamp();
    let resource = Rc::new(());
    let weak = Rc::downgrade(&resource);
    let mailbox = Rc::new(RefCell::new(Mailbox {
        request: NonZeroU64::new(7).unwrap(),
        origin: stamp.clone(),
        expires_at: now + Duration::from_secs(1),
        pending: None,
        redraw: false,
    }));
    let request = FrameRequest {
        id: NonZeroU64::new(7).unwrap(),
        origin: stamp.clone(),
        video: "original-video".into(),
        expires_at: now + Duration::from_secs(1),
        frame_budget: NonZeroUsize::new(budget).unwrap(),
    };
    (
        Flow {
            request,
            backend: Port {
                prepares: 0,
                consumes: 0,
                steps: steps.into(),
                _resource: resource,
                contexts: Cell::new(0),
                context_failure: Cell::new(false),
            },
            origin: Some(stamp.clone()),
            last_scope: Some(stamp.clone()),
            window: None,
            last_frame: None,
            left: budget,
            phase: Phase::Waiting,
            first_error: None,
            buffered: None,
            buffered_redraw: false,
            dirty: false,
            redraw_requested: false,
            mailbox: mailbox.clone(),
        },
        FrameReceiver { mailbox },
        stamp,
        now,
        weak,
    )
}

#[test]
fn pending_redraw_then_same_frame_replay_and_owned_completion_do_not_repeat_native_work() {
    let (mut flow, receiver, stamp, now, _) = flow(
        4,
        vec![Progress::Pending { redraw: true }, Progress::Observed],
    );
    let win = window();
    let first = frame(&stamp, win, 1);
    assert!(flow.observe_at(&first, now).unwrap());
    let mut turn = completion(&stamp);
    turn.reason = HostTurnReason::Input;
    assert!(receiver.receive(&turn).unwrap().result.is_none());
    turn.reason = HostTurnReason::Completion;
    assert!(receiver.receive(&turn).unwrap().redraw);
    assert!(!flow.observe_at(&first, now).unwrap());
    assert_eq!(
        (flow.backend.prepares, flow.backend.consumes, flow.left),
        (1, 1, 3)
    );
    let next = frame(&turn.stamp, win, 2);
    assert!(flow.observe_at(&next, now).unwrap());
    let update = receiver.receive(&completion(&turn.stamp)).unwrap();
    assert!(matches!(
        update.result.unwrap().state,
        FrameResultState::ObservedDisabled { .. }
    ));
    assert!(!update.redraw);
    assert!(
        !flow
            .observe_at(&frame(&completion(&turn.stamp).stamp, win, 3), now)
            .unwrap()
    );
    assert_eq!((flow.backend.prepares, flow.backend.consumes), (1, 2));
}
#[test]
fn budget_and_deadline_stop_new_work_and_retain_original_owner() {
    let (mut flow, receiver, stamp, now, owner) = flow(1, vec![Progress::Pending { redraw: true }]);
    let win = window();
    flow.observe_at(&frame(&stamp, win, 1), now).unwrap();
    receiver.receive(&completion(&stamp)).unwrap();
    let next = completion(&stamp).stamp;
    flow.observe_at(&frame(&next, win, 2), now).unwrap();
    assert!(matches!(
        receiver
            .receive(&completion(&next))
            .unwrap()
            .result
            .unwrap()
            .state,
        FrameResultState::Rejected(FrameRejection::BudgetExhausted)
    ));
    assert_eq!(flow.backend.consumes, 1);
    assert!(owner.upgrade().is_some());
    drop(flow);
    assert!(owner.upgrade().is_none()); // receiver carries data, not native owner authority
    let (mut expired, receiver, stamp, _, _) = self::flow(2, vec![]);
    let cutoff = expired.request.expires_at;
    expired
        .observe_at(&frame(&stamp, window(), 1), cutoff)
        .unwrap();
    assert_eq!((expired.backend.prepares, expired.backend.consumes), (0, 0));
    assert!(matches!(
        receiver
            .receive(&completion(&stamp))
            .unwrap()
            .result
            .unwrap()
            .state,
        FrameResultState::Rejected(FrameRejection::Expired)
    ));
}
#[test]
fn foreign_source_and_window_never_retarget_original_request() {
    let (mut flow, _, stamp, now, _) = flow(3, vec![]);
    let mut foreign = stamp.clone();
    foreign.owner.source_generation += 1;
    flow.observe_at(&frame(&foreign, window(), 1), now).unwrap();
    assert_eq!((flow.backend.prepares, flow.backend.consumes), (0, 0));
    let contexts = flow.backend.contexts.get();
    flow.observe_at(&frame(&stamp, window(), 2), now).unwrap();
    assert_eq!(flow.backend.contexts.get(), contexts);
    let (mut original, _, stamp, now, _) = self::flow(3, vec![Progress::Pending { redraw: true }]);
    original
        .observe_at(&frame(&stamp, window(), 1), now)
        .unwrap();
    original
        .observe_at(&frame(&completion(&stamp).stamp, window(), 2), now)
        .unwrap();
    assert_eq!(
        (original.backend.prepares, original.backend.consumes),
        (1, 1)
    );
    assert!(matches!(
        original.first_error,
        Some(FrameFailure::Rejected(FrameRejection::WindowChanged))
    ));
}
#[test]
fn once_redraw_and_no_budget_renewal_across_distinct_pending_frames() {
    let (mut flow, receiver, stamp, now, _) = flow(
        3,
        vec![
            Progress::Pending { redraw: true },
            Progress::Pending { redraw: true },
        ],
    );
    let win = window();
    let deadline = flow.request.expires_at;
    flow.observe_at(&frame(&stamp, win, 1), now).unwrap();
    assert!(receiver.receive(&completion(&stamp)).unwrap().redraw);
    let next = completion(&stamp).stamp;
    flow.observe_at(&frame(&next, win, 2), now).unwrap();
    assert!(!receiver.receive(&completion(&next)).unwrap().redraw);
    assert_eq!(flow.request.expires_at, deadline);
    assert_eq!(flow.left, 1);
}
#[test]
fn mailbox_backpressure_retains_owned_result_without_second_proof_consumption() {
    let (mut flow, receiver, stamp, now, _) = flow(2, vec![Progress::Observed]);
    let mailbox = flow.mailbox.clone();
    let held = mailbox.borrow_mut();
    let notice = frame(&stamp, window(), 1);
    assert!(flow.observe_at(&notice, now).is_err());
    assert_eq!(flow.backend.consumes, 1);
    assert!(flow.dirty);
    drop(held);
    assert!(flow.observe_at(&notice, now).unwrap());
    assert_eq!(flow.backend.consumes, 1);
    assert!(matches!(
        receiver
            .receive(&completion(&stamp))
            .unwrap()
            .result
            .unwrap()
            .state,
        FrameResultState::ObservedDisabled { .. }
    ));
}
#[test]
fn foreign_completion_preserves_pending_data_and_redraw_for_original_scope() {
    let (mut flow, receiver, stamp, now, _) = flow(2, vec![Progress::Pending { redraw: true }]);
    flow.observe_at(&frame(&stamp, window(), 1), now).unwrap();
    let mut wrong = completion(&stamp);
    wrong.stamp.document_revision += 1;
    assert!(matches!(
        receiver.receive(&wrong),
        Err(FrameReceiveError::ForeignScope)
    ));
    let correct = receiver.receive(&completion(&stamp)).unwrap();
    assert!(correct.redraw);
    assert!(correct.result.is_some());
    assert!(
        receiver
            .receive(&completion(&stamp))
            .unwrap()
            .result
            .is_none()
    );
}
#[test]
fn document_change_invalidates_historical_observation_without_new_native_work() {
    let (mut flow, receiver, stamp, now, _) = flow(2, vec![Progress::Observed]);
    let win = window();
    flow.observe_at(&frame(&stamp, win, 1), now).unwrap();
    receiver.receive(&completion(&stamp)).unwrap();
    let mut changed = completion(&stamp).stamp;
    changed.document_revision += 1;
    flow.observe_at(&frame(&changed, win, 2), now).unwrap();
    assert_eq!(flow.backend.consumes, 1);
    assert!(matches!(
        receiver
            .receive(&completion(&changed))
            .unwrap()
            .result
            .unwrap()
            .state,
        FrameResultState::Rejected(FrameRejection::DocumentChanged)
    ));
}
#[test]
fn concrete_consumer_is_typed_for_original_quarantine_child_without_starting_a_loop() {
    let request = FrameRequest {
        id: NonZeroU64::new(1).unwrap(),
        origin: stamp(),
        video: "video".into(),
        expires_at: Instant::now() + Duration::from_secs(1),
        frame_budget: NonZeroUsize::new(2).unwrap(),
    };
    let (mut consumer, _) = NoMediaFrameConsumer::new(request);
    let _: &mut dyn ShellFrameConsumer<QuarantineChild> = &mut consumer;
    assert!(consumer.first_error().is_none());
}

#[test]
fn consumed_notice_publication_retry_does_not_revalidate_the_current_next_turn() {
    let (mut flow, receiver, stamp, now, _) = flow(2, vec![Progress::Observed]);
    let mailbox = flow.mailbox.clone();
    let held = mailbox.borrow_mut();
    let notice = frame(&stamp, window(), 1);
    assert!(flow.observe_at(&notice, now).is_err());
    let reads = flow.backend.contexts.get();
    flow.backend.context_failure.set(true); // current document moved to N+1; old N would fail context
    drop(held);
    assert!(flow.observe_at(&notice, now).unwrap());
    assert_eq!(flow.backend.contexts.get(), reads);
    assert_eq!(flow.backend.consumes, 1);
    assert!(matches!(
        receiver
            .receive(&completion(&stamp))
            .unwrap()
            .result
            .unwrap()
            .state,
        FrameResultState::ObservedDisabled { .. }
    ));
}
#[test]
fn new_source_notice_is_not_treated_as_owned_publication_replay() {
    let (mut flow, receiver, stamp, now, owner) = flow(2, vec![Progress::Observed]);
    let mailbox = flow.mailbox.clone();
    let held = mailbox.borrow_mut();
    flow.observe_at(&frame(&stamp, window(), 1), now)
        .unwrap_err();
    drop(held);
    let mut foreign = completion(&stamp).stamp;
    foreign.owner.source_generation += 1;
    flow.observe_at(&frame(&foreign, flow.window.unwrap(), 2), now)
        .unwrap();
    assert_eq!(flow.backend.consumes, 1);
    assert!(owner.upgrade().is_some());
    assert!(matches!(
        flow.first_error,
        Some(FrameFailure::Rejected(FrameRejection::SourceChanged))
    ));
    assert!(matches!(
        receiver
            .receive(&completion(&stamp))
            .unwrap()
            .result
            .unwrap()
            .state,
        FrameResultState::Rejected(FrameRejection::SourceChanged)
    ));
}
#[test]
fn deadline_crossed_during_advance_skips_proof_and_late_verify_is_not_observed() {
    let start = Instant::now();
    let expires = start + Duration::from_secs(1);
    let clock = Cell::new(start);
    let proofs = Cell::new(0);
    let result = consume_checked(
        expires,
        &|| clock.get(),
        || {
            clock.set(expires);
            Ok(ConstructionStatus::Constructed)
        },
        || {
            proofs.set(proofs.get() + 1);
            Ok(())
        },
    );
    assert!(matches!(
        result,
        Err(FrameFailure::Rejected(FrameRejection::Expired))
    ));
    assert_eq!(proofs.get(), 0);
    clock.set(start);
    let result = consume_checked(
        expires,
        &|| clock.get(),
        || Ok(ConstructionStatus::Constructed),
        || {
            proofs.set(proofs.get() + 1);
            clock.set(expires);
            Ok(())
        },
    );
    assert!(matches!(
        result,
        Err(FrameFailure::Rejected(FrameRejection::Expired))
    ));
    assert_eq!(proofs.get(), 1);
}
#[test]
fn deadline_crossed_before_buffered_publication_retains_owner_and_reports_expired() {
    let (mut flow, receiver, stamp, now, owner) = flow(2, vec![Progress::Observed]);
    let mailbox = flow.mailbox.clone();
    let held = mailbox.borrow_mut();
    let notice = frame(&stamp, window(), 1);
    flow.observe_at(&notice, now).unwrap_err();
    assert_eq!(flow.backend.consumes, 1);
    drop(held);
    let expires = flow.request.expires_at;
    flow.observe_at(&notice, expires).unwrap();
    assert_eq!(flow.backend.consumes, 1);
    assert!(owner.upgrade().is_some());
    assert!(matches!(
        receiver
            .receive(&completion(&stamp))
            .unwrap()
            .result
            .unwrap()
            .state,
        FrameResultState::Rejected(FrameRejection::Expired)
    ));
}

#[test]
fn same_turn_completion_moves_data_but_older_turn_and_foreign_request_do_not() {
    let (mut flow, receiver, mut stamp, now, _) = flow(2, vec![Progress::Pending { redraw: true }]);
    stamp.turn_sequence = 5;
    flow.observe_at(&frame(&stamp, window(), 1), now).unwrap();
    let mut turn = completion(&stamp);
    turn.stamp.turn_sequence = 4;
    assert!(matches!(
        receiver.receive_at(&turn, now),
        Err(FrameReceiveError::BeforeObservation)
    ));
    turn.stamp.turn_sequence = 5;
    {
        let mut mailbox = receiver.mailbox.borrow_mut();
        mailbox.pending.as_mut().unwrap().request = NonZeroU64::new(8).unwrap();
    }
    assert!(matches!(
        receiver.receive_at(&turn, now),
        Err(FrameReceiveError::ForeignRequest)
    ));
    receiver
        .mailbox
        .borrow_mut()
        .pending
        .as_mut()
        .unwrap()
        .request = NonZeroU64::new(7).unwrap();
    let update = receiver.receive_at(&turn, now).unwrap();
    assert!(update.redraw);
    assert!(update.result.is_some());
}
#[test]
fn expired_pending_redraw_is_cancelled_but_completed_history_can_be_reflected_late() {
    let (mut flow, receiver, stamp, now, owner) = flow(2, vec![Progress::Pending { redraw: true }]);
    flow.observe_at(&frame(&stamp, window(), 1), now).unwrap();
    let update = receiver
        .receive_at(&completion(&stamp), flow.request.expires_at)
        .unwrap();
    assert!(!update.redraw);
    assert!(matches!(
        update.result.unwrap().state,
        FrameResultState::Rejected(FrameRejection::Expired)
    ));
    assert!(owner.upgrade().is_some());
    assert_eq!(flow.backend.consumes, 1);
    let (mut completed, receiver, stamp, now, _) = self::flow(2, vec![Progress::Observed]);
    completed
        .observe_at(&frame(&stamp, window(), 1), now)
        .unwrap();
    let update = receiver
        .receive_at(
            &completion(&stamp),
            completed.request.expires_at + Duration::from_secs(1),
        )
        .unwrap();
    assert!(matches!(
        update.result.unwrap().state,
        FrameResultState::ObservedDisabled { .. }
    ));
    assert!(!update.redraw);
}
