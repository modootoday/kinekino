//! Minimal Cargo.toml reader (xtask has no dependencies): name and dependency lines.
use std::{fs, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepKind {
    Normal,
    Dev,
    Build,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dep {
    pub name: String,
    pub kind: DepKind,
    pub workspace: bool,
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub name: String,
    /// Relative to the workspace root, e.g. `crates/kinekino-app`.
    pub dir: String,
    pub deps: Vec<Dep>,
}

pub fn workspace(root: &Path) -> Vec<Manifest> {
    let mut dirs: Vec<String> = fs::read_dir(root.join("crates"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().join("Cargo.toml").is_file())
        .map(|e| format!("crates/{}", e.file_name().to_string_lossy()))
        .collect();
    dirs.sort();
    dirs.push("xtask".into());
    dirs.into_iter()
        .filter_map(|d| {
            let text = fs::read_to_string(root.join(&d).join("Cargo.toml")).ok()?;
            Some(parse(&d, &text))
        })
        .collect()
}

pub fn parse(dir: &str, text: &str) -> Manifest {
    let mut name = String::new();
    let mut section = String::new();
    let mut deps = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') {
            section = line.trim_matches(|c| c == '[' || c == ']').to_owned();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        if section == "package" && key == "name" {
            name = value.trim_matches('"').to_owned();
        }
        let kind = match section.rsplit('.').next() {
            Some("dependencies") => DepKind::Normal,
            Some("dev-dependencies") => DepKind::Dev,
            Some("build-dependencies") => DepKind::Build,
            _ => continue,
        };
        if key.starts_with('#') {
            continue;
        }
        let (dep, workspace) = match key.strip_suffix(".workspace") {
            Some(d) => (d, value == "true"),
            None => (key, value.replace(' ', "").contains("workspace=true")),
        };
        deps.push(Dep {
            name: dep.trim_matches('"').to_owned(),
            kind,
            workspace,
        });
    }
    Manifest {
        name,
        dir: dir.to_owned(),
        deps,
    }
}
