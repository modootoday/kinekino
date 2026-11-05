//! JSON file adapter for [`PlayerPrefsStore`] (M1-7): versioned v1 body, atomic per-process
//! tmp + rename writes, corrupt files preserved as `.corrupt`, newer-version files never overwritten.
mod format;
mod location;

pub use location::default_player_prefs_file;

use super::store::PlayerPrefsStore;
use super::value::PlayerPrefs;
use crate::port::error::{ErrorCode, PortError, PortResult};
use format::Decoded;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FilePlayerPrefsStore {
    path: PathBuf,
}

/// File state before a write; `Corrupt` is moved aside on the first write.
enum Loaded {
    Ready(PlayerPrefs),
    Missing,
    Corrupt,
    Newer,
}

impl FilePlayerPrefsStore {
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn sibling(&self, suffix: &str) -> PathBuf {
        let mut name = self.path.file_name().unwrap_or_default().to_owned();
        name.push(suffix);
        self.path.with_file_name(name)
    }

    fn temp_path(&self) -> PathBuf {
        self.sibling(&format!(".{}.tmp", std::process::id()))
    }

    fn error(&self, code: ErrorCode, cause: impl std::fmt::Display) -> PortError {
        PortError::new(
            code,
            format!("player prefs file {}: {cause}", self.path.display()),
        )
    }

    fn load_state(&self) -> PortResult<Loaded> {
        match std::fs::read(&self.path) {
            Ok(body) => Ok(match format::decode(&body) {
                Decoded::V1(prefs) => Loaded::Ready(prefs),
                Decoded::Corrupt => Loaded::Corrupt,
                Decoded::Newer => Loaded::Newer,
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Loaded::Missing),
            Err(e) => Err(self.error(ErrorCode::Io, e)),
        }
    }

    fn load_for_write(&self) -> PortResult<()> {
        match self.load_state()? {
            Loaded::Ready(_) | Loaded::Missing => Ok(()),
            Loaded::Corrupt => {
                std::fs::rename(&self.path, self.sibling(".corrupt"))
                    .map_err(|e| self.error(ErrorCode::Io, e))?;
                Ok(())
            }
            Loaded::Newer => Err(self.error(
                ErrorCode::Unsupported,
                "written by a newer version; not overwritten",
            )),
        }
    }

    fn write(&self, prefs: &PlayerPrefs) -> PortResult<()> {
        let body = format::encode(prefs)
            .map_err(|_| self.error(ErrorCode::Internal, "could not encode prefs"))?;
        let io = |e: std::io::Error| self.error(ErrorCode::Io, e);
        if let Some(dir) = self.path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(io)?;
        }
        let tmp = self.temp_path();
        let written = write_then_rename(&tmp, &self.path, &body);
        if written.is_err() {
            let _ = std::fs::remove_file(&tmp);
        }
        written.map_err(io)
    }
}

fn write_then_rename(tmp: &Path, path: &Path, body: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::File::create(tmp)?;
    f.write_all(body)?;
    f.sync_all()?;
    drop(f);
    std::fs::rename(tmp, path)
}

impl PlayerPrefsStore for FilePlayerPrefsStore {
    fn load(&self) -> PortResult<PlayerPrefs> {
        Ok(match self.load_state()? {
            Loaded::Ready(prefs) => prefs,
            Loaded::Missing | Loaded::Corrupt | Loaded::Newer => PlayerPrefs::default(),
        })
    }

    fn save(&mut self, prefs: &PlayerPrefs) -> PortResult<()> {
        self.load_for_write()?;
        self.write(&prefs.checked()?)
    }
}

#[cfg(test)]
mod tests;
