//! Launch-free custody and one-use launch. Product replay issuance stays outside.
mod close;
use crate::parent_host::{HostCloseFacts, ParentHost};
use kinekino_process::{
    application::raw_records::RecordCursor,
    domain::deadline::Deadline,
    infrastructure::{
        evidence_files::{EvidencePaths, PreparedEvidence},
        family::SpawnSpec,
        ipc::{ClosePoll, PreparedSourceIpc, spawn_source},
    },
};
use std::io;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenOnceError {
    Spent,
    Closing,
    Current,
    Launch,
    Ipc,
    Evidence,
    Unreturned,
}
struct LaunchInputs {
    spec: SpawnSpec,
    evidence: Option<PreparedEvidence>,
    ipc: PreparedSourceIpc,
}
/// Resource guard is supplied and validated by the existing trusted product
/// adapter. Holding G is lifetime custody, not executable/replay validation.
pub struct ParentLaunch<G: Send> {
    prepared: Option<LaunchInputs>,
    host: Option<ParentHost>,
    evidence: EvidencePaths,
    deadline: Deadline,
    raw_limit: u64,
    _guard: G,
    spent: bool,
    closing: bool,
    unreturned: bool,
    first: Option<OpenOnceError>,
    io_errors: Vec<io::Error>,
    io_overflow: bool,
    cursors: [RecordCursor; 2],
    raw_eof: [bool; 2],
    raw_failed: [bool; 2],
}
#[derive(Clone, Copy, Debug)]
pub struct LaunchCloseFacts {
    pub host: Option<HostCloseFacts>,
    pub prepared_ipc: Option<ClosePoll>,
    pub raw_eof: [bool; 2],
    pub launch_unreturned: bool,
}
impl LaunchCloseFacts {
    /// Resource readiness only. Product adapter must persist facts, preserve
    /// errors and never equate this with source/download/publication success.
    pub fn ready_to_ack(&self) -> bool {
        if self.launch_unreturned {
            return false;
        }
        if let Some(host) = self.host {
            return host.ready_to_ack();
        }
        self.prepared_ipc
            .is_some_and(|ipc| ipc.io_reaped && ipc.local_endpoints_closed)
            && self.raw_eof == [true; 2]
    }
}
impl<G: Send> ParentLaunch<G> {
    /// Connect only while the original launch owner remains retained. No spawn.
    pub fn poll_prepared(&mut self) -> std::io::Result<bool> {
        if self.closing || self.spent {
            return Err(std::io::Error::other(
                "original launch preparation unavailable",
            ));
        }
        self.prepared
            .as_mut()
            .ok_or_else(|| std::io::Error::other("original launch inputs absent"))?
            .ipc
            .poll_connected()
    }
    /// Infallible custody only: no launch, current check, network or new timer.
    /// The jobs core must own this before try_launch invokes any external code.
    pub fn new(
        spec: SpawnSpec,
        evidence: PreparedEvidence,
        deadline: Deadline,
        ipc: PreparedSourceIpc,
        raw_limit: u64,
        guard: G,
    ) -> Self {
        let paths = evidence.paths.clone();
        Self {
            prepared: Some(LaunchInputs {
                spec,
                evidence: Some(evidence),
                ipc,
            }),
            host: None,
            evidence: paths,
            deadline,
            raw_limit,
            _guard: guard,
            spent: false,
            closing: false,
            unreturned: false,
            first: None,
            io_errors: Vec::new(),
            io_overflow: false,
            cursors: Default::default(),
            raw_eof: [false; 2],
            raw_failed: [false; 2],
        }
    }
    pub fn first_error(&self) -> Option<OpenOnceError> {
        self.first
    }
    pub fn io_errors(&self) -> &[io::Error] {
        &self.io_errors
    }
    pub fn io_overflow(&self) -> bool {
        self.io_overflow
    }
    pub fn spent(&self) -> bool {
        self.spent
    }
    pub fn host(&self) -> Option<&ParentHost> {
        self.host.as_ref()
    }
    pub fn host_mut(&mut self) -> Option<&mut ParentHost> {
        self.host.as_mut()
    }
    /// Callback validates original current/attempt/deadline. It does not create
    /// replay permission. Original D admission is a separate prerequisite.
    pub fn try_launch(
        &mut self,
        requested: Deadline,
        mut check: impl FnMut() -> Result<(), OpenOnceError>,
    ) -> Result<(), OpenOnceError> {
        if self.closing {
            return Err(OpenOnceError::Closing);
        }
        if self.spent {
            return Err(OpenOnceError::Spent);
        }
        self.spent = true; // even rejection/unwind before launch consumes the attempt
        self.deadline = self.deadline.constrained_by(requested);
        if let Err(error) = check() {
            self.first.get_or_insert(error);
            return Err(error);
        }
        let Some(mut inputs) = self.prepared.take() else {
            self.first.get_or_insert(OpenOnceError::Unreturned);
            return Err(OpenOnceError::Unreturned);
        };
        self.unreturned = true; // no invented no-child ACK if the launch call unwinds
        let result = spawn_source(
            inputs.spec,
            inputs.evidence.take().expect("original prepared evidence"),
            self.deadline,
            inputs.ipc,
            self.raw_limit,
        );
        // Store the returned original owner before any post-launch check/unwind.
        let failed = match result {
            Ok(family) => {
                self.host = Some(ParentHost::from_running(family));
                false
            }
            Err(failure) => {
                self.host = Some(ParentHost::from_launch_failure(
                    failure,
                    self.deadline,
                    self.raw_limit,
                ));
                true
            }
        };
        self.unreturned = false;
        if failed {
            self.first.get_or_insert(OpenOnceError::Launch);
            return Err(OpenOnceError::Launch);
        }
        if let Err(error) = check() {
            self.first.get_or_insert(error);
            return Err(error);
        }
        Ok(())
    }
    fn remember_io(&mut self, error: io::Error) {
        self.first.get_or_insert(OpenOnceError::Evidence);
        if self.io_errors.len() < 8 {
            self.io_errors.push(error);
        } else {
            self.io_overflow = true;
        }
    }
}
