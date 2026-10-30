use crate::{
    domain::cleanup,
    infrastructure::{cleanup_files, files},
};
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::path::Path;

pub fn execute(root: &Path, mode: &str) -> Result<Value> {
    ensure!(matches!(mode, "plan" | "apply"), "cleanup-vm plan|apply");
    let root = cleanup_files::root(root)?;
    cleanup_files::dependencies(&root)?;
    let mut entries = cleanup_files::inventory(&root)?;
    let plan = json!({"mode":mode,"target":root.join(cleanup::TARGET),"logical_bytes":entries.iter().map(|e|e.bytes).sum::<u64>(),"entries":entries,"vm_registry":"target absent","deletion_performed":false});
    files::write(
        &root.join("runtime/verification/cleanup-vm-plan.json"),
        &plan,
    )?;
    if mode == "plan" {
        return Ok(plan);
    }
    ensure!(
        cleanup_files::inventory(&root)? == entries,
        "target changed after planning"
    );
    entries.sort_by(|a, b| {
        b.relative
            .matches('/')
            .count()
            .cmp(&a.relative.matches('/').count())
            .then_with(|| a.directory.cmp(&b.directory))
    });
    let mut journal = json!({"target":root.join(cleanup::TARGET),"status":"applying","removed":[],"deletion_performed":false});
    let journal_path = root.join("runtime/verification/cleanup-vm-result.json");
    files::write(&journal_path, &journal)?;
    for entry in &entries {
        if let Err(error) = cleanup_files::remove_entry(&root, entry) {
            journal["status"] = json!("failed-partial");
            journal["error"] = json!(error.to_string());
            files::write(&journal_path, &journal)?;
            return Err(error);
        }
        journal["removed"]
            .as_array_mut()
            .expect("owned journal array")
            .push(json!(entry.relative));
        journal["deletion_performed"] = json!(true);
        files::write(&journal_path, &journal)?;
    }
    cleanup_files::remove_empty_target(&root)?;
    journal["status"] = json!("complete");
    journal["target_exists"] = json!(root.join(cleanup::TARGET).try_exists()?);
    files::write(&journal_path, &journal)?;
    Ok(journal)
}
