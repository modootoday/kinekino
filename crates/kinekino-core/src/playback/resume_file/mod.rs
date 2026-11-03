//! JSON file adapter for [`ResumeStore`] (#9): versioned v1 body, atomic per-process tmp + rename writes,
//! corrupt files preserved as `.corrupt`, newer-version files never overwritten.
mod format;
mod location;

pub use location::default_resume_file;

use super::resume::ResumePoint;
use super::store::ResumeStore;
use crate::port::error::{ErrorCode, PortError, PortResult};
use format::{Decoded, Entries, Entry};
use std::path::{Path, PathBuf};

/// Most entries kept; the least recently updated are dropped first.
pub const MAX_ENTRIES: usize = 1000;
/// Entries not updated for this long are dropped on the next write.
pub const MAX_AGE_MS: u64 = 180 * 24 * 60 * 60 * 1000;

/// Unix epoch milliseconds.
pub type Clock = fn() -> u64;

fn system_now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Debug, Clone)]
pub struct FileResumeStore {
    path: PathBuf,
    clock: Clock,
}

/// File state before a write; `Corrupt` is moved aside on the first write.
enum Loaded {
    Ready(Entries),
    Corrupt,
    Newer,
}

impl FileResumeStore {
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self::with_clock(path, system_now_ms)
    }
    pub fn with_clock(path: impl Into<PathBuf>, clock: Clock) -> Self {
        Self {
            path: path.into(),
            clock,
        }
    }
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// `<file name><suffix>` next to the store file, e.g. `resume.json.tmp`.
    fn sibling(&self, suffix: &str) -> PathBuf {
        let mut name = self.path.file_name().unwrap_or_default().to_owned();
        name.push(suffix);
        self.path.with_file_name(name)
    }

    /// `<file name>.<pid>.tmp`: each process writes its own temp file, so concurrent
    /// writers never truncate each other's and the last rename wins.
    fn temp_path(&self) -> PathBuf {
        self.sibling(&format!(".{}.tmp", std::process::id()))
    }

    /// Errors carry only the store file path and the cause, never a media key.
    fn error(&self, code: ErrorCode, cause: impl std::fmt::Display) -> PortError {
        PortError::new(
            code,
            format!("resume file {}: {cause}", self.path.display()),
        )
    }

    fn load(&self) -> PortResult<Loaded> {
        match std::fs::read(&self.path) {
            Ok(body) => Ok(match format::decode(&body) {
                Decoded::V1(entries) => Loaded::Ready(entries),
                Decoded::Corrupt => Loaded::Corrupt,
                Decoded::Newer => Loaded::Newer,
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Loaded::Ready(Entries::new())),
            Err(e) => Err(self.error(ErrorCode::Io, e)),
        }
    }

    /// Entries ready to modify: corrupt files are preserved as `.corrupt`, newer ones refused.
    fn load_for_write(&self) -> PortResult<Entries> {
        match self.load()? {
            Loaded::Ready(entries) => Ok(entries),
            Loaded::Corrupt => {
                std::fs::rename(&self.path, self.sibling(".corrupt"))
                    .map_err(|e| self.error(ErrorCode::Io, e))?;
                Ok(Entries::new())
            }
            Loaded::Newer => Err(self.error(
                ErrorCode::Unsupported,
                "written by a newer version; not overwritten",
            )),
        }
    }

    fn save(&self, entries: &mut Entries) -> PortResult<()> {
        format::prune(entries, (self.clock)(), MAX_ENTRIES, MAX_AGE_MS);
        let body = format::encode(entries)
            .map_err(|_| self.error(ErrorCode::Internal, "could not encode entries"))?;
        let io = |e: std::io::Error| self.error(ErrorCode::Io, e);
        if let Some(dir) = self.path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(io)?;
        }
        let tmp = self.temp_path();
        let written = write_then_rename(&tmp, &self.path, &body);
        if written.is_err() {
            // Best effort: a failed write must not leave its temp file behind.
            let _ = std::fs::remove_file(&tmp);
        }
        written.map_err(io)
    }
}

/// Writes and syncs `body` to `tmp`, then renames it over `path`.
fn write_then_rename(tmp: &Path, path: &Path, body: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::File::create(tmp)?;
    f.write_all(body)?;
    f.sync_all()?;
    drop(f);
    std::fs::rename(tmp, path)
}

impl ResumeStore for FileResumeStore {
    fn get(&self, key: &str) -> PortResult<Option<ResumePoint>> {
        Ok(match self.load()? {
            Loaded::Ready(entries) => entries.get(key).map(Entry::point),
            Loaded::Corrupt | Loaded::Newer => None,
        })
    }
    fn put(&mut self, key: &str, p: ResumePoint) -> PortResult<()> {
        let mut entries = self.load_for_write()?;
        entries.insert(key.into(), Entry::from_point(p, (self.clock)()));
        self.save(&mut entries)
    }
    fn clear(&mut self, key: &str) -> PortResult<bool> {
        // Corrupt or newer files hold no readable key, so there is nothing to clear.
        let Loaded::Ready(mut entries) = self.load()? else {
            return Ok(false);
        };
        let had = entries.remove(key).is_some();
        if had {
            self.save(&mut entries)?;
        }
        Ok(had)
    }
}

#[cfg(test)]
mod tests;
