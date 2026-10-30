use crate::domain::resources;
use crate::infrastructure::{files, processes};
use anyhow::Result;
use serde_json::{Value, json};
use std::path::Path;

pub fn inspect(root: &Path) -> Result<Value> {
    let list = processes::inventory()?;
    let protected: Vec<_> = list
        .iter()
        .filter(|p| {
            [
                "chatgpt.exe",
                "windowsterminal.exe",
                "terminal.exe",
                "openconsole.exe",
                "codex.exe",
                "pwsh.exe",
                "powershell.exe",
            ]
            .contains(&p.name.to_lowercase().as_str())
        })
        .collect();
    let validation: Vec<_> = list
        .iter()
        .filter(|p| resources::is_player_process(&p.name))
        .collect();
    let (free, cpu) = processes::resources()?;
    let report = json!({"free_ram_bytes":free,"cpu_percent":cpu,"protected_processes":protected,
        "validation_apps":validation,"ready":resources::may_benchmark(free,cpu,!validation.is_empty()),
        "modifications":"read-only; no application/service/display/VM/power/memory-cache changes",
        "worker_policy":"one owned worker; BelowNormal; Rayon/OMP/MKL 4; offline; unload by process exit"});
    files::write(
        &root.join("runtime/ai-benchmark/preflight-rust.json"),
        &report,
    )?;
    Ok(report)
}
