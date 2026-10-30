//! Explicit, separately launched control host. No process/pipe is started by module initialization.
use super::{Failure, GateSpec, Mode, Pin, Stage, owner, run};
use kinekino_player::infrastructure::content_digest;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    ffi::c_void,
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    os::windows::{ffi::OsStringExt, io::AsRawHandle},
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::windows::named_pipe::{ClientOptions, NamedPipeServer, ServerOptions},
    time::timeout,
};
use windows::{
    Win32::{
        Foundation::{CloseHandle, FILETIME, HANDLE, HLOCAL, LocalFree},
        Security::{
            Authorization::{
                ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
                SDDL_REVISION_1,
            },
            Cryptography::{BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom},
            GetTokenInformation, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, TOKEN_QUERY,
            TOKEN_USER, TokenUser,
        },
        System::{
            Pipes::GetNamedPipeServerProcessId,
            Threading::{
                GetCurrentProcess, GetProcessTimes, OpenProcess, OpenProcessToken,
                PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
            },
        },
    },
    core::{PCWSTR, PWSTR},
};

const MAX_FRAME: usize = 65536;
const IO_BUDGET: Duration = Duration::from_secs(3);
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    spec: GateSpec,
    host_sha256: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Ready {
    version: u32,
    pipe: String,
    host_pid: u32,
    host_creation: u64,
    host_image: PathBuf,
    host_sha256: String,
    config_sha256: String,
    mode: Mode,
    controller_nonce: String,
    run_wait_ms: u64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Command {
    Run { mode: Mode },
    State,
    Close,
    ReleaseClosed,
    FinishHost,
    AbortHost { approval: String },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Request {
    pub(super) version: u32,
    pub(super) request_id: String,
    pub(super) nonce: String,
    pub(super) case_id: Option<String>,
    pub(super) command: Command,
}
fn failed(message: impl ToString + 'static) -> Failure {
    Failure::new(Stage::Admission, message)
}
fn digest(bytes: &[u8]) -> Result<String, Failure> {
    content_digest::sha256(&mut io::Cursor::new(bytes), MAX_FRAME as u64)
        .map(|(_, sha)| sha)
        .map_err(|e| Failure::debug(Stage::Admission, e))
}
fn read_json<T: for<'a> Deserialize<'a>>(path: &Path, expected: &str) -> Result<T, Failure> {
    if !path.is_absolute() {
        return Err(failed("Absolute pinned JSON path required"));
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(failed)?
        .take(MAX_FRAME as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(failed)?;
    if bytes.len() > MAX_FRAME {
        return Err(failed("JSON bound exceeded"));
    }
    super::verify_pin(expected, &digest(&bytes)?)?;
    serde_json::from_slice(&bytes).map_err(failed)
}
fn write_new(path: &Path, value: &Value) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    serde_json::to_writer(&mut file, value).map_err(io::Error::other)?;
    file.write_all(b"\n")?;
    file.sync_all()
}
pub(super) fn valid_header(request: &Request) -> bool {
    request.version == 1
        && super::valid_sha256(&request.nonce)
        && !request.request_id.is_empty()
        && request.request_id.len() <= 64
        && request
            .request_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        && request
            .case_id
            .as_deref()
            .is_none_or(|id| id == "original-1")
        && (!matches!(
            request.command,
            Command::Close | Command::ReleaseClosed | Command::AbortHost { .. }
        ) || request.case_id.as_deref() == Some("original-1"))
}
pub(super) fn permitted(command: &Command, admin: bool) -> bool {
    match command {
        Command::Run { .. } => !admin,
        Command::State => true,
        _ => admin,
    }
}
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Replay {
    Miss,
    Same,
    Conflict,
}
pub(super) fn replay(previous: Option<&str>, current: &str) -> Replay {
    match previous {
        None => Replay::Miss,
        Some(value) if value == current => Replay::Same,
        Some(_) => Replay::Conflict,
    }
}
pub(super) fn run_allowed(used: bool, fenced: bool, expected: Mode, actual: Mode) -> bool {
    !used && !fenced && expected == actual
}
pub(super) fn finish_allowed(carrier_present: bool) -> bool {
    !carrier_present
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ErrorLane {
    Control,
    Observation,
    Transport,
}
struct OwnedFailure {
    lane: ErrorLane,
    original: Failure,
}
#[derive(Default)]
pub(super) struct FailureLog {
    first: Option<OwnedFailure>,
    secondary: Vec<OwnedFailure>,
    overflow: usize,
    transport_seen: bool,
}
impl FailureLog {
    pub(super) fn record(&mut self, lane: ErrorLane, original: Failure) -> Value {
        let evidence = original.evidence(); // Borrow before moving the same object into the bounded log.
        self.transport_seen |= lane == ErrorLane::Transport;
        let entry = OwnedFailure { lane, original };
        let retained = if self.first.is_none() {
            self.first = Some(entry);
            true
        } else if self.secondary.len() < 8 {
            self.secondary.push(entry);
            true
        } else {
            self.overflow = self.overflow.saturating_add(1);
            false
        };
        json!({"lane":format!("{lane:?}"),"error":evidence,"original_retained":retained,"overflow":self.overflow})
    }
    pub(super) fn retained(&self) -> impl Iterator<Item = &Failure> {
        self.first
            .iter()
            .chain(self.secondary.iter())
            .map(|entry| &entry.original)
    }
    pub(super) fn evidence(&self) -> Value {
        let view = |entry: &OwnedFailure| json!({"lane":format!("{:?}",entry.lane),"error":entry.original.evidence()});
        json!({"first":self.first.as_ref().map(view),"secondary":self.secondary.iter().map(view).collect::<Vec<_>>(),
            "overflow":self.overflow,"transport_fenced":self.transport_seen})
    }
}
struct Host {
    spec: GateSpec,
    controller: String,
    admin: String,
    abort: String,
    used: bool,
    run_request: Option<(String, String)>,
    original: Option<owner::Outcome>,
    released_original: Option<Value>,
    cache: BTreeMap<String, (String, Value)>,
    emergency: Option<(String, String, Value)>,
    errors: FailureLog,
    stop: bool,
    aborted: bool,
}
impl Host {
    fn state(&self) -> Value {
        json!({"case_used":self.used,"case_id":"original-1","original_owner":self.original.as_ref().map(owner::Outcome::description),
            "released_original_summary":self.released_original,"control_observation_errors":self.errors.evidence(),
            "run_request":self.run_request,"owner_host_pid":std::process::id(),"owner_transfer":false,"aborted_by_operator":self.aborted,
            "abort_mechanism":if self.aborted {Some("owned_job_close_on_host_exit")} else {None},"abort_completion_confirmed":false})
    }
    fn handle(&mut self, request: Request, body_sha: String) -> Value {
        if !valid_header(&request) {
            return json!({"ok":false,"error":"invalid_header"});
        }
        let admin = request.nonce == self.admin;
        if (!admin && request.nonce != self.controller) || !permitted(&request.command, admin) {
            return json!({"ok":false,"error":"unauthorized"});
        }
        if let Some((previous, response)) = self.cache.get(&request.request_id) {
            return if replay(Some(previous), &body_sha) == Replay::Same {
                response.clone()
            } else {
                json!({"ok":false,"error":"request_id_conflict"})
            };
        }
        if let Some((id, previous, response)) = &self.emergency
            && id == &request.request_id
        {
            return if replay(Some(previous), &body_sha) == Replay::Same {
                response.clone()
            } else {
                json!({"ok":false,"error":"request_id_conflict"})
            };
        }
        // Recovery reads remain available even if ordinary request records are exhausted.
        // State is deliberately live, not an old mutation-response replay.
        if matches!(request.command, Command::State) {
            let result = self
                .original
                .as_mut()
                .map(|original| super::capture_phase(|| original.physically_closed()));
            let observation = match result {
                Some(Ok(closed)) => Some(json!({"physical_closed":closed})),
                Some(Err(original)) => Some(self.errors.record(ErrorLane::Observation, original)),
                None => None,
            };
            return json!({"version":1,"request_id":request.request_id,"ok":true,"state":self.state(),"observation":observation});
        }
        let emergency_abort = admin
            && matches!(request.command,Command::AbortHost{ref approval} if approval==&self.abort);
        if self.cache.len() >= 64 && (!emergency_abort || self.emergency.is_some()) {
            return json!({"ok":false,"error":"request_capacity"});
        }
        let action = super::capture_phase(|| -> Result<(), Failure> {
            match request.command {
                Command::Run { mode } => {
                    if !run_allowed(self.used, self.errors.transport_seen, self.spec.mode, mode) {
                        return Err(failed("Run not admitted"));
                    }
                    self.used = true; // Consume admission before any process creation; never retry.
                    self.run_request = Some((request.request_id.clone(), body_sha.clone()));
                    self.original = Some(run(self.spec.clone()));
                }
                Command::State => {}
                Command::Close => self
                    .original
                    .as_mut()
                    .ok_or_else(|| failed("No original owner"))?
                    .request_normal_close()?,
                Command::ReleaseClosed => {
                    let original = self
                        .original
                        .as_mut()
                        .ok_or_else(|| failed("No original owner"))?;
                    if !original.physically_closed()? {
                        return Err(failed("Original child/family not physically closed"));
                    }
                    self.released_original = Some(original.description());
                    // Empty was observed through the same original handles; no Completion is synthesized.
                    let released = self.original.take();
                    drop(released);
                }
                Command::FinishHost => {
                    if !finish_allowed(self.original.is_some()) {
                        return Err(failed(
                            "FinishHost rejected while an original carrier remains",
                        ));
                    }
                    self.stop = true;
                }
                Command::AbortHost { approval } => {
                    if approval != self.abort || self.original.is_none() {
                        return Err(failed("Separate abort authority required"));
                    }
                    // Explicit PM decision, never generated by N/C controllers. Persist before host exit.
                    self.aborted = true;
                }
            };
            Ok(())
        });
        let error = action
            .err()
            .map(|original| self.errors.record(ErrorLane::Control, original));
        let mut response = json!({"version":1,"request_id":request.request_id,"ok":error.is_none(),
            "error":error,"state":self.state()});
        if let Err(error) = write_new(
            &self
                .spec
                .evidence
                .join(format!("control-{}.json", request.request_id)),
            &response,
        ) {
            self.errors.record(ErrorLane::Transport, failed(error));
            self.aborted = false;
            self.stop = false; // No abort/stop without its durable decision record.
            response = json!({"ok":false,"error":"decision_record_failed","state":self.state()});
        }
        if self.cache.len() >= 64 {
            self.emergency = Some((request.request_id, body_sha, response.clone()));
        } else {
            self.cache
                .insert(request.request_id, (body_sha, response.clone()));
        }
        response
    }
}
struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
struct Local(*mut c_void);
impl Drop for Local {
    fn drop(&mut self) {
        unsafe {
            let _ = LocalFree(Some(HLOCAL(self.0)));
        }
    }
}
fn creation(process: HANDLE) -> io::Result<u64> {
    let (mut c, mut e, mut k, mut u) = (
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
    );
    // SAFETY: live process handle and initialized writable time outputs.
    unsafe { GetProcessTimes(process, &mut c, &mut e, &mut k, &mut u) }
        .map_err(io::Error::other)?;
    Ok((u64::from(c.dwHighDateTime) << 32) | u64::from(c.dwLowDateTime))
}
fn random() -> io::Result<String> {
    let mut bytes = [0u8; 32];
    // SAFETY: initialized bounded writable byte buffer, system RNG provider.
    unsafe { BCryptGenRandom(None, &mut bytes, BCRYPT_USE_SYSTEM_PREFERRED_RNG) }
        .ok()
        .map_err(io::Error::other)?;
    Ok(bytes.iter().map(|v| format!("{v:02x}")).collect())
}
fn user_sid() -> io::Result<String> {
    let mut token = HANDLE::default();
    // SAFETY: current process pseudo-handle; returned token gets one private owner.
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }
        .map_err(io::Error::other)?;
    let token = Handle(token);
    let mut length = 0;
    unsafe {
        let _ = GetTokenInformation(token.0, TokenUser, None, 0, &mut length);
    }
    if length == 0 || length > 65536 {
        return Err(io::Error::other("Token size rejected"));
    }
    let mut buffer = vec![0usize; (length as usize).div_ceil(std::mem::size_of::<usize>())];
    // SAFETY: aligned bounded token storage and returned SID allocation remain live.
    unsafe {
        GetTokenInformation(
            token.0,
            TokenUser,
            Some(buffer.as_mut_ptr().cast()),
            length,
            &mut length,
        )
        .map_err(io::Error::other)?;
        let user = &*buffer.as_ptr().cast::<TOKEN_USER>();
        let mut sid = PWSTR::null();
        ConvertSidToStringSidW(user.User.Sid, &mut sid).map_err(io::Error::other)?;
        let _allocation = Local(sid.0.cast());
        sid.to_string().map_err(io::Error::other)
    }
}
fn pipe(path: &str, first: bool) -> io::Result<NamedPipeServer> {
    let sddl: Vec<u16> = format!("D:P(A;;GA;;;{})", user_sid()?)
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let mut descriptor = PSECURITY_DESCRIPTOR::default();
    // SAFETY: SDDL buffer is terminated; Local owns the returned allocation through creation.
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            PCWSTR(sddl.as_ptr()),
            SDDL_REVISION_1,
            &mut descriptor,
            None,
        )
    }
    .map_err(io::Error::other)?;
    let _allocation = Local(descriptor.0);
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: false.into(),
    };
    // SAFETY: Windows copies these non-inheritable current-user-only attributes synchronously.
    unsafe {
        ServerOptions::new()
            .first_pipe_instance(first)
            .max_instances(2)
            .reject_remote_clients(true)
            .in_buffer_size(MAX_FRAME as u32)
            .out_buffer_size(MAX_FRAME as u32)
            .create_with_security_attributes_raw(
                path,
                (&attributes as *const SECURITY_ATTRIBUTES)
                    .cast_mut()
                    .cast(),
            )
    }
}
async fn read_frame<T: AsyncRead + Unpin>(io: &mut T) -> io::Result<Vec<u8>> {
    let length = io.read_u32().await? as usize;
    if length == 0 || length > MAX_FRAME {
        return Err(io::Error::other("Frame bound rejected"));
    }
    let mut bytes = vec![0u8; length];
    io.read_exact(&mut bytes).await?;
    Ok(bytes)
}
async fn write_frame<T: AsyncWrite + Unpin>(io: &mut T, value: &Value) -> io::Result<()> {
    let bytes = serde_json::to_vec(value).map_err(io::Error::other)?;
    if bytes.len() > MAX_FRAME {
        return Err(io::Error::other("Response bound exceeded"));
    }
    io.write_u32(bytes.len() as u32).await?;
    io.write_all(&bytes).await?;
    io.flush().await
}
fn env_pin(path_name: &str, sha_name: &str) -> Result<(PathBuf, String), Failure> {
    Ok((
        std::env::var_os(path_name)
            .map(PathBuf::from)
            .ok_or_else(|| failed(format!("Missing {path_name}")))?,
        std::env::var(sha_name).map_err(failed)?,
    ))
}
fn verify_server(
    client: &tokio::net::windows::named_pipe::NamedPipeClient,
    ready: &Ready,
) -> Result<(), Failure> {
    let mut pid = 0;
    // SAFETY: live connected pipe handle, read-only server identity query.
    unsafe { GetNamedPipeServerProcessId(HANDLE(client.as_raw_handle()), &mut pid) }
        .map_err(failed)?;
    if pid != ready.host_pid {
        return Err(failed("Control server PID mismatch"));
    }
    if std::env::current_exe()
        .map_err(failed)?
        .canonicalize()
        .map_err(failed)?
        != ready.host_image
    {
        return Err(failed(
            "Controller and host must use the same fixed test BIN",
        ));
    }
    // This read-only verification handle is not a replacement owner of the product child.
    let process = Handle(
        unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.map_err(failed)?,
    );
    if creation(process.0).map_err(failed)? != ready.host_creation {
        return Err(failed("Control host generation mismatch"));
    }
    let mut buffer = vec![0u16; 32768];
    let mut length = buffer.len() as u32;
    unsafe {
        QueryFullProcessImageNameW(
            process.0,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut length,
        )
    }
    .map_err(failed)?;
    buffer.truncate(length as usize);
    let image = PathBuf::from(std::ffi::OsString::from_wide(&buffer))
        .canonicalize()
        .map_err(failed)?;
    if image != ready.host_image {
        return Err(failed("Control host image mismatch"));
    }
    let (_, sha) =
        content_digest::sha256(&mut File::open(image).map_err(failed)?, 1024 * 1024 * 1024)
            .map_err(|e| Failure::debug(Stage::Admission, e))?;
    super::verify_pin(&ready.host_sha256, &sha)
}
pub(super) fn ready_for_mode(version: u32, expected: Mode, actual: Mode) -> bool {
    version == 1 && expected == actual
}
async fn exchange(ready: &Ready, request: &Request, wait: Duration) -> Result<Value, Failure> {
    let mut client = ClientOptions::new().open(&ready.pipe).map_err(failed)?;
    verify_server(&client, ready)?;
    let value = serde_json::to_value(request).map_err(failed)?;
    timeout(IO_BUDGET, write_frame(&mut client, &value))
        .await
        .map_err(failed)?
        .map_err(failed)?;
    let bytes = timeout(wait, read_frame(&mut client))
        .await
        .map_err(failed)?
        .map_err(failed)?;
    let response: Value = serde_json::from_slice(&bytes).map_err(failed)?;
    if response["version"] != 1 || response["request_id"] != request.request_id {
        return Err(failed(
            "Control response identity mismatch; inspect the original host, do not retry Run",
        ));
    }
    Ok(response)
}
pub(super) fn run_controller(mode: Mode) -> Result<(), Failure> {
    let (path, sha) = env_pin(
        "NATIVE_DEFAULT_HOST_READY",
        "NATIVE_DEFAULT_HOST_READY_SHA256",
    )?;
    let ready: Ready = read_json(&path, &sha)?;
    if !ready_for_mode(ready.version, mode, ready.mode)
        || !ready.pipe.starts_with(r"\\.\pipe\player-default-gate-")
        || ready.run_wait_ms > 65000
    {
        return Err(failed("Control handoff scope rejected"));
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(failed)?;
    runtime.block_on(async {
        let request = Request {
            version: 1,
            request_id: format!("run-{}", std::process::id()),
            nonce: ready.controller_nonce.clone(),
            case_id: None,
            command: Command::Run { mode },
        };
        let response = exchange(&ready, &request, Duration::from_millis(ready.run_wait_ms)).await?;
        if response["ok"] != true || response["state"]["original_owner"]["normal_success"] != true {
            return Err(failed(format!(
                "Original owner remains at host {}: {response}",
                ready.host_pid
            )));
        }
        Ok(())
    })
}

#[test]
#[ignore = "Explicit PM control request against the separately pinned live host; never auto-aborts"]
fn retained_owner_control_request() -> Result<(), String> {
    kinekino_platform::install();
    operator_request().map_err(|e| format!("{:?}: {}", e.stage, e.message))
}
fn operator_request() -> Result<(), Failure> {
    let (path, sha) = env_pin(
        "NATIVE_DEFAULT_HOST_READY",
        "NATIVE_DEFAULT_HOST_READY_SHA256",
    )?;
    let ready: Ready = read_json(&path, &sha)?;
    if ready.version != 1 || !ready.pipe.starts_with(r"\\.\pipe\player-default-gate-") {
        return Err(failed("Control scope rejected"));
    }
    let (request_path, request_sha) = env_pin(
        "NATIVE_DEFAULT_CONTROL_REQUEST",
        "NATIVE_DEFAULT_CONTROL_REQUEST_SHA256",
    )?;
    let request: Request = read_json(&request_path, &request_sha)?;
    if !valid_header(&request) || matches!(request.command, Command::Run { .. }) {
        return Err(failed("Use N/C controller for Run"));
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(failed)?;
    let response = runtime.block_on(exchange(&ready, &request, IO_BUDGET))?;
    let parent = path
        .parent()
        .ok_or_else(|| failed("Missing control evidence directory"))?;
    write_new(
        &parent.join(format!("operator-response-{}.json", request.request_id)),
        &response,
    )
    .map_err(failed)?;
    if response["ok"] != true {
        return Err(failed(
            "Original host rejected control request; response preserved",
        ));
    }
    Ok(())
}

#[test]
#[ignore = "Explicit separately owned control service; stays alive for PM commands; OS lease required"]
fn retained_owner_host_service() -> Result<(), String> {
    kinekino_platform::install();
    serve_host().map_err(|e| format!("{:?}: {}", e.stage, e.message))
}
fn serve_host() -> Result<(), Failure> {
    let (path, sha) = env_pin(
        "NATIVE_DEFAULT_HOST_CONFIG",
        "NATIVE_DEFAULT_HOST_CONFIG_SHA256",
    )?;
    let config: Config = read_json(&path, &sha)?;
    if config.spec.total > Duration::from_secs(60)
        || config.spec.total <= config.spec.cleanup
        || config.spec.cleanup.is_zero()
    {
        return Err(failed("Host case budget rejected"));
    }
    super::arguments(config.spec.mode, config.spec.project.as_ref())?;
    let image = std::env::current_exe().map_err(failed)?;
    let own_spec = GateSpec {
        mode: Mode::NoArgs,
        binary: image,
        sha256: config.host_sha256.clone(),
        cwd: config.spec.cwd.clone(),
        evidence: config.spec.evidence.clone(),
        project: None,
        total: config.spec.total,
        cleanup: config.spec.cleanup,
    };
    let host_guard = Pin::open(&own_spec)?;
    let controller = random().map_err(failed)?;
    let admin = random().map_err(failed)?;
    let abort = random().map_err(failed)?;
    if controller == admin || controller == abort || admin == abort {
        return Err(failed("Independent control authorities required"));
    }
    let name = format!(
        r"\\.\pipe\player-default-gate-{}-{}",
        std::process::id(),
        random().map_err(failed)?
    );
    let ready = Ready {
        version: 1,
        pipe: name.clone(),
        host_pid: std::process::id(),
        host_creation: creation(unsafe { GetCurrentProcess() }).map_err(failed)?,
        host_image: host_guard.path.clone(),
        host_sha256: host_guard.sha256.clone(),
        config_sha256: sha,
        mode: config.spec.mode,
        controller_nonce: controller.clone(),
        run_wait_ms: config.spec.total.as_millis() as u64 + 3000,
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(failed)?;
    runtime.block_on(async {
        let mut server = pipe(&name, true).map_err(failed)?; // Security is installed before any product owner exists.
        write_new(
            &config.spec.evidence.join("host-ready.json"),
            &serde_json::to_value(&ready).map_err(failed)?,
        )
        .map_err(failed)?;
        write_new(
            &config.spec.evidence.join("host-admin.json"),
            &json!({"version":1,"admin_nonce":admin,"abort_nonce":abort}),
        )
        .map_err(failed)?;
        let mut host = Host {
            spec: config.spec,
            controller,
            admin,
            abort,
            used: false,
            run_request: None,
            original: None,
            released_original: None,
            cache: BTreeMap::new(),
            emergency: None,
            errors: FailureLog::default(),
            stop: false,
            aborted: false,
        };
        loop {
            // Explicit service lifetime, not a case-budget retry or failed-case wait loop.
            if let Err(error) = server.connect().await {
                host.errors.record(ErrorLane::Transport, failed(error));
                let _ = server.disconnect();
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
            let result = async {
                let bytes = timeout(IO_BUDGET, read_frame(&mut server))
                    .await
                    .map_err(failed)?
                    .map_err(failed)?;
                let request: Request = serde_json::from_slice(&bytes).map_err(failed)?;
                let sha = digest(&bytes)?;
                let response = host.handle(request, sha);
                timeout(IO_BUDGET, write_frame(&mut server, &response))
                    .await
                    .map_err(failed)?
                    .map_err(failed)
            }
            .await;
            if let Err(error) = result {
                host.errors.record(ErrorLane::Transport, error);
            }
            if host.aborted {
                // This explicit admin-only exit drops its own Job. It never certifies child cleanup.
                return Err(failed(
                    "Operator-authorized host abort; original completion unconfirmed",
                ));
            }
            if host.stop && host.original.is_none() {
                return Ok(());
            }
            // Keep a name owner while replacing buffered session I/O state.
            match pipe(&name, false) {
                Ok(next) => {
                    let old = std::mem::replace(&mut server, next);
                    let _ = old.disconnect();
                }
                Err(error) => {
                    host.errors.record(ErrorLane::Transport, failed(error));
                    // Fenced state forbids Run. Existing scope/id replay checks remain in force.
                    let _ = server.disconnect();
                }
            }
        }
    })
}
