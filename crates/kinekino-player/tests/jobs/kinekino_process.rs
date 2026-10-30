//! Original-handle test orchestration, not a process state machine or product ACK.
#[path = "../support/owned_process_evidence.rs"]
pub(super) mod evidence;
pub(super) use evidence::CliFailure;
use evidence::{Ownership, receipt_json, write_json};
use kinekino_process::{
    application::{
        raw_records::{RecordCursor, RecordPoll},
        supervisor::{
            FinishOutcome, PhaseObservation, ProcessFamily, ReceiptSink, RecordObservation,
            Supervisor, SystemClock,
        },
    },
    domain::{
        deadline::Deadline,
        observation::{FamilyObservation, ProcessReceipt},
    },
    infrastructure::{
        evidence_files::{PreparedEvidence, RawFile},
        family::{OwnedProcessFamily, SpawnSpec, spawn},
    },
};
use serde_json::{Value, json};
use std::{
    any::Any,
    ffi::OsString,
    fmt, fs,
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub(super) const RECORD_LIMIT: usize = kinekino_player::application::local_host::MAX_RESPONSE_BYTES;
const STREAM_LIMIT: u64 = RECORD_LIMIT as u64 * 2 + 2;
#[derive(Debug)]
pub(super) struct SdkFailure(pub kinekino_player::application::api::ApiError);
impl fmt::Display for SdkFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl std::error::Error for SdkFailure {}
pub(super) fn sdk_failure(
    stage: &'static str,
    error: kinekino_player::application::api::ApiError,
    root: &Path,
) -> Box<CliFailure> {
    CliFailure::not_spawned(stage, io::Error::other(SdkFailure(error)), root)
}
// Also used by the unchanged shared adapter's test module when imported here.
fn hash(bytes: &[u8]) -> String {
    kinekino_player::infrastructure::content_digest::sha256(
        &mut io::Cursor::new(bytes),
        STREAM_LIMIT,
    )
    .unwrap()
    .1
}
pub(super) fn phase_deadline(original: Deadline, cap: Duration) -> io::Result<Deadline> {
    Ok(original.constrained_by(Deadline::new(
        Instant::now(),
        cap,
        evidence::CLEANUP_RESERVE,
    )?))
}
pub(super) fn transport_at(
    endpoint: &str,
    original: Deadline,
) -> io::Result<kinekino_player::infrastructure::local_host::EndpointClient> {
    let remaining = original
        .operation_remaining(Instant::now())
        .min(Duration::from_secs(5));
    if remaining < Duration::from_millis(10) {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "Original endpoint budget exhausted",
        ));
    }
    let options =
        kinekino_player::infrastructure::local_host::TransportOptions { timeout: remaining };
    kinekino_player::infrastructure::local_host::EndpointClient::new(endpoint, options)
        .map_err(|error| io::Error::other(SdkFailure(error)))
}
pub(super) struct OwnedHost<P = OwnedProcessFamily> {
    owner: Supervisor<P>,
    cursor: RecordCursor,
    root: PathBuf,
    original: Deadline,
    active_cap: Deadline,
}
pub(super) struct HostTerminal {
    pub receipt: ProcessReceipt,
    pub tail: Vec<u8>,
    pub deadline: Deadline,
}
struct Sink(PathBuf);
impl ReceiptSink for Sink {
    fn record(&mut self, receipt: &ProcessReceipt) -> io::Result<()> {
        write_json(&self.0, &receipt_json(receipt))
    }
}
pub(super) fn export_raw(
    raw: &RawFile,
    target: &Path,
    offset: u64,
    maximum: usize,
    terminal: bool,
) -> io::Result<Value> {
    let mut file = raw.open_read()?;
    let length = file.metadata()?.len();
    if length < offset {
        return Err(io::Error::other("Raw stream shrank before export"));
    }
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = Vec::new();
    file.take(maximum as u64 + 1).read_to_end(&mut bytes)?;
    fs::write(target, &bytes)?;
    let digest = kinekino_player::infrastructure::content_digest::sha256(
        &mut io::Cursor::new(&bytes),
        maximum as u64 + 1,
    )
    .map_err(|error| io::Error::other(format!("{error:?}")))?
    .1;
    Ok(
        json!({"origin":raw.path(),"offset":offset,"preserved_bytes":bytes.len(),"observed_length":length,"sha256":digest,"complete":terminal&&length-offset==bytes.len() as u64&&bytes.len()<=maximum,"limit_exceeded":length-offset>maximum as u64,"live_writer":false}),
    )
}
impl OwnedHost {
    pub(super) fn launch(
        executable: PathBuf,
        arguments: Vec<OsString>,
        root: &Path,
        original: Deadline,
    ) -> Result<Self, Box<CliFailure>> {
        Self::launch_with_environment(executable, arguments, root, original, None)
    }
    pub(super) fn launch_with_environment(
        executable: PathBuf,
        arguments: Vec<OsString>,
        root: &Path,
        original: Deadline,
        environment: Option<std::collections::BTreeMap<String, OsString>>,
    ) -> Result<Self, Box<CliFailure>> {
        let failure = |stage, error| CliFailure::not_spawned(stage, error, root);
        if original.operation_remaining(Instant::now()).is_zero() {
            return Err(failure(
                "host budget",
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Host deadline expired before launch",
                ),
            ));
        }
        fs::create_dir(root.join("owned")).map_err(|error| failure("host evidence", error))?;
        let prepared = PreparedEvidence::create(&root.join("owned"), None, 0)
            .map_err(|error| failure("host raw preparation", error))?;
        let cwd = std::env::current_dir().map_err(|error| failure("host cwd", error))?;
        let (family, paths) = spawn(
            SpawnSpec {
                executable,
                cwd,
                arguments,
                environment,
            },
            prepared,
            original,
        )
        .map_err(|error| {
            let mut result = match error.owner {
                Some(owner) => CliFailure::live(
                    error.stage,
                    error.error,
                    root,
                    Supervisor::new(*owner, error.evidence, STREAM_LIMIT),
                    original,
                ),
                None => failure(error.stage, error.error),
            };
            if let Some(error) = error.termination_error {
                result
                    .secondary
                    .push(format!("launch termination: {error}"));
            }
            result
        })?;
        Ok(Self {
            owner: Supervisor::new(family, paths, STREAM_LIMIT),
            cursor: RecordCursor::default(),
            root: root.into(),
            original,
            active_cap: original,
        })
    }
}
impl<P: ProcessFamily> OwnedHost<P> {
    pub(super) fn pid(&self) -> u32 {
        self.owner.process().identity().pid
    }
    pub(super) fn deadline(&self) -> Deadline {
        self.original
    }
    /// Borrow the original launch identity; no PID discovery or replacement ownership.
    pub(super) fn original_identity(&self) -> &kinekino_process::domain::identity::ProcessIdentity {
        self.owner.process().identity()
    }
    /// Observe the next complete original stdout record without consuming this GUI owner.
    /// An error leaves the caller holding the same owner for its existing closeout path.
    pub(super) fn next_observation_record(
        &mut self,
        maximum: usize,
        requested: Deadline,
    ) -> io::Result<Vec<u8>> {
        let deadline = self.original.constrained_by(requested);
        let mut observation = RecordObservation::default();
        let record = self
            .owner
            .wait_stdout_record_observed(
                &mut self.cursor,
                maximum,
                deadline,
                &SystemClock,
                &mut observation,
            )
            .map_err(|failure| failure.error)?;
        let bytes = match record {
            RecordPoll::Record { bytes, .. } => bytes,
            RecordPoll::Pending => {
                return Err(io::Error::other("Original observation remained pending"));
            }
        };
        let receipt = self.owner.poll_state().map_err(|failure| failure.error)?;
        if receipt.child_exit.is_some() || !matches!(receipt.family, FamilyObservation::Active(_)) {
            return Err(io::Error::other(
                "Original GUI is no longer alive in its owned family",
            ));
        }
        write_record_observation(
            &self
                .root
                .join(format!("owned/observation-{}.json", self.cursor.offset())),
            &observation,
        )?;
        if deadline.operation_remaining(Instant::now()).is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Original observation cutoff expired",
            ));
        }
        Ok(bytes)
    }
    pub(super) fn active_cap(&self) -> Deadline {
        self.active_cap
    }
    pub(super) fn ready<T>(
        self,
        requested: Deadline,
        legacy: &Path,
        validate: impl FnOnce(&[u8]) -> io::Result<T>,
    ) -> Result<(Self, T), Box<CliFailure<P>>> {
        self.ready_inner(requested, legacy, validate, false)
    }
    pub(super) fn ready_observed<T>(
        self,
        requested: Deadline,
        legacy: &Path,
        validate: impl FnOnce(&[u8]) -> io::Result<T>,
    ) -> Result<(Self, T), Box<CliFailure<P>>> {
        self.ready_inner(requested, legacy, validate, true)
    }
    fn ready_inner<T>(
        mut self,
        requested: Deadline,
        legacy: &Path,
        validate: impl FnOnce(&[u8]) -> io::Result<T>,
        observed: bool,
    ) -> Result<(Self, T), Box<CliFailure<P>>> {
        let deadline = self.original.constrained_by(requested);
        self.active_cap = deadline;
        let (result, observation_error) = if observed {
            let mut observation = RecordObservation::default();
            let result = self.owner.wait_stdout_record_observed(
                &mut self.cursor,
                RECORD_LIMIT,
                deadline,
                &SystemClock,
                &mut observation,
            );
            let error = write_record_observation(
                &self.root.join("owned/ready-observation.json"),
                &observation,
            )
            .err();
            (result, error)
        } else {
            (
                self.owner.wait_stdout_record(
                    &mut self.cursor,
                    RECORD_LIMIT,
                    deadline,
                    &SystemClock,
                ),
                None,
            )
        };
        let record = match result {
            Ok(RecordPoll::Record { offset, bytes }) => (offset, bytes),
            Ok(RecordPoll::Pending) => {
                let mut failure = self.fail(
                    "ready record",
                    io::Error::other("Ready remained pending"),
                    deadline,
                );
                if let Some(error) = observation_error {
                    failure
                        .secondary
                        .push(format!("ready observation evidence: {error}"));
                }
                return Err(failure);
            }
            Err(error) => {
                let mut failure = self.fail("ready record", error.error, deadline);
                if let Some(error) = observation_error {
                    failure
                        .secondary
                        .push(format!("ready observation evidence: {error}"));
                }
                return Err(failure);
            }
        };
        if let Some(error) = observation_error {
            return Err(self.fail("ready observation evidence", error, deadline));
        }
        if let Err(error) = write_json(
            &self.root.join("owned/ready-offset.json"),
            &json!({"offset":record.0,"next_offset":self.cursor.offset(),"bytes":record.1.len()}),
        )
        .and_then(|_| fs::write(legacy, &record.1))
        {
            return Err(self.fail("ready evidence", error, deadline));
        }
        let parsed = match validate(&record.1) {
            Ok(value) => value,
            Err(error) => return Err(self.fail("ready validation", error, deadline)),
        };
        match self.owner.poll_state() {
            Ok(receipt)
                if receipt.child_exit.is_none()
                    && matches!(receipt.family, FamilyObservation::Active(_)) => {}
            Ok(_) => {
                return Err(self.fail(
                    "ready ownership",
                    io::Error::other("Ready child not observed alive in owned family"),
                    deadline,
                ));
            }
            Err(error) => return Err(self.fail("ready ownership", error.error, deadline)),
        }
        if deadline.operation_remaining(Instant::now()).is_zero() {
            return Err(self.fail(
                "ready deadline",
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Ready processing exceeded original phase cutoff",
                ),
                deadline,
            ));
        }
        Ok((self, parsed))
    }
    pub(super) fn export_stderr(&self, legacy: &Path, terminal: bool) -> io::Result<()> {
        let value = export_raw(&self.owner.evidence().stderr, legacy, 0, 8192, terminal)?;
        write_json(&self.root.join("owned/stderr-export.json"), &value)?;
        if self
            .active_cap
            .operation_remaining(Instant::now())
            .is_zero()
        {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Ready stderr export exceeded original cutoff",
            ));
        }
        Ok(())
    }
    pub(super) fn fail(
        self,
        stage: &'static str,
        error: io::Error,
        requested: Deadline,
    ) -> Box<CliFailure<P>> {
        CliFailure::live(stage, error, &self.root, self.owner, requested)
    }
    pub(super) fn finish(
        self,
        cap: Duration,
        legacy_stop: &Path,
        legacy_stderr: &Path,
    ) -> Result<HostTerminal, Box<CliFailure<P>>> {
        let original = self.original;
        let deadline = match phase_deadline(original, cap) {
            Ok(value) => value,
            Err(error) => return Err(self.fail("stop budget", error, original)),
        };
        let paths = self.owner.evidence().clone();
        let offset = self.cursor.offset();
        let root = self.root.clone();
        let mut sink = Sink(root.join("owned/receipt.json"));
        let receipt = match self.owner.finish(deadline, &SystemClock, &mut sink) {
            FinishOutcome::Complete(value) => value.receipt().clone(),
            FinishOutcome::Incomplete(value) => {
                let mut error = CliFailure::live(
                    "host finish",
                    value.cause.error,
                    &root,
                    value.owner,
                    deadline,
                );
                if let Some(cause) = value.termination_error {
                    error.secondary.push(format!("termination: {cause}"));
                }
                if let Some(cause) = value.receipt_error {
                    error.secondary.push(format!("receipt: {cause}"));
                }
                return Err(error);
            }
        };
        let failed = |stage, error| CliFailure::terminal(stage, error, &root, receipt.clone());
        let tail = export_raw(&paths.stdout, legacy_stop, offset, RECORD_LIMIT, true)
            .map_err(|error| failed("stop stdout", error))?;
        export_raw(&paths.stderr, legacy_stderr, 0, 8192, true)
            .and_then(|value| write_json(&root.join("owned/stderr-export.json"), &value))
            .map_err(|error| failed("stop stderr", error))?;
        write_json(&root.join("owned/stop-export.json"), &tail)
            .map_err(|error| failed("stop metadata", error))?;
        if tail["limit_exceeded"] == true
            || receipt.forced
            || deadline.remaining(Instant::now()).is_zero()
        {
            return Err(failed(
                "host stop observation",
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Forced, oversized or late host closeout is not graceful ACK",
                ),
            ));
        }
        let bytes = fs::read(legacy_stop).map_err(|error| failed("stop bytes", error))?;
        if deadline.remaining(Instant::now()).is_zero() {
            return Err(failed(
                "stop bytes deadline",
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Original closeout deadline elapsed",
                ),
            ));
        }
        Ok(HostTerminal {
            receipt,
            tail: bytes,
            deadline,
        })
    }
}
fn write_record_observation(path: &Path, observation: &RecordObservation) -> io::Result<()> {
    use std::io::Write;
    let phase = |value: PhaseObservation| json!({"calls":value.calls,"total_us":value.total_us,"max_us":value.max_us});
    let value = json!({"schema_version":1,"iterations":observation.iterations,
        "limit_check":phase(observation.limit_check),"record_poll":phase(observation.record_poll),"child_poll":phase(observation.child_poll),
        "offset_before":observation.offset_before,"offset_after":observation.offset_after,
        "last_record":format!("{:?}",observation.last_record),"exit":format!("{:?}",observation.exit),
        "elapsed_us":observation.elapsed_us,"has_unknown":observation.has_unknown(),"host_cause_established":false});
    let mut bytes = kinekino_player::application::local_host::encode_bounded(&value, 2047)
        .map_err(|error| io::Error::other(format!("{error:?}")))?;
    bytes.push(b'\n');
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    output.write_all(&bytes)?;
    output.flush()
}
pub(super) type ActionOutcome<P = OwnedProcessFamily> =
    Result<Result<(), Box<CliFailure<P>>>, Box<dyn Any + Send>>;
