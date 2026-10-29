//! QueryPerformanceCounter domain anchored on a live original parent.
use kinekino_core::platform::clock::{CounterIdentity, CounterSample, LiveCreator, ParentCounter};
use std::{
    ffi::c_void,
    io,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
};

const KIND: &str = "windows-qpc/1";
const WAIT_TIMEOUT: u32 = 258;
const WAIT_OBJECT_0: u32 = 0;
const TH32CS_SNAPPROCESS: u32 = 2;
const SYNCHRONIZE_QUERY_LIMITED: u32 = 0x0010_1000;

#[repr(C)]
#[derive(Default)]
struct FileTime {
    low: u32,
    high: u32,
}
#[repr(C)]
struct ProcessEntry {
    size: u32,
    usage: u32,
    pid: u32,
    heap: usize,
    module: u32,
    threads: u32,
    parent_pid: u32,
    priority: i32,
    flags: u32,
    executable: [u16; 260],
}
#[link(name = "kernel32")]
unsafe extern "system" {
    #[link_name = "CreateToolhelp32Snapshot"]
    fn snapshot(flags: u32, pid: u32) -> *mut c_void;
    #[link_name = "Process32FirstW"]
    fn first(handle: *mut c_void, entry: *mut ProcessEntry) -> i32;
    #[link_name = "Process32NextW"]
    fn next(handle: *mut c_void, entry: *mut ProcessEntry) -> i32;
    #[link_name = "OpenProcess"]
    fn open_process(access: u32, inherit: i32, pid: u32) -> *mut c_void;
    #[link_name = "WaitForSingleObject"]
    fn wait(handle: *mut c_void, milliseconds: u32) -> u32;
    #[link_name = "GetCurrentProcess"]
    fn current_process() -> *mut c_void;
    #[link_name = "GetProcessTimes"]
    fn process_times(
        handle: *mut c_void,
        creation: *mut FileTime,
        exit: *mut FileTime,
        kernel: *mut FileTime,
        user: *mut FileTime,
    ) -> i32;
    #[link_name = "QueryPerformanceCounter"]
    fn qpc(value: *mut i64) -> i32;
    #[link_name = "QueryPerformanceFrequency"]
    fn qpf(value: *mut i64) -> i32;
}

/// Windows QPC counter adapter.
pub struct WindowsCounter;
/// Process-wide adapter instance.
pub static COUNTER: WindowsCounter = WindowsCounter;

fn sample_for(process: *mut c_void, pid: u32) -> io::Result<CounterSample> {
    let mut creation = FileTime::default();
    let mut exit = FileTime::default();
    let mut kernel = FileTime::default();
    let mut user = FileTime::default();
    let mut frequency = 0i64;
    let mut ticks = 0i64;
    // SAFETY: `process` is a live handle (or the current-process pseudo-handle)
    // and every out-pointer refers to a local of the documented layout.
    let success = unsafe {
        process_times(process, &mut creation, &mut exit, &mut kernel, &mut user) != 0
            && qpf(&mut frequency) != 0
            && qpc(&mut ticks) != 0
    };
    if !success {
        return Err(io::Error::last_os_error());
    }
    if frequency <= 0 || ticks < 0 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "QPC range is unsupported",
        ));
    }
    Ok(CounterSample {
        identity: CounterIdentity {
            kind: KIND,
            frequency_hz: frequency as u64,
            parent_pid: pid,
            parent_creation: u64::from(creation.low) | (u64::from(creation.high) << 32),
        },
        ticks: ticks as u64,
    })
}

fn observed_creator() -> io::Result<u32> {
    // SAFETY: plain snapshot request; the result is validated before use.
    let raw = unsafe { snapshot(TH32CS_SNAPPROCESS, 0) };
    if raw.is_null() || raw as isize == -1 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `raw` is a fresh owned snapshot handle.
    let handle = unsafe { OwnedHandle::from_raw_handle(raw) };
    // SAFETY: ProcessEntry is plain data; zero is a valid bit pattern.
    let mut entry: ProcessEntry = unsafe { std::mem::zeroed() };
    entry.size = std::mem::size_of::<ProcessEntry>() as u32;
    // SAFETY: valid snapshot handle and sized entry.
    if unsafe { first(handle.as_raw_handle(), &mut entry) } == 0 {
        return Err(io::Error::last_os_error());
    }
    loop {
        if entry.pid == std::process::id() {
            return Ok(entry.parent_pid);
        }
        // SAFETY: as above.
        if unsafe { next(handle.as_raw_handle(), &mut entry) } == 0 {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Current process creator was not observed",
            ));
        }
    }
}

struct WindowsCreator {
    handle: OwnedHandle,
    pid: u32,
}
impl WindowsCreator {
    fn live(&self) -> io::Result<bool> {
        // SAFETY: owned live process handle with SYNCHRONIZE access.
        match unsafe { wait(self.handle.as_raw_handle(), 0) } {
            WAIT_TIMEOUT => Ok(true),
            WAIT_OBJECT_0 => Ok(false),
            _ => Err(io::Error::last_os_error()),
        }
    }
}
impl LiveCreator for WindowsCreator {
    fn pid(&self) -> u32 {
        self.pid
    }
    fn sample(&self) -> io::Result<Option<CounterSample>> {
        if !self.live()? {
            return Ok(None);
        }
        let sample = sample_for(self.handle.as_raw_handle(), self.pid)?;
        Ok(self.live()?.then_some(sample))
    }
}

impl ParentCounter for WindowsCounter {
    fn kinds(&self) -> &'static [&'static str] {
        &[KIND]
    }
    fn sample_current(&self) -> io::Result<CounterSample> {
        // SAFETY: the current-process pseudo-handle is borrowed and never closed.
        sample_for(unsafe { current_process() }, std::process::id())
    }
    fn open_creator(&self) -> io::Result<Box<dyn LiveCreator>> {
        let pid = observed_creator()?;
        if pid == 0 {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No creator"));
        }
        // SAFETY: plain OpenProcess; the result is validated before ownership.
        let raw = unsafe { open_process(SYNCHRONIZE_QUERY_LIMITED, 0, pid) };
        if raw.is_null() {
            return Err(io::Error::last_os_error());
        }
        Ok(Box::new(WindowsCreator {
            // SAFETY: `raw` is a fresh owned process handle.
            handle: unsafe { OwnedHandle::from_raw_handle(raw) },
            pid,
        }))
    }
}
