//! Typed Rust SDK host options; media addresses are never release permissions.
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::path::PathBuf;
pub struct ProductVadHostOptions {
    pub release_root: PathBuf,
    pub workspace_root: PathBuf,
}
pub struct ProductVadHostPlan {
    pub release_root: PathBuf,
    pub audio_root: PathBuf,
    pub output_root: PathBuf,
    pub cache_root: PathBuf,
    pub evidence_root: PathBuf,
}
impl ProductVadHostOptions {
    pub fn plan(self) -> Result<ProductVadHostPlan, ApiError> {
        if !self.release_root.is_absolute() || !self.workspace_root.is_absolute() {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Product VAD requires absolute release/workspace addresses",
            ));
        }
        Ok(ProductVadHostPlan {
            release_root: self.release_root,
            audio_root: self.workspace_root.join("audio"),
            output_root: self.workspace_root.join("outputs"),
            cache_root: self.workspace_root.join("cache"),
            evidence_root: self.workspace_root.join("evidence"),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn relative_media_addresses_do_not_become_a_host_plan() {
        let options = ProductVadHostOptions {
            release_root: "relative".into(),
            workspace_root: std::env::temp_dir(),
        };
        assert_eq!(
            options.plan().err().unwrap().code,
            ErrorCode::InvalidArgument
        );
    }
}
