//! Local create-only destination for project exports (temp write, sync, no-replace link).
use crate::application::projects::ProjectExportTarget;
use kinekino_media_contracts::errors::ApiError;

pub struct ProjectExportFiles;

impl ProjectExportTarget for ProjectExportFiles {
    fn publish_new(&self, path: &str, bytes: &[u8]) -> Result<(), ApiError> {
        kinekino_core::file_publication::publish(path, bytes)
    }
}
