//! Original owned-process supervisor and bounded raw/receipt evidence only.
use kinekino_process::{
    application::supervisor::{FinishOutcome, ReceiptSink, Supervisor, SystemClock},
    domain::{
        deadline::Deadline,
        observation::{FamilyObservation, ProcessReceipt},
    },
    infrastructure::{
        evidence_files::PreparedEvidence,
        family::{OwnedProcessFamily, SpawnSpec, spawn},
    },
};
use std::io;
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    io::{Read, Write},
    path::PathBuf,
    time::{Duration, Instant, SystemTime},
};
pub(super) const CHILD_MODE: &str = "KINEKINO_VIEWER_NO_MEDIA_CHILD";
pub(super) const CHILD_EVIDENCE: &str = "KINEKINO_VIEWER_NO_MEDIA_EVIDENCE";
const RAW_LIMIT: u64 = 256 * 1024;

pub(super) struct RunFailure {
    pub(super) stage: &'static str,
    pub(super) error: String,
    pub(super) evidence: PathBuf,
    pub(super) owner: Option<Box<Supervisor<OwnedProcessFamily>>>,
    pub(super) receipt: Option<ProcessReceipt>,
    pub(super) secondary: Vec<String>,
}
impl std::fmt::Debug for RunFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ViewerNoMediaFailure")
            .field("stage", &self.stage)
            .field("error", &self.error)
            .field("evidence", &self.evidence)
            .field("original_owner_retained", &self.owner.is_some())
            .field("receipt", &self.receipt)
            .field("secondary", &self.secondary)
            .finish()
    }
}
struct ReceiptFile(PathBuf);
impl ReceiptSink for ReceiptFile {
    fn record(&mut self, receipt: &ProcessReceipt) -> io::Result<()> {
        let value = serde_json::json!({
            "schema":"viewer-no-media-family-v1",
            "pid":receipt.identity.pid,"creation_filetime":receipt.identity.creation_filetime,
            "executable":receipt.identity.executable,"child_exit":receipt.child_exit,
            "family":format!("{:?}",receipt.family),"forced":receipt.forced,
            "stdin_eof_requested":receipt.stdin_eof_requested,
        });
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.0)?;
        file.write_all(&serde_json::to_vec_pretty(&value)?)?;
        file.flush()
    }
}
pub(super) fn parent_run(exact: &'static str, mode: &'static str) -> Result<(), Box<RunFailure>> {
    let started = Instant::now();
    let mut root = PathBuf::new();
    let early = |stage, error: io::Error, root: &PathBuf| {
        Box::new(RunFailure {
            stage,
            error: error.to_string(),
            evidence: root.clone(),
            owner: None,
            receipt: None,
            secondary: Vec::new(),
        })
    };
    let deadline = Deadline::new(started, Duration::from_secs(20), Duration::from_secs(2))
        .map_err(|error| early("budget", error, &root))?;
    let executable = std::env::current_exe()
        .and_then(fs::canonicalize)
        .map_err(|error| early("original executable", error, &root))?;
    let cwd = std::env::current_dir().map_err(|error| early("cwd", error, &root))?;
    let suffix = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|error| early("clock", io::Error::other(error), &root))?
        .as_nanos();
    root = cwd
        .join("runtime/quality-review/ui-viewer-no-media-process")
        .join(format!("{}-{suffix}", std::process::id()));
    fs::create_dir_all(root.parent().unwrap())
        .map_err(|error| early("evidence parent", error, &root))?;
    fs::create_dir(&root).map_err(|error| early("new evidence", error, &root))?;
    let prepared = PreparedEvidence::create(&root, None, 0)
        .map_err(|error| early("raw evidence", error, &root))?;
    let mut environment = BTreeMap::new();
    for name in [
        "SystemRoot",
        "WINDIR",
        "TEMP",
        "TMP",
        "USERPROFILE",
        "LOCALAPPDATA",
        "APPDATA",
        "PATH",
        "PATHEXT",
        "COMSPEC",
        "NUMBER_OF_PROCESSORS",
        "PROCESSOR_ARCHITECTURE",
        "ProgramData",
    ] {
        if let Some(value) = std::env::var_os(name) {
            environment.insert(name.to_owned(), value);
        }
    }
    environment.insert(CHILD_MODE.into(), OsString::from(mode));
    environment.insert(CHILD_EVIDENCE.into(), root.as_os_str().to_owned());
    let spec = SpawnSpec {
        executable,
        cwd,
        arguments: vec![
            OsString::from(exact),
            OsString::from("--ignored"),
            OsString::from("--exact"),
            OsString::from("--test-threads=1"),
            OsString::from("--nocapture"),
        ],
        environment: Some(environment),
    };
    let (family, paths) = spawn(spec, prepared, deadline).map_err(|failure| {
        let kinekino_process::infrastructure::family::LaunchFailure {
            stage,
            error,
            owner,
            termination_error,
            evidence,
        } = *failure;
        Box::new(RunFailure {
            stage,
            error: error.to_string(),
            evidence: root.clone(),
            owner: owner.map(|owner| Box::new(Supervisor::new(*owner, evidence, RAW_LIMIT))),
            receipt: None,
            secondary: termination_error
                .map(|error| vec![error.to_string()])
                .unwrap_or_default(),
        })
    })?;
    let supervisor = Supervisor::new(family, paths, RAW_LIMIT);
    let receipt = match supervisor.finish(
        deadline,
        &SystemClock,
        &mut ReceiptFile(root.join("family-receipt.json")),
    ) {
        FinishOutcome::Complete(completion) => completion.receipt().clone(),
        FinishOutcome::Incomplete(incomplete) => {
            let receipt = incomplete.owner.receipt().clone();
            let secondary = incomplete
                .termination_error
                .into_iter()
                .chain(incomplete.receipt_error)
                .map(|error| error.to_string())
                .collect();
            return Err(Box::new(RunFailure {
                stage: "owned-family finish",
                error: format!("{:?}: {}", incomplete.cause.stage, incomplete.cause.error),
                evidence: root,
                owner: Some(Box::new(incomplete.owner)),
                receipt: Some(receipt),
                secondary,
            }));
        }
    };
    let terminal_failure = |stage, error: String| {
        Box::new(RunFailure {
            stage,
            error,
            evidence: root.clone(),
            owner: None,
            receipt: Some(receipt.clone()),
            secondary: Vec::new(),
        })
    };
    if receipt.forced || receipt.child_exit != Some(0) || receipt.family != FamilyObservation::Empty
    {
        return Err(terminal_failure(
            "inner process",
            "forced/failed/unknown family is not GUI cleanup success".into(),
        ));
    }
    // Exit zero alone could mean an empty --exact selection. Require the inner protocol record.
    let mut bytes = Vec::new();
    fs::File::open(root.join("inner-success.json"))
        .and_then(|file| file.take(524289).read_to_end(&mut bytes))
        .map_err(|error| terminal_failure("inner record", error.to_string()))?;
    if bytes.len() > 524288 {
        return Err(terminal_failure(
            "inner record",
            "record exceeds bound".into(),
        ));
    }
    let record: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| terminal_failure("inner record", error.to_string()))?;
    if record["schema"] != "viewer-no-media-inner-v1"
        || record["test"] != exact
        || record["mode"] != mode
        || record["scenario_passed"] != true
    {
        return Err(terminal_failure(
            "inner record",
            "missing exact inner native result".into(),
        ));
    }
    println!("owned Viewer NoMedia fixture evidence: {}", root.display());
    Ok(())
}
