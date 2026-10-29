//! Persistent credential store: one owner-only (0600) file under the user's config dir.
//! Interim adapter until a Secret Service (D-Bus) store lands; values are not encrypted,
//! so the file mode and parent dir (0700) are the protection boundary.
use kinekino_core::port::credential::{CredentialStore, Secret};
use kinekino_core::port::error::{ErrorCode, PortError, PortResult};
use std::collections::BTreeMap;
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// File name inside the kinekino config dir.
pub const TOKEN_FILE: &str = "credentials.tsv";

#[derive(Debug)]
pub struct FileCredentialStore {
    path: PathBuf,
}

fn io(e: std::io::Error) -> PortError {
    PortError::new(ErrorCode::Io, e.to_string())
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
}

fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some(o) => out.push(o),
            None => out.push('\\'),
        }
    }
    out
}

type Entries = BTreeMap<(String, String), String>;

impl FileCredentialStore {
    /// Store at `path`; the file is created on first `set`.
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn load(&self) -> PortResult<Entries> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Entries::new()),
            Err(e) => return Err(io(e)),
        };
        let mode = std::fs::metadata(&self.path)
            .map_err(io)?
            .permissions()
            .mode();
        if mode & 0o077 != 0 {
            return Err(io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "credential file is readable by group/other; refusing to use it",
            )));
        }
        let mut entries = Entries::new();
        for line in text.lines().filter(|l| !l.is_empty()) {
            let mut f = line.splitn(3, '\t');
            match (f.next(), f.next(), f.next()) {
                (Some(s), Some(a), Some(v)) => {
                    entries.insert((unescape(s), unescape(a)), unescape(v));
                }
                _ => {
                    return Err(io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "malformed credential file line",
                    )));
                }
            }
        }
        Ok(entries)
    }

    /// Atomic replace: write a 0600 temp file next to the target, fsync, rename.
    fn save(&self, entries: &Entries) -> PortResult<()> {
        if let Some(dir) = self.path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(dir)
                .map_err(io)?;
        }
        let tmp = self.path.with_extension("tsv.tmp");
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp)
            .map_err(io)?;
        for ((s, a), v) in entries {
            writeln!(file, "{}\t{}\t{}", escape(s), escape(a), escape(v)).map_err(io)?;
        }
        file.sync_all().map_err(io)?;
        std::fs::rename(&tmp, &self.path).map_err(io)
    }
}

impl CredentialStore for FileCredentialStore {
    fn get(&self, service: &str, account: &str) -> PortResult<Option<Secret>> {
        Ok(self
            .load()?
            .remove(&(service.into(), account.into()))
            .map(Secret::new))
    }
    fn set(&mut self, service: &str, account: &str, secret: Secret) -> PortResult<()> {
        let mut entries = self.load()?;
        entries.insert((service.into(), account.into()), secret.expose().into());
        self.save(&entries)
    }
    fn delete(&mut self, service: &str, account: &str) -> PortResult<bool> {
        let mut entries = self.load()?;
        let removed = entries.remove(&(service.into(), account.into())).is_some();
        if removed {
            self.save(&entries)?;
        }
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinekino_core::port::contract::check_credential_store;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("kk-token-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("cfg").join(TOKEN_FILE)
    }

    #[test]
    fn meets_contract_and_persists_across_instances() {
        let path = temp("persist");
        check_credential_store(&mut FileCredentialStore::at(&path)).unwrap();
        let mut a = FileCredentialStore::at(&path);
        a.set(
            "kinekino.mock",
            "tab\there",
            Secret::new("line\nwith\\slash"),
        )
        .unwrap();
        let b = FileCredentialStore::at(&path);
        let got = b.get("kinekino.mock", "tab\there").unwrap();
        assert_eq!(got.as_ref().map(Secret::expose), Some("line\nwith\\slash"));
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
        let dir_mode = std::fs::metadata(path.parent().unwrap())
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(dir_mode & 0o777, 0o700);
        assert!(!path.with_extension("tsv.tmp").exists());
    }

    #[test]
    fn refuses_world_readable_and_malformed_files() {
        let path = temp("perm");
        let mut s = FileCredentialStore::at(&path);
        s.set("kinekino.mock", "a", Secret::new("v")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(s.get("kinekino.mock", "a").is_err());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::write(&path, "only-one-field\n").unwrap();
        assert!(s.get("kinekino.mock", "a").is_err());
    }

    #[test]
    fn missing_file_is_empty_and_delete_absent_writes_nothing() {
        let path = temp("absent");
        let mut s = FileCredentialStore::at(&path);
        assert_eq!(s.get("x", "y").unwrap(), None);
        assert!(!s.delete("x", "y").unwrap());
        assert!(!path.exists());
    }
}
