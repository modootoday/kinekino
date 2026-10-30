//! Original registrar, frame consumption, child ownership and diagnostic observations.
use crate::assembly::meari::quarantine_surface::{
    ConstructionStatus, OriginalInput, QuarantineChild, QuarantineRegistration,
    QuarantineRegistrationFailure, register_inert_failure_for_test, register_no_media,
};
use crate::infrastructure::mpv::ProbeRelease;
use meari_native::{
    application::window_lifecycle::{CleanupStatus, WindowLifecycle},
    infrastructure::{
        native_host::{HostStamp, HostTurn, HostTurnReason, NativeDocumentHandle},
        window::{
            NativeSurfaceRegistrar, NativeSurfaceRegistrationFailure, NativeWindowNotice,
            NativeWindowNoticeKind, NativeWindowWake,
        },
    },
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Instant,
};

#[derive(Default)]
pub(crate) struct Evidence {
    pub(super) inert_partial: bool,
    pub(super) partial_retained: bool,
    pub(super) partial_released: bool,
    pub(super) disabled_read: bool,
    pub(super) expired_cleanup_rejected: bool,
    pub(super) remaining_zero: bool,
    pub(super) consumed_frame: Option<HostStamp>,
    pub(super) reflected_turn: Option<HostStamp>,
    pub(super) reflected_reason: Option<HostTurnReason>,
    pub(super) reflection_wake: Option<bool>,
    pub(super) failure: Option<String>,
    pub(super) close_seen: bool,
}
impl Evidence {
    pub(super) fn reflect(&mut self, turn: &HostTurn) -> Result<(), String> {
        if self.reflected_turn.is_some() {
            return Ok(());
        }
        let frame = self
            .consumed_frame
            .as_ref()
            .ok_or("missing consumed original frame")?;
        check(
            matches_reflection(frame, turn, self.reflection_wake),
            "expected original own-wake Completion at exact next full stamp; intervening turn is not accepted",
        )?;
        self.reflected_turn = Some(turn.stamp.clone());
        self.reflected_reason = Some(turn.reason);
        Ok(())
    }
}
fn matches_reflection(frame: &HostStamp, turn: &HostTurn, wake: Option<bool>) -> bool {
    wake.is_some()
        && turn.reason == HostTurnReason::Completion
        && frame.host_id == turn.stamp.host_id
        && frame.owner == turn.stamp.owner
        && frame.definition_revision == turn.stamp.definition_revision
        && frame.document_revision == turn.stamp.document_revision
        && frame.turn_sequence.checked_add(1) == Some(turn.stamp.turn_sequence)
}
pub(super) enum Stage {
    Register,
    FreshFrame,
    Finished,
}
pub(crate) struct Fixture {
    pub(super) document: NativeDocumentHandle,
    pub(super) video: String,
    pub(super) children: Rc<WindowLifecycle<QuarantineChild>>,
    pub(super) surfaces: Option<NativeSurfaceRegistrar<QuarantineChild>>,
    pub(super) registration: Option<QuarantineRegistration>,
    pub(super) registration_failure: Option<Box<QuarantineRegistrationFailure>>,
    pub(super) wake: Option<NativeWindowWake>,
    pub(super) stage: Stage,
    pub(super) redraw: bool,
    pub(super) deadline: Instant,
    pub(super) evidence: Rc<RefCell<Evidence>>,
}
fn check(condition: bool, message: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}
pub(super) fn explain(error: impl std::fmt::Debug) -> String {
    format!("{error:?}")
}

