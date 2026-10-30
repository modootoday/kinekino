//! Scoped, bounded test observations. Instrumentation never owns a shutdown executor.
pub(crate) mod live_retained;
use super::quarantine_surface::frame_delivery::{FrameOwnedResult, FrameResultState};
use meari_native::{
    application::window_lifecycle::{CleanupReport, WindowStamp},
    infrastructure::native_host::{HostEffects, HostStamp, HostTurn},
};
use serde::Serialize;
use std::{
    cell::{Cell, RefCell},
    io::{self, Write},
    rc::Rc,
};

// The fixture can reflect 64 budgeted frames plus bind/proof/close/return events.
const EVENT_LIMIT: usize = 80;
const PAYLOAD_LIMIT: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) enum Kind {
    Bind,
    ProofObserved,
    Reflection,
    CloseIntent,
    InnerClose,
    RunReturned,
    CleanupReturned,
    JobsInitialFrame,
    JobsEntered,
    JobsReturnCommitted,
    JobsRetainedFrame,
    JobsOwnersReleased,
    JobsRestored,
    JobsFault,
    LiveInitialFrame,
    LiveEntryCommitted,
    LiveEntryFrame,
    LiveActivated,
    LiveInputDisplayed,
    LiveInputRestored,
    LiveSaveRequested,
    LiveSaveObservationWake,
    LiveSaveSettled,
    LiveHostReturnRequested,
    LiveSourceNavigationAction,
    LiveSourceNavigationRedraw,
    LiveSourceNavigationAdmitted,
    LiveSourceReturnRequested,
    LiveReturnCommitted,
    LiveViewingFrame,
    LiveOwnersReleased,
    LiveArmBusy,
    LiveArmRecovered,
    LiveArmRetryWake,
    LiveWakeCounts,
    LiveWakeError,
    LiveModelProofError,
    LiveModelRouteCancelled,
    LiveFault,
    ChooserSourceAction,
    ChooserGeneration,
    ChooserJourneyComplete,
    ChooserActiveIdentity,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Fault {
    Reentrant,
    Capacity,
    Sequence,
    Payload,
    Observation,
}
#[derive(Clone, Copy)]
struct Payload {
    bytes: [u8; PAYLOAD_LIMIT],
    len: usize,
}
impl Default for Payload {
    fn default() -> Self {
        Self {
            bytes: [0; PAYLOAD_LIMIT],
            len: 0,
        }
    }
}
impl Write for Payload {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let end = self
            .len
            .checked_add(bytes.len())
            .filter(|end| *end <= PAYLOAD_LIMIT)
            .ok_or_else(|| io::Error::other("probe payload capacity"))?;
        self.bytes[self.len..end].copy_from_slice(bytes);
        self.len = end;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[derive(Clone, Copy)]
pub(crate) struct Event {
    pub sequence: u64,
    pub kind: Kind,
    payload: Payload,
}
impl Event {
    pub(crate) fn payload(&self) -> &[u8] {
        &self.payload.bytes[..self.payload.len]
    }
}
struct Trace {
    events: [Option<Event>; EVENT_LIMIT],
    len: usize,
    sequence: u64,
}
impl Default for Trace {
    fn default() -> Self {
        Self {
            events: [None; EVENT_LIMIT],
            len: 0,
            sequence: 0,
        }
    }
}
type CloseRequest = Box<dyn Fn() -> Result<bool, &'static str>>;
type ChildrenReport = Box<dyn Fn() -> Option<(CleanupReport, WindowStamp)>>;
#[derive(Clone, Copy, PartialEq, Eq)]
enum JobsPhase {
    Initial,
    Jobs,
    Returning,
    Release,
    Complete,
}
struct JobsTrace {
    phase: JobsPhase,
    list_completed: std::sync::Arc<std::sync::atomic::AtomicBool>,
    proof: Option<(
        meari_native::infrastructure::native_host::NativeDocumentHandle,
        meari_native::infrastructure::native_host::RetainedCommitProof,
    )>,
    released: bool,
}
#[derive(Default)]
struct Checkpoint {
    enabled: bool,
    lines: usize,
    notices: usize,
    error: Option<io::Error>,
    failed: bool,
}
pub(crate) struct Probe {
    checkpoint: RefCell<Checkpoint>,
    trace: RefCell<Trace>,
    fault: Cell<Option<Fault>>,
    close: RefCell<Option<CloseRequest>>,
    children: RefCell<Option<ChildrenReport>>,
    close_requested: Cell<bool>,
    inner_close_seen: Cell<bool>,
    jobs: RefCell<Option<JobsTrace>>,
    live: RefCell<Option<live_retained::LiveTrace>>,
    initial_reflected: Cell<bool>,
}
impl Probe {
    pub(crate) fn enable_checkpoints(&self) {
        self.checkpoint.borrow_mut().enabled = true;
    }
    pub(crate) fn take_checkpoint_error(&self) -> Option<io::Error> {
        self.checkpoint.borrow_mut().error.take()
    }
    pub(crate) fn checkpoint(&self, message: &str) {
        let mut checkpoint = self.checkpoint.borrow_mut();
        if !checkpoint.enabled || checkpoint.failed || checkpoint.lines == 96 {
            return;
        }
        let line = format!("live-checkpoint {} {message}\n", checkpoint.lines + 1);
        if line.len() > 512 {
            checkpoint.failed = true;
            checkpoint.error = Some(io::Error::other("checkpoint line exceeds 512 bytes"));
            return;
        }
        checkpoint.lines += 1;
        // Bounded bytes do not guarantee hard preemption of synchronous diagnostic I/O.
        if let Err(error) = io::stderr().lock().write_all(line.as_bytes()) {
            checkpoint.failed = true;
            checkpoint.error = Some(error);
        }
    }
    fn checkpoint_notice_allowed(&self) -> bool {
        let mut checkpoint = self.checkpoint.borrow_mut();
        if !checkpoint.enabled || checkpoint.failed || checkpoint.notices == 8 {
            return false;
        }
        checkpoint.notices += 1;
        true
    }
    pub(crate) fn new() -> Rc<Self> {
        Rc::new(Self {
            checkpoint: RefCell::new(Checkpoint::default()),
            trace: RefCell::new(Trace::default()),
            fault: Cell::new(None),
            close: RefCell::new(None),
            children: RefCell::new(None),
            close_requested: Cell::new(false),
            inner_close_seen: Cell::new(false),
            jobs: RefCell::new(None),
            live: RefCell::new(None),
            initial_reflected: Cell::new(false),
        })
    }
    pub(crate) fn for_jobs(
        list_completed: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Rc<Self> {
        let probe = Self::new();
        *probe.jobs.borrow_mut() = Some(JobsTrace {
            phase: JobsPhase::Initial,
            list_completed,
            proof: None,
            released: false,
        });
        probe
    }
    pub(crate) fn jobs_commit(
        &self,
        document: &meari_native::infrastructure::native_host::NativeDocumentHandle,
        proof: &meari_native::infrastructure::native_host::RetainedCommitProof,
    ) {
        let Ok(mut jobs) = self.jobs.try_borrow_mut() else {
            self.fail(Fault::Reentrant);
            return;
        };
        let Some(jobs) = jobs.as_mut() else {
            return;
        };
        if jobs.phase != JobsPhase::Returning || jobs.proof.is_some() {
            self.fail(Fault::Observation);
            return;
        }
        jobs.proof = Some((document.clone(), proof.clone()));
        self.record(
            Kind::JobsReturnCommitted,
            &(
                stamp(proof.receipt().before()),
                stamp(proof.receipt().after()),
            ),
        );
    }
    pub(crate) fn jobs_owners_released(&self) {
        let Ok(mut jobs) = self.jobs.try_borrow_mut() else {
            self.fail(Fault::Reentrant);
            return;
        };
        let Some(jobs) = jobs.as_mut() else {
            return;
        };
        if jobs.phase != JobsPhase::Release || jobs.released {
            self.fail(Fault::Observation);
            return;
        }
        jobs.released = true;
        self.record(Kind::JobsOwnersReleased, &true);
    }
    pub(crate) fn jobs_notice(
        &self,
        document: &meari_native::infrastructure::native_host::NativeDocumentHandle,
        notice: &meari_native::infrastructure::window::NativeWindowNotice,
        observed: Option<&str>,
    ) -> bool {
        match self.jobs.try_borrow() {
            Ok(jobs) if jobs.is_none() => return false,
            Err(_) => {
                self.fail(Fault::Reentrant);
                return false;
            }
            _ => (),
        }
        if self.fault().is_some() {
            self.request_close();
            return false;
        }
        let result = (|| -> Result<(bool, bool), String> {
            let mut jobs = self
                .jobs
                .try_borrow_mut()
                .map_err(|_| "jobs probe borrowed")?;
            let jobs = jobs.as_mut().ok_or("jobs probe unavailable")?;
            let Some(frame) = notice.frame.as_ref() else {
                return Ok((false, false));
            };
            let Some(submitted) = frame.retained.as_ref() else {
                return Ok((false, false));
            };
            if submitted.stamp() != &frame.document
                || submitted.ticket() != &frame.submission.ticket
            {
                return Err("proof/notice mismatch".into());
            }
            match jobs.phase {
                JobsPhase::Initial
                    if observed == Some("view.start") && self.initial_reflected.get() =>
                {
                    activate(document, "onOpenJobs", None)?;
                    jobs.phase = JobsPhase::Jobs;
                    self.record(
                        Kind::JobsInitialFrame,
                        &(stamp(&frame.document), &frame.submission.ticket),
                    );
                    Ok((true, false))
                }
                JobsPhase::Jobs
                    if observed == Some("studio.jobs")
                        && jobs
                            .list_completed
                            .load(std::sync::atomic::Ordering::Acquire)
                        && contains_text(
                            document,
                            crate::presentation::web_ui_adapter::studio::jobs::labels::EMPTY,
                        )? =>
                {
                    self.record(
                        Kind::JobsEntered,
                        &(stamp(&frame.document), &frame.submission.ticket, true),
                    );
                    activate(document, "onNavigate", Some("시청으로 돌아가기"))?;
                    jobs.phase = JobsPhase::Returning;
                    Ok((true, false))
                }
                JobsPhase::Returning if observed == Some("view.start") => {
                    let (owner, proof) =
                        jobs.proof.as_ref().ok_or("original commit proof missing")?;
                    owner
                        .validate_retained_frame(proof, submitted)
                        .map_err(|error| format!("{error:?}"))?;
                    self.record(
                        Kind::JobsRetainedFrame,
                        &(
                            stamp(proof.receipt().after()),
                            stamp(submitted.stamp()),
                            submitted.ticket(),
                            true,
                        ),
                    );
                    jobs.phase = JobsPhase::Release;
                    Ok((false, false))
                }
                JobsPhase::Release
                    if jobs.released
                        && observed == Some("view.start")
                        && contains_text(document, "작업 대기열 열기")? =>
                {
                    self.record(
                        Kind::JobsRestored,
                        &(stamp(&frame.document), &frame.submission.ticket),
                    );
                    jobs.phase = JobsPhase::Complete;
                    Ok((false, true))
                }
                _ => Ok((false, false)),
            }
        })();
        match result {
            Ok((notify, close)) => {
                if close {
                    self.request_close();
                }
                notify
            }
            Err(error) => {
                self.record(Kind::JobsFault, &error);
                self.fail(Fault::Observation);
                self.request_close();
                false
            }
        }
    }
    pub(crate) fn fail(&self, fault: Fault) {
        if self.fault.get().is_none() {
            self.fault.set(Some(fault));
        }
    }
    pub(crate) fn fault(&self) -> Option<Fault> {
        self.fault.get()
    }
    pub(crate) fn record(&self, kind: Kind, value: &impl Serialize) {
        // Failures stay sticky, but later cleanup events can still be collected.
        let mut payload = Payload::default();
        if serde_json::to_writer(&mut payload, value).is_err() {
            self.fail(Fault::Payload);
            return;
        }
        let Ok(mut trace) = self.trace.try_borrow_mut() else {
            self.fail(Fault::Reentrant);
            return;
        };
        if trace.len == EVENT_LIMIT {
            self.fail(Fault::Capacity);
            return;
        }
        let Some(sequence) = trace.sequence.checked_add(1) else {
            self.fail(Fault::Sequence);
            return;
        };
        let index = trace.len;
        trace.events[index] = Some(Event {
            sequence,
            kind,
            payload,
        });
        trace.len += 1;
        trace.sequence = sequence;
        drop(trace);
        self.checkpoint(&format!("event sequence={sequence} kind={kind:?}"));
    }
    pub(crate) fn inspect<T>(&self, visit: impl FnOnce(&[Option<Event>]) -> T) -> Option<T> {
        match self.trace.try_borrow() {
            Ok(trace) => Some(visit(&trace.events[..trace.len])),
            Err(_) => {
                self.fail(Fault::Reentrant);
                None
            }
        }
    }
    pub(crate) fn attach(
        &self,
        close: impl Fn() -> Result<bool, &'static str> + 'static,
        children: impl Fn() -> Option<(CleanupReport, WindowStamp)> + 'static,
    ) {
        match (self.close.try_borrow_mut(), self.children.try_borrow_mut()) {
            (Ok(mut close_slot), Ok(mut children_slot)) => {
                *close_slot = Some(Box::new(close));
                *children_slot = Some(Box::new(children));
            }
            _ => self.fail(Fault::Reentrant),
        }
    }
    fn request_close(&self) {
        if self.close_requested.replace(true) {
            return;
        }
        let result = match self.close.try_borrow() {
            Ok(close) => close
                .as_ref()
                .map_or(Err("close unavailable"), |close| close()),
            Err(_) => Err("close borrowed"),
        };
        if result.is_err() {
            self.fail(Fault::Observation);
        }
        self.record(Kind::CloseIntent, &result);
        self.checkpoint(&format!("close result={result:?}"));
    }
}

#[derive(Serialize)]
pub(crate) struct Stamp<'a> {
    host: u64,
    owner: &'a meari::domain::plugins::PluginScope,
    definition: u64,
    document: u64,
    turn: u64,
}
pub(crate) fn stamp(value: &HostStamp) -> Stamp<'_> {
    Stamp {
        host: value.host_id,
        owner: &value.owner,
        definition: value.definition_revision,
        document: value.document_revision,
        turn: value.turn_sequence,
    }
}
pub(crate) fn reflection(
    turn: &HostTurn,
    effects: &HostEffects,
    current_stamp: Option<&HostStamp>,
    result: &FrameOwnedResult,
) -> bool {
    let Some(probe) = current() else {
        return false;
    };
    let observed = matches!(result.state, FrameResultState::ObservedDisabled { .. });
    let (state, frame) = match &result.state {
        FrameResultState::Pending => ("pending", None),
        FrameResultState::ObservedDisabled { frame } => ("observed_disabled", Some(frame)),
        FrameResultState::Rejected(_) => ("rejected", None),
    };
    probe.record(
        Kind::Reflection,
        &(
            result.request.get(),
            stamp(&turn.stamp),
            format!("{:?}", turn.reason),
            turn.remaining_callbacks,
            stamp(&result.observed),
            result
                .window
                .map(|window| (window.owner(), window.epoch(), window.surface_generation())),
            state,
            frame,
            effects
                .committed
                .as_ref()
                .map(|receipt| (stamp(receipt.before()), stamp(receipt.after()))),
            current_stamp.map(stamp),
        ),
    );
    // The caller has finished SDK/document/receiver borrows. Trace recording has ended too.
    let jobs_mode = match probe.jobs.try_borrow() {
        Ok(jobs) => jobs.is_some(),
        Err(_) => {
            probe.fail(Fault::Reentrant);
            return false;
        }
    };
    let live_mode = probe.is_live();
    if observed && !jobs_mode && !live_mode {
        probe.request_close();
    }
    // A later real redraw gives the probe a safe post-turn action point. Never
    // dispatch input here while the native runtime still verifies this receipt.
    observed && (jobs_mode || live_mode) && !probe.initial_reflected.replace(true)
}

fn contains_text(
    document: &meari_native::infrastructure::native_host::NativeDocumentHandle,
    text: &str,
) -> Result<bool, String> {
    document
        .try_with(|document| {
            document.elements().iter().any(|element| {
                element
                    .text_override
                    .as_deref()
                    .is_some_and(|value| value.contains(text))
                    || element.text_runs.iter().any(|run| run.text.contains(text))
            })
        })
        .map_err(|error| format!("{error:?}"))
}
fn activate(
    document: &meari_native::infrastructure::native_host::NativeDocumentHandle,
    binding: &str,
    label: Option<&str>,
) -> Result<(), String> {
    let path = document
        .try_with(|document| {
            document.module_projection().map(|projection| {
                projection
                    .callback_bindings()
                    .into_iter()
                    .find(|(path, callback)| {
                        callback.reference.binding == binding
                            && label.is_none_or(|label| {
                                document.elements().iter().any(|element| {
                                    element.owns_path(path)
                                        && element
                                            .attributes
                                            .get("aria-label")
                                            .is_some_and(|value| value == label)
                                })
                            })
                    })
                    .map(|(path, _)| path)
            })
        })
        .map_err(|error| format!("{error:?}"))??
        .ok_or("current source callback missing")?;
    if !document
        .dispatch_action(meari_native::domain::widgets::Action::Activate { path })
        .map_err(|error| format!("{error:?}"))?
    {
        return Err("current source action was not activated".into());
    }
    Ok(())
}
pub(crate) fn inner_close() {
    let Some(probe) = current() else {
        return;
    };
    if probe.inner_close_seen.replace(true) {
        return;
    }
    let report = probe
        .children
        .try_borrow()
        .ok()
        .and_then(|reader| reader.as_ref().and_then(|read| read()));
    if report.is_none() {
        probe.fail(Fault::Observation);
    }
    probe.record(
        Kind::InnerClose,
        &report.map(|(report, window)| {
            (
                format!("{:?}", report.status),
                report.remaining,
                report.failed,
                report.timed_out,
                report.unavailable,
                (window.owner(), window.epoch(), window.surface_generation()),
            )
        }),
    );
}
thread_local! {
    static ACTIVE: RefCell<Option<Rc<Probe>>> = const { RefCell::new(None) };
}
pub(crate) struct Scope {
    previous: Option<Rc<Probe>>,
}
impl Scope {
    pub(crate) fn enter(probe: Rc<Probe>) -> Self {
        let previous = ACTIVE.with(|active| active.replace(Some(probe)));
        Self { previous }
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        ACTIVE.with(|active| *active.borrow_mut() = self.previous.take());
    }
}
pub(crate) fn current() -> Option<Rc<Probe>> {
    // Clone and end the TLS borrow before invoking observation code.
    ACTIVE.with(|active| active.borrow().clone())
}

#[test]
fn trace_records_order_and_keeps_payloads_independent() {
    let probe = Probe::new();
    probe.record(Kind::Bind, &17_u64);
    probe.record(Kind::ProofObserved, &18_u64);
    probe
        .inspect(|events| {
            let first = events[0].as_ref().unwrap();
            let second = events[1].as_ref().unwrap();
            assert_eq!((first.sequence, second.sequence), (1, 2));
            assert_eq!(first.payload(), b"17");
            assert_eq!(second.payload(), b"18");
            assert_eq!(second.kind, Kind::ProofObserved);
        })
        .unwrap();
    assert_eq!(probe.fault(), None);
}
#[test]
fn overflow_does_not_replace_prior_events_or_grow_storage() {
    let probe = Probe::new();
    for index in 0..=EVENT_LIMIT {
        probe.record(Kind::Bind, &index);
    }
    assert_eq!(probe.fault(), Some(Fault::Capacity));
    probe
        .inspect(|events| {
            assert_eq!(events.len(), EVENT_LIMIT);
            assert_eq!(
                events.last().unwrap().as_ref().unwrap().sequence,
                EVENT_LIMIT as u64
            );
        })
        .unwrap();
}
#[test]
fn budgeted_reflections_keep_space_for_original_close_and_cleanup_evidence() {
    let probe = Probe::new();
    probe.record(Kind::Bind, &true);
    for frame in 0..63_u64 {
        probe.record(Kind::Reflection, &frame);
    }
    probe.record(Kind::ProofObserved, &64_u64);
    probe.record(Kind::Reflection, &64_u64);
    probe.record(Kind::CloseIntent, &Ok::<bool, &str>(false));
    probe.record(Kind::InnerClose, &0_usize);
    probe.record(Kind::RunReturned, &true);
    probe.record(Kind::CleanupReturned, &true);
    assert_eq!(probe.fault(), None);
    probe
        .inspect(|events| {
            assert_eq!(events.len(), 70);
            assert_eq!(
                events.last().unwrap().as_ref().unwrap().kind,
                Kind::CleanupReturned
            );
            assert!(
                events
                    .iter()
                    .flatten()
                    .map(|event| event.sequence)
                    .eq(1..=70)
            );
        })
        .unwrap();
}
#[test]
fn sequence_overflow_is_explicit_and_does_not_append_a_wrapped_event() {
    let probe = Probe::new();
    probe.trace.borrow_mut().sequence = u64::MAX;
    probe.record(Kind::Bind, &0_u8);
    assert_eq!(probe.fault(), Some(Fault::Sequence));
    probe.inspect(|events| assert!(events.is_empty())).unwrap();
}
#[test]
fn reentrant_and_oversize_observation_fail_without_blocking_cleanup_collection() {
    let probe = Probe::new();
    let borrowed = probe.trace.borrow_mut();
    probe.record(Kind::Bind, &0_u8);
    drop(borrowed);
    assert_eq!(probe.fault(), Some(Fault::Reentrant));
    probe.record(Kind::InnerClose, &0_usize);
    probe.inspect(|events| assert_eq!(events.len(), 1)).unwrap();
    let oversized = Probe::new();
    oversized.record(Kind::Bind, &"x".repeat(PAYLOAD_LIMIT + 1));
    assert_eq!(oversized.fault(), Some(Fault::Payload));
    oversized.record(Kind::InnerClose, &0_usize);
    oversized
        .inspect(|events| assert_eq!(events.len(), 1))
        .unwrap();
}
#[test]
fn nested_scope_restores_previous_probe_and_releases_scope_ownership() {
    let first = Probe::new();
    let second = Probe::new();
    let weak = Rc::downgrade(&second);
    {
        let _first = Scope::enter(first.clone());
        {
            let _second = Scope::enter(second.clone());
            assert!(Rc::ptr_eq(&current().unwrap(), &second));
        }
        assert!(Rc::ptr_eq(&current().unwrap(), &first));
    }
    drop(second);
    assert!(weak.upgrade().is_none());
    assert!(current().is_none());
}

fn reflected_policy_fixture() -> (HostTurn, FrameOwnedResult) {
    let source = meari::parse(
        "export function Root(){return <main/>}",
        meari::application::Language::Tsx,
    )
    .unwrap();
    let document = meari_native::infrastructure::native_host::NativeDocumentHandle::new(
        meari_native::infrastructure::document::InteractiveDocument::new(
            source,
            "",
            meari_native::domain::Viewport {
                width: 160,
                height: 80,
                dark: false,
            },
        )
        .unwrap(),
        1,
    )
    .unwrap();
    let current = document.stamp().unwrap();
    let turn = HostTurn {
        stamp: current.clone(),
        reason: meari_native::infrastructure::native_host::HostTurnReason::Completion,
        accepted_callbacks: 0,
        rejected_callbacks: 0,
        remaining_callbacks: 0,
    };
    // Instrumentation policy DTO only, never a retained checked submission proof.
    let result = FrameOwnedResult {
        request: std::num::NonZeroU64::new(92).unwrap(),
        observed: current,
        window: None,
        state: FrameResultState::ObservedDisabled {
            frame: meari_native::domain::graphics::FrameTicket {
                context_id: "probe-policy-only".into(),
                context_generation: 1,
                scene_generation: 1,
            },
        },
    };
    (turn, result)
}
#[test]
fn jobs_mode_requests_one_safe_redraw_after_reflection_without_early_close() {
    let (turn, result) = reflected_policy_fixture();
    let probe = Probe::for_jobs(std::sync::Arc::new(std::sync::atomic::AtomicBool::new(
        false,
    )));
    let closes = Rc::new(Cell::new(0));
    let observed = closes.clone();
    probe.attach(
        move || {
            observed.set(observed.get() + 1);
            Ok(false)
        },
        || None,
    );
    let _scope = Scope::enter(probe.clone());
    assert!(reflection(
        &turn,
        &HostEffects::default(),
        Some(&turn.stamp),
        &result
    ));
    assert!(!reflection(
        &turn,
        &HostEffects::default(),
        Some(&turn.stamp),
        &result
    ));
    assert_eq!(closes.get(), 0);
    assert_eq!(probe.fault(), None);
}
#[test]
fn original_probe_mode_still_closes_once_on_observed_reflection() {
    let (turn, result) = reflected_policy_fixture();
    let probe = Probe::new();
    let closes = Rc::new(Cell::new(0));
    let observed = closes.clone();
    probe.attach(
        move || {
            observed.set(observed.get() + 1);
            Ok(false)
        },
        || None,
    );
    let _scope = Scope::enter(probe.clone());
    assert!(!reflection(
        &turn,
        &HostEffects::default(),
        Some(&turn.stamp),
        &result
    ));
    assert!(!reflection(
        &turn,
        &HostEffects::default(),
        Some(&turn.stamp),
        &result
    ));
    assert_eq!(closes.get(), 1);
    assert_eq!(probe.fault(), None);
}
