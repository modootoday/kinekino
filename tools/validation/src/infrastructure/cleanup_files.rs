use crate::{domain::cleanup, infrastructure::files};
use anyhow::{Result, ensure};
use serde::Serialize;
use std::{
    fs,
    os::windows::fs::MetadataExt,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Entry {
    pub relative: String,
    pub directory: bool,
    pub bytes: u64,
    modified: u64,
}

fn plain(path: &Path) -> Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(path)?;
    ensure!(
        metadata.file_attributes() & 0x400 == 0,
        "reparse point rejected: {}",
        path.display()
    );
    Ok(metadata)
}

pub fn root(root: &Path) -> Result<PathBuf> {
    let root = root.canonicalize()?;
    ensure!(
        fs::read_to_string(root.join("Cargo.toml"))?.contains("crates/kinekino-player"),
        "wrong project root"
    );
    plain(&root.join("runtime"))?;
    Ok(root)
}

pub fn dependencies(root: &Path) -> Result<()> {
    ensure!(
        files::hash(&root.join("runtime/display-driver-review/devcon.exe"))?
            == "77e87e7c4e23d6b9a4f5d86d15ed9ec61a16a242fb299c390af6f179225bfa05",
        "independent reviewed devcon missing or changed"
    );
    let provenance = root.join("runtime/validation-vm/media/noprompt-provenance.json");
    if provenance.exists() {
        ensure!(
            files::hash(&provenance)?
                == files::hash(
                    &root.join(".spec/evidence/environment/vm-iso-provenance-before-cleanup.json")
                )?,
            "ISO provenance must be preserved first"
        );
    }
    let output = Command::new(r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Get-VM -ErrorAction Stop | Select-Object Name,Path | ConvertTo-Json -Depth 2",
        ])
        .creation_flags(0x08000000)
        .output()?;
    ensure!(
        output.status.success(),
        "cannot verify Hyper-V registry; cleanup stopped"
    );
    let text = String::from_utf8_lossy(&output.stdout);
    if !text.trim().is_empty() {
        let value: serde_json::Value =
            serde_json::from_str(text.trim_start_matches('\u{feff}').trim())?;
        let vms = value.as_array().cloned().unwrap_or_else(|| vec![value]);
        let target = root.join(cleanup::TARGET);
        for vm in vms {
            ensure!(
                vm["Name"] != "Source-DPI-Lab",
                "target VM is still registered"
            );
            let path = vm["Path"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("VM path unavailable"))?;
            ensure!(
                !Path::new(path).canonicalize()?.starts_with(&target),
                "registered VM uses target directory"
            );
        }
    }
    Ok(())
}

pub fn inventory(root: &Path) -> Result<Vec<Entry>> {
    let target = root.join(cleanup::TARGET);
    if !target.try_exists()? {
        return Ok(Vec::new());
    }
    plain(&target)?;
    let canonical = target.canonicalize()?;
    ensure!(canonical.starts_with(root), "target outside project");
    let mut entries = Vec::new();
    walk(&target, &target, &canonical, &mut entries)?;
    entries.sort_by(|a, b| a.relative.cmp(&b.relative));
    Ok(entries)
}

fn walk(target: &Path, folder: &Path, canonical: &Path, entries: &mut Vec<Entry>) -> Result<()> {
    for item in fs::read_dir(folder)? {
        let path = item?.path();
        let metadata = plain(&path)?;
        ensure!(
            path.canonicalize()?.starts_with(canonical),
            "entry outside target"
        );
        let relative = path
            .strip_prefix(target)?
            .to_string_lossy()
            .replace('\\', "/");
        ensure!(
            cleanup::owned_entry(&relative, metadata.is_dir()),
            "unexpected asset: {relative}"
        );
        ensure!(metadata.is_dir() || metadata.is_file(), "unsupported asset");
        entries.push(Entry {
            relative,
            directory: metadata.is_dir(),
            bytes: if metadata.is_file() {
                metadata.len()
            } else {
                0
            },
            modified: metadata.last_write_time(),
        });
        if metadata.is_dir() {
            walk(target, &path, canonical, entries)?;
        }
    }
    Ok(())
}

pub fn remove_entry(root: &Path, entry: &Entry) -> Result<()> {
    let target = root.join(cleanup::TARGET);
    plain(&root.join("runtime"))?;
    plain(&target)?;
    let path = target.join(&entry.relative);
    // Recheck every ancestor before operating; never follow a junction/symlink.
    let mut ancestor = path.parent();
    while let Some(folder) = ancestor {
        plain(folder)?;
        if folder == target {
            break;
        }
        ensure!(folder.starts_with(&target), "ancestor outside target");
        ancestor = folder.parent();
    }
    let metadata = plain(&path)?;
    ensure!(
        path.canonicalize()?.starts_with(target.canonicalize()?),
        "entry moved outside target"
    );
    ensure!(metadata.is_dir() == entry.directory, "entry type changed");
    if entry.directory {
        fs::remove_dir(path)?;
    } else {
        ensure!(
            metadata.len() == entry.bytes && metadata.last_write_time() == entry.modified,
            "file changed after planning"
        );
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn remove_empty_target(root: &Path) -> Result<()> {
    plain(&root.join("runtime"))?;
    let target = root.join(cleanup::TARGET);
    if target.try_exists()? {
        plain(&target)?;
        fs::remove_dir(target)?;
    }
    Ok(())
}
