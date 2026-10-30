//! Test evidence adapter. Process-family state and deadlines belong to owned-process.
use kinekino_process::{
    application::supervisor::{FinishOutcome, ProcessFamily, ReceiptSink, Supervisor, SystemClock},
    domain::{deadline::Deadline, observation::ProcessReceipt},
    infrastructure::{
        evidence_files::{EvidencePaths, PreparedEvidence, RawFile},
        family::{OwnedProcessFamily, SpawnSpec, spawn},
    },
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fmt, fs,
    io::{self, Read},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub const CLEANUP_RESERVE: Duration = Duration::from_secs(2);
const STDOUT_LIMIT: usize = kinekino_player::application::local_host::MAX_RESPONSE_BYTES;
const STDERR_LIMIT: usize = 8192;
const RAW_LIMIT: u64 = STDOUT_LIMIT as u64 + 1;

#[must_use = "Explicitly close out or retain the original failed owner"]
pub enum Ownership<P> {
    NotSpawned,
    Live(Box<Supervisor<P>>),
    ObservedTerminal(ProcessReceipt),
}
#[must_use = "A failure is never a successful CLI tuple or cleanup ACK"]
pub struct CliFailure<P = OwnedProcessFamily> {
    pub stage: &'static str,
    pub error: io::Error,
    pub secondary: Vec<String>,
    pub ownership: Ownership<P>,
    pub evidence: PathBuf,
    pub streams: Value,
    deadline: Option<Deadline>,
}
impl<P> fmt::Debug for CliFailure<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ownership = match &self.ownership {
            Ownership::NotSpawned => "not_spawned",
            Ownership::Live(_) => "live_owner_retained",
            Ownership::ObservedTerminal(_) => "observed_terminal",
        };
        f.debug_struct("CliFailure")
            .field("stage", &self.stage)
            .field("error", &self.error)
            .field("secondary", &self.secondary)
            .field("ownership", &ownership)
            .field("evidence", &self.evidence)
            .finish()
    }
}
impl<P> CliFailure<P> {
    pub fn not_spawned(stage: &'static str, error: io::Error, evidence: &Path) -> Box<Self> {
        Box::new(Self {
            stage,
            error,
            secondary: Vec::new(),
            ownership: Ownership::NotSpawned,
            evidence: evidence.into(),
            streams: Value::Null,
            deadline: None,
        })
    }
    pub fn terminal(
        stage: &'static str,
        error: io::Error,
        evidence: &Path,
        receipt: ProcessReceipt,
    ) -> Box<Self> {
        Box::new(Self {
            stage,
            error,
            secondary: Vec::new(),
            ownership: Ownership::ObservedTerminal(receipt),
            evidence: evidence.into(),
            streams: Value::Null,
            deadline: None,
        })
    }
}
impl<P: ProcessFamily> CliFailure<P> {
    pub fn live(
        stage: &'static str,
        error: io::Error,
        evidence: &Path,
        owner: Supervisor<P>,
        deadline: Deadline,
    ) -> Box<Self> {
        let deadline = owner.process().deadline().constrained_by(deadline);
        let mut failure = Self::not_spawned(stage, error, evidence);
        failure.ownership = Ownership::Live(Box::new(owner));
        failure.deadline = Some(deadline);
        failure
    }
    /// Same original cutoff; even successful cleanup preserves the original failure.
    pub fn closeout(mut self: Box<Self>) -> Box<Self> {
        match std::mem::replace(&mut self.ownership, Ownership::NotSpawned) {
            Ownership::Live(owner) => {
                let deadline = self.deadline.unwrap_or_else(|| owner.process().deadline());
                let paths = owner.evidence().clone();
                let mut sink = JsonReceipt(self.evidence.join("owned/receipt.json"));
                match owner.finish(deadline, &SystemClock, &mut sink) {
                    FinishOutcome::Complete(completion) => {
                        self.ownership = Ownership::ObservedTerminal(completion.receipt().clone());
                    }
                    FinishOutcome::Incomplete(incomplete) => {
                        self.secondary
                            .push(format!("closeout: {:?}", incomplete.cause));
                        if let Some(error) = incomplete.termination_error {
                            self.secondary.push(format!("termination: {error}"));
                        }
                        if let Some(error) = incomplete.receipt_error {
                            self.secondary.push(format!("receipt: {error}"));
                        }
                        self.ownership = Ownership::Live(Box::new(incomplete.owner));
                    }
                }
                let mut partial = |file: &RawFile, name: &str, maximum| match export_prefix(
                    file,
                    &self.evidence.join(name),
                    maximum,
                ) {
                    Ok((_, value)) => value,
                    Err(error) => {
                        self.secondary.push(format!("closeout {name}: {error}"));
                        json!({"state":"unavailable","error":error.to_string()})
                    }
                };
                let stdout = partial(&paths.stdout, "stdout.bin", STDOUT_LIMIT);
                let stderr = partial(&paths.stderr, "stderr.bin", STDERR_LIMIT);
                self.streams = json!({"stdout":stdout,"stderr":stderr});
            }
            ownership => self.ownership = ownership,
        }
        if let Err(error) = write_json(&self.evidence.join("failure.json"), &self.observation()) {
            self.secondary.push(format!("failure evidence: {error}"));
        }
        self
    }
    pub fn observation(&self) -> Value {
        let (state, receipt) = match &self.ownership {
            Ownership::NotSpawned => ("not_spawned", Value::Null),
            Ownership::Live(owner) => ("live_owner_retained", receipt_json(owner.receipt())),
            Ownership::ObservedTerminal(receipt) => ("observed_terminal", receipt_json(receipt)),
        };
        json!({"stage":self.stage,"error":self.error.to_string(),"secondary":self.secondary,"ownership":state,"owned_family":receipt,"streams":self.streams})
    }
}
pub fn write_json(path: &Path, value: &Value) -> io::Result<()> {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(io::Error::other)?,
    )
}
pub fn receipt_json(receipt: &ProcessReceipt) -> Value {
    json!({"identity":{"pid":receipt.identity.pid,"creation_filetime":receipt.identity.creation_filetime,"executable":receipt.identity.executable},"child_exit":receipt.child_exit,"family":format!("{:?}",receipt.family),"forced":receipt.forced,"stdin_eof_requested":receipt.stdin_eof_requested})
}
struct JsonReceipt(PathBuf);
impl ReceiptSink for JsonReceipt {
    fn record(&mut self, receipt: &ProcessReceipt) -> io::Result<()> {
        write_json(&self.0, &receipt_json(receipt))
    }
}
pub struct ProcessOutput {
    pub deadline: Deadline,
    pub receipt: ProcessReceipt,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stdout_evidence: Value,
    pub stderr_evidence: Value,
    pub elapsed_ms: u128,
}
impl ProcessOutput {
    pub fn check_deadline(&self, root: &Path) -> Result<(), Box<CliFailure>> {
        if self.deadline.remaining(Instant::now()).is_zero() {
            return Err(self.failure(
                "post-process deadline",
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    "original observation deadline elapsed",
                ),
                root,
            ));
        }
        Ok(())
    }
    pub fn failure(&self, stage: &'static str, error: io::Error, root: &Path) -> Box<CliFailure> {
        CliFailure::terminal(stage, error, root, self.receipt.clone())
    }
}
fn environment(clear_path: bool) -> io::Result<Option<BTreeMap<String, OsString>>> {
    if !clear_path {
        return Ok(None);
    }
    explicit_environment(std::env::vars_os()).map(Some)
}
fn explicit_environment(
    entries: impl IntoIterator<Item = (OsString, OsString)>,
) -> io::Result<BTreeMap<String, OsString>> {
    let mut result = BTreeMap::new();
    for (name, value) in entries {
        let name = name.into_string().map_err(|name| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                format!("non-UTF8 environment name: {name:?}"),
            )
        })?;
        if !name.is_ascii() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("non-ASCII environment name: {name:?}"),
            ));
        }
        if !name.eq_ignore_ascii_case("PATH") {
            result.insert(name, value);
        }
    }
    result.insert("PATH".into(), OsString::new());
    Ok(result)
}
pub fn run_process(
    executable: PathBuf,
    root: &Path,
    input: &[u8],
    endpoint: Option<&str>,
    total: Duration,
    clear_path: bool,
) -> Result<ProcessOutput, Box<CliFailure>> {
    let mut arguments = vec![OsString::from("--input"), OsString::from("-")];
    if let Some(endpoint) = endpoint {
        arguments.extend([OsString::from("--endpoint"), OsString::from(endpoint)]);
    }
    run_process_with_arguments(executable, root, input, arguments, total, clear_path)
}
pub fn run_process_until(
    executable: PathBuf,
    root: &Path,
    input: &[u8],
    endpoint: Option<&str>,
    original: Deadline,
    cap: Duration,
    clear_path: bool,
) -> Result<ProcessOutput, Box<CliFailure>> {
    let started = Instant::now();
    let requested = Deadline::new(started, cap, CLEANUP_RESERVE)
        .map_err(|error| CliFailure::not_spawned("request budget", error, root))?;
    let deadline = original.constrained_by(requested);
    if deadline.operation_remaining(started).is_zero() {
        return Err(CliFailure::not_spawned(
            "request budget",
            io::Error::new(
                io::ErrorKind::TimedOut,
                "Original operation deadline exhausted",
            ),
            root,
        ));
    }
    run_process_at(
        executable, root, input, endpoint, deadline, clear_path, started,
    )
}
fn run_process_at(
    executable: PathBuf,
    root: &Path,
    input: &[u8],
    endpoint: Option<&str>,
    deadline: Deadline,
    clear_path: bool,
    started: Instant,
) -> Result<ProcessOutput, Box<CliFailure>> {
    let mut arguments = vec![OsString::from("--input"), OsString::from("-")];
    if let Some(endpoint) = endpoint {
        arguments.extend([OsString::from("--endpoint"), OsString::from(endpoint)]);
    }
    run_arguments_at(
        executable, root, input, arguments, deadline, clear_path, started,
    )
}

