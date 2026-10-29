//! Original process/IPC close custody. No launcher, replay grant or SDK registry.
#[path = "parent_host/ports.rs"]
mod ports;
pub use ports::ParentReceiptSink;
mod close;
mod unassigned;
use kinekino_process::{
    application::{
        raw_records::RecordCursor,
        supervisor::{ProcessFamily, ReceiptSink, Supervisor},
    },
    domain::{
        deadline::Deadline,
        identity::ProcessIdentity,
        observation::{FamilyObservation, ProcessReceipt},
    },
    infrastructure::{
        evidence_files::EvidencePaths,
        family::{LaunchFailure, LaunchState, OwnedProcessFamily},
        ipc::{ClosePoll, SourceFamily, SourceIpc, SourceLaunchFailure},
    },
};
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseStage {
    Ipc,
    Process,
    RawStdout,
    RawStderr,
    Receipt,
}
/// The original error remains owned here or in the original supervisor/launch
/// carrier. Do not format this into user-visible URL/body/command diagnostics.
pub struct CloseError {
    pub stage: CloseStage,
    pub error: io::Error,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessClosure {
    NotCreated,
    Pending,
    Closed,
    UnassignedClosed,
}
/// The child was never assigned/resumed. Empty describes only the original
/// unassigned Job; no ProcessReceipt family=Empty is fabricated for this child.
#[derive(Debug)]
pub struct UnassignedCloseReceipt {
    pub identity: ProcessIdentity,
    pub child_exit: u32,
    pub original_job: FamilyObservation,
    pub launch_state: LaunchState,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiptState {
    NotApplicable,
    Unrecorded,
    Recorded,
}
#[derive(Clone, Copy, Debug)]
pub struct HostCloseFacts {
    pub ipc: ClosePoll,
    pub process: ProcessClosure,
    pub receipt: ReceiptState,
    pub stdout_eof: bool,
    pub stderr_eof: bool,
}
impl HostCloseFacts {
    /// Caller still persists transport facts and checks retained errors. A true
    /// value is resource closure readiness, never download/job/publication success.
    pub fn ready_to_ack(&self) -> bool {
        self.ipc.io_reaped
            && self.ipc.local_endpoints_closed
            && self.stdout_eof
            && self.stderr_eof
            && matches!(
                (self.process, self.receipt),
                (ProcessClosure::NotCreated, ReceiptState::NotApplicable)
                    | (ProcessClosure::Closed, ReceiptState::Recorded)
                    | (ProcessClosure::UnassignedClosed, ReceiptState::Recorded)
            )
    }
}
/// Same original handles survive open failure, diagnostic failure and unwind.
/// Caller retains worker/input/ancestor guards alongside this host until close.
pub struct ParentHost {
    process: Option<Supervisor<OwnedProcessFamily>>,
    ipc: SourceIpc,
    launch_failure: Option<Box<LaunchFailure>>,
    evidence: EvidencePaths,
    deadline: Deadline,
    closing: bool,
    receipt_attempted: bool,
    cursors: [RecordCursor; 2],
    raw_eof: [bool; 2],
    raw_failed: [bool; 2],
    first: Option<CloseError>,
    secondary: Vec<CloseError>,
    overflow: bool,
    process_error_seen: bool,
    ipc_error_seen: bool,
    unassigned_receipt: Option<UnassignedCloseReceipt>,
    unassigned_recorded: bool,
}
impl ParentHost {
    pub fn source_cutoffs(&self) -> kinekino_process::domain::source_clock::SourceCutoffs {
        self.ipc.source_cutoffs()
    }
    pub fn from_running(family: SourceFamily) -> Self {
        let deadline = family.owner.process().deadline();
        let evidence = family.owner.evidence().clone();
        Self::new(Some(family.owner), family.ipc, None, evidence, deadline)
    }
    /// raw_limit/requested must be the original launch budget, never a fresh
    /// cleanup timer. Assigned OwnedProcessFamily moves once into a Supervisor;
    /// unassigned owner stays in the original launch carrier for scoped closure.
    /// No PID is searched or reopened and no child is retried.
    pub fn from_launch_failure(
        failure: Box<SourceLaunchFailure>,
        requested: Deadline,
        raw_limit: u64,
    ) -> Self {
        let SourceLaunchFailure { mut launch, ipc } = *failure;
        let evidence = launch.evidence.clone();
        let mut deadline = requested;
        let process = launch.owner.take().and_then(|original| {
            deadline = deadline.constrained_by(original.deadline());
            if original.state() == LaunchState::UnassignedSuspended {
                launch.owner = Some(original);
                None
            } else {
                Some(Supervisor::new(*original, evidence.clone(), raw_limit))
            }
        });
        Self::new(process, ipc, Some(launch), evidence, deadline)
    }
    fn new(
        process: Option<Supervisor<OwnedProcessFamily>>,
        ipc: SourceIpc,
        launch_failure: Option<Box<LaunchFailure>>,
        evidence: EvidencePaths,
        deadline: Deadline,
    ) -> Self {
        Self {
            process,
            ipc,
            launch_failure,
            evidence,
            deadline,
            closing: false,
            receipt_attempted: false,
            cursors: Default::default(),
            raw_eof: [false; 2],
            raw_failed: [false; 2],
            first: None,
            secondary: Vec::new(),
            overflow: false,
            process_error_seen: false,
            ipc_error_seen: false,
            unassigned_receipt: None,
            unassigned_recorded: false,
        }
    }
    pub fn launch_failure(&self) -> Option<&LaunchFailure> {
        self.launch_failure.as_deref()
    }
    pub fn first_close_error(&self) -> Option<&CloseError> {
        self.first.as_ref()
    }
    pub fn secondary_close_errors(&self) -> &[CloseError] {
        &self.secondary
    }
    pub fn secondary_overflow(&self) -> bool {
        self.overflow
    }
    pub fn process(&self) -> Option<&Supervisor<OwnedProcessFamily>> {
        self.process.as_ref()
    }
    pub fn process_mut(&mut self) -> Option<&mut Supervisor<OwnedProcessFamily>> {
        self.process.as_mut()
    }
    pub fn receipt(&self) -> Option<&ProcessReceipt> {
        self.process.as_ref().map(Supervisor::receipt)
    }
    pub fn unassigned_receipt(&self) -> Option<&UnassignedCloseReceipt> {
        self.unassigned_receipt.as_ref()
    }
    pub fn evidence(&self) -> &EvidencePaths {
        &self.evidence
    }
    pub fn ipc_mut(&mut self) -> Option<&mut SourceIpc> {
        if self.closing {
            None
        } else {
            Some(&mut self.ipc)
        }
    }
    fn remember(&mut self, stage: CloseStage, error: io::Error) {
        let error = CloseError { stage, error };
        if self.first.is_none() {
            self.first = Some(error);
        } else if self.secondary.len() < 8 {
            self.secondary.push(error);
        } else {
            self.overflow = true;
        }
    }
}
