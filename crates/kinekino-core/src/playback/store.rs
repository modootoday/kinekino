//! Resume position storage (#9): a port, an in-memory fake, and a portable JSON file store.
use super::resume::ResumePoint;
use crate::port::error::{ErrorCode, PortError, PortResult};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Keyed by a stable media key (e.g. source id + episode, or canonical local path).
pub trait ResumeStore {
    fn get(&self, key: &str) -> PortResult<Option<ResumePoint>>;
    fn put(&mut self, key: &str, point: ResumePoint) -> PortResult<()>;
    /// Returns whether an entry existed.
    fn clear(&mut self, key: &str) -> PortResult<bool>;
}

#[derive(Debug, Default)]
pub struct MemoryResumeStore(BTreeMap<String, ResumePoint>);

impl ResumeStore for MemoryResumeStore {
    fn get(&self, key: &str) -> PortResult<Option<ResumePoint>> {
        Ok(self.0.get(key).copied())
    }
    fn put(&mut self, key: &str, point: ResumePoint) -> PortResult<()> {
        self.0.insert(key.into(), point);
        Ok(())
    }
    fn clear(&mut self, key: &str) -> PortResult<bool> {
        Ok(self.0.remove(key).is_some())
    }
}

/// JSON object `{key: [position_us, duration_us]}`, rewritten atomically (tmp + rename).
#[derive(Debug, Clone)]
pub struct FileResumeStore {
    path: PathBuf,
}

fn io(e: std::io::Error) -> PortError {
    PortError::new(ErrorCode::Io, e.to_string())
}

impl FileResumeStore {
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    fn load(&self) -> PortResult<BTreeMap<String, (u64, u64)>> {
        match std::fs::read(&self.path) {
            Ok(b) => serde_json::from_slice(&b).map_err(|e| {
                PortError::new(
                    ErrorCode::Io,
                    format!("{}: malformed resume file: {e}", self.path.display()),
                )
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
            Err(e) => Err(io(e)),
        }
    }
    fn save(&self, map: &BTreeMap<String, (u64, u64)>) -> PortResult<()> {
        if let Some(dir) = self.path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(io)?;
        }
        let tmp = self.path.with_extension("tmp");
        let body = serde_json::to_vec(map)
            .map_err(|e| PortError::new(ErrorCode::Internal, e.to_string()))?;
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&tmp).map_err(io)?;
            f.write_all(&body).map_err(io)?;
            f.sync_all().map_err(io)?;
        }
        std::fs::rename(&tmp, &self.path).map_err(io)
    }
}

impl ResumeStore for FileResumeStore {
    fn get(&self, key: &str) -> PortResult<Option<ResumePoint>> {
        Ok(self
            .load()?
            .get(key)
            .map(|&(position_us, duration_us)| ResumePoint {
                position_us,
                duration_us,
            }))
    }
    fn put(&mut self, key: &str, p: ResumePoint) -> PortResult<()> {
        let mut map = self.load()?;
        map.insert(key.into(), (p.position_us, p.duration_us));
        self.save(&map)
    }
    fn clear(&mut self, key: &str) -> PortResult<bool> {
        let mut map = self.load()?;
        let had = map.remove(key).is_some();
        if had {
            self.save(&map)?;
        }
        Ok(had)
    }
}

/// Shared contract for [`ResumeStore`] adapters; `store` must start empty.
pub fn check_resume_store(store: &mut dyn ResumeStore) -> PortResult<()> {
    let fail = |m: &str| PortError::new(ErrorCode::Internal, format!("contract: {m}"));
    let p = ResumePoint {
        position_us: 42_000_000,
        duration_us: 1_200_000_000,
    };
    if store.get("a")?.is_some() || store.clear("a")? {
        return Err(fail("store must start empty"));
    }
    store.put("a", p)?;
    store.put(
        "b",
        ResumePoint {
            position_us: 1,
            ..p
        },
    )?;
    store.put(
        "a",
        ResumePoint {
            position_us: 50_000_000,
            ..p
        },
    )?;
    if store.get("a")?.map(|x| x.position_us) != Some(50_000_000) || store.get("b")?.is_none() {
        return Err(fail("put must upsert per key"));
    }
    if !store.clear("a")? || store.get("a")?.is_some() || store.get("b")?.is_none() {
        return Err(fail("clear must remove only its key"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("kk-resume-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d.join("nested").join("resume.json")
    }

    #[test]
    fn memory_and_file_stores_meet_contract() {
        check_resume_store(&mut MemoryResumeStore::default()).unwrap();
        let path = temp("contract");
        check_resume_store(&mut FileResumeStore::at(&path)).unwrap();
        let reopened = FileResumeStore::at(&path);
        assert!(
            reopened.get("b").unwrap().is_some(),
            "entries persist across instances"
        );
        let _ = std::fs::remove_dir_all(path.parent().unwrap().parent().unwrap());
    }

    #[test]
    fn malformed_file_is_an_error_not_silent_reset() {
        let path = temp("bad");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "not json").unwrap();
        let mut s = FileResumeStore::at(&path);
        assert_eq!(s.get("x").unwrap_err().code, ErrorCode::Io);
        let p = ResumePoint {
            position_us: 1,
            duration_us: 2,
        };
        assert!(s.put("x", p).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "not json");
        let _ = std::fs::remove_dir_all(path.parent().unwrap().parent().unwrap());
    }
}
