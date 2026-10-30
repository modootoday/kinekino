use super::preflight;
use crate::infrastructure::{engine_commands, files, processes};
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub fn execute(root: &Path, engine: &str) -> Result<Value> {
    let before = preflight::inspect(root)?;
    ensure!(
        before["ready"] == true,
        "resource preflight waiting; preserve existing applications"
    );
    let folder = root.join("runtime/ai-benchmark").canonicalize()?;
    let model = files::json(&folder.join("q4-model.json"))?;
    let declared = PathBuf::from(
        model["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing model path"))?,
    );
    let path = if declared.is_absolute() {
        declared
    } else {
        root.join(declared)
    };
    let resolved = path.canonicalize()?;
    ensure!(
        resolved.starts_with(&folder) && path.extension().is_some_and(|e| e == "gguf"),
        "invalid model location"
    );
    ensure!(
        files::hash(&path)? == model["sha256"].as_str().unwrap_or_default(),
        "model hash mismatch"
    );
    let (relative, args) = engine_commands::benchmark(engine, &path)?;
    let exe = folder.join(relative);
    let mut result = processes::measure(
        &exe,
        &args,
        &folder.join(format!("{engine}-rust.stdout")),
        &folder.join(format!("{engine}-rust.stderr")),
        Some(&folder),
    )?;
    result["engine"] = json!(engine);
    result["model"] = model;
    result["executable_sha256"] = json!(files::hash(&exe)?);
    result["runtime_directory_bytes"] = json!(files::directory_bytes(exe.parent().unwrap())?);
    result["arguments"] = json!(args);
    result["preflight"] = before;
    result["after"] = preflight::inspect(root)?;
    result["scope_note"] = json!(
        "Q4 CPU synthetic throughput; engine benchmark protocols differ; no quality or GPU claim"
    );
    files::write(&folder.join(format!("{engine}-rust.json")), &result)?;
    ensure!(
        result["success"] == true,
        "benchmark failed; retained result and stderr"
    );
    Ok(result)
}
