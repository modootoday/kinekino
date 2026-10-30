use super::{Failure, Pin, Stage};
use kinekino_process::{
    application::supervisor::{
        Completion, FinishOutcome, Incomplete, ProcessFamily, ReceiptSink, Supervisor, SystemClock,
    },
    domain::{
        deadline::Deadline,
        identity::ProcessIdentity,
        observation::{FamilyObservation, ProcessReceipt},
    },
    infrastructure::family::{LaunchFailure, OwnedProcessFamily},
};
use serde_json::{Value, json};
use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::Path,
};

pub(super) struct NoForce<P> {
    inner: P,
    denied: usize,
}
impl<P> NoForce<P> {
    pub(super) fn new(inner: P) -> Self {
        Self { inner, denied: 0 }
    }
    pub(super) fn denied_requests(&self) -> usize {
        self.denied
    }
}
impl<P: ProcessFamily> ProcessFamily for NoForce<P> {
    fn deadline(&self) -> Deadline {
        self.inner.deadline()
    }
    fn identity(&self) -> &ProcessIdentity {
        self.inner.identity()
    }
    fn close_stdin(&mut self) -> io::Result<()> {
        self.inner.close_stdin()
    }
    fn poll_child(&mut self) -> io::Result<Option<u32>> {
        self.inner.poll_child()
    }
    fn poll_family(&mut self) -> io::Result<FamilyObservation> {
        self.inner.poll_family()
    }
    fn terminate_owned_family(&mut self) -> io::Result<()> {
        self.denied = self.denied.saturating_add(1);
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Normal-close gate refuses forced termination",
        ))
    }
}
pub(super) type Original = Supervisor<NoForce<OwnedProcessFamily>>;
impl NoForce<OwnedProcessFamily> {
    fn outer_job_observed(&self) -> bool {
        self.inner.outer_job_observed()
    }
}
#[must_use = "Retained holds the original kill-on-close Job; do not discard or end its host process"]
pub(super) enum Retained {
    Launch {
        original: Box<LaunchFailure>,
        pin: Pin,
    },
    Observing {
        supervisor: Box<Original>,
        pin: Pin,
        first: Failure,
        events: Vec<Value>,
        close_attempted: bool,
    },
    Finishing {
        original: Box<Incomplete<NoForce<OwnedProcessFamily>>>,
        pin: Pin,
        events: Vec<Value>,
    },
}
#[must_use = "This is not a transferable live-handle receipt; retain the original outcome"]
pub(super) enum Outcome {
    NotStarted(Failure),
    Closed {
        completion: Completion,
        pin: Pin,
        events: Vec<Value>,
        outer_job_observed: bool,
    },
    Retained(Box<Retained>),
}
pub(super) fn receipt_value(receipt: &ProcessReceipt) -> Value {
    json!({"pid":receipt.identity.pid,"creation_filetime":receipt.identity.creation_filetime,
        "image":receipt.identity.executable,"child_exit":receipt.child_exit,
        "family":format!("{:?}",receipt.family),"force_requested":receipt.forced,
        "stdin_eof_requested":receipt.stdin_eof_requested})
}
impl Outcome {
    pub(super) fn description(&self) -> Value {
        match self {
            Self::NotStarted(error) => {
                json!({"admitted":false,"error":error.evidence()})
            }
            Self::Closed {
                completion,
                pin,
                events,
                outer_job_observed,
            } => json!({"physical_completion":true,
                "normal_success":completion.receipt().child_exit == Some(0) && completion.receipt().family == FamilyObservation::Empty && !completion.receipt().forced,
                "receipt":receipt_value(completion.receipt()),"image_pin":pin.evidence(),"event_count":events.len(),"outer_job_observed":outer_job_observed}),
            Self::Retained(original) => original.description(),
        }
    }
}
impl Retained {
    pub(super) fn description(&self) -> Value {
        match self {
            Self::Launch { original, pin } => json!({"stage":format!("{:?}",Stage::Spawn),
                "launch_stage":original.stage,"error":original.error.to_string(),
                "original_owner_present":original.owner.is_some(),"original_spawn_may_have_requested_termination":original.owner.is_some(),"original_identity":original.owner.as_ref().map(|family|json!({"pid":family.identity().pid,"creation_filetime":family.identity().creation_filetime,"image":family.identity().executable,"launch_state":format!("{:?}",family.state())})),"termination_error":original.termination_error.as_ref().map(ToString::to_string),
                "image_pin":pin.evidence(),"host_lifetime_only":true,"externally_handed_off":false}),
            Self::Observing {
                supervisor,
                pin,
                first,
                events,
                close_attempted,
            } => json!({"stage":format!("{:?}",first.stage),
                "first_error":first.evidence(),"receipt":receipt_value(supervisor.receipt()),
                "denied_force_requests":supervisor.process().denied_requests(),"image_pin":pin.evidence(),"event_count":events.len(),"close_attempted":close_attempted,"outer_job_observed":supervisor.process().outer_job_observed(),
                "host_lifetime_only":true,"externally_handed_off":false}),
            Self::Finishing {
                original,
                pin,
                events,
            } => json!({"stage":format!("{:?}",original.cause.stage),
                "first_error":original.cause.error.to_string(),"receipt":receipt_value(original.owner.receipt()),
                "denied_force_requests":original.owner.process().denied_requests(),"image_pin":pin.evidence(),"event_count":events.len(),"outer_job_observed":original.owner.process().outer_job_observed(),
                "host_lifetime_only":true,"externally_handed_off":false}),
        }
    }
}
struct Sink<'a>(&'a Path);
impl Outcome {
    pub(super) fn physically_closed(&mut self) -> Result<bool, Failure> {
        let state = match self {
            Self::NotStarted(_) | Self::Closed { .. } => return Ok(true),
            Self::Retained(retained) => match retained.as_mut() {
                Retained::Launch { original, .. } => match &mut original.owner {
                    None => return Ok(true),
                    Some(family) => {
                        return Ok(family
                            .poll_child()
                            .map_err(|e| Failure::new(Stage::Identity, e))?
                            .is_some()
                            && family
                                .poll_family()
                                .map_err(|e| Failure::new(Stage::Identity, e))?
                                == FamilyObservation::Empty);
                    }
                },
                Retained::Observing { supervisor, .. } => supervisor.poll_state(),
                Retained::Finishing { original, .. } => original.owner.poll_state(),
            },
        }
        .map_err(|e| Failure::debug(Stage::Identity, e))?;
        Ok(state.child_exit.is_some() && state.family == FamilyObservation::Empty)
    }
    pub(super) fn request_normal_close(&mut self) -> Result<(), Failure> {
        let Self::Retained(retained) = self else {
            return Err(Failure::new(Stage::Close, "No pending original owner"));
        };
        let Retained::Observing {
            supervisor,
            pin,
            close_attempted,
            ..
        } = retained.as_mut()
        else {
            return Err(Failure::new(
                Stage::Close,
                "No unattempted window close for this retained phase",
            ));
        };
        if *close_attempted {
            return Err(Failure::new(
                Stage::Close,
                "Original close already attempted",
            ));
        }
        let receipt = supervisor
            .poll_state()
            .map_err(|e| Failure::debug(Stage::Identity, e))?
            .clone();
        super::window::check_identity(&receipt.identity, &pin.path, receipt.child_exit)?;
        let window = super::window::find(receipt.identity.pid)?
            .ok_or_else(|| Failure::new(Stage::Window, "No original visible root"))?;
        *close_attempted = true;
        window.close(receipt.identity.pid)
    }
}
impl ReceiptSink for Sink<'_> {
    fn record(&mut self, receipt: &ProcessReceipt) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.0.join("family-receipt.json"))?;
        serde_json::to_writer(&mut file, &receipt_value(receipt)).map_err(io::Error::other)?;
        file.write_all(b"\n")?;
        file.sync_all()
    }
}
pub(super) fn finish(
    supervisor: Original,
    pin: Pin,
    directory: &Path,
    events: Vec<Value>,
    deadline: Deadline,
) -> Outcome {
    // Only called after the original poll has observed child exit and Empty.
    let outer_job_observed = supervisor.process().outer_job_observed();
    match supervisor.finish(deadline, &SystemClock, &mut Sink(directory)) {
        FinishOutcome::Complete(completion) => Outcome::Closed {
            completion,
            pin,
            events,
            outer_job_observed,
        },
        FinishOutcome::Incomplete(original) => Outcome::Retained(Box::new(Retained::Finishing {
            original,
            pin,
            events,
        })),
    }
}
