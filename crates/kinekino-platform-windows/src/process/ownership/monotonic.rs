use windows::Win32::System::SystemInformation::GetTickCount64;

/// Host-wide boot-relative tick shared by parent and child processes.
pub(super) fn uptime_millis() -> u64 {
    unsafe { GetTickCount64() }
}