/// Uses the same owned family/evidence path for standalone CLI root arguments.
pub fn run_process_with_arguments(
    executable: PathBuf,
    root: &Path,
    input: &[u8],
    arguments: Vec<OsString>,
    total: Duration,
    clear_path: bool,
) -> Result<ProcessOutput, Box<CliFailure>> {
    let started = Instant::now();
    let deadline = Deadline::new(started, total, CLEANUP_RESERVE)
        .map_err(|error| CliFailure::not_spawned("budget", error, root))?;
    run_arguments_at(
        executable, root, input, arguments, deadline, clear_path, started,
    )
}

fn run_arguments_at(
    executable: PathBuf,
    root: &Path,
    input: &[u8],
    arguments: Vec<OsString>,
    deadline: Deadline,
    clear_path: bool,
    started: Instant,
) -> Result<ProcessOutput, Box<CliFailure>> {
    let before = |stage, error| CliFailure::not_spawned(stage, error, root);
    let environment = environment(clear_path).map_err(|error| before("environment", error))?;
    let cwd = std::env::current_dir().map_err(|error| before("cwd", error))?;
    let owned = root.join("owned");
    fs::create_dir(&owned).map_err(|error| before("evidence directory", error))?;
    let prepared = PreparedEvidence::create(
        &owned,
        Some(input),
        kinekino_player::application::local_host::MAX_REQUEST_BYTES,
    )
    .map_err(|error| before("prepare stdin/raw", error))?;
    let spec = SpawnSpec {
        executable,
        cwd,
        arguments,
        environment,
    };
    let (family, paths) = spawn(spec, prepared, deadline).map_err(|failure| {
        let mut result = before(failure.stage, failure.error);
        if let Some(error) = failure.termination_error {
            result
                .secondary
                .push(format!("launch termination: {error}"));
        }
        if let Some(owner) = failure.owner {
            result.ownership = Ownership::Live(Box::new(Supervisor::new(
                *owner,
                failure.evidence,
                RAW_LIMIT,
            )));
            result.deadline = Some(deadline);
        }
        result
    })?;
    finish_owned(Supervisor::new(family, paths, RAW_LIMIT), root, started)
}
fn export_prefix(
    file: &RawFile,
    destination: &Path,
    maximum: usize,
) -> io::Result<(Vec<u8>, Value)> {
    let reader = file.open_read()?;
    let length = reader.metadata()?.len();
    let mut bytes = Vec::new();
    reader.take(maximum as u64 + 1).read_to_end(&mut bytes)?;
    fs::write(destination, &bytes)?;
    let digest = kinekino_player::infrastructure::content_digest::sha256(
        &mut std::io::Cursor::new(&bytes),
        maximum as u64 + 1,
    )
    .map_err(|error| io::Error::other(format!("{error:?}")))?
    .1;
    let value = json!({"state":"received","file":destination.file_name().and_then(|name|name.to_str()),"bytes":bytes.len(),"sha256":digest,"read_error":null,"limit_exceeded":length>maximum as u64,"complete":length==bytes.len() as u64&&length<=maximum as u64,"origin":file.path(),"offset":0,"observed_length":length});
    Ok((bytes, value))
}
fn finish_owned<P: ProcessFamily>(
    owner: Supervisor<P>,
    root: &Path,
    started: Instant,
) -> Result<ProcessOutput, Box<CliFailure<P>>> {
    let deadline = owner.process().deadline();
    let paths: EvidencePaths = owner.evidence().clone();
    let mut sink = JsonReceipt(root.join("owned/receipt.json"));
    let receipt = match owner.finish(deadline, &SystemClock, &mut sink) {
        FinishOutcome::Complete(completion) => completion.receipt().clone(),
        FinishOutcome::Incomplete(incomplete) => {
            let mut secondary = vec![format!("finish stage: {:?}", incomplete.cause.stage)];
            let mut partial = |file: &RawFile, name: &str, maximum| match export_prefix(
                file,
                &root.join(name),
                maximum,
            ) {
                Ok((_, value)) => value,
                Err(error) => {
                    secondary.push(format!("partial {name}: {error}"));
                    json!({"state":"unavailable","error":error.to_string()})
                }
            };
            let stdout = partial(&paths.stdout, "stdout.bin", STDOUT_LIMIT);
            let stderr = partial(&paths.stderr, "stderr.bin", STDERR_LIMIT);
            if let Some(error) = incomplete.termination_error {
                secondary.push(format!("termination: {error}"));
            }
            if let Some(error) = incomplete.receipt_error {
                secondary.push(format!("receipt: {error}"));
            }
            return Err(Box::new(CliFailure {
                stage: "finish owned family",
                error: incomplete.cause.error,
                secondary,
                ownership: Ownership::Live(Box::new(incomplete.owner)),
                evidence: root.into(),
                streams: json!({"stdout":stdout,"stderr":stderr}),
                deadline: Some(deadline),
            }));
        }
    };
    let terminal_error = |stage, error| CliFailure::terminal(stage, error, root, receipt.clone());
    let (stdout, stdout_evidence) =
        export_prefix(&paths.stdout, &root.join("stdout.bin"), STDOUT_LIMIT)
            .map_err(|error| terminal_error("stdout export", error))?;
    let (stderr, stderr_evidence) =
        export_prefix(&paths.stderr, &root.join("stderr.bin"), STDERR_LIMIT)
            .map_err(|error| terminal_error("stderr export", error))?;
    if receipt.forced || deadline.remaining(Instant::now()).is_zero() {
        return Err(terminal_error(
            "operation deadline",
            io::Error::new(
                io::ErrorKind::TimedOut,
                "owned family required forced termination or evidence completed after deadline",
            ),
        ));
    }
    Ok(ProcessOutput {
        deadline,
        receipt,
        stdout,
        stderr,
        stdout_evidence,
        stderr_evidence,
        elapsed_ms: started.elapsed().as_millis(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinekino_process::domain::{identity::ProcessIdentity, observation::FamilyObservation};
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    fn root() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "snapshot-owned-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("owned")).unwrap();
        root
    }
    struct Family {
        deadline: Deadline,
        identity: ProcessIdentity,
        exit: Option<u32>,
        unknown: bool,
    }
    impl ProcessFamily for Family {
        fn deadline(&self) -> Deadline {
            self.deadline
        }
        fn identity(&self) -> &ProcessIdentity {
            &self.identity
        }
        fn close_stdin(&mut self) -> io::Result<()> {
            Ok(())
        }
        fn poll_child(&mut self) -> io::Result<Option<u32>> {
            Ok(self.exit)
        }
        fn poll_family(&mut self) -> io::Result<FamilyObservation> {
            if self.unknown {
                Err(io::Error::other("original family query failure"))
            } else {
                Ok(FamilyObservation::Empty)
            }
        }
        fn terminate_owned_family(&mut self) -> io::Result<()> {
            Err(io::Error::other("owned termination unconfirmed"))
        }
    }
    fn owner(root: &Path, exit: Option<u32>, unknown: bool) -> Supervisor<Family> {
        let prepared = PreparedEvidence::create(&root.join("owned"), Some(b"{}"), 2).unwrap();
        let deadline = Deadline::new(
            Instant::now(),
            Duration::from_secs(2),
            Duration::from_secs(1),
        )
        .unwrap();
        Supervisor::new(
            Family {
                deadline,
                identity: ProcessIdentity {
                    pid: 123,
                    creation_filetime: Some(456),
                    executable: PathBuf::from("fixture-only"),
                },
                exit,
                unknown,
            },
            prepared.paths.clone(),
            RAW_LIMIT,
        )
    }
    #[test]
    fn finite_file_transport_preserves_prefix_hash_and_nonzero_exit_without_fake_workers() {
        kinekino_platform::install();
        let root = root();
        let owner = owner(&root, Some(7), false);
        fs::write(root.join("owned/stdout.raw"), b"{\"ok\":false}\n").unwrap();
        let result = finish_owned(owner, &root, Instant::now()).unwrap();
        assert_eq!(result.receipt.child_exit, Some(7));
        assert_eq!(fs::read(root.join("stdout.bin")).unwrap(), result.stdout);
        assert_eq!(
            result.stdout_evidence["sha256"],
            super::super::hash(&result.stdout)
        );
        assert!(result.stderr.is_empty());
    }
    #[test]
    fn empty_output_remains_empty_and_export_failure_keeps_terminal_receipt() {
        kinekino_platform::install();
        let root = root();
        let result = finish_owned(owner(&root, Some(0), false), &root, Instant::now()).unwrap();
        assert!(result.stdout.is_empty());
        let root = self::root();
        fs::create_dir(root.join("stdout.bin")).unwrap();
        let error = finish_owned(owner(&root, Some(0), false), &root, Instant::now())
            .err()
            .unwrap();
        assert_eq!(error.stage, "stdout export");
        assert!(matches!(error.ownership, Ownership::ObservedTerminal(_)));
    }
    #[test]
    fn unknown_family_and_receipt_failure_keep_original_owner_and_primary_error() {
        kinekino_platform::install();
        let root = root();
        fs::create_dir(root.join("owned/receipt.json")).unwrap();
        let error = finish_owned(owner(&root, None, true), &root, Instant::now())
            .err()
            .unwrap();
        assert_eq!(error.error.to_string(), "original family query failure");
        assert!(matches!(error.ownership, Ownership::Live(_)));
        let error = error.closeout();
        assert_eq!(error.error.to_string(), "original family query failure");
        assert!(matches!(error.ownership, Ownership::Live(_)));
        assert!(!error.secondary.is_empty());
    }
    #[test]
    fn invalid_budget_and_nonascii_environment_fail_before_spawn_without_dropping_fields() {
        kinekino_platform::install();
        assert!(Deadline::new(Instant::now(), CLEANUP_RESERVE, CLEANUP_RESERVE).is_err());
        let error =
            explicit_environment([(OsString::from("환경"), OsString::from("value"))]).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
        assert!(error.to_string().contains("환경"));
        let env = explicit_environment([
            (OsString::from("Path"), OsString::from("old")),
            (OsString::from("KEEP"), OsString::from("value")),
        ])
        .unwrap();
        assert_eq!(env["PATH"], OsString::new());
        assert_eq!(env["KEEP"], "value");
    }
    #[test]
    fn invalid_budget_returns_not_spawned_and_closeout_cannot_invent_an_owner() {
        kinekino_platform::install();
        let root = root();
        let original = Deadline::new(
            Instant::now() - Duration::from_secs(10),
            Duration::from_secs(3),
            CLEANUP_RESERVE,
        )
        .unwrap();
        let expired = run_process_until(
            root.join("must-never-launch.exe"),
            &root,
            b"{}",
            None,
            original,
            Duration::from_secs(30),
            false,
        )
        .err()
        .unwrap();
        assert!(matches!(expired.ownership, Ownership::NotSpawned));
        assert_eq!(expired.error.kind(), io::ErrorKind::TimedOut);
        assert!(!root.join("owned/stdin.raw").exists());
        let failure = run_process(
            root.join("must-never-launch.exe"),
            &root,
            b"{}",
            None,
            CLEANUP_RESERVE,
            false,
        )
        .err()
        .unwrap();
        assert_eq!(failure.stage, "budget");
        assert!(matches!(failure.ownership, Ownership::NotSpawned));
        let failure = failure.closeout();
        assert!(matches!(failure.ownership, Ownership::NotSpawned));
        assert!(!root.join("owned/stdin.raw").exists());
    }
    #[test]
    fn receipt_write_failure_retains_terminal_child_as_live_owner_until_recorded() {
        kinekino_platform::install();
        let root = root();
        fs::create_dir(root.join("owned/receipt.json")).unwrap();
        let error = finish_owned(owner(&root, Some(0), false), &root, Instant::now())
            .err()
            .unwrap();
        assert!(
            matches!(&error.ownership, Ownership::Live(owner) if owner.receipt().child_exit == Some(0))
        );
        assert!(
            error
                .secondary
                .iter()
                .any(|error| error.contains("Receipt"))
        );
        let error = error.closeout();
        assert!(matches!(error.ownership, Ownership::Live(_)));
    }
    #[test]
    fn elapsed_original_deadline_cannot_be_renewed_by_failure_closeout() {
        kinekino_platform::install();
        let root = root();
        let prepared = PreparedEvidence::create(&root.join("owned"), Some(b"{}"), 2).unwrap();
        let deadline = Deadline::new(
            Instant::now() - Duration::from_secs(1),
            Duration::from_millis(20),
            Duration::from_millis(10),
        )
        .unwrap();
        let family = Family {
            deadline,
            identity: ProcessIdentity {
                pid: 123,
                creation_filetime: Some(456),
                executable: PathBuf::from("fixture-only"),
            },
            exit: None,
            unknown: false,
        };
        let error = finish_owned(
            Supervisor::new(family, prepared.paths.clone(), RAW_LIMIT),
            &root,
            Instant::now(),
        )
        .err()
        .unwrap();
        assert_eq!(error.error.kind(), io::ErrorKind::TimedOut);
        let error = error.closeout();
        assert!(
            matches!(&error.ownership, Ownership::Live(owner) if owner.process().deadline().remaining(Instant::now()).is_zero())
        );
        assert_eq!(error.error.kind(), io::ErrorKind::TimedOut);
    }
    #[test]
    fn stderr_probe_limit_is_independent_of_coarse_spool_limit() {
        kinekino_platform::install();
        let root = root();
        let owner = owner(&root, Some(0), false);
        fs::write(root.join("owned/stderr.raw"), vec![b'x'; STDERR_LIMIT + 2]).unwrap();
        let output = finish_owned(owner, &root, Instant::now()).unwrap();
        assert_eq!(output.stderr.len(), STDERR_LIMIT + 1);
        assert_eq!(output.stderr_evidence["limit_exceeded"], true);
        assert_eq!(output.stderr_evidence["complete"], false);
        assert_eq!(output.stderr_evidence["observed_length"], STDERR_LIMIT + 2);
    }
}
