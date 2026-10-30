mod export;
mod paths;
mod playback_media;
mod session;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        model_review::{ReviewSession, ReviewStore},
    },
    domain::model_review::{ReviewPackage, valid_sha256},
    infrastructure::content_digest,
};
pub use playback_media::{GuardedAdmittedReviewMedia, GuardedReviewMedia};
use std::{
    collections::BTreeSet,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Arc,
};
pub struct ReviewFiles {
    root: Arc<Root>,
}
struct Root {
    path: PathBuf,
    _guards: Vec<File>,
}
impl ReviewFiles {
    pub fn open(path: &Path) -> Result<Self, ApiError> {
        let (path, mut guards) = paths::root(path)?;
        for name in ["packages", "objects", "decisions"] {
            guards.push(paths::lock_directory(&path.join(name))?);
        }
        Ok(Self {
            root: Arc::new(Root {
                path,
                _guards: guards,
            }),
        })
    }
}
impl ReviewStore for ReviewFiles {
    fn open(&self, sha: &str) -> Result<Box<dyn ReviewSession>, ApiError> {
        if !valid_sha256(sha) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Review package SHA256 required",
            ));
        }
        let mut package_file =
            paths::file(&self.root.path.join("packages").join(format!("{sha}.json")))?;
        let (bytes, actual) = content_digest::sha256(&mut package_file, 524288)?;
        if actual != sha {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Review package identity mismatch",
            ));
        }
        package_file.seek(SeekFrom::Start(0)).map_err(io_error)?;
        let mut body = Vec::with_capacity(bytes as usize);
        package_file.read_to_end(&mut body).map_err(io_error)?;
        let package: ReviewPackage = serde_json::from_slice(&body).map_err(|_| {
            ApiError::new(ErrorCode::InvalidArgument, "Invalid review package JSON")
        })?;
        package
            .validate()
            .map_err(|m| ApiError::new(ErrorCode::InvalidArgument, m))?;
        let mut guards = vec![package_file];
        let mut seen = BTreeSet::new();
        for asset in package.assets() {
            if !seen.insert(&asset.sha256) {
                continue;
            }
            let mut file = paths::file(&self.root.path.join("objects").join(&asset.sha256))?;
            let observed = content_digest::sha256(&mut file, 1_073_741_824)?;
            if observed != (asset.bytes.0, asset.sha256.clone()) {
                return Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Review evidence asset identity mismatch",
                ));
            }
            guards.push(file);
        }
        Ok(Box::new(session::Session::new(
            self.root.clone(),
            sha.into(),
            package,
            guards,
        )))
    }
}
fn io_error(error: std::io::Error) -> ApiError {
    match error.kind() {
        std::io::ErrorKind::NotFound => {
            let mut e = ApiError::new(
                ErrorCode::NotFound,
                "Review package or fixed evidence is missing; prepare the review package",
            );
            e.remediation = "prepare_review_package".into();
            e
        }
        std::io::ErrorKind::AlreadyExists => ApiError::new(
            ErrorCode::RevisionConflict,
            "Review decision identity already exists",
        ),
        std::io::ErrorKind::PermissionDenied => {
            ApiError::new(ErrorCode::PermissionDenied, "Review storage access denied")
        }
        _ => ApiError::new(ErrorCode::EngineFailed, "Review storage operation failed"),
    }
}
