use anyhow::Result;
use serde::Serialize;
use std::{
    mem::size_of,
    os::windows::process::CommandExt,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use windows::Win32::{
    Foundation::{CloseHandle, FILETIME},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        },
        ProcessStatus::{
            K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
        },
        SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX},
        Threading::{
            BELOW_NORMAL_PRIORITY_CLASS, GetSystemTimes, OpenProcess, PROCESS_QUERY_INFORMATION,
            PROCESS_VM_READ,
        },
    },
};

#[derive(Serialize)]
pub struct Process {
    pub id: u32,
    pub parent: u32,
    pub name: String,
}

pub fn inventory() -> Result<Vec<Process>> {
    // SAFETY: valid owned snapshot, initialized structure size, closed before returning.
    unsafe {
        let handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut result = Vec::new();
        if Process32FirstW(handle, &mut entry).is_ok() {
            loop {
                let len = entry
                    .szExeFile
                    .iter()
                    .position(|c| *c == 0)
                    .unwrap_or(entry.szExeFile.len());
                result.push(Process {
                    id: entry.th32ProcessID,
                    parent: entry.th32ParentProcessID,
                    name: String::from_utf16_lossy(&entry.szExeFile[..len]),
                });
                if Process32NextW(handle, &mut entry).is_err() {
                    break;
                }
            }
        }
        CloseHandle(handle)?;
        Ok(result)
    }
}

fn times() -> Result<(u64, u64)> {
    let (mut idle, mut kernel, mut user) = (
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
    );
    // SAFETY: all output pointers are valid local FILETIME values.
    unsafe {
        GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user))?;
    }
    let ticks = |v: FILETIME| ((v.dwHighDateTime as u64) << 32) | v.dwLowDateTime as u64;
    Ok((ticks(idle), ticks(kernel) + ticks(user)))
}

pub fn resources() -> Result<(u64, f64)> {
    let mut memory = MEMORYSTATUSEX {
        dwLength: size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    // SAFETY: initialized size and valid output structure.
    unsafe {
        GlobalMemoryStatusEx(&mut memory)?;
    }
    let before = times()?;
    thread::sleep(Duration::from_secs(1));
    let after = times()?;
    let total = after.1.saturating_sub(before.1);
    anyhow::ensure!(total > 0, "system CPU sample unavailable");
    Ok((
        memory.ullAvailPhys,
        100.0 * (1.0 - after.0.saturating_sub(before.0) as f64 / total as f64),
    ))
}

fn memory(id: u32) -> Option<(usize, usize)> {
    // SAFETY: read-only process handle and correctly sized output; handle always closed.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, id).ok()?;
        let mut counters = PROCESS_MEMORY_COUNTERS_EX::default();
        let ok = K32GetProcessMemoryInfo(
            handle,
            (&mut counters as *mut PROCESS_MEMORY_COUNTERS_EX).cast::<PROCESS_MEMORY_COUNTERS>(),
            size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
        )
        .as_bool();
        let _ = CloseHandle(handle);
        ok.then_some((counters.PrivateUsage, counters.WorkingSetSize))
    }
}

pub fn measure(
    executable: &Path,
    args: &[String],
    stdout: &Path,
    stderr: &Path,
    cache: Option<&Path>,
) -> Result<serde_json::Value> {
    measure_in(executable, args, stdout, stderr, None, cache)
}

pub fn measure_in(
    executable: &Path,
    args: &[String],
    stdout: &Path,
    stderr: &Path,
    directory: Option<&Path>,
    cache: Option<&Path>,
) -> Result<serde_json::Value> {
    let mut command = Command::new(executable);
    if let Some(directory) = directory {
        command.current_dir(directory);
    }
    if let Some(cache) = cache {
        command.env("HF_HOME", cache);
    }
    let mut child = command
        .args(args)
        .env("RAYON_NUM_THREADS", "4")
        .env("OMP_NUM_THREADS", "4")
        .env("MKL_NUM_THREADS", "4")
        .env("HF_HUB_OFFLINE", "1")
        .env("TOKENIZERS_PARALLELISM", "false")
        .stdout(Stdio::from(std::fs::File::create(stdout)?))
        .stderr(Stdio::from(std::fs::File::create(stderr)?))
        .creation_flags(BELOW_NORMAL_PRIORITY_CLASS.0)
        .spawn()?;
    let clock = Instant::now();
    let (mut private, mut working, mut samples) = (0, 0, 0);
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if clock.elapsed() > Duration::from_secs(600) {
            timed_out = true;
            child.kill()?;
            break child.wait()?;
        }
        let mut ids = vec![child.id()];
        let list = inventory()?;
        loop {
            let before = ids.len();
            for process in &list {
                if ids.contains(&process.parent) && !ids.contains(&process.id) {
                    ids.push(process.id);
                }
            }
            if ids.len() == before {
                break;
            }
        }
        let totals = ids
            .into_iter()
            .filter_map(memory)
            .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
        private = private.max(totals.0);
        working = working.max(totals.1);
        samples += 1;
        thread::sleep(Duration::from_millis(100));
    };
    Ok(
        serde_json::json!({"exit_code":status.code(),"success":status.success() && !timed_out,"timed_out":timed_out,"peak_private_bytes":private,
        "peak_working_set_bytes":working,"samples":samples,"sample_interval_ms":100,
        "elapsed_seconds":clock.elapsed().as_secs_f64(),"priority":"BelowNormal",
        "scope":"owned worker tree; sampled peak includes load; OS file cache not cleared"}),
    )
}
