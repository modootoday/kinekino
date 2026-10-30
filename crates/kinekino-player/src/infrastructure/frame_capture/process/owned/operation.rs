//! Original parent operation, retained before launch and between nonblocking polls.
use super::*;
use crate::application::{review_source::MutationControl, sdk_execution::ReceiptRecoveryControl};
use kinekino_process::infrastructure::{evidence_files::PreparedEvidence, family::SpawnSpec};
use std::{fs::File, path::PathBuf};
struct Prepared {
    spec: SpawnSpec,
    evidence: PreparedEvidence,
    root: PathBuf,
    deadline: Deadline,
    maximum: usize,
}
enum Phase {
    Prepared(Box<Prepared>),
    Running(Box<Session>),
    Finished(Result<ProcessOutput, ApiError>),
    Consumed,
}
/// This value is not serializable or reconstructible from process IDs/receipts.
pub(crate) struct NativeProcessOperation {
    phase: Phase,
    settlement: Option<NativeProcessProgress>,
    observer: Option<std::sync::Arc<crate::application::review_source::NativeProcessState>>,
}

/// Observation from this original parent only; serialized diagnostics are not authority.
#[derive(Debug, serde::Serialize)]
pub(crate) struct NativeProcessProgress {
    pub not_started: bool,
    pub child_exit: Option<u32>,
    pub family_empty: bool,
    pub receipt_recorded: bool,
    pub regular_files_eof: bool,
    pub forced: bool,
}
impl NativeProcessProgress {
    fn not_started() -> Self {
        Self {
            not_started: true,
            child_exit: None,
            family_empty: false,
            receipt_recorded: false,
            regular_files_eof: false,
            forced: false,
        }
    }
    fn observed(session: &Session) -> Self {
        let receipt = session.owner.receipt();
        Self {
            not_started: false,
            child_exit: receipt.child_exit,
            family_empty: receipt.family == FamilyObservation::Empty,
            receipt_recorded: session.owner.receipt_recorded(),
            regular_files_eof: session.output_drain.complete(),
            forced: receipt.forced,
        }
    }
    pub(crate) fn may_release(&self) -> bool {
        self.not_started
            || (self.child_exit.is_some()
                && self.family_empty
                && self.receipt_recorded
                && self.regular_files_eof)
    }
}

