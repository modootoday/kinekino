//! Observation adapter only. OS entry stays closed until a real retained-owner host is available.
#[path = "default_launch/control.rs"]
mod control;
#[cfg(test)]
#[path = "default_launch/file_guard.rs"]
mod file_guard;
#[path = "default_launch/observation.rs"]
mod observation;
#[path = "default_launch/owner.rs"]
mod owner;
use observation::Observations;
#[cfg(test)]
#[path = "default_launch/tests.rs"]
mod tests;
#[path = "default_launch/window.rs"]
mod window;

use kinekino_player::{
    domain::identifiers::{valid_id, valid_sha256},
    infrastructure::content_digest,
};
use kinekino_process::{
    application::{
        raw_records::RecordCursor,
        supervisor::{ProcessFamily, Supervisor},
    },
    domain::{deadline::Deadline, observation::FamilyObservation},
    infrastructure::{
        evidence_files::PreparedEvidence,
        family::{SpawnSpec, spawn},
    },
};
use serde::{Deserialize, Serialize};
use std::{
    any::{Any, type_name},
    ffi::OsString,
    fs::{File, OpenOptions},
    io::Write,
    os::windows::ffi::OsStringExt,
    os::windows::fs::{MetadataExt, OpenOptionsExt},
    os::windows::io::AsRawHandle,
    path::{Component, Path, PathBuf, Prefix},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Mode {
    NoArgs,
    Configured,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Admission,
    Pin,
    Spawn,
    Identity,
    Window,
    Raw,
    Sdk,
    Close,
    Deadline,
    Exit,
    Report,
    Panic,
}
struct Failure {
    stage: Stage,
    message: String,
    original: Box<dyn Any>,
    original_type: &'static str,
}
impl std::fmt::Debug for Failure {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("Failure")
            .field("stage", &self.stage)
            .field("message", &self.message)
            .field("original_type", &self.original_type)
            .finish()
    }
}
impl Failure {
    fn new<T: ToString + 'static>(stage: Stage, message: T) -> Self {
        Self {
            stage,
            message: message.to_string(),
            original: Box::new(message),
            original_type: type_name::<T>(),
        }
    }
    fn debug<T: std::fmt::Debug + 'static>(stage: Stage, original: T) -> Self {
        Self {
            stage,
            message: format!("{original:?}"),
            original: Box::new(original),
            original_type: type_name::<T>(),
        }
    }
    fn evidence(&self) -> serde_json::Value {
        let io = self.original.downcast_ref::<std::io::Error>().or_else(|| {
            self.original
                .downcast_ref::<kinekino_process::domain::observation::FailureCause>()
                .map(|cause| &cause.error)
        });
        serde_json::json!({"stage":format!("{:?}",self.stage),"message":self.message,"original_type":self.original_type,
            "io_kind":io.map(|error|format!("{:?}",error.kind())),"raw_os_error":io.and_then(std::io::Error::raw_os_error),
            "windows_hresult":self.original.downcast_ref::<windows::core::Error>().map(|error|error.code().0),
            "api_code":self.original.downcast_ref::<kinekino_player::application::api::ApiError>().map(|error|format!("{:?}",error.code))})
    }
    fn panic(payload: Box<dyn Any + Send>) -> Self {
        let message = payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| {
                payload
                    .downcast_ref::<&'static str>()
                    .map(|text| (*text).to_owned())
            })
            .unwrap_or_else(|| "Non-string panic payload retained".into());
        Self {
            stage: Stage::Panic,
            message,
            original: Box::new(payload),
            original_type: "original panic payload",
        }
    }
}
fn capture_phase<T>(body: impl FnOnce() -> Result<T, Failure>) -> Result<T, Failure> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(body)) {
        Ok(result) => result,
        Err(original) => Err(Failure::panic(original)),
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe {
    root: PathBuf,
    id: String,
    revision: u64,
    sha256: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GateSpec {
    mode: Mode,
    binary: PathBuf,
    sha256: String,
    cwd: PathBuf,
    evidence: PathBuf,
    project: Option<Probe>,
    total: Duration,
    cleanup: Duration,
}
struct Pin {
    _guard: File,
    _ancestors: Vec<File>,
    path: PathBuf,
    sha256: String,
    bytes: u64,
    volume: u32,
    file_id: u64,
    mtime: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct ImageIdentity {
    volume: u32,
    file_id: u64,
    final_path: PathBuf,
}
fn check_binding(expected: &ImageIdentity, actual: &ImageIdentity) -> Result<(), Failure> {
    if expected != actual {
        return Err(Failure::new(
            Stage::Pin,
            "Original file identity/final path binding changed",
        ));
    }
    Ok(())
}
fn ordinary_executable(path: &Path) -> bool {
    path.is_absolute()
        && path.file_name().is_some()
        && matches!(path.components().next(),Some(Component::Prefix(prefix)) if matches!(prefix.kind(),Prefix::Disk(_)))
        && !path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
}
fn file_information(
    file: &File,
) -> Result<windows::Win32::Storage::FileSystem::BY_HANDLE_FILE_INFORMATION, Failure> {
    let mut info = windows::Win32::Storage::FileSystem::BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: original retained file handle and correctly sized initialized output.
    unsafe {
        windows::Win32::Storage::FileSystem::GetFileInformationByHandle(
            windows::Win32::Foundation::HANDLE(file.as_raw_handle()),
            &mut info,
        )
    }
    .map_err(|error| Failure::new(Stage::Pin, error))?;
    Ok(info)
}
fn final_path(file: &File) -> Result<PathBuf, Failure> {
    use windows::Win32::Storage::FileSystem::{
        FILE_NAME_NORMALIZED, GETFINALPATHNAMEBYHANDLE_FLAGS, GetFinalPathNameByHandleW,
        VOLUME_NAME_DOS,
    };
    let mut name = vec![0u16; 32768];
    // SAFETY: original file handle and bounded writable UTF-16 buffer. No input path is re-resolved.
    let count = unsafe {
        GetFinalPathNameByHandleW(
            windows::Win32::Foundation::HANDLE(file.as_raw_handle()),
            &mut name,
            GETFINALPATHNAMEBYHANDLE_FLAGS(FILE_NAME_NORMALIZED.0 | VOLUME_NAME_DOS.0),
        )
    } as usize;
    if count == 0 {
        return Err(Failure::new(Stage::Pin, std::io::Error::last_os_error()));
    }
    if count >= name.len() {
        return Err(Failure::new(Stage::Pin, "Final handle path exceeds bound"));
    }
    Ok(PathBuf::from(OsString::from_wide(&name[..count])))
}
fn open_ordinary_file(path: &Path) -> Result<File, Failure> {
    let file = OpenOptions::new()
        .read(true)
        .share_mode(1)
        .custom_flags(0x00200000)
        .open(path)
        .map_err(|error| Failure::new(Stage::Pin, error))?;
    let info = file_information(&file)?;
    if info.dwFileAttributes & (0x400 | 0x10) != 0 {
        return Err(Failure::new(
            Stage::Pin,
            "Ordinary non-reparse executable required",
        ));
    }
    Ok(file)
}
fn identity_from(file: &File) -> Result<ImageIdentity, Failure> {
    let info = file_information(file)?;
    Ok(ImageIdentity {
        volume: info.dwVolumeSerialNumber,
        file_id: (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
        final_path: final_path(file)?,
    })
}
fn verify_pin(expected: &str, actual: &str) -> Result<(), Failure> {
    if !valid_sha256(&expected.to_ascii_lowercase()) || !expected.eq_ignore_ascii_case(actual) {
        return Err(Failure::new(Stage::Pin, "Executable SHA256 mismatch"));
    }
    Ok(())
}
impl Pin {
    fn open(spec: &GateSpec) -> Result<Self, Failure> {
        if !ordinary_executable(&spec.binary)
            || !spec.cwd.is_absolute()
            || !spec.evidence.is_absolute()
        {
            return Err(Failure::new(Stage::Pin, "Explicit absolute paths required"));
        }
        // Same ordinary local ancestor/no-delete guard mechanism as immutable_storage_paths.
        // That module is private; do not broaden product API or misuse the mpv-only public wrapper.
        let parent = spec
            .binary
            .parent()
            .ok_or_else(|| Failure::new(Stage::Pin, "Missing executable parent"))?;
        let mut paths = parent.ancestors().collect::<Vec<_>>();
        paths.reverse();
        let mut ancestors = Vec::new();
        for path in paths {
            let metadata =
                std::fs::symlink_metadata(path).map_err(|error| Failure::new(Stage::Pin, error))?;
            if !metadata.is_dir() || metadata.file_attributes() & 0x400 != 0 {
                return Err(Failure::new(
                    Stage::Pin,
                    "Reparse executable ancestor rejected",
                ));
            }
            let directory = OpenOptions::new()
                .read(true)
                .share_mode(3)
                .custom_flags(0x02200000)
                .open(path)
                .map_err(|error| Failure::new(Stage::Pin, error))?;
            let info = file_information(&directory)?;
            if info.dwFileAttributes & 0x400 != 0 || info.dwFileAttributes & 0x10 == 0 {
                return Err(Failure::new(
                    Stage::Pin,
                    "Executable ancestor identity changed",
                ));
            }
            ancestors.push(directory);
        }
        let mut guard = open_ordinary_file(&spec.binary)?;
        let info = file_information(&guard)?;
        let (bytes, sha256) = content_digest::sha256(&mut guard, 1024 * 1024 * 1024)
            .map_err(|e| Failure::debug(Stage::Pin, e))?;
        let recorded_bytes = (u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow);
        if bytes != recorded_bytes {
            return Err(Failure::new(Stage::Pin, "Pinned file size changed"));
        }
        verify_pin(&spec.sha256, &sha256)?;
        let path = final_path(&guard)?;
        let pin = Self {
            _guard: guard,
            _ancestors: ancestors,
            path,
            sha256,
            bytes,
            volume: info.dwVolumeSerialNumber,
            file_id: (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
            mtime: (u64::from(info.ftLastWriteTime.dwHighDateTime) << 32)
                | u64::from(info.ftLastWriteTime.dwLowDateTime),
        };
        pin.verify_bound()?;
        Ok(pin)
    }
    fn identity(&self) -> ImageIdentity {
        ImageIdentity {
            volume: self.volume,
            file_id: self.file_id,
            final_path: self.path.clone(),
        }
    }
    fn verify_bound(&self) -> Result<(), Failure> {
        check_binding(&self.identity(), &identity_from(&self._guard)?)?;
        let resolved = open_ordinary_file(&self.path)?;
        check_binding(&self.identity(), &identity_from(&resolved)?)
    }
    fn verify_spawned(
        &self,
        original: &kinekino_process::domain::identity::ProcessIdentity,
    ) -> Result<(), Failure> {
        if original.creation_filetime.is_none() || original.executable != self.path {
            return Err(Failure::new(
                Stage::Pin,
                "Original process image readback differs from guarded final path",
            ));
        }
        self.verify_bound()
    }
    fn evidence(&self) -> serde_json::Value {
        serde_json::json!({"path":self.path,"sha256":self.sha256,"bytes":self.bytes,
            "volume":self.volume,"file_id":self.file_id,"mtime_filetime":self.mtime})
    }
}
fn arguments(mode: Mode, probe: Option<&Probe>) -> Result<Vec<OsString>, Failure> {
    match (mode, probe) {
        (Mode::NoArgs, None) => Ok(Vec::new()),
        (Mode::Configured, Some(probe))
            if probe.root.is_absolute()
                && valid_id(&probe.id)
                && probe.revision > 0
                && valid_sha256(&probe.sha256) =>
        {
            Ok(vec![
                "--record-validation".into(),
                "--project-root".into(),
                probe.root.as_os_str().to_owned(),
            ])
        }
        _ => Err(Failure::new(
            Stage::Admission,
            "Mode and explicit project probe disagree",
        )),
    }
}
pub(super) fn run_gate(mode: Mode) -> Result<(), String> {
    control::run_controller(mode).map_err(|e| format!("{mode:?}: {:?}: {}", e.stage, e.message))
}
fn run(spec: GateSpec) -> owner::Outcome {
    let args = match arguments(spec.mode, spec.project.as_ref()) {
        Ok(v) => v,
        Err(e) => return owner::Outcome::NotStarted(e),
    };
    let pin = match Pin::open(&spec) {
        Ok(v) => v,
        Err(e) => return owner::Outcome::NotStarted(e),
    };
    let deadline = match Deadline::new(Instant::now(), spec.total, spec.cleanup) {
        Ok(v) => v,
        Err(e) => return owner::Outcome::NotStarted(Failure::new(Stage::Admission, e)),
    };
    let evidence = match PreparedEvidence::create(&spec.evidence, None, 0) {
        Ok(v) => v,
        Err(e) => return owner::Outcome::NotStarted(Failure::new(Stage::Raw, e)),
    };
    if let Err(error) = pin.verify_bound() {
        return owner::Outcome::NotStarted(error);
    }
    let pin_before = pin.evidence();
    let requested_final_path = pin.path.clone();
    let spawned = spawn(
        SpawnSpec {
            executable: requested_final_path.clone(),
            cwd: spec.cwd,
            arguments: args,
            environment: None,
        },
        evidence,
        deadline,
    );
    let (family, evidence) = match spawned {
        Ok(v) => v,
        Err(original) => {
            return owner::Outcome::Retained(Box::new(owner::Retained::Launch { original, pin }));
        }
    };
    let returned_launch_state = family.state();
    let mut supervisor = Supervisor::new(owner::NoForce::new(family), evidence, 1024 * 1024);
    let mut observed = Observations::new();
    let mut window_witness = WindowWitness::new();
    let result = capture_phase(|| {
        let original_process_identity = supervisor.process().identity();
        pin.verify_spawned(original_process_identity)?;
        // Both cached snapshots have just passed the existing original-handle/reopen equality checks.
        // The image query belongs to spawn's original HANDLE admission, not a new PID lookup here.
        let witness = serde_json::json!({
            "kind":"default-native-spawn-admission",
            "requested_final_path":requested_final_path,
            "pin_before":pin_before,"pin_after":pin.evidence(),
            "pin_metadata_source":"cached fields checked against original and reopened handles before and after spawn",
            "original_process_identity":{"pid":original_process_identity.pid,
                "creation_filetime":original_process_identity.creation_filetime,
                "cached_image_path":original_process_identity.executable},
            "returned_launch_state":format!("{returned_launch_state:?}"),
            "original_spawn_readback_checked":matches!(returned_launch_state,
                kinekino_process::infrastructure::family::LaunchState::Running),
            "readback_source":"kinekino_process::spawn original process HANDLE before resume",
            "fresh_independent_process_image_query":false,
            "process_section_file_id_observed":false
        });
        let mut record = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(spec.evidence.join("spawn-admission.json"))
            .map_err(|error| Failure::new(Stage::Raw, error))?;
        serde_json::to_writer(&mut record, &witness)
            .map_err(|error| Failure::new(Stage::Raw, error))?;
        record
            .sync_all()
            .map_err(|error| Failure::new(Stage::Raw, error))?;
        observe(
            &spec.project,
            spec.mode,
            &pin,
            &mut supervisor,
            &mut observed,
            &mut window_witness,
            deadline,
        )
    });
    let outcome = if let Err(first) = result {
        owner::Outcome::Retained(Box::new(owner::Retained::Observing {
            supervisor: Box::new(supervisor),
            pin,
            first,
            events: observed.events,
            close_attempted: observed.close_attempted,
        }))
    } else {
        owner::finish(supervisor, pin, &spec.evidence, observed.events, deadline)
    };
    // The original lifetime result and any live owner are established before diagnostic IO.
    window_witness.flush(&spec.evidence);
    outcome
}
fn observe(
    project: &Option<Probe>,
    mode: Mode,
    pin: &Pin,
    supervisor: &mut Supervisor<
        owner::NoForce<kinekino_process::infrastructure::family::OwnedProcessFamily>,
    >,
    observed: &mut Observations,
    window_witness: &mut WindowWitness,
    deadline: Deadline,
) -> Result<(), Failure> {
    let mut cursor = RecordCursor::default();
    loop {
        let receipt = supervisor
            .poll_state()
            .map_err(|e| Failure::debug(Stage::Identity, e))?
            .clone();
        observed.records(
            supervisor,
            &mut cursor,
            receipt.child_exit.is_some() && receipt.family == FamilyObservation::Empty,
        )?;
        if let Some(exit) = receipt.child_exit {
            if receipt.family == FamilyObservation::Empty {
                if exit != 0 || !observed.close_sent {
                    return Err(Failure::new(Stage::Exit, "Exit without normal owned close"));
                }
                let window = observed
                    .window
                    .as_ref()
                    .ok_or_else(|| Failure::new(Stage::Window, "Window not observed"))?;
                if window.still_present() {
                    return Err(Failure::new(Stage::Window, "Original window remains"));
                }
                observed.final_raw_eof(supervisor)?;
                return observed.check_report(mode);
            }
        } else if !observed.close_sent {
            if deadline.operation_remaining(Instant::now()).is_zero() {
                return Err(Failure::new(Stage::Deadline, "Observation cutoff"));
            }
            let identity = supervisor.process().identity();
            window::check_identity(identity, &pin.path, receipt.child_exit)?;
            if observed.window.is_none() {
                observed.window = window::find(identity.pid)?;
                if let Some(window) = &observed.window {
                    let selected = window.evidence();
                    if window_witness.enabled {
                        window_witness.selected = Some(selected.clone());
                    }
                    observed.events.push(selected);
                }
            }
            let ready = match project {
                Some(probe) if !observed.sdk_read => observed.inspect(probe, deadline)?,
                _ => true,
            };
            if ready && let Some(window) = &observed.window {
                let current = supervisor
                    .poll_state()
                    .map_err(|e| Failure::debug(Stage::Identity, e))?
                    .clone();
                window::check_identity(&current.identity, &pin.path, current.child_exit)?;
                if window_witness.enabled {
                    window_witness.attempted = Some(
                        serde_json::json!({"kind":"wm-close-attempt","window":window.evidence(),
                        "process_pid":current.identity.pid,
                        "process_creation_filetime":current.identity.creation_filetime,
                        "post_message_result":"not_yet_observed","callback_received":null}),
                    );
                }
                observed.close_attempted = true;
                let posted = window.close(current.identity.pid);
                if window_witness.enabled {
                    window_witness.returned = Some(
                        serde_json::json!({"kind":"wm-close-post-return","window":window.evidence(),
                        "process_pid":current.identity.pid,
                        "process_creation_filetime":current.identity.creation_filetime,
                        "close_method_returned_ok":posted.is_ok(),
                        "post_message_returned_ok":if posted.is_ok(){Some(true)}else{None},
                        "close_error":posted.as_ref().err().map(|error|serde_json::json!({
                            "stage":format!("{:?}",error.stage),"original_type":error.original_type,
                            "windows_hresult":error.original.downcast_ref::<windows::core::Error>().map(|error|error.code().0)})),
                        "callback_received":null}),
                    );
                }
                posted?;
                observed.close_sent = true;
            }
        }
        if deadline.operation_remaining(Instant::now()).is_zero() {
            return Err(Failure::new(
                Stage::Deadline,
                "Close/exit observation cutoff",
            ));
        }
        thread::sleep(Duration::from_millis(10).min(deadline.operation_remaining(Instant::now())));
    }
}
struct WindowWitness {
    enabled: bool,
    selected: Option<serde_json::Value>,
    attempted: Option<serde_json::Value>,
    returned: Option<serde_json::Value>,
}
impl WindowWitness {
    fn new() -> Self {
        Self {
            enabled: std::env::var_os("MEARI_NATIVE_DIAG_CLOSE").as_deref()
                == Some(std::ffi::OsStr::new("1")),
            selected: None,
            attempted: None,
            returned: None,
        }
    }
    fn flush(&self, root: &Path) {
        if !self.enabled {
            return;
        }
        let mut written = 0;
        let mut failed = 0;
        let mut present = 0;
        let mut report = std::io::stderr().lock();
        for (name, record) in [
            ("window-selection.json", &self.selected),
            ("window-close-attempt.json", &self.attempted),
            ("window-close-post-return.json", &self.returned),
        ] {
            let Some(record) = record else { continue };
            present += 1;
            match write_window_witness(root, name, record) {
                Ok(()) => written += 1,
                Err(error) => {
                    failed += 1;
                    let code = error
                        .original
                        .downcast_ref::<std::io::Error>()
                        .and_then(std::io::Error::raw_os_error);
                    // Diagnostic failure is separate; never replace the original Close/lifetime result.
                    let _ = writeln!(
                        report,
                        "player.window-witness.v1 p={} name={name} ok=0 stage={:?} io={code:?}",
                        std::process::id(),
                        error.stage
                    );
                }
            }
        }
        let _ = writeln!(
            report,
            "player.window-witness.v1 p={} enabled=1 records={present} written={written} failed={failed}",
            std::process::id()
        );
    }
}
fn write_window_witness(
    root: &Path,
    name: &'static str,
    value: &serde_json::Value,
) -> Result<(), Failure> {
    // At most three gate-only records, after the original outcome; no process or close operation.
    let bytes = serde_json::to_vec(value).map_err(|error| Failure::new(Stage::Raw, error))?;
    if bytes.len() > 4096 {
        return Err(Failure::new(Stage::Raw, "Window witness exceeds 4KiB"));
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(name))
        .map_err(|error| Failure::new(Stage::Raw, error))?;
    file.write_all(&bytes)
        .map_err(|error| Failure::new(Stage::Raw, error))?;
    file.sync_all()
        .map_err(|error| Failure::new(Stage::Raw, error))
}
