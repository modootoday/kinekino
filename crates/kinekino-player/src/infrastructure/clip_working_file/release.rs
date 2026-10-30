//! Close only the original handle, then observe identity/absence. Never remove a pathname.
use super::*;
struct DeletionWitness {
    path: PathBuf,
    identity: StorageDirectoryIdentity,
    _guards: Vec<File>,
    released: bool,
}
impl DeletionWitness {
    fn poll(&mut self) -> Result<bool, ApiError> {
        if self.released {
            return Ok(true);
        }
        let mut options = OpenOptions::new();
        options.read(true);
        apply_share_mode(&mut options, ShareMode::All);
        apply_open_flags(
            &mut options,
            OpenFlags {
                no_follow: true,
                directory_handle: true,
                ..OpenFlags::default()
            },
        );
        match options.open(&self.path) {
            Ok(file) => self.released = StorageDirectoryIdentity::observe(&file)? != self.identity,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => self.released = true,
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => return Ok(false),
            Err(error) => return Err(kinekino_core::file_publication::io_error(error)),
        }
        Ok(self.released)
    }
}
enum Phase {
    Active(ClipWorkingFile),
    Observing(DeletionWitness),
    // Never re-close or reconstruct a File from a handle with uncertain close state.
    Uncertain {
        witness: DeletionWitness,
        _original_raw: usize,
        code: i32,
    },
    Transition,
}
#[must_use = "Keep the original release carrier until the original file is gone"]
pub(crate) struct ClipWorkingFileRelease {
    phase: Phase,
    #[cfg(test)]
    fail_next_close: bool,
}
fn uncertain(code: i32) -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        format!(
            "Original clip input close is uncertain (HRESULT {:08x})",
            code as u32
        ),
    )
}
impl ClipWorkingFileRelease {
    pub(super) fn new(owner: ClipWorkingFile) -> Self {
        Self {
            phase: Phase::Active(owner),
            #[cfg(test)]
            fail_next_close: false,
        }
    }
    pub(crate) fn poll(&mut self) -> Result<bool, ApiError> {
        if let Phase::Active(owner) = &self.phase {
            owner.check()?;
            #[cfg(test)]
            if self.fail_next_close {
                self.fail_next_close = false;
                return Err(ApiError::new(
                    ErrorCode::EngineFailed,
                    "Injected original close failure before OS call",
                ));
            }
            let Phase::Active(owner) = std::mem::replace(&mut self.phase, Phase::Transition) else {
                unreachable!()
            };
            let ClipWorkingFile {
                file,
                path,
                identity,
                _guards,
                ..
            } = owner;
            // This valid original File was identity-checked; the port closes exactly it once.
            let closed = crate::infrastructure::os_policy::close_original(file);
            let witness = DeletionWitness {
                path,
                identity,
                _guards,
                released: false,
            };
            match closed {
                Ok(()) => self.phase = Phase::Observing(witness),
                Err(failure) => {
                    let code = failure.code;
                    self.phase = Phase::Uncertain {
                        witness,
                        _original_raw: failure.raw,
                        code,
                    };
                    return Err(uncertain(code));
                }
            }
        }
        match &mut self.phase {
            Phase::Observing(witness) => witness.poll(),
            Phase::Uncertain { witness, code, .. } => {
                if witness.poll()? {
                    Ok(true)
                } else {
                    Err(uncertain(*code))
                }
            }
            _ => Err(ApiError::new(
                ErrorCode::Internal,
                "Original clip input release transition is incomplete",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io::Read};
    struct Control;
    impl MutationControl for Control {
        fn checkpoint(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }
    #[test]
    #[ignore = "Actual Windows original handle close/reader lifetime/foreign replacement; fresh evidence, fixed cache TMP"]
    fn original_release_retains_failed_close_and_waits_for_reader_without_deleting_replacement() {
        let evidence = PathBuf::from(
            std::env::var_os("CLIP_WORKING_RELEASE_EVIDENCE_DIR").expect("Fresh evidence"),
        );
        fs::create_dir(&evidence).unwrap();
        let mut working = ClipWorkingFile::create(&std::env::temp_dir(), 4096, &Control).unwrap();
        let facts = working
            .seal(
                b"original bytes remain readable across explicit owner close",
                &Control,
            )
            .unwrap();
        let path = working.path().to_path_buf();
        let mut reader = working.with_reader(Ok).unwrap();
        let mut release = working.into_release();
        release.fail_next_close = true;
        let first = release.poll().unwrap_err();
        assert_eq!(first.code, ErrorCode::EngineFailed);
        assert!(OpenOptions::new().write(true).open(&path).is_err());
        assert!(!release.poll().unwrap());
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).unwrap();
        assert_eq!(
            bytes,
            b"original bytes remain readable across explicit owner close"
        );
        drop(reader);
        assert!(!path.exists());
        fs::write(&path, b"foreign replacement must be preserved").unwrap();
        assert!(release.poll().unwrap());
        assert!(release.poll().unwrap());
        drop(release);
        assert_eq!(
            fs::read(&path).unwrap(),
            b"foreign replacement must be preserved"
        );
        let mut ordinary = ClipWorkingFile::create(&std::env::temp_dir(), 4096, &Control).unwrap();
        ordinary.seal(b"ordinary release", &Control).unwrap();
        let ordinary_path = ordinary.path().to_path_buf();
        let mut ordinary = ordinary.into_release();
        assert!(ordinary.poll().unwrap());
        assert!(!ordinary_path.exists());
        fs::write(
            evidence.join("input-facts.json"),
            serde_json::to_vec_pretty(&facts).unwrap(),
        )
        .unwrap();
        fs::write(evidence.join("verification.json"),serde_json::to_vec_pretty(&serde_json::json!({"close_failure_was_injected_before_os_call":true,"original_owner_retained_on_error":true,"reader_outlived_explicit_original_close":true,"pending_until_original_reader_closed":true,"foreign_replacement_path":path,"replacement_preserved_by_actual_identity_check":true,"ordinary_original_absence_confirmed":true,"no_path_delete_api_used":true})).unwrap()).unwrap();
    }
}
