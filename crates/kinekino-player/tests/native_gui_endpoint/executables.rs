use super::fixture::{Fixture, TestResult};
use serde::Serialize;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    os::windows::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};
use std::{os::windows::io::AsRawHandle, process::Child};
use windows::Win32::Security::Cryptography::{
    BCRYPT_ALG_HANDLE, BCRYPT_HASH_HANDLE, BCRYPT_SHA256_ALGORITHM, BCryptCloseAlgorithmProvider,
    BCryptCreateHash, BCryptDestroyHash, BCryptFinishHash, BCryptHashData,
    BCryptOpenAlgorithmProvider,
};
use windows::{
    Win32::{
        Foundation::HANDLE,
        System::Threading::{PROCESS_NAME_WIN32, QueryFullProcessImageNameW},
    },
    core::PWSTR,
};

struct Algorithm(BCRYPT_ALG_HANDLE);
impl Drop for Algorithm {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptCloseAlgorithmProvider(self.0, 0);
        }
    }
}
struct Hash(BCRYPT_HASH_HANDLE);
impl Drop for Hash {
    fn drop(&mut self) {
        unsafe {
            let _ = BCryptDestroyHash(self.0);
        }
    }
}
fn digest(reader: &mut impl Read) -> TestResult<(u64, String)> {
    let mut algorithm = BCRYPT_ALG_HANDLE::default();
    unsafe {
        BCryptOpenAlgorithmProvider(
            &mut algorithm,
            BCRYPT_SHA256_ALGORITHM,
            None,
            Default::default(),
        )
        .ok()
    }
    .map_err(|_| "Cannot initialize executable SHA256")?;
    let algorithm = Algorithm(algorithm);
    let mut hash = BCRYPT_HASH_HANDLE::default();
    unsafe { BCryptCreateHash(algorithm.0, &mut hash, None, None, 0).ok() }
        .map_err(|_| "Cannot create executable hash")?;
    let hash = Hash(hash);
    let mut buffer = [0u8; 65536];
    let mut bytes = 0u64;
    loop {
        let length = reader
            .read(&mut buffer)
            .map_err(|_| "Cannot read executable bytes")?;
        if length == 0 {
            break;
        }
        unsafe { BCryptHashData(hash.0, &buffer[..length], 0).ok() }
            .map_err(|_| "Cannot hash executable bytes")?;
        bytes += length as u64;
    }
    let mut output = [0u8; 32];
    unsafe { BCryptFinishHash(hash.0, &mut output, 0).ok() }
        .map_err(|_| "Cannot finalize executable hash")?;
    Ok((bytes, output.iter().map(|b| format!("{b:02x}")).collect()))
}
#[derive(Clone, Serialize)]
pub struct ExecutableIdentity {
    pub role: String,
    pub original_path: PathBuf,
    pub copied_path: PathBuf,
    pub executed_path: PathBuf,
    pub bytes: u64,
    pub sha256: String,
    pub copied_bytes_verified: bool,
}
pub struct Executables {
    pub identities: Vec<ExecutableIdentity>,
    inputs: Vec<File>,
    copies: Vec<File>,
}
fn locked(path: &Path) -> TestResult<File> {
    OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(path)
        .map_err(|_| "Cannot lock executable snapshot input".into())
}
impl Executables {
    pub fn freeze(fixture: &Fixture, output: &Path) -> TestResult<Self> {
        let folder = output.join("binaries");
        fs::create_dir(&folder).map_err(|_| "Cannot create fresh executable snapshot directory")?;
        let mut snapshot = Self {
            identities: Vec::new(),
            inputs: Vec::new(),
            copies: Vec::new(),
        };
        for (role, source, name) in [
            (
                "gui-and-local-host",
                fixture.gui_executable.clone(),
                "player.exe",
            ),
            ("native-engine", fixture.mpv_executable.clone(), "mpv.exe"),
            (
                "cli",
                PathBuf::from(crate::cli_exe::kinekino_cli_exe()),
                "kinekino.exe",
            ),
            (
                "test-harness",
                std::env::current_exe().map_err(|_| "Cannot identify test executable")?,
                "test-harness.exe",
            ),
        ] {
            let mut input = locked(&source)?;
            let before = digest(&mut input)?;
            input
                .seek(SeekFrom::Start(0))
                .map_err(|_| "Cannot rewind executable")?;
            let destination = folder.join(name);
            let mut copy = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&destination)
                .map_err(|_| "Executable snapshot already exists")?;
            std::io::copy(&mut input, &mut copy).map_err(|_| "Cannot copy executable")?;
            copy.sync_all().map_err(|_| "Cannot sync executable")?;
            drop(copy);
            input
                .seek(SeekFrom::Start(0))
                .map_err(|_| "Cannot rewind executable")?;
            let after = digest(&mut input)?;
            let mut copied = locked(&destination)?;
            let stored = digest(&mut copied)?;
            if before != after || before != stored {
                return Err("Executable source/copy bytes changed during snapshot".into());
            }
            let mut permissions = fs::metadata(&destination)
                .map_err(|_| "Cannot inspect executable copy")?
                .permissions();
            permissions.set_readonly(true);
            fs::set_permissions(&destination, permissions)
                .map_err(|_| "Cannot protect executable copy")?;
            snapshot.identities.push(ExecutableIdentity {
                role: role.into(),
                original_path: source.clone(),
                copied_path: destination.clone(),
                executed_path: if role == "cli" { destination } else { source },
                bytes: before.0,
                sha256: before.1,
                copied_bytes_verified: true,
            });
            snapshot.inputs.push(input);
            snapshot.copies.push(copied);
        }
        Ok(snapshot)
    }
    pub fn cli(&self) -> &Path {
        &self.identities[2].executed_path
    }
    pub fn verify_cli_child(&self, child: &Child) -> TestResult<serde_json::Value> {
        let mut buffer = vec![0u16; 32768];
        let mut length = buffer.len() as u32;
        unsafe {
            QueryFullProcessImageNameW(
                HANDLE(child.as_raw_handle()),
                PROCESS_NAME_WIN32,
                PWSTR(buffer.as_mut_ptr()),
                &mut length,
            )
        }
        .map_err(|_| "Cannot inspect owned CLI child image")?;
        let image = PathBuf::from(
            String::from_utf16(&buffer[..length as usize]).map_err(|_| "Invalid CLI image path")?,
        );
        if image.canonicalize().ok() != self.cli().canonicalize().ok() {
            return Err("Owned CLI child did not execute pinned copy".into());
        }
        Ok(
            serde_json::json!({"pid":child.id(),"image_path":image,"verified_copied_cli":true,"bytes":self.identities[2].bytes,"sha256":self.identities[2].sha256}),
        )
    }
    pub fn verify_after(&mut self) -> TestResult<()> {
        for ((identity, input), copy) in self
            .identities
            .iter()
            .zip(&mut self.inputs)
            .zip(&mut self.copies)
        {
            input
                .seek(SeekFrom::Start(0))
                .map_err(|_| "Cannot rewind executable source")?;
            copy.seek(SeekFrom::Start(0))
                .map_err(|_| "Cannot rewind executable copy")?;
            let expected = (identity.bytes, identity.sha256.clone());
            if digest(input)? != expected || digest(copy)? != expected {
                return Err("Executable bytes changed during integration".into());
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn executable_sha256_matches_published_vector_without_running_any_process() {
        kinekino_platform::install();
        assert_eq!(
            digest(&mut &b"abc"[..]).unwrap(),
            (
                3,
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into()
            )
        );
    }
}
