//! Fixed child directory over the existing immutable, content-addressed CatalogFiles adapter.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        catalog::CatalogArtifacts,
    },
    infrastructure::{catalog::CatalogFiles, immutable_storage_paths},
};
use std::{path::Path, sync::Arc};
pub fn open(project_root: &Path) -> Result<Arc<dyn CatalogArtifacts>, ApiError> {
    let (_canonical, _guards) = immutable_storage_paths::root(project_root)?;
    // Keep the validated input while its ancestor handles prevent path replacement.
    // canonicalize returns VerbatimDisk on Windows; it is an identity, not fresh root input.
    let directory = project_root.join("viewer-execution-evidence");
    match std::fs::create_dir(&directory) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                format!("Viewer evidence directory: {error}"),
            ));
        }
    }
    Ok(Arc::new(CatalogFiles::open(&directory)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Component, Prefix};
    #[test]
    fn validated_disk_root_preserves_canonical_identity_and_strict_input_guard() {
        let root = std::env::current_dir()
            .unwrap()
            .join("runtime/quality-review/ui-capture-archive-root")
            .join(format!(
                "{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        std::fs::create_dir_all(&root).unwrap();
        let (canonical, _guards) = immutable_storage_paths::root(&root).unwrap();
        eprintln!(
            "capture_archive_root input={} canonical={} input_prefix={:?} canonical_prefix={:?}",
            root.display(),
            canonical.display(),
            root.components().next(),
            canonical.components().next()
        );
        assert!(
            matches!(root.components().next(),Some(Component::Prefix(prefix)) if matches!(prefix.kind(),Prefix::Disk(_)))
        );
        assert!(
            matches!(canonical.components().next(),Some(Component::Prefix(prefix)) if matches!(prefix.kind(),Prefix::VerbatimDisk(_)))
        );
        let error = match open(&canonical) {
            Ok(_) => panic!("Fresh extended-prefix input must remain denied"),
            Err(error) => error,
        };
        assert_eq!(error.code, ErrorCode::PermissionDenied);
        assert!(open(&root).is_ok());
    }
}
