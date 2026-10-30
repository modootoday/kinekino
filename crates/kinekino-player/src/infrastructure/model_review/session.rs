use super::{Root, io_error, paths};
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        model_review::{ReviewSession, StoredDecision},
    },
    domain::model_review::{
        RecordReviewDecision, ReviewDecision, ReviewPackage, valid_id, valid_sha256,
    },
    infrastructure::{content_digest, new_file},
};
use std::{
    fs::{self, File},
    io::{Cursor, Read},
    sync::Arc,
};
const MAX_HISTORY: usize = 32;
fn key(prefix: &str, id: &str) -> String {
    format!(
        "{prefix}-{}",
        id.bytes().map(|b| format!("{b:02x}")).collect::<String>()
    )
}
fn conflict() -> ApiError {
    ApiError::new(
        ErrorCode::RevisionConflict,
        "Stored review decision identity or digest mismatch",
    )
}
pub(super) struct Session {
    root: Arc<Root>,
    sha: String,
    package: ReviewPackage,
    _guards: Vec<File>,
}
impl Session {
    pub(super) fn new(
        root: Arc<Root>,
        sha: String,
        package: ReviewPackage,
        guards: Vec<File>,
    ) -> Self {
        Self {
            root,
            sha,
            package,
            _guards: guards,
        }
    }
    fn checked(&self, record: &ReviewDecision) -> Result<(), ApiError> {
        let input = RecordReviewDecision {
            package_sha256: record.package_sha256.clone(),
            item_id: record.item_id.clone(),
            decision_id: record.decision_id.clone(),
            verdict: record.verdict,
            note: record.note.clone(),
            issue_ranges: record.issue_ranges.clone(),
        };
        let expected = input
            .bind(&self.package)
            .map_err(|m| ApiError::new(ErrorCode::InvalidArgument, m))?;
        if record.package_sha256 != self.sha
            || serde_json::to_value(expected).ok() != serde_json::to_value(record).ok()
        {
            return Err(conflict());
        }
        Ok(())
    }
}
impl ReviewSession for Session {
    fn package(&self) -> &ReviewPackage {
        &self.package
    }
    fn read_evidence_asset(
        &self,
        asset_id: &str,
        sha256: &str,
        max_bytes: u64,
    ) -> Result<Vec<u8>, ApiError> {
        use crate::application::model_review::MAX_BINARY_EVIDENCE_BYTES;
        let conflict = || {
            ApiError::new(
                ErrorCode::RevisionConflict,
                "Review evidence asset identity or digest mismatch",
            )
        };
        if !valid_id(asset_id)
            || !valid_sha256(sha256)
            || max_bytes == 0
            || max_bytes > MAX_BINARY_EVIDENCE_BYTES
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid bounded review evidence request",
            ));
        }
        let asset = self
            .package
            .assets()
            .into_iter()
            .find(|asset| asset.asset_id == asset_id)
            .ok_or_else(|| {
                ApiError::new(
                    ErrorCode::NotFound,
                    "Evidence asset ID is not declared by this package",
                )
            })?;
        if asset.sha256 != sha256 {
            return Err(conflict());
        }
        if asset.bytes.0 > max_bytes {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Review evidence exceeds requested byte bound",
            ));
        }
        let mut file = paths::file(&self.root.path.join("objects").join(&asset.sha256))?;
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(max_bytes + 1)
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        let observed = content_digest::sha256(&mut Cursor::new(&bytes), max_bytes)?;
        if observed != (asset.bytes.0, asset.sha256.clone()) {
            return Err(conflict());
        }
        Ok(bytes)
    }
    fn read_evidence_prefix(
        &self,
        asset_id: &str,
        sha256: &str,
        max_prefix: u64,
    ) -> Result<Vec<u8>, ApiError> {
        if !valid_id(asset_id) || !valid_sha256(sha256) || max_prefix == 0 || max_prefix > 65536 {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid review prefix request",
            ));
        }
        let asset = self
            .package
            .assets()
            .into_iter()
            .find(|asset| asset.asset_id == asset_id)
            .ok_or_else(|| {
                ApiError::new(ErrorCode::NotFound, "Review prefix asset is not declared")
            })?;
        if asset.sha256 != sha256 {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Review prefix asset identity differs",
            ));
        }
        // ReviewFiles::open has verified the entire asset and retains its immutable guard.
        let mut file = paths::file(&self.root.path.join("objects").join(&asset.sha256))?;
        if file.metadata().map_err(io_error)?.len() != asset.bytes.0 {
            return Err(conflict());
        }
        let mut bytes =
            vec![0; usize::try_from(max_prefix.min(asset.bytes.0)).map_err(|_| conflict())?];
        file.read_exact(&mut bytes).map_err(io_error)?;
        Ok(bytes)
    }
    fn read_text_asset(
        &self,
        asset_sha256: &str,
    ) -> Result<crate::application::model_review::ReviewTextAsset, ApiError> {
        use crate::application::model_review::{MAX_TEXT_ASSET_BYTES, ReviewTextAsset};
        crate::application::model_review::text_asset(&self.package, asset_sha256)?;
        let asset = self
            .package
            .assets()
            .into_iter()
            .find(|a| a.sha256 == asset_sha256)
            .ok_or_else(|| ApiError::new(ErrorCode::NotFound, "Review asset not found"))?;
        if asset.bytes.0 > MAX_TEXT_ASSET_BYTES {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Review text exceeds 64 KiB",
            ));
        }
        let mut file = paths::file(&self.root.path.join("objects").join(asset_sha256))?;
        let mut bytes = Vec::new();
        std::io::Read::by_ref(&mut file)
            .take(MAX_TEXT_ASSET_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        let (length, sha) =
            content_digest::sha256(&mut Cursor::new(&bytes), MAX_TEXT_ASSET_BYTES + 1)?;
        if length != asset.bytes.0 || sha != asset.sha256 {
            return Err(conflict());
        }
        let utf8 = String::from_utf8(bytes).map_err(|_| {
            ApiError::new(ErrorCode::NeedsInput, "Review evidence is not strict UTF-8")
        })?;
        Ok(ReviewTextAsset {
            package_sha256: self.sha.clone(),
            asset_sha256: asset.sha256.clone(),
            bytes: asset.bytes,
            media_type: asset.media_type.clone(),
            utf8,
        })
    }
    fn append(&self, decision: &ReviewDecision) -> Result<StoredDecision, ApiError> {
        self.checked(decision)?;
        let body = serde_json::to_vec_pretty(decision)
            .map_err(|_| ApiError::new(ErrorCode::Internal, "Cannot encode review decision"))?;
        let sha = content_digest::sha256(&mut Cursor::new(&body), 65536)?.1;
        let package_dir = self.root.path.join("decisions").join(&self.sha);
        let _package_guard = paths::create_locked(&package_dir)?;
        let item_dir = package_dir.join(key("i", &decision.item_id));
        let _item_guard = paths::create_locked(&item_dir)?;
        let _writer = crate::infrastructure::os_policy::create_transient_lock(
            &item_dir.join(".publish.lock"),
        )
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                ApiError::new(ErrorCode::ResourceExhausted, "Review publication is busy")
            } else {
                io_error(e)
            }
        })?;
        let mut count = 0;
        for entry in fs::read_dir(&item_dir).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            if entry.file_name() == ".publish.lock" {
                continue;
            }
            if !entry.file_type().map_err(io_error)?.is_dir() {
                return Err(conflict());
            }
            count += 1;
        }
        let directory = item_dir.join(key("d", &decision.decision_id));
        if directory.exists() {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Review decision identity already exists",
            ));
        }
        if count >= MAX_HISTORY {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Review decision history is full",
            ));
        }
        fs::create_dir(&directory).map_err(io_error)?;
        let _decision_guard = paths::lock_directory(&directory)?;
        let path = directory.join(format!("{sha}.json"));
        new_file::publish(path.to_str().ok_or_else(conflict)?, &body)?;
        let mut permissions = fs::metadata(&path).map_err(io_error)?.permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&path, permissions).map_err(io_error)?;
        Ok(StoredDecision {
            decision_sha256: sha,
            decision: decision.clone(),
        })
    }
    fn decisions(&self, item_id: &str) -> Result<Vec<StoredDecision>, ApiError> {
        if !valid_id(item_id) {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid review item identity",
            ));
        }
        let package_dir = self.root.path.join("decisions").join(&self.sha);
        if paths::absent(&package_dir)? {
            return Ok(Vec::new());
        }
        let _package_guard = paths::lock_directory(&package_dir)?;
        let item_dir = package_dir.join(key("i", item_id));
        if paths::absent(&item_dir)? {
            return Ok(Vec::new());
        }
        let _item_guard = paths::lock_directory(&item_dir)?;
        if item_dir.join(".publish.lock").exists() {
            return Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Review publication is busy",
            ));
        }
        let mut directories = Vec::new();
        for entry in fs::read_dir(&item_dir).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            let name = entry.file_name().into_string().map_err(|_| conflict())?;
            if !entry.file_type().map_err(io_error)?.is_dir() || !name.starts_with("d-") {
                return Err(conflict());
            }
            directories.push(name);
            if directories.len() > MAX_HISTORY {
                return Err(conflict());
            }
        }
        directories.sort();
        let mut result = Vec::new();
        for name in directories {
            let directory = item_dir.join(&name);
            let _guard = paths::lock_directory(&directory)?;
            let entries = fs::read_dir(&directory)
                .map_err(io_error)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(io_error)?;
            if entries.len() != 1 {
                return Err(ApiError::new(
                    ErrorCode::NeedsInput,
                    "Review publication is incomplete; prepare a new decision identity",
                ));
            }
            let path = entries[0].path();
            let digest = path
                .file_stem()
                .and_then(|v| v.to_str())
                .filter(|v| valid_sha256(v))
                .ok_or_else(conflict)?;
            if path.extension().and_then(|v| v.to_str()) != Some("json") {
                return Err(conflict());
            }
            let mut file = paths::file(&path)?;
            let mut body = Vec::new();
            file.by_ref()
                .take(65537)
                .read_to_end(&mut body)
                .map_err(io_error)?;
            let observed = content_digest::sha256(&mut Cursor::new(&body), 65536)?;
            if observed.1 != digest {
                return Err(conflict());
            }
            let decision: ReviewDecision = serde_json::from_slice(&body).map_err(|_| conflict())?;
            self.checked(&decision)?;
            if decision.item_id != item_id || key("d", &decision.decision_id) != name {
                return Err(conflict());
            }
            result.push(StoredDecision {
                decision_sha256: observed.1,
                decision,
            });
        }
        Ok(result)
    }
}
