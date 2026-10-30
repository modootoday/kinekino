use super::tool::PinnedTimelineTool;
use super::*;
use crate::application::review_source::MutationControl;
use kinekino_process::{
    application::supervisor::ProcessFamily,
    domain::observation::{FamilyObservation, ProcessReceipt},
    infrastructure::{
        evidence_files::{EvidencePaths, PreparedEvidence},
        family::{OwnedProcessFamily, PreparedFamilyJob, SpawnSpec, spawn_prepared},
    },
};
use std::{
    ffi::OsString,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};
#[must_use = "Retain and poll the original pending owner on a post-create failure"]
pub struct TimelineProcessFailure<'s, 'a> {
    pub error: ApiError,
    pub pending: Option<PendingTimelineProcess<'s, 'a>>,
}
impl std::fmt::Debug for TimelineProcessFailure<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimelineProcessFailure")
            .field("error", &self.error)
            .field("owner_retained", &self.pending.is_some())
            .finish()
    }
}
#[must_use = "Keep source/tool/slot and original Job until physical cleanup"]
pub struct PendingTimelineProcess<'s, 'a> {
    family: OwnedProcessFamily,
    _session: &'s TimelineProductionSession<'a>,
    _tool: &'s PinnedTimelineTool,
    _derived_input: Option<&'s RetainedProducedPcm>,
    _video_input: Option<&'s ProducedVideoFrames<'s, 'a>>,
    pub(super) paths: EvidencePaths,
    pub(super) root: PathBuf,
    pub(super) _records: Vec<File>,
    forced: bool,
    terminal: Option<ProcessReceipt>,
    cleanup_error: Option<String>,
    permit: ComponentPermit<'s>,
    launch_failed: bool,
    stdin_eof_requested: bool,
}
struct ComponentPermit<'a> {
    active: &'a std::cell::Cell<u8>,
    root: &'a Path,
    spawned: bool,
    closed: bool,
}
impl Drop for ComponentPermit<'_> {
    fn drop(&mut self) {
        if !self.spawned || self.closed {
            self.active.set(0);
        } else {
            self.active.set(2);
            let _ = write_new(
                &self.root.join("ABANDONED.json"),
                &serde_json::json!({"physical_cleanup_confirmed":false,"reason":"pending timeline owner dropped before observed close"}),
            );
        }
    }
}
impl PendingTimelineProcess<'_, '_> {
    pub fn receipt(&self) -> Option<&ProcessReceipt> {
        self.terminal.as_ref()
    }
    pub fn output_root(&self) -> &Path {
        &self.root
    }
    pub fn cleanup_error(&self) -> Option<&str> {
        self.cleanup_error.as_deref()
    }
    fn observe(&mut self) -> Result<Option<ProcessReceipt>, ApiError> {
        let exit = self.family.poll_child().map_err(io_error)?;
        let family = self.family.poll_owned_job().map_err(io_error)?;
        if exit.is_some() && family == FamilyObservation::Empty {
            let receipt = ProcessReceipt {
                identity: self.family.identity().clone(),
                child_exit: exit,
                family,
                forced: self.forced || self.launch_failed,
                stdin_eof_requested: self.stdin_eof_requested,
            };
            self.terminal = Some(receipt.clone());
            self.permit.closed = true;
            return Ok(Some(receipt));
        }
        Ok(None)
    }
    /// One bounded observation/termination request on exact owned handles. No PID reopen.
    pub fn poll_cleanup(&mut self) -> bool {
        match self.observe() {
            Ok(Some(_)) => return true,
            Ok(None) => {}
            Err(error) => {
                self.cleanup_error.get_or_insert(error.message);
            }
        }
        if !self.forced {
            match self.family.terminate_owned_family() {
                Ok(()) => self.forced = true,
                Err(error) => {
                    self.cleanup_error.get_or_insert(error.to_string());
                }
            }
        }
        false
    }
}
pub(super) fn before(error: ApiError) -> Box<TimelineProcessFailure<'static, 'static>> {
    Box::new(TimelineProcessFailure {
        error,
        pending: None,
    })
}
pub(super) fn failed<'s, 'a>(
    error: ApiError,
    owner: PendingTimelineProcess<'s, 'a>,
) -> Box<TimelineProcessFailure<'s, 'a>> {
    Box::new(TimelineProcessFailure {
        error,
        pending: Some(owner),
    })
}
pub(super) fn receipt_json(receipt: &ProcessReceipt) -> serde_json::Value {
    serde_json::json!({"pid":receipt.identity.pid,"creation_filetime":receipt.identity.creation_filetime,"child_exit":receipt.child_exit,"family_empty":receipt.family==FamilyObservation::Empty,"forced":receipt.forced,"stdin_eof_requested":receipt.stdin_eof_requested})
}
impl<'a> TimelineProductionSession<'a> {
    pub(super) fn execute<'s>(
        &'s self,
        tool: &'s PinnedTimelineTool,
        component: &str,
        arguments: Vec<OsString>,
        derived_input: Option<&'s RetainedProducedPcm>,
        video_input: Option<&'s ProducedVideoFrames<'s, 'a>>,
        control: &dyn MutationControl,
    ) -> Result<PendingTimelineProcess<'s, 'a>, Box<TimelineProcessFailure<'s, 'a>>> {
        if self.active.get() != 0 {
            return Err(before(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Timeline component is active or ownership was abandoned",
            )));
        }
        self.active.set(1);
        let mut permit = ComponentPermit {
            active: &self.active,
            root: &self.root,
            spawned: false,
            closed: false,
        };
        control.checkpoint().map_err(before)?;
        self.source.check_current().map_err(before)?;
        // The tool already retains its canonical parent; do not feed a
        // VerbatimDisk path into ordinary-root admission again.
        let tool_root = tool
            .path
            .parent()
            .ok_or_else(|| before(invalid("Probe parent missing")))?;
        let (root, _) = rooted::root(&self.root).map_err(before)?;
        if overlaps(&root, tool_root) {
            return Err(before(invalid(
                "Timeline runtime and tool root must be disjoint",
            )));
        }
        if self.deadline.operation_remaining(Instant::now()).is_zero() {
            return Err(before(ApiError::new(
                ErrorCode::Timeout,
                "Timeline producer operation deadline expired",
            )));
        }
        let leaf = format!(
            "{}-{}-{}",
            component,
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| before(io_error(std::io::Error::other(e))))?
                .as_nanos()
        );
        let root = self.root.join(leaf);
        std::fs::create_dir(&root).map_err(|e| before(io_error(e)))?;
        let mut records = vec![rooted::directory(&root).map_err(before)?];
        let argv = arguments
            .iter()
            .map(|a| {
                a.to_str()
                    .ok_or_else(|| before(invalid("Unicode producer argv required")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        records.push(write_new(&root.join("INPUT.json"),&serde_json::json!({"source":self.source.identity(),"source_path":self.source.path(),"tool_path":tool.path,"tool_sha256":tool.sha256(),"tool_file_identity":tool.file_identity(),"arguments":argv,"max_output_bytes":self.max_output_bytes,"memory_committed_bytes":self.budget.max_committed_bytes,"affinity_mask":self.budget.affinity_mask,"cpu_threads":1,"verified_timeline":false})).map_err(before)?);
        if let Some(input) = derived_input {
            records.push(
                write_new(
                    &root.join("DERIVED-INPUT.json"),
                    &serde_json::json!({
                        "path":input.path,"sha256":input.sha256(),"samples":input.samples(),
                        "first_pts":input.first_pts(),"end_pts":input.end_pts(),
                        "format":"mono16k_s16le","authority_granted":false
                    }),
                )
                .map_err(before)?,
            );
        }
        if let Some(input) = video_input {
            let (n, d) = input.frames().time_base().ratio();
            records.push(
                write_new(
                    &root.join("VIDEO-INPUT.json"),
                    &serde_json::json!({
                        "root":input.output_root(),"sha256":input.frames().sha256(),
                        "time_base":format!("{n}/{d}"),"stream_index":input.frames().stream_index(),
                        "authority_granted":false
                    }),
                )
                .map_err(before)?,
            );
        }
        let job = PreparedFamilyJob::reserve(self.budget).map_err(|e| before(io_error(e)))?;
        control.checkpoint().map_err(before)?;
        self.source.check_current().map_err(before)?;
        let evidence = PreparedEvidence::create(&root, None, 1).map_err(|e| before(io_error(e)))?;
        let spec = SpawnSpec {
            executable: tool.path.clone(),
            cwd: root.clone(),
            arguments,
            environment: None,
        };
        let (family, paths) = match spawn_prepared(spec, evidence, self.deadline, job) {
            Ok(parts) => parts,
            Err(failure) => {
                let failure = *failure;
                let error = ApiError::new(
                    ErrorCode::EngineFailed,
                    format!(
                        "Timeline probe spawn {}: {}; termination={:?}",
                        failure.stage, failure.error, failure.termination_error
                    ),
                );
                return Err(match failure.owner {
                    Some(owner) => {
                        permit.spawned = true;
                        failed(
                            error,
                            PendingTimelineProcess {
                                family: *owner,
                                _session: self,
                                _tool: tool,
                                _derived_input: derived_input,
                                _video_input: video_input,
                                paths: failure.evidence,
                                root,
                                _records: records,
                                forced: false,
                                terminal: None,
                                cleanup_error: None,
                                permit,
                                launch_failed: true,
                                stdin_eof_requested: false,
                            },
                        )
                    }
                    None => before(error),
                });
            }
        };
        permit.spawned = true;
        let mut owner = PendingTimelineProcess {
            family,
            _session: self,
            _tool: tool,
            _derived_input: derived_input,
            _video_input: video_input,
            paths,
            root,
            _records: records,
            forced: false,
            terminal: None,
            cleanup_error: None,
            permit,
            launch_failed: false,
            stdin_eof_requested: false,
        };
        let started = serde_json::json!({"pid":owner.family.identity().pid,"creation_filetime":owner.family.identity().creation_filetime});
        match write_new(&owner.root.join("STARTED.json"), &started) {
            Ok(file) => owner._records.push(file),
            Err(error) => return Err(failed(error, owner)),
        }
        if let Err(error) = owner.family.close_stdin() {
            return Err(failed(io_error(error), owner));
        }
        owner.stdin_eof_requested = true;
        let receipt = loop {
            let checks = control
                .checkpoint()
                .and_then(|()| self.source.check_current())
                .and_then(|()| {
                    owner
                        .paths
                        .check_limit(self.max_output_bytes)
                        .map_err(io_error)
                });
            if let Err(error) = checks {
                return Err(failed(error, owner));
            }
            if self.deadline.operation_remaining(Instant::now()).is_zero() {
                return Err(failed(
                    ApiError::new(
                        ErrorCode::Timeout,
                        "Timeline component observation deadline expired",
                    ),
                    owner,
                ));
            }
            match owner.observe() {
                Ok(Some(receipt)) => break receipt,
                Ok(None) => {}
                Err(error) => return Err(failed(error, owner)),
            }
            thread::sleep(Duration::from_millis(5));
        };
        if receipt.child_exit != Some(0) || receipt.forced {
            return Err(failed(
                ApiError::new(
                    ErrorCode::EngineFailed,
                    "Timeline component process did not complete normally",
                ),
                owner,
            ));
        }
        Ok(owner)
    }
}
