//! Normal product file guards; unadmitted legacy assembly stays test-only.
//! Application supplies an admitted destination; this adapter preserves ordinary
//! local files and the existing ancestor directory guards for its entire life.
use super::immutable_storage_paths;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad_executor::infrastructure::{
    EvidenceRootGuard, ExecutableGuard, GuardedOutputFile,
};
#[cfg(test)]
use kinekino_vad_executor::{
    ArtifactPin, Deadline,
    application::VadExecutor,
    infrastructure::{JsonOutputPublication, NativeTransportFactory, PinnedRequestEncoder},
};
#[cfg(test)]
use std::{collections::BTreeMap, ffi::OsString, sync::Arc};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

/// Internal application-resolved locations, not a public request/CLI schema.
#[cfg(test)]
pub(super) struct VadExecutionPaths {
    pub worker: PathBuf,
    pub evidence: PathBuf,
    pub output: PathBuf,
}
#[cfg(test)]
pub(super) struct VadExecutionConfiguration {
    pub paths: VadExecutionPaths,
    pub worker_pin: ArtifactPin,
    pub original: Deadline,
    pub environment: BTreeMap<String, OsString>,
}
/// Concrete, inert composition. This function registers no operation and starts
/// no worker; native admission/hash/read/load remain inside original session run.
#[cfg(test)]
pub(super) fn assemble(configuration: VadExecutionConfiguration) -> Result<VadExecutor, ApiError> {
    let worker = Arc::new(VadJobFile::from_application_path(
        &configuration.paths.worker,
    )?);
    let evidence = Arc::new(VadJobEvidenceRoot::from_application_root(
        &configuration.paths.evidence,
    )?);
    let output = Arc::new(VadJobFile::from_application_path(
        &configuration.paths.output,
    )?);
    let factory = NativeTransportFactory::new(
        worker,
        configuration.worker_pin,
        evidence.path.clone(),
        evidence,
        configuration.original,
        configuration.environment,
    );
    Ok(VadExecutor::new(
        Arc::new(PinnedRequestEncoder),
        Arc::new(factory),
        Arc::new(JsonOutputPublication::new(output)),
    ))
}

