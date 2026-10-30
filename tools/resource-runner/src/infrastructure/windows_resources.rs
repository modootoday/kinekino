use crate::{
    execution_admission::{QueryFailure, Resources},
    execution_profile::Snapshot,
};
use std::{
    io,
    mem::size_of,
    time::{Duration, Instant},
};

#[repr(C)]
#[derive(Default)]
struct FileTime {
    low: u32,
    high: u32,
}
impl FileTime {
    fn ticks(self) -> u64 {
        u64::from(self.high) << 32 | u64::from(self.low)
    }
}
#[repr(C)]
#[derive(Default)]
struct MemoryStatus {
    length: u32,
    load: u32,
    total_physical: u64,
    available_physical: u64,
    total_page_file: u64,
    available_page_file: u64,
    total_virtual: u64,
    available_virtual: u64,
    available_extended_virtual: u64,
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetSystemTimes(idle: *mut FileTime, kernel: *mut FileTime, user: *mut FileTime) -> i32;
    fn GlobalMemoryStatusEx(memory: *mut MemoryStatus) -> i32;
    fn GetActiveProcessorGroupCount() -> u16;
}
pub struct WindowsResources;
impl Resources for WindowsResources {
    fn snapshot(&mut self) -> Result<Snapshot, QueryFailure> {
        let mut idle = FileTime::default();
        let mut kernel = FileTime::default();
        let mut user = FileTime::default();
        let mut memory = MemoryStatus {
            length: size_of::<MemoryStatus>() as u32,
            ..MemoryStatus::default()
        };
        // SAFETY: initialized, correctly sized Win32 output structures; queries do not mutate processes.
        unsafe {
            // GetSystemTimes is group-local on hosts with >64 processors. Do not call that global load.
            if GetActiveProcessorGroupCount() != 1 {
                return Err(QueryFailure {
                    stage: "unsupported_system_cpu_groups",
                    os_code: None,
                });
            }
            if GetSystemTimes(&mut idle, &mut kernel, &mut user) == 0 {
                return Err(failure("system_times"));
            }
            if GlobalMemoryStatusEx(&mut memory) == 0 {
                return Err(failure("physical_memory"));
            }
        }
        Ok(Snapshot {
            idle: idle.ticks(),
            kernel: kernel.ticks(),
            user: user.ticks(),
            free_bytes: memory.available_physical,
        })
    }
    fn wait_interval(&mut self) -> u128 {
        let start = Instant::now();
        std::thread::sleep(Duration::from_secs(1));
        start.elapsed().as_millis()
    }
}
fn failure(stage: &'static str) -> QueryFailure {
    QueryFailure {
        stage,
        os_code: io::Error::last_os_error().raw_os_error(),
    }
}