impl Fixture {
    pub(super) fn submitted(&mut self, notice: NativeWindowNotice) -> Result<(), String> {
        if matches!(self.stage, Stage::Finished)
            || notice.kind != NativeWindowNoticeKind::FrameSubmitted
        {
            return Ok(());
        }
        check(
            Instant::now() < self.deadline,
            "own-window deadline expired before frame consumption",
        )?;
        let frame = notice
            .frame
            .ok_or("FrameSubmitted missing original frame")?;
        check(
            frame.document == self.document.stamp().map_err(explain)?,
            "delayed frame must not authorize a new turn",
        )?;
        check(
            notice.window == Some(self.children.stamp().map_err(explain)?),
            "foreign window frame",
        )?;
        match self.stage {
            Stage::Register => self.register_inert_and_request_frame(),
            Stage::FreshFrame => self.consume_and_release(frame.document),
            Stage::Finished => Ok(()),
        }
    }

    fn register_inert_and_request_frame(&mut self) -> Result<(), String> {
        let surfaces = self.surfaces.take().ok_or("missing original registrar")?;
        let reservation = surfaces
            .reserve_video(&self.document.stamp().map_err(explain)?, &self.video)
            .map_err(explain)?;
        let failed = match register_inert_failure_for_test(surfaces, reservation) {
            Err(failure) => failure,
            Ok(registration) => {
                self.registration = Some(registration);
                return Err("inert fault did not fail actual registration".into());
            }
        };
        let QuarantineRegistrationFailure {
            registrar,
            failure,
            checkpoint,
        } = *failed;
        self.surfaces = Some(registrar);
        let NativeSurfaceRegistrationFailure::Child(failure) = failure else {
            return Err("fault bypassed actual child registration".into());
        };
        let lease = failure
            .cleanup
            .ok_or("actual partial failure lost cleanup lease")?;
        let checkpoint = checkpoint.ok_or("partial failure lost checkpoint")?;
        {
            let owner = checkpoint
                .owner
                .upgrade()
                .ok_or("coordinator discarded partial child")?;
            let state = owner.borrow();
            let probe = state
                .probe
                .as_ref()
                .ok_or("inert failure lost original OS owner")?;
            check(
                matches!(
                    probe.observe_quarantine_release().map_err(explain)?,
                    ProbeRelease::NotCreated
                ),
                "factory created an OS child",
            )?;
            let mut evidence = self.evidence.borrow_mut();
            evidence.inert_partial = true;
            evidence.partial_retained = self.children.report().map_err(explain)?.remaining == 1;
        }
        check(
            self.children.release_child(&lease).map_err(explain)?.status == CleanupStatus::Complete,
            "inert partial cleanup did not complete",
        )?;
        check(
            checkpoint.owner.upgrade().is_none(),
            "partial owner escaped original coordinator cleanup",
        )?;
        self.evidence.borrow_mut().partial_released = true;
        let surfaces = self
            .surfaces
            .take()
            .ok_or("partial failure lost original registrar")?;
        let reservation = surfaces
            .reserve_video(&self.document.stamp().map_err(explain)?, &self.video)
            .map_err(explain)?;
        let registration = match register_no_media(surfaces, reservation) {
            Ok(registration) => registration,
            Err(failure) => {
                let error = explain(&failure.failure);
                self.registration_failure = Some(failure);
                return Err(error);
            }
        };
        let status = registration.checkpoint.advance().map_err(explain);
        self.registration = Some(registration);
        check(
            matches!(
                status?,
                ConstructionStatus::Pending {
                    request_redraw: true
                }
            ),
            "new reservation must require its own fresh frame",
        )?;
        self.stage = Stage::FreshFrame;
        self.redraw = true;
        // Intent only; the future host turn applies redraw after this observer/borrow returns.
        self.wake
            .as_ref()
            .ok_or("missing own wake")?
            .wake()
            .map_err(explain)?;
        Ok(())
    }

