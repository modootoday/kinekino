use kinekino_player::domain::local_endpoint::LocalEndpoint;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    os::windows::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};
use windows::{
    Win32::{
        Foundation::{CloseHandle, FILETIME, HANDLE},
        System::Threading::{
            GetExitCodeProcess, GetProcessTimes, OpenProcess, PROCESS_NAME_WIN32,
            PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
        },
    },
    core::PWSTR,
};
pub type TestResult<T> = Result<T, String>;
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub fixture_kind: String,
    pub run_id: String,
    pub exclusive_control: bool,
    pub gui_pid: u32,
    pub gui_executable: PathBuf,
    pub mpv_pid: u32,
    pub mpv_executable: PathBuf,
    pub mpv_pipe: String,
    pub endpoint: String,
    pub source_path: PathBuf,
}
impl Fixture {
    pub fn load() -> TestResult<Self> {
        let path = std::env::var_os("PLAYER_NATIVE_GUI_FIXTURE")
            .ok_or("Missing explicit owned fixture file")?;
        let bytes = std::fs::read(path).map_err(|_| "Cannot read fixture")?;
        if bytes.len() > 16384 {
            return Err("Fixture exceeds size bound".into());
        }
        let fixture: Self = serde_json::from_slice(&bytes).map_err(|_| "Invalid fixture JSON")?;
        if fixture.fixture_kind != "owned-normal-gui-sintel-v1"
            || !fixture.exclusive_control
            || fixture.run_id.is_empty()
            || fixture.run_id.len() > 80
            || !fixture
                .run_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err("Explicit owned GUI lease is required".into());
        }
        let endpoint = LocalEndpoint::parse(&fixture.endpoint).map_err(str::to_owned)?;
        if endpoint.process_id != fixture.gui_pid || fixture.gui_pid == fixture.mpv_pid {
            return Err("Fixture process identities disagree".into());
        }
        let prefix = format!(r"\\.\pipe\kinekino-{}-", fixture.gui_pid);
        if !fixture.mpv_pipe.strip_prefix(&prefix).is_some_and(|nonce| {
            !nonce.is_empty() && nonce.len() <= 40 && nonce.bytes().all(|b| b.is_ascii_digit())
        }) {
            return Err("Invalid owned mpv pipe".into());
        }
        if fixture.source_path.file_name().and_then(|v| v.to_str()) != Some("Sintel.2010.720p.mkv")
            || !fixture.source_path.is_absolute()
        {
            return Err("Only explicitly owned Sintel fixture is permitted".into());
        }
        Ok(fixture)
    }
}
struct Process {
    handle: HANDLE,
    start: u64,
}
impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}
impl Process {
    fn open(pid: u32, path: &Path) -> TestResult<Self> {
        let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }
            .map_err(|_| "Cannot inspect owned process")?;
        let mut process = Self { handle, start: 0 };
        let mut buffer = vec![0u16; 32768];
        let mut length = buffer.len() as u32;
        unsafe {
            QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_WIN32,
                PWSTR(buffer.as_mut_ptr()),
                &mut length,
            )
        }
        .map_err(|_| "Cannot inspect owned executable")?;
        let actual = PathBuf::from(
            String::from_utf16(&buffer[..length as usize])
                .map_err(|_| "Invalid executable identity")?,
        );
        if actual.canonicalize().ok() != path.canonicalize().ok() || !path.is_file() {
            return Err("Owned executable identity mismatch".into());
        }
        let (mut creation, mut exit, mut kernel, mut user) = (
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
        );
        unsafe { GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) }
            .map_err(|_| "Cannot inspect process generation")?;
        process.start =
            (u64::from(creation.dwHighDateTime) << 32) | u64::from(creation.dwLowDateTime);
        process.check()?;
        Ok(process)
    }
    fn check(&self) -> TestResult<()> {
        let mut code = 0;
        unsafe { GetExitCodeProcess(self.handle, &mut code) }
            .map_err(|_| "Owned process observation failed")?;
        if code != 259 {
            return Err("Owned process ended; no further controls permitted".into());
        }
        Ok(())
    }
}
pub struct Owned {
    gui: Process,
    mpv: Process,
    _source: File,
}
impl Owned {
    pub fn open(fixture: &Fixture) -> TestResult<Self> {
        let gui = Process::open(fixture.gui_pid, &fixture.gui_executable)?;
        if gui.start
            != LocalEndpoint::parse(&fixture.endpoint)
                .map_err(str::to_owned)?
                .process_start
        {
            return Err("Endpoint generation mismatch".into());
        }
        if fixture.mpv_executable.file_name().and_then(|v| v.to_str()) != Some("mpv.exe") {
            return Err("Expected owned native executable".into());
        }
        let mpv = Process::open(fixture.mpv_pid, &fixture.mpv_executable)?;
        // Existing readers remain allowed; write/delete opens conflict while this handle lives.
        let source = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&fixture.source_path)
            .map_err(|_| "Cannot freeze owned source against writes/replacement")?;
        if source
            .metadata()
            .map_err(|_| "Cannot inspect owned source")?
            .len()
            == 0
        {
            return Err("Owned source is empty".into());
        }
        Ok(Self {
            gui,
            mpv,
            _source: source,
        })
    }
    pub fn check(&self) -> TestResult<()> {
        self.gui.check()?;
        self.mpv.check()
    }
    pub fn validation(&self) -> serde_json::Value {
        serde_json::json!({"gui_process_start":self.gui.start.to_string(),"mpv_process_start":self.mpv.start.to_string(),"executable_paths_verified":true,"source_write_delete_guard":true})
    }
}