impl NativeProcessOperation {
    pub(crate) fn prepare(
        command: &Command,
        cutoff: Instant,
        maximum: usize,
        control: Option<&dyn MutationControl>,
        input: Option<File>,
    ) -> Result<Self, ApiError> {
        if let Some(control) = control {
            control.checkpoint()?;
        }
        let start = Instant::now();
        let work = cutoff.saturating_duration_since(start);
        if work.is_zero() {
            return Err(timeout());
        }
        if maximum == 0 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Positive frame output limit required",
            ));
        }
        let deadline = Deadline::new(start, work + CLEANUP_RESERVE, CLEANUP_RESERVE)
            .map_err(|_| failed("deadline"))?;
        let (spec, evidence, root) = match input {
            Some(input) => launch::prepare_with_input(command, input)?,
            None => launch::prepare(command)?,
        };
        Ok(Self {
            observer: control
                .and_then(MutationControl::native_processes)
                .map(|ledger| ledger.register()),
            settlement: None,
            phase: Phase::Prepared(Box::new(Prepared {
                spec,
                evidence,
                root,
                deadline,
                maximum,
            })),
        })
    }
    /// Explicit single launch; polling a prepared operation never creates a child.
    pub(crate) fn launch(&mut self) -> Result<(), ApiError> {
        if !matches!(self.phase, Phase::Prepared(_)) {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Original native operation was already launched or consumed",
            ));
        }
        if let Some(observer) = &self.observer {
            observer.record(crate::application::review_source::NativeProcessObservation::Pending);
        }
        let Phase::Prepared(prepared) = std::mem::replace(&mut self.phase, Phase::Consumed) else {
            unreachable!()
        };
        let Prepared {
            spec,
            evidence,
            root,
            deadline,
            maximum,
        } = *prepared;
        let raw_limit = maximum.max(STDERR_LIMIT) as u64;
        let (owner, first) = match spawn(spec, evidence, deadline) {
            Ok((family, evidence)) => (Supervisor::new(family, evidence, raw_limit), None),
            Err(failure) => {
                let mut error = if failure.error.kind() == std::io::ErrorKind::TimedOut {
                    timeout()
                } else {
                    failed(failure.stage)
                };
                error.details = serde_json::json!({"stage":failure.stage,"os_error":failure.error.raw_os_error(),"termination_os_error":failure.termination_error.as_ref().and_then(std::io::Error::raw_os_error),"evidence_root":root});
                if let Some(owner) = failure.owner {
                    (
                        Supervisor::new(*owner, failure.evidence, raw_limit),
                        Some(error),
                    )
                } else {
                    self.settlement = Some(NativeProcessProgress::not_started());
                    self.phase = Phase::Finished(Err(error));
                    self.record_observation();
                    return Ok(());
                }
            }
        };
        self.phase = Phase::Running(Box::new(Session {
            owner,
            deadline,
            max_stdout: maximum,
            first,
            sink: receipt::FileSink::new(root),
            receipt_attempted: false,
            recovery_generation: None,
            output_drain: Default::default(),
        }));
        self.record_observation();
        Ok(())
    }
    /// Original Session owns physical family and sticky receipt recovery throughout.
    pub(crate) fn poll(
        &mut self,
        control: Option<&dyn MutationControl>,
        recovery: Option<&dyn ReceiptRecoveryControl>,
    ) -> bool {
        let done = match &mut self.phase {
            Phase::Prepared(_) => false,
            Phase::Running(session) => {
                if !session.poll(control, recovery) {
                    self.record_observation();
                    return false;
                }
                self.settlement = Some(NativeProcessProgress::observed(session));
                let Phase::Running(session) = std::mem::replace(&mut self.phase, Phase::Consumed)
                else {
                    unreachable!()
                };
                self.phase = Phase::Finished(session.complete());
                true
            }
            Phase::Finished(_) => true,
            Phase::Consumed => false,
        };
        self.record_observation();
        done
    }
    fn record_observation(&self) {
        use crate::application::review_source::NativeProcessObservation;
        if let Some(observer) = &self.observer {
            let observation = match self.progress() {
                Some(p) if p.not_started => NativeProcessObservation::NotStarted,
                Some(p) => NativeProcessObservation::Observed {
                    child_exit: p.child_exit,
                    family_empty: p.family_empty,
                    receipt_recorded: p.receipt_recorded,
                    regular_files_eof: p.regular_files_eof,
                    forced: p.forced,
                },
                None => NativeProcessObservation::Pending,
            };
            observer.record(observation);
        }
    }
    pub(crate) fn take_result(&mut self) -> Option<Result<ProcessOutput, ApiError>> {
        if !matches!(self.phase, Phase::Finished(_)) || !self.progress()?.may_release() {
            return None;
        }
        let Phase::Finished(result) = std::mem::replace(&mut self.phase, Phase::Consumed) else {
            unreachable!()
        };
        Some(result)
    }
    /// Pending receipt failure remains visible after physical closure, without PID lookup.
    pub(crate) fn progress(&self) -> Option<NativeProcessProgress> {
        match &self.phase {
            Phase::Prepared(_) => Some(NativeProcessProgress::not_started()),
            Phase::Running(session) => Some(NativeProcessProgress::observed(session)),
            Phase::Finished(_) | Phase::Consumed => {
                self.settlement.as_ref().map(|p| NativeProcessProgress {
                    not_started: p.not_started,
                    child_exit: p.child_exit,
                    family_empty: p.family_empty,
                    receipt_recorded: p.receipt_recorded,
                    regular_files_eof: p.regular_files_eof,
                    forced: p.forced,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Real original Windows operation: prepare before launch, one child, polling and one result; central allocation"]
    fn prepared_operation_launches_once_and_consumes_only_settled_result_once() {
        use crate::application::review_source::{NativeProcessLedger, NativeProcessObservation};
        struct Control(NativeProcessLedger);
        impl MutationControl for Control {
            fn checkpoint(&self) -> Result<(), ApiError> {
                Ok(())
            }
            fn native_processes(&self) -> Option<&NativeProcessLedger> {
                Some(&self.0)
            }
        }
        let control = Control(NativeProcessLedger::default());
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.args(["--exact", "native_operation_zero_case_fixture"]);
        let mut operation = NativeProcessOperation::prepare(
            &command,
            Instant::now() + Duration::from_secs(7),
            4096,
            Some(&control),
            None,
        )
        .unwrap();
        let Phase::Prepared(prepared) = &operation.phase else {
            panic!("Not prepared")
        };
        let evidence = prepared.root.clone();
        assert!(operation.progress().unwrap().not_started);
        assert_eq!(
            control.0.observations(),
            [NativeProcessObservation::NotStarted]
        );
        assert!(!operation.poll(None, None));
        assert!(operation.take_result().is_none());
        assert!(!evidence.join("receipt-0000.json").exists());
        operation.launch().unwrap();
        assert!(!control.0.observations()[0].closed());
        assert!(!operation.progress().unwrap().not_started);
        assert_eq!(
            operation.launch().unwrap_err().code,
            ErrorCode::RevisionConflict
        );
        while !operation.poll(None, None) {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(operation.poll(None, None));
        let progress = operation.progress().unwrap();
        assert!(progress.family_empty && progress.receipt_recorded);
        assert!(progress.regular_files_eof);
        assert_eq!(progress.child_exit, Some(0));
        assert!(!progress.forced);
        assert_eq!(
            control.0.observations(),
            [NativeProcessObservation::Observed {
                child_exit: Some(0),
                family_empty: true,
                receipt_recorded: true,
                regular_files_eof: true,
                forced: false,
            }]
        );
        assert!(operation.launch().is_err());
        let output = operation.take_result().unwrap().unwrap();
        assert!(String::from_utf8_lossy(&output.stdout).contains("0 passed"));
        assert!(operation.take_result().is_none());
        assert!(!operation.poll(None, None));
        drop(operation);
        assert!(control.0.observations()[0].closed());
        assert_eq!(
            match control.0.observations()[0] {
                NativeProcessObservation::Observed { child_exit, .. } => child_exit,
                _ => None,
            },
            progress.child_exit
        );
        let receipt: serde_json::Value =
            serde_json::from_slice(&std::fs::read(evidence.join("receipt-0000.json")).unwrap())
                .unwrap();
        assert_eq!(receipt["child_exit"], 0);
        assert_eq!(receipt["family"], "Empty");
        assert_eq!(receipt["forced"], false);
        std::fs::write(evidence.join("operation-verification.json"),serde_json::to_vec_pretty(&serde_json::json!({"prepare_poll_did_not_launch":true,"duplicate_launch_rejected_without_replacing_owner":true,"result_only_after_original_family_and_receipt_settled":true,"result_consumed_once":true,"jobs_owned_attempt_migration_verified":false})).unwrap()).unwrap();
    }
}