pub(super) struct VadJobFile {
    path: PathBuf,
    text: String,
    _ancestors: Vec<File>,
}
pub(super) struct VadJobEvidenceRoot {
    path: PathBuf,
    _ancestors: Vec<File>,
    _writer: File,
}
impl VadJobEvidenceRoot {
    pub(super) fn from_application_root(path: &Path) -> Result<Self, ApiError> {
        let (_, ancestors) = immutable_storage_paths::root(path)?;
        let writer = immutable_storage_paths::lock(&path.join(".vad-worker.lock"))?;
        Ok(Self {
            // Keep the validated disk-prefixed spelling for later root admission;
            // canonicalize returns a verbatim prefix that root deliberately rejects.
            path: path.to_path_buf(),
            _ancestors: ancestors,
            _writer: writer,
        })
    }
}
impl EvidenceRootGuard for VadJobEvidenceRoot {
    fn path(&self) -> &Path {
        &self.path
    }
}
impl VadJobFile {
    pub(super) fn from_application_path(path: &Path) -> Result<Self, ApiError> {
        let invalid = || {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "VAD storage requires one ordinary filename in an admitted absolute directory",
            )
        };
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .filter(|s| !s.contains(':') && !s.chars().any(char::is_control))
            .ok_or_else(invalid)?;
        let parent = path.parent().ok_or_else(invalid)?;
        let (root, ancestors) = immutable_storage_paths::root(parent)?;
        let path = root.join(name);
        let text = path.to_str().ok_or_else(invalid)?.to_owned();
        Ok(Self {
            path,
            text,
            _ancestors: ancestors,
        })
    }
}
impl GuardedOutputFile for VadJobFile {
    fn path(&self) -> &str {
        &self.text
    }
    fn open_actual(&self) -> Result<File, ApiError> {
        immutable_storage_paths::file(&self.path)
    }
}
impl ExecutableGuard for VadJobFile {
    fn path(&self) -> &Path {
        &self.path
    }
    fn open_guarded(&self) -> Result<File, ApiError> {
        immutable_storage_paths::file(&self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        io::Read,
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    fn root() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "vad-job-file-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        root
    }
    #[test]
    fn original_executable_file_guard_prevents_replacement_until_release() {
        let root = root();
        let path = root.join("worker.bin");
        fs::write(&path, b"original").unwrap();
        let guard = VadJobFile::from_application_path(&path).unwrap();
        let mut original = ExecutableGuard::open_guarded(&guard).unwrap();
        // POSIX: the unlinked original stays readable through the retained handle.
        let removed = crate::infrastructure::os_policy::guarded_mutation(fs::remove_file(&path));
        if !removed {
            crate::infrastructure::os_policy::guarded_mutation(fs::rename(
                &path,
                root.join("changed.bin"),
            ));
        }
        let mut bytes = Vec::new();
        original.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"original");
        drop(original);
        drop(guard);
        if !removed {
            fs::remove_file(path).unwrap();
        }
        fs::remove_dir(root).unwrap();
    }
    #[test]
    fn parent_directory_guard_prevents_replacement_before_output_publication() {
        let root = root();
        let guard = VadJobFile::from_application_path(&root.join("result.json")).unwrap();
        let changed = root.with_extension("changed");
        let applied =
            crate::infrastructure::os_policy::guarded_mutation(fs::rename(&root, &changed));
        drop(guard);
        if !applied {
            fs::rename(&root, &changed).unwrap();
        }
        fs::remove_dir(changed).unwrap();
    }
    #[test]
    fn actual_published_bytes_are_read_without_overwriting_existing_output() {
        let root = root();
        let guard = VadJobFile::from_application_path(&root.join("result.json")).unwrap();
        let expected = b"{\"original\":true}";
        kinekino_core::file_publication::publish_before(
            GuardedOutputFile::path(&guard),
            expected,
            Some(std::time::Instant::now() + std::time::Duration::from_secs(5)),
        )
        .unwrap();
        assert!(
            kinekino_core::file_publication::publish(
                GuardedOutputFile::path(&guard),
                b"replacement"
            )
            .is_err()
        );
        let mut bytes = Vec::new();
        GuardedOutputFile::open_actual(&guard)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        assert_eq!(bytes, expected);
        let path = guard.path.clone();
        drop(guard);
        fs::remove_file(path).unwrap();
        fs::remove_dir(root).unwrap();
    }
    #[test]
    fn output_reparse_file_is_rejected_instead_of_following_link() -> std::io::Result<()> {
        let root = root();
        let actual = root.join("actual.json");
        let link = root.join("result.json");
        fs::write(&actual, b"outside-target")?;
        // Capability failures remain real test errors, never silent skips/passes.
        kinekino_platform::fs::symlink_file(&actual, &link)?;
        let guard = VadJobFile::from_application_path(&link).unwrap();
        assert!(GuardedOutputFile::open_actual(&guard).is_err());
        drop(guard);
        fs::remove_file(link)?;
        fs::remove_file(actual)?;
        fs::remove_dir(root)
    }
    #[test]
    fn evidence_directory_lease_rejects_second_writer_until_original_release() {
        let root = root();
        let first = VadJobEvidenceRoot::from_application_root(&root).unwrap();
        assert!(VadJobEvidenceRoot::from_application_root(&root).is_err());
        drop(first);
        let second = VadJobEvidenceRoot::from_application_root(&root).unwrap();
        drop(second);
        fs::remove_file(root.join(".vad-worker.lock")).unwrap();
        fs::remove_dir(root).unwrap();
    }
    #[test]
    fn concrete_product_assembly_retains_guards_without_launching_worker() {
        let root = root();
        let worker = root.join("worker.bin");
        fs::write(&worker, b"not executable").unwrap();
        let executor = assemble(VadExecutionConfiguration {
            paths: VadExecutionPaths {
                worker: worker.clone(),
                evidence: root.clone(),
                output: root.join("result.json"),
            },
            worker_pin: ArtifactPin::new(
                "worker".into(),
                "a".repeat(64),
                kinekino_media_contracts::api_values::DecimalU64(14),
            )
            .unwrap(),
            original: Deadline::new(
                std::time::Instant::now(),
                std::time::Duration::from_secs(5),
                std::time::Duration::from_secs(1),
            )
            .unwrap(),
            environment: BTreeMap::new(),
        })
        .unwrap();
        assert!(!root.join("result.json").exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
        assert!(VadJobEvidenceRoot::from_application_root(&root).is_err());
        drop(executor);
        let after = VadJobEvidenceRoot::from_application_root(&root).unwrap();
        drop(after);
        fs::remove_file(root.join(".vad-worker.lock")).unwrap();
        fs::remove_file(worker).unwrap();
        fs::remove_dir(root).unwrap();
    }
}
