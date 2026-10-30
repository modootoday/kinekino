//! Config file location and schema (JSON, unknown keys rejected so secrets cannot slip in).
use crate::Env;
use kinekino_core::port::{ErrorCode, PortError, PortResult};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct FileConfig {
    browser: BrowserSection,
    inference: InferenceSection,
    output: OutputSection,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct BrowserSection {
    engine: Option<String>,
    headless: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct InferenceSection {
    mode: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct OutputSection {
    format: Option<String>,
}

pub(super) struct FileValues(FileConfig);

impl FileValues {
    pub(super) fn get(&self, key: &str) -> Option<String> {
        let c = &self.0;
        match key {
            "browser.engine" => c.browser.engine.clone(),
            "browser.headless" => c.browser.headless.map(|b| b.to_string()),
            "inference.mode" => c.inference.mode.clone(),
            "output.format" => c.output.format.clone(),
            _ => None,
        }
    }
}

/// `KINEKINO_CONFIG`, else `$XDG_CONFIG_HOME|$HOME/.config/kinekino/config.json`, else
/// `%APPDATA%\kinekino\config.json`.
pub fn config_path(env: &Env) -> Option<PathBuf> {
    if let Some(p) = env.get("KINEKINO_CONFIG") {
        return Some(PathBuf::from(p));
    }
    let base = env
        .get("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env.get("HOME").map(|h| Path::new(h).join(".config")))
        .or_else(|| env.get("APPDATA").map(PathBuf::from))?;
    Some(base.join("kinekino").join("config.json"))
}

/// `Ok(None)` when the file does not exist.
pub(super) fn read_file(path: &Path) -> PortResult<Option<FileValues>> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(PortError::new(
                ErrorCode::Io,
                format!("{}: {e}", path.display()),
            ));
        }
    };
    serde_json::from_str(&text)
        .map(|c| Some(FileValues(c)))
        .map_err(|e| PortError::new(ErrorCode::InvalidInput, format!("{}: {e}", path.display())))
}
