use super::{
    ProbeContainer, ProbePlan, ProbeToolPin,
    result::{self, MAX_PROBE_OUTPUT},
};
use crate::infrastructure::content_digest::{DigestFailure, sha256_with_checkpoint};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        download::{PendingMediaVerification, VerificationLease, VerificationPoll},
        jobs::{ExecutionControl, owned_attempt::OwnerObservation},
    },
    infrastructure::immutable_storage_paths,
};
use kinekino_process::{
    application::supervisor::{ReceiptSink, Supervisor, SystemClock},
    domain::{
        deadline::Deadline,
        observation::{FamilyObservation, ProcessReceipt},
    },
    infrastructure::{
        evidence_files::PreparedEvidence,
        family::{LaunchFailure, OwnedProcessFamily, SpawnSpec, spawn},
    },
};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::windows::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Capabilities,
    Inspect,
    Closed,
}
/// Core stores this prepared value before poll may launch. Every error path
/// retains original process/launch failure/output lease until explicit closure.
pub(super) struct ProbeAttempt {
    tool: Arc<ProbeToolPin>,
    plan: Arc<ProbePlan>,
    lease: Arc<dyn VerificationLease>,
    root: PathBuf,
    _root_guards: Vec<File>,
    _tool_guards: Vec<File>,
    _tool: File,
    input: Option<File>,
    owner: Option<Supervisor<OwnedProcessFamily>>,
    launch_failure: Option<Box<LaunchFailure>>,
    started: Instant,
    deadline: Deadline,
    active_deadline: Deadline,
    reserve: Duration,
    phase: Phase,
    first: Option<ApiError>,
    cancelled: bool,
    receipt_sequence: u8,
    last_observation: OwnerObservation,
    terminal: Option<Result<(), ApiError>>,
}
impl ProbeAttempt {
    pub(super) fn prepare(
        tool: Arc<ProbeToolPin>,
        plan: Arc<ProbePlan>,
        lease: Arc<dyn VerificationLease>,
        base: &Path,
        name: &str,
        reserve: Duration,
        control: &dyn ExecutionControl,
    ) -> Result<Self, ApiError> {
        let started = Instant::now();
        let total = control.deadline().saturating_duration_since(started);
        let deadline = Deadline::new(started, total, reserve).map_err(|_| {
            ApiError::new(
                ErrorCode::Timeout,
                "Insufficient original budget for probe and cleanup",
            )
        })?;
        check(control)?;
        lease.check_current()?;
        if !crate::domain::identifiers::valid_sha256(&tool.sha256) || tool.bytes == 0 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid host-selected probe tool pin",
            ));
        }
        let parent = tool
            .executable
            .parent()
            .ok_or_else(|| invalid("Probe executable parent is absent"))?;
        let (_, tool_guards) = immutable_storage_paths::root(parent)?;
        let mut tool_file = immutable_storage_paths::file(&tool.executable)?;
        if tool_file.metadata().map_err(io_error)?.len() != tool.bytes {
            return Err(expired("Probe executable bytes differ from host pin"));
        }
        let (bytes, digest) = sha256_with_checkpoint(&mut tool_file, tool.bytes, || check(control))
            .map_err(|failure| match failure {
                DigestFailure::Read(error) => io_error(error),
                DigestFailure::ByteCountOverflow => invalid("Tool byte count overflow"),
                DigestFailure::ByteBoundExceeded { .. } => expired("Probe executable changed"),
                other => other.into_api_error(),
            })?;
        if bytes != tool.bytes || digest != tool.sha256 {
            return Err(expired("Probe executable digest differs from host pin"));
        }
        let (base, mut root_guards) = immutable_storage_paths::root(base)?;
        let root = base.join(name);
        fs::create_dir(&root).map_err(io_error)?;
        root_guards.push(immutable_storage_paths::directory(&root)?);
        let input = OpenOptions::new()
            .read(true)
            .share_mode(7)
            .custom_flags(0x00200000)
            .open(lease.path())
            .map_err(io_error)?;
        let metadata = input.metadata().map_err(io_error)?;
        if !metadata.is_file() || metadata.len() == 0 || metadata.file_attributes() & 0x400 != 0 {
            return Err(invalid(
                "Probe input must be original nonempty regular media",
            ));
        }
        lease.check_current()?;
        check(control)?;
        Ok(Self {
            tool,
            plan,
            lease,
            root,
            _root_guards: root_guards,
            _tool_guards: tool_guards,
            _tool: tool_file,
            input: Some(input),
            owner: None,
            launch_failure: None,
            started,
            deadline,
            active_deadline: deadline,
            reserve,
            phase: Phase::Capabilities,
            first: None,
            cancelled: false,
            receipt_sequence: 0,
            last_observation: OwnerObservation::NotStarted,
            terminal: None,
        })
    }
    fn fail(&mut self, error: ApiError) {
        if self.first.is_none() {
            self.first = Some(error);
        }
    }
    fn stop(&mut self) {
        self.cancelled = true;
        let elapsed = self.started.elapsed().max(Duration::from_nanos(1));
        if let Some(total) = elapsed.checked_add(self.reserve)
            && let Ok(short) = Deadline::new(self.started, total, self.reserve)
        {
            self.active_deadline = self.active_deadline.constrained_by(short);
        }
    }
    fn finish_error(&mut self, error: ApiError) {
        self.fail(error);
        self.phase = Phase::Closed;
        self.terminal = Some(Err(copy_error(self.first.as_ref().unwrap())));
    }
    fn launch(&mut self) -> Result<(), ApiError> {
        let name = if self.phase == Phase::Capabilities {
            "capabilities"
        } else {
            "inspect"
        };
        let directory = self.root.join(name);
        fs::create_dir(&directory).map_err(io_error)?;
        self._root_guards
            .push(immutable_storage_paths::directory(&directory)?);
        let mut evidence = PreparedEvidence::create(&directory, None, 0).map_err(io_error)?;
        let args: Vec<OsString> = if self.phase == Phase::Capabilities {
            ["-v", "error", "-protocols"]
                .into_iter()
                .map(Into::into)
                .collect()
        } else {
            evidence = evidence
                .with_input_file(
                    self.input
                        .take()
                        .ok_or_else(|| invalid("Original probe reader was already consumed"))?,
                )
                .map_err(io_error)?;
            let demuxer = match self.plan.container {
                ProbeContainer::Mp4 => "mov",
                ProbeContainer::Matroska => "matroska",
                ProbeContainer::MpegTs => "mpegts",
            };
            let mut args: Vec<OsString> =
                ["-v", "error", "-protocol_whitelist", "fd", "-f", demuxer]
                    .into_iter()
                    .map(Into::into)
                    .collect();
            if matches!(self.plan.container, ProbeContainer::Mp4) {
                args.extend(["-enable_drefs".into(), "0".into()]);
            }
            args.extend(["-fd","0","-blocksize","65536","-count_packets","-show_entries","stream=index,codec_type,codec_name,width,height,channels,sample_rate,nb_read_packets:format=format_name,duration","-of","json","-i","fd:"].into_iter().map(OsString::from));
            args
        };
        let mut environment = BTreeMap::new();
        if let Some(root) = std::env::var_os("SystemRoot") {
            environment.insert("SystemRoot".into(), root);
        }
        let spec = SpawnSpec {
            executable: self.tool.executable.clone(),
            cwd: directory,
            arguments: args,
            environment: Some(environment),
        };
        match spawn(spec, evidence, self.active_deadline) {
            Ok((process, paths)) => {
                self.owner = Some(Supervisor::new(process, paths, MAX_PROBE_OUTPUT as u64));
                Ok(())
            }
            Err(mut failure) => {
                if let Some(process) = failure.owner.take() {
                    self.owner = Some(Supervisor::new(
                        *process,
                        failure.evidence.clone(),
                        MAX_PROBE_OUTPUT as u64,
                    ));
                }
                let mut error = ApiError::new(
                    ErrorCode::EngineFailed,
                    "Original probe process launch failed",
                );
                error.details = serde_json::json!({"launch_stage":failure.stage});
                self.launch_failure = Some(failure);
                Err(error)
            }
        }
    }
    fn raw(owner: &Supervisor<OwnedProcessFamily>) -> Result<(Vec<u8>, Vec<u8>), ApiError> {
        let read = |mut file: File| -> Result<Vec<u8>, ApiError> {
            let length = file.metadata().map_err(io_error)?.len();
            let mut bytes = Vec::new();
            Read::by_ref(&mut file)
                .take(MAX_PROBE_OUTPUT as u64 + 1)
                .read_to_end(&mut bytes)
                .map_err(io_error)?;
            if length != bytes.len() as u64
                || file.metadata().map_err(io_error)?.len() != length
                || bytes.len() > MAX_PROBE_OUTPUT
            {
                return Err(ApiError::new(
                    ErrorCode::ResourceExhausted,
                    "Probe raw response is incomplete or oversized",
                ));
            }
            Ok(bytes)
        };
        Ok((
            read(owner.evidence().stdout.open_read().map_err(io_error)?)?,
            read(owner.evidence().stderr.open_read().map_err(io_error)?)?,
        ))
    }
}
impl PendingMediaVerification for ProbeAttempt {
    fn first_error(&self) -> Option<&ApiError> {
        self.first.as_ref()
    }
    fn owner_observation(&self) -> OwnerObservation {
        if self.phase == Phase::Closed {
            self.last_observation
        } else {
            OwnerObservation::Pending
        }
    }
    fn request_cancel(&mut self) -> Result<(), ApiError> {
        self.stop();
        Ok(())
    }
    fn poll(&mut self, control: &dyn ExecutionControl) -> VerificationPoll {
        if self.phase == Phase::Closed {
            return VerificationPoll::Closed(
                self.terminal
                    .take()
                    .unwrap_or_else(|| Err(invalid("Probe was already settled"))),
            );
        }
        if let Err(error) = check(control).and_then(|()| self.lease.check_current()) {
            self.fail(error);
            self.stop();
        }
        if self.deadline.operation_remaining(Instant::now()).is_zero() {
            self.fail(ApiError::new(
                ErrorCode::Timeout,
                "Original probe operation budget expired",
            ));
            self.stop();
        }
        if self.owner.is_none() {
            if self.cancelled {
                self.finish_error(ApiError::new(
                    ErrorCode::Cancelled,
                    "Probe stopped before next launch",
                ));
                return self.poll(control);
            }
            if let Err(error) = self.launch() {
                self.fail(error);
                self.stop();
                if self.owner.is_none() {
                    self.finish_error(copy_error(self.first.as_ref().unwrap()));
                    return self.poll(control);
                }
            }
        }
        let owner = self.owner.as_mut().unwrap();
        let progress = owner.poll_cleanup(self.active_deadline, &SystemClock);
        let physical = progress.physical_closed();
        let cleanup_error = progress.first_error().map(|cause| {
            ApiError::new(
                ErrorCode::EngineFailed,
                format!("Original probe cleanup failed at {:?}", cause.stage),
            )
        });
        if let Some(error) = cleanup_error
            && self.first.is_none()
        {
            self.first = Some(error);
        }
        if physical
            && !owner.cleanup_progress().unwrap().final_receipt_recorded()
            && self.receipt_sequence < 8
        {
            self.receipt_sequence += 1;
            let path = self.root.join(format!(
                "{}-close-{}.json",
                if self.phase == Phase::Capabilities {
                    "capabilities"
                } else {
                    "inspect"
                },
                self.receipt_sequence
            ));
            if let Err(error) = owner.record_cleanup(&mut Receipt(path), &SystemClock)
                && self.first.is_none()
            {
                self.first = Some(io_error(error));
            }
        }
        if !owner.cleanup_progress().unwrap().settled() {
            return VerificationPoll::Pending;
        }
        let receipt = owner.receipt().clone();
        let successful = receipt.child_exit == Some(0)
            && !receipt.forced
            && !owner.cleanup_progress().unwrap().deadline_expired()
            && owner.cleanup_progress().unwrap().first_error().is_none();
        let (stdout, stderr) = match Self::raw(owner) {
            Ok(raw) => raw,
            Err(error) => {
                if self.first.is_none() {
                    self.first = Some(error);
                }
                return VerificationPoll::Pending;
            }
        };
        self.last_observation = OwnerObservation::Exited {
            exit_code: receipt.child_exit.unwrap_or(u32::MAX) as i32,
            family_empty: receipt.family == FamilyObservation::Empty,
            // Bounded reads reached both original regular-file EOFs after family
            // closure. This is distinct from merely requesting stdin EOF.
            streams_eof: true,
            forced: receipt.forced,
        };
        let raw = if successful && self.first.is_none() && stderr.is_empty() {
            Ok(stdout)
        } else {
            Err(copy_error(self.first.as_ref().unwrap_or(&ApiError::new(
                ErrorCode::EngineFailed,
                "Probe exited unsuccessfully",
            ))))
        };
        // Original process/family is physically closed and close receipt persisted.
        self.owner.take();
        self.launch_failure.take();
        self.receipt_sequence = 0;
        match raw {
            Err(error) => self.finish_error(error),
            Ok(bytes) if self.phase == Phase::Capabilities => {
                let fd = std::str::from_utf8(&bytes)
                    .is_ok_and(|text| text.lines().any(|line| line.trim() == "fd"));
                if !fd {
                    self.finish_error(ApiError::new(
                        ErrorCode::Unsupported,
                        "Pinned probe tool has no observed fd protocol",
                    ));
                } else {
                    self.phase = Phase::Inspect;
                    return VerificationPoll::Pending;
                }
            }
            Ok(bytes) => match result::inspect(&bytes, &self.plan) {
                Ok(_) => {
                    self.phase = Phase::Closed;
                    self.terminal = Some(Ok(()));
                }
                Err(error) => self.finish_error(error),
            },
        }
        self.poll(control)
    }
}
struct Receipt(PathBuf);
impl ReceiptSink for Receipt {
    fn record(&mut self, receipt: &ProcessReceipt) -> std::io::Result<()> {
        let value = serde_json::json!({"identity":{"pid":receipt.identity.pid,"creation_filetime":receipt.identity.creation_filetime,"executable":receipt.identity.executable},"child_exit":receipt.child_exit,"family":format!("{:?}",receipt.family),"forced":receipt.forced,"stdin_eof_requested":receipt.stdin_eof_requested});
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.0)?;
        file.write_all(&serde_json::to_vec_pretty(&value).map_err(std::io::Error::other)?)?;
        file.sync_all()
    }
}
fn check(control: &dyn ExecutionControl) -> Result<(), ApiError> {
    if control.cancellation_requested() {
        Err(ApiError::new(ErrorCode::Cancelled, "Probe cancelled"))
    } else if Instant::now() >= control.deadline() {
        Err(ApiError::new(
            ErrorCode::Timeout,
            "Original probe deadline expired",
        ))
    } else {
        Ok(())
    }
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::Internal, message)
}
fn expired(message: &str) -> ApiError {
    ApiError::new(ErrorCode::SourceExpired, message)
}
fn io_error(error: std::io::Error) -> ApiError {
    let raw = error.raw_os_error();
    let mut failure = ApiError::new(ErrorCode::EngineFailed, "Probe filesystem operation failed");
    failure.details =
        serde_json::json!({"io_kind":format!("{:?}",error.kind()),"raw_os_error":raw});
    failure
}
fn copy_error(error: &ApiError) -> ApiError {
    ApiError {
        code: error.code,
        message: error.message.clone(),
        retryable: error.retryable,
        details: error.details.clone(),
        remediation: error.remediation.clone(),
        request_id: error.request_id.clone(),
    }
}