    fn consume_and_release(&mut self, frame: HostStamp) -> Result<(), String> {
        let registration = self
            .registration
            .as_ref()
            .ok_or("missing retained registration")?;
        check(
            matches!(
                registration.checkpoint.advance().map_err(explain)?,
                ConstructionStatus::Constructed
            ),
            "submitted current frame did not construct hidden child",
        )?;
        registration
            .checkpoint
            .with_disabled_child(|proof| proof.verify())
            .map_err(explain)?
            .map_err(explain)?;
        self.evidence.borrow_mut().disabled_read = true;
        // Keep a diagnostic borrow of the SAME state across coordinator removal, never a child/lease clone.
        let owner = registration
            .checkpoint
            .owner
            .upgrade()
            .ok_or("original child owner missing")?;
        check(
            self.children
                .release_child(&registration.lease)
                .map_err(explain)?
                .status
                == CleanupStatus::Complete,
            "real hidden child cleanup not acknowledged",
        )?;
        {
            let state = owner.borrow();
            let request = state.request.ok_or("original cleanup request missing")?;
            let OriginalInput::Ready(input) = &state.input else {
                return Err("registered input changed identity".into());
            };
            let called = Cell::new(false);
            let rejected = input.with_cleanup_parent(request, |_| called.set(true));
            check(
                rejected.is_err() && !called.get(),
                "expired original request entered cleanup callback",
            )?;
            // This is expired authority rejection, not evidence of pre-revocation admission preservation.
            self.evidence.borrow_mut().expired_cleanup_rejected = true;
        }
        drop(owner);
        check(
            registration.checkpoint.owner.upgrade().is_none(),
            "OS owner retained after diagnostic borrow ended",
        )?;
        check(
            self.children.report().map_err(explain)?.remaining == 0,
            "original coordinator still owns children",
        )?;
        {
            let mut evidence = self.evidence.borrow_mut();
            evidence.remaining_zero = true;
            evidence.consumed_frame = Some(frame);
        }
        self.registration.take();
        self.stage = Stage::Finished;
        // No proof/snapshot is retained. Completion may reflect these owned observations only.
        let enqueued = self
            .wake
            .as_ref()
            .ok_or("missing own wake")?
            .wake()
            .map_err(explain)?;
        self.evidence.borrow_mut().reflection_wake = Some(enqueued);
        Ok(())
    }
}

#[test]
fn reflection_requires_own_wake_completion_and_exact_full_next_stamp() {
    use meari_native::{domain::Viewport, infrastructure::document::InteractiveDocument};
    let source = meari::parse(
        "type Props={};export function App(props:Props){return <main/>;}",
        meari::application::Language::Tsx,
    )
    .unwrap();
    let document = NativeDocumentHandle::new(
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
    .unwrap();
    let frame = document.stamp().unwrap();
    let mut next = frame.clone();
    next.turn_sequence = frame.turn_sequence.checked_add(1).unwrap();
    let good = HostTurn {
        stamp: next,
        reason: HostTurnReason::Completion,
        accepted_callbacks: 0,
        rejected_callbacks: 0,
        remaining_callbacks: 0,
    };
    assert!(matches_reflection(&frame, &good, Some(true)));
    assert!(matches_reflection(&frame, &good, Some(false))); // coalesced original wake, not a missing request
    assert!(!matches_reflection(&frame, &good, None));
    for reason in [
        HostTurnReason::Input,
        HostTurnReason::Accessibility,
        HostTurnReason::ResumePending,
    ] {
        assert!(!matches_reflection(
            &frame,
            &HostTurn {
                reason,
                ..good.clone()
            },
            Some(true)
        ));
    }
    let mut alternatives = Vec::new();
    let mut wrong = good.stamp.clone();
    wrong.host_id += 1;
    alternatives.push(wrong);
    let mut wrong = good.stamp.clone();
    wrong.owner.source_generation += 1;
    alternatives.push(wrong);
    let mut wrong = good.stamp.clone();
    wrong.definition_revision += 1;
    alternatives.push(wrong);
    let mut wrong = good.stamp.clone();
    wrong.document_revision += 1;
    alternatives.push(wrong);
    let mut wrong = good.stamp.clone();
    wrong.turn_sequence += 1;
    alternatives.push(wrong);
    for stamp in alternatives {
        assert!(!matches_reflection(
            &frame,
            &HostTurn {
                stamp,
                ..good.clone()
            },
            Some(true)
        ));
    }
}