pub(super) fn close_request<P: ProcessFamily>(result: ActionOutcome<P>) -> ActionOutcome<P> {
    match result {
        Ok(Err(error)) => Ok(Err(error.closeout())),
        other => other,
    }
}
pub(super) struct ScenarioFailure<P = OwnedProcessFamily> {
    request: Option<Box<CliFailure<P>>>,
    host: Option<Box<CliFailure<P>>>,
    panic: Option<Box<dyn Any + Send>>,
    host_receipt: Option<Box<ProcessReceipt>>,
    evidence_error: Option<io::Error>,
}
impl<P> fmt::Debug for ScenarioFailure<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let panic = self.panic.as_ref().map(|value| {
            value
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| value.downcast_ref::<&str>().copied())
                .unwrap_or("non-string panic")
        });
        f.debug_struct("ScenarioFailure")
            .field("request", &self.request)
            .field("host", &self.host)
            .field("panic", &panic)
            .field("host_receipt", &self.host_receipt)
            .field("evidence_error", &self.evidence_error)
            .finish()
    }
}
impl<P: ProcessFamily> ScenarioFailure<P> {
    pub(super) fn host(error: Box<CliFailure<P>>) -> Self {
        Self {
            request: None,
            host: Some(close_host(error)),
            panic: None,
            host_receipt: None,
            evidence_error: None,
        }
    }
    pub(super) fn conclude(
        action: ActionOutcome<P>,
        stopped: Result<ProcessReceipt, Box<CliFailure<P>>>,
        evidence_error: Option<io::Error>,
    ) -> Result<(), Self> {
        let (request, panic) = match action {
            Ok(Ok(())) => (None, None),
            Ok(Err(error)) => (Some(error), None),
            Err(error) => (None, Some(error)),
        };
        let (host, host_receipt) = match stopped {
            Ok(receipt) => (None, Some(Box::new(receipt))),
            Err(error) => (Some(close_host(error)), None),
        };
        if request.is_none() && panic.is_none() && host.is_none() && evidence_error.is_none() {
            Ok(())
        } else {
            Err(Self {
                request,
                host,
                panic,
                host_receipt,
                evidence_error,
            })
        }
    }
}
fn close_host<P: ProcessFamily>(error: Box<CliFailure<P>>) -> Box<CliFailure<P>> {
    let mut error = error.closeout();
    let root = error.evidence.clone();
    let preserve = || -> io::Result<()> {
        let offset = fs::read(root.join("owned/ready-offset.json"))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
            .and_then(|value| value["next_offset"].as_u64())
            .unwrap_or(0);
        if let Ok(mut file) = fs::File::open(root.join("stdout.bin")) {
            file.seek(SeekFrom::Start(offset))?;
            let mut bytes = Vec::new();
            file.take(RECORD_LIMIT as u64 + 1).read_to_end(&mut bytes)?;
            fs::write(root.join("logs/native-failure-stop.stdout"), bytes)?;
        }
        write_json(
            &root.join("logs/native-failure-cleanup.json"),
            &json!({"schema_version":2,"forced_owned_child_stop":"not_applicable_owned_job","wait_succeeded":"not_applicable_owned_handle_poll","failure":error.observation()}),
        )
    };
    if let Err(cause) = preserve() {
        error
            .secondary
            .push(format!("native failure aliases: {cause}"));
    }
    error
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinekino_process::domain::identity::ProcessIdentity;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    fn root() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "bounded-endpoint-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("logs")).unwrap();
        root
    }
    struct Family {
        deadline: Deadline,
        identity: ProcessIdentity,
        closed: bool,
        unknown: bool,
        drops: Arc<AtomicUsize>,
    }
    impl Drop for Family {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }
    impl ProcessFamily for Family {
        fn deadline(&self) -> Deadline {
            self.deadline
        }
        fn identity(&self) -> &ProcessIdentity {
            &self.identity
        }
        fn close_stdin(&mut self) -> io::Result<()> {
            self.closed = true;
            Ok(())
        }
        fn poll_child(&mut self) -> io::Result<Option<u32>> {
            Ok(if self.closed { Some(0) } else { None })
        }
        fn poll_family(&mut self) -> io::Result<FamilyObservation> {
            if self.unknown {
                Err(io::Error::other("original family query failure"))
            } else if self.closed {
                Ok(FamilyObservation::Empty)
            } else {
                Ok(FamilyObservation::Active(1))
            }
        }
        fn terminate_owned_family(&mut self) -> io::Result<()> {
            Err(io::Error::other("termination unconfirmed"))
        }
    }
    fn fake(root: &Path, unknown: bool, drops: Arc<AtomicUsize>) -> OwnedHost<Family> {
        fs::create_dir(root.join("owned")).unwrap();
        let prepared = PreparedEvidence::create(&root.join("owned"), None, 0).unwrap();
        let original = Deadline::new(
            Instant::now(),
            Duration::from_secs(20),
            Duration::from_secs(2),
        )
        .unwrap();
        OwnedHost {
            owner: Supervisor::new(
                Family {
                    deadline: original,
                    identity: ProcessIdentity {
                        pid: 123,
                        creation_filetime: Some(456),
                        executable: "fake-only".into(),
                    },
                    closed: false,
                    unknown,
                    drops,
                },
                prepared.paths.clone(),
                STREAM_LIMIT,
            ),
            cursor: RecordCursor::default(),
            root: root.into(),
            original,
            active_cap: original,
        }
    }
    #[test]
    fn ready_observation_is_explicit_bounded_and_preserves_original_owner() {
        kinekino_platform::install();
        for observed in [false, true] {
            let root = root();
            let drops = Arc::new(AtomicUsize::new(0));
            let host = fake(&root, false, drops.clone());
            let original = host.deadline();
            fs::write(root.join("owned/stdout.raw"), b"ready\n").unwrap();
            let result = if observed {
                host.ready_observed(original, &root.join("logs/ready.stdout"), |bytes| {
                    Ok(bytes.to_vec())
                })
            } else {
                host.ready(original, &root.join("logs/ready.stdout"), |bytes| {
                    Ok(bytes.to_vec())
                })
            };
            let (host, bytes) = result.unwrap();
            assert_eq!(bytes, b"ready\n");
            assert_eq!(host.pid(), 123);
            assert_eq!(drops.load(Ordering::SeqCst), 0);
            let path = root.join("owned/ready-observation.json");
            assert_eq!(path.exists(), observed);
            if observed {
                let bytes = fs::read(&path).unwrap();
                assert!(bytes.len() <= 2048);
                let value: Value = serde_json::from_slice(&bytes).unwrap();
                assert_eq!(value["exit"], "Record");
                assert_eq!(value["offset_after"], 6);
                assert_eq!(value["child_poll"]["calls"], 0);
                assert_eq!(value["host_cause_established"], false);
                assert!(write_record_observation(&path, &RecordObservation::default()).is_err());
            }
            drop(host);
            assert_eq!(drops.load(Ordering::SeqCst), 1);
            fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn ready_observation_evidence_error_remains_secondary_to_original_cutoff() {
        kinekino_platform::install();
        let root = root();
        let drops = Arc::new(AtomicUsize::new(0));
        let host = fake(&root, false, drops.clone());
        fs::write(root.join("owned/ready-observation.json"), b"original proof").unwrap();
        let expired = Deadline::new(
            Instant::now() - Duration::from_secs(3),
            Duration::from_secs(2),
            Duration::from_secs(1),
        )
        .unwrap();
        let failure = host
            .ready_observed(expired, &root.join("logs/ready.stdout"), |_| Ok(()))
            .err()
            .unwrap();
        assert_eq!(failure.stage, "ready record");
        assert_eq!(
            failure.error.to_string(),
            "record observation budget exhausted"
        );
        assert_eq!(failure.secondary.len(), 1);
        assert!(matches!(failure.ownership, Ownership::Live(_)));
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        assert_eq!(
            fs::read(root.join("owned/ready-observation.json")).unwrap(),
            b"original proof"
        );
        drop(failure);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn elapsed_parent_deadline_is_not_renewed_by_ready_or_request_cap() {
        kinekino_platform::install();
        let original = Deadline::new(
            Instant::now() - Duration::from_secs(10),
            Duration::from_secs(2),
            Duration::from_secs(1),
        )
        .unwrap();
        let cap = phase_deadline(original, Duration::from_secs(30)).unwrap();
        assert!(cap.remaining(Instant::now()).is_zero());
        let error = evidence::run_process_until(
            PathBuf::from(crate::cli_exe::kinekino_cli_exe()),
            Path::new("unused"),
            b"{}",
            None,
            original,
            Duration::from_secs(30),
            false,
        )
        .err()
        .unwrap();
        assert!(matches!(error.ownership, Ownership::NotSpawned));
    }
    #[test]
    fn panic_and_secondary_closeout_error_are_both_retained() {
        kinekino_platform::install();
        let action = Err(Box::new("original assertion") as Box<dyn Any + Send>);
        let stop: Result<ProcessReceipt, Box<CliFailure>> = Err(CliFailure::not_spawned(
            "stop failure",
            io::Error::other("secondary"),
            Path::new("missing-evidence"),
        ));
        let error = ScenarioFailure::conclude(action, stop, None).unwrap_err();
        assert!(error.panic.is_some());
        assert_eq!(error.host.as_ref().unwrap().stage, "stop failure");
    }
    #[test]
    fn ready_and_terminal_offsets_keep_legacy_bytes_and_real_family_observation_separate() {
        kinekino_platform::install();
        let root = root();
        let drops = Arc::new(AtomicUsize::new(0));
        let host = fake(&root, false, drops.clone());
        fs::write(
            root.join("owned/stdout.raw"),
            b"{\"event\":\"ready\"}\n{\"event\":\"stopped\"}\n",
        )
        .unwrap();
        let ready = phase_deadline(host.deadline(), Duration::from_secs(8)).unwrap();
        let (host, value) = host
            .ready(ready, &root.join("logs/ready.stdout"), |bytes| {
                serde_json::from_slice::<Value>(bytes).map_err(io::Error::other)
            })
            .map_err(|error| error.closeout())
            .unwrap();
        assert_eq!(value["event"], "ready");
        let terminal = host
            .finish(
                Duration::from_secs(8),
                &root.join("logs/stop.stdout"),
                &root.join("logs/host.stderr"),
            )
            .map_err(|error| error.closeout())
            .unwrap();
        assert_eq!(terminal.tail, b"{\"event\":\"stopped\"}\n");
        assert_eq!(terminal.receipt.family, FamilyObservation::Empty);
        assert!(!terminal.receipt.forced);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn bad_ready_and_evidence_failure_return_original_owner_before_any_drop() {
        kinekino_platform::install();
        for bad_json in [true, false] {
            let root = root();
            let drops = Arc::new(AtomicUsize::new(0));
            let host = fake(&root, false, drops.clone());
            fs::write(root.join("owned/stdout.raw"), b"not-json\n").unwrap();
            let legacy = root.join("logs/ready.stdout");
            if !bad_json {
                fs::create_dir(&legacy).unwrap();
            }
            let deadline = phase_deadline(host.deadline(), Duration::from_secs(8)).unwrap();
            let error = host
                .ready(deadline, &legacy, |bytes| {
                    serde_json::from_slice::<Value>(bytes).map_err(io::Error::other)
                })
                .err()
                .unwrap();
            assert!(matches!(error.ownership, Ownership::Live(_)));
            assert_eq!(drops.load(Ordering::SeqCst), 0);
            let error = error.closeout();
            assert!(matches!(error.ownership, Ownership::ObservedTerminal(_)));
            assert_eq!(drops.load(Ordering::SeqCst), 1);
        }
    }
    #[test]
    fn request_and_host_incomplete_keep_both_original_owners_and_first_errors() {
        kinekino_platform::install();
        let request_root = root();
        let host_root = root();
        let drops = Arc::new(AtomicUsize::new(0));
        let request = fake(&request_root, true, drops.clone());
        let deadline = request.deadline();
        let request = request.fail(
            "original request",
            io::Error::other("first error"),
            deadline,
        );
        let host = fake(&host_root, true, drops.clone());
        let deadline = host.deadline();
        let host = host.fail(
            "original host",
            io::Error::other("secondary host error"),
            deadline,
        );
        let failure = ScenarioFailure::conclude(close_request(Ok(Err(request))), Err(host), None)
            .unwrap_err();
        assert!(matches!(
            failure.request.as_ref().unwrap().ownership,
            Ownership::Live(_)
        ));
        assert!(matches!(
            failure.host.as_ref().unwrap().ownership,
            Ownership::Live(_)
        ));
        assert_eq!(
            failure.request.as_ref().unwrap().error.to_string(),
            "first error"
        );
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(failure);
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
    #[test]
    fn live_stderr_alias_is_explicitly_a_snapshot_not_a_child_writer() {
        kinekino_platform::install();
        let root = root();
        let host = fake(&root, false, Arc::new(AtomicUsize::new(0)));
        fs::write(root.join("owned/stderr.raw"), b"original diagnostic").unwrap();
        host.export_stderr(&root.join("logs/host.stderr"), false)
            .unwrap();
        let value: Value =
            serde_json::from_slice(&fs::read(root.join("owned/stderr-export.json")).unwrap())
                .unwrap();
        assert_eq!(value["complete"], false);
        assert_eq!(value["live_writer"], false);
        let deadline = host.deadline();
        let _failure = host
            .fail("test teardown", io::Error::other("fixture end"), deadline)
            .closeout();
    }
    #[test]
    fn no_media_jobs_host_uses_finite_request_and_owned_eof_without_fake_join()
    -> Result<(), ScenarioFailure> {
        kinekino_platform::install();
        use super::super::fixture::{Fixture, request};
        use std::panic::{AssertUnwindSafe, catch_unwind};
        let mut fixture = Fixture::named("bounded-jobs-owner");
        fixture.keep_evidence();
        let original = Deadline::new(
            Instant::now(),
            Duration::from_secs(30),
            Duration::from_secs(5),
        )
        .unwrap();
        let ready = phase_deadline(original, Duration::from_secs(10)).unwrap();
        let host = OwnedHost::launch(
            fixture.cli.clone(),
            vec![
                "--serve-jobs".into(),
                "--job-root".into(),
                fixture.root.join("store").into_os_string(),
            ],
            &fixture.root,
            original,
        )
        .map_err(ScenarioFailure::host)?;
        let (host, endpoint) = host
            .ready(
                ready,
                &fixture.root.join("logs/bounded-ready.stdout"),
                |bytes| {
                    let value: Value = serde_json::from_slice(bytes).map_err(io::Error::other)?;
                    if value["event"] != "job_host_ready" {
                        return Err(io::Error::other("Unexpected jobs readiness"));
                    }
                    value["endpoint"]
                        .as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| io::Error::other("Missing endpoint"))
                },
            )
            .map_err(ScenarioFailure::host)?;
        let action = catch_unwind(AssertUnwindSafe(|| {
            let response = fixture.try_call_until(
                "bounded-version",
                &request("version", json!({})),
                &endpoint,
                original,
                Duration::from_secs(10),
                0,
            )?;
            assert_eq!(response["ok"], true);
            Ok(())
        }));
        let action = close_request(action);
        let stopped = host
            .finish(
                Duration::from_secs(10),
                &fixture.root.join("logs/bounded-stop.stdout"),
                &fixture.root.join("logs/bounded-host.stderr"),
            )
            .and_then(|terminal| {
                let failed = |error| {
                    CliFailure::terminal(
                        "jobs stop validation",
                        error,
                        &fixture.root,
                        terminal.receipt.clone(),
                    )
                };
                let value: Value = serde_json::from_slice(&terminal.tail)
                    .map_err(|error| failed(io::Error::other(error)))?;
                if terminal.receipt.child_exit != Some(0)
                    || value["event"] != "job_host_stopped"
                    || value["local_host_shutdown"]["pipe"] != "joined"
                    || value["local_host_shutdown"]["sdk"] != "joined"
                {
                    return Err(failed(io::Error::other("Original jobs stop proof missing")));
                }
                Ok(terminal.receipt)
            });
        ScenarioFailure::conclude(action, stopped, None)
    }
    #[test]
    #[ignore = "Explicit centrally allocated owned-process-test-child Windows failure gate; no browser/media"]
    fn windows_missing_partial_malformed_exit_and_live_descendant_keep_original_families() {
        kinekino_platform::install();
        let executable = PathBuf::from(
            std::env::var_os("OWNED_PROCESS_TEST_CHILD")
                .expect("Pinned existing Rust child fixture path"),
        );
        let expected = std::env::var("OWNED_PROCESS_TEST_CHILD_SHA256")
            .expect("Pinned Rust child fixture SHA256");
        let actual = kinekino_player::infrastructure::content_digest::sha256(
            &mut fs::File::open(&executable).unwrap(),
            1024 * 1024 * 1024,
        )
        .unwrap()
        .1;
        assert_eq!(actual, expected);
        for mode in ["silent", "partial", "malformed", "exit", "parent-exit"] {
            let root = root();
            let original = Deadline::new(
                Instant::now(),
                Duration::from_secs(10),
                Duration::from_secs(2),
            )
            .unwrap();
            let ready = phase_deadline(original, Duration::from_secs(4)).unwrap();
            let host = OwnedHost::launch(executable.clone(), vec![mode.into()], &root, original)
                .map_err(|error| error.closeout())
                .unwrap();
            let error = if mode == "parent-exit" {
                host.finish(
                    Duration::from_secs(4),
                    &root.join("logs/stop.stdout"),
                    &root.join("logs/host.stderr"),
                )
                .err()
                .expect("Parent exit cannot prove descendant quiescence")
            } else {
                host.ready(ready, &root.join("logs/ready.stdout"), |bytes| {
                    serde_json::from_slice::<Value>(bytes).map_err(io::Error::other)
                })
                .err()
                .expect("Invalid readiness cannot publish host")
            };
            let error = error.closeout();
            assert!(!matches!(error.ownership, Ownership::NotSpawned));
        }
    }
}
