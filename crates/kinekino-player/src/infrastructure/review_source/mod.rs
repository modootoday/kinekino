//! Separate user-review transaction root. Never modifies model review packages.
#[cfg(test)]
mod case_tests;
pub mod media_preparation;
mod records;
mod transaction;
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        review_source::*,
    },
    domain::review_source::ReviewCase,
    infrastructure::{content_digest, immutable_storage_paths as rooted},
};
use std::{
    fs::{self, File},
    io::Cursor,
    path::{Path, PathBuf},
    sync::Arc,
};

pub const MAX_TRANSACTION_BYTES: u64 = 262_144;
pub const MAX_CASE_TRANSACTIONS: usize = 4096;
/// Trusted local assembly. Roots must be disjoint; package evidence is never a user store.
pub fn open_api(
    review_root: &Path,
    user_root: &Path,
    writable: bool,
) -> Result<crate::application::review_source::api::ReviewSourceService, ApiError> {
    let (review_path, _review_guards) = rooted::root(review_root)?;
    let (user_path, _user_guards) = rooted::root(user_root)?;
    let review_comparison = PathBuf::from(review_path.to_string_lossy().to_lowercase());
    let user_comparison = PathBuf::from(user_path.to_string_lossy().to_lowercase());
    if review_comparison.starts_with(&user_comparison)
        || user_comparison.starts_with(&review_comparison)
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Review evidence and user-review roots must be disjoint",
        ));
    }
    Ok(
        crate::application::review_source::api::ReviewSourceService::new(
            ReviewCases::new(crate::infrastructure::model_review::ReviewFiles::open(
                review_root,
            )?),
            ReviewPackageMedia::new(crate::infrastructure::model_review::ReviewFiles::open(
                review_root,
            )?),
            UserReviewFiles::open(user_root, writable)?,
            writable,
        ),
    )
}
pub struct UserReviewFiles {
    root: Arc<Root>,
}
struct Root {
    path: PathBuf,
    _guards: Vec<File>,
    writable: bool,
    #[cfg(test)]
    lose_ack: std::sync::atomic::AtomicBool,
}
impl UserReviewFiles {
    /// The trusted caller creates/selects the root. Read-only open creates nothing.
    pub fn open(path: &Path, writable: bool) -> Result<Self, ApiError> {
        let (path, mut guards) = rooted::root(path)?;
        if writable {
            create_directory(&path.join("cases"))?;
        }
        guards.push(rooted::directory(&path.join("cases"))?);
        Ok(Self {
            root: Arc::new(Root {
                path,
                _guards: guards,
                writable,
                #[cfg(test)]
                lose_ack: std::sync::atomic::AtomicBool::new(false),
            }),
        })
    }
}
impl UserReviewStore for UserReviewFiles {
    fn limits(&self) -> Option<StoreLimits> {
        Some(StoreLimits {
            max_case_transactions: crate::domain::api_values::DecimalU64(
                MAX_CASE_TRANSACTIONS as u64,
            ),
            max_transaction_bytes: crate::domain::api_values::DecimalU64(MAX_TRANSACTION_BYTES),
        })
    }
    fn open(
        &self,
        case: &ReviewCase,
        access: StoreAccess,
    ) -> Result<Box<dyn CaseTransaction + '_>, ApiError> {
        case.validate()
            .map_err(|_| ApiError::new(ErrorCode::InvalidArgument, "Invalid user review case"))?;
        if access == StoreAccess::Write && !self.root.writable {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "User review root is read-only",
            ));
        }
        let key = content_digest::sha256(
            &mut Cursor::new(serde_json::to_vec(case).map_err(|_| incomplete("case_encoding"))?),
            4096,
        )?
        .1;
        let directory = self.root.path.join("cases").join(key);
        if access == StoreAccess::Write {
            create_directory(&directory)?;
        }
        let mut guards = vec![rooted::directory(&directory)?];
        let lock = directory.join("writer.lock");
        let reservation = if access == StoreAccess::Write {
            rooted::lock(&lock)?
        } else {
            read_reservation(&lock)?
        };
        let transactions = directory.join("transactions");
        if access == StoreAccess::Write {
            create_directory(&transactions)?;
        }
        guards.push(rooted::directory(&transactions)?);
        let (records, files) = records::load(&transactions, case)?;
        Ok(Box::new(transaction::Transaction {
            root: self.root.clone(),
            case: case.clone(),
            directory: transactions,
            _reservation: reservation,
            _guards: guards,
            _files: files,
            records,
            access,
        }))
    }
}
fn create_directory(path: &Path) -> Result<(), ApiError> {
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            rooted::directory(path).map(|_| ())
        }
        Err(e) => Err(io_error(e)),
    }
}
/// Read-only reservation belongs to this adapter; the shared rooted-path module
/// is also included by older fixtures that only need its writer API.
fn read_reservation(path: &Path) -> Result<File, ApiError> {
    use crate::infrastructure::os_policy::{
        is_reparse_point, is_sharing_violation, open_reserved_read,
    };
    let file = open_reserved_read(path).map_err(|error| {
        if is_sharing_violation(&error) {
            ApiError::new(ErrorCode::RevisionConflict, "User review writer is active")
        } else {
            io_error(error)
        }
    })?;
    let metadata = file.metadata().map_err(io_error)?;
    if !metadata.is_file() || is_reparse_point(&metadata) {
        return Err(ApiError::new(
            ErrorCode::PermissionDenied,
            "User review reservation must be an ordinary file",
        ));
    }
    // Where share modes are advisory (POSIX), the writer's exclusive lock is the exclusion; a
    // shared lock on the same original reservation keeps readers and writers mutually exclusive.
    crate::infrastructure::os_policy::try_lock_shared(&file).map_err(|error| {
        if error.kind() == std::io::ErrorKind::WouldBlock {
            ApiError::new(ErrorCode::RevisionConflict, "User review writer is active")
        } else {
            io_error(error)
        }
    })?;
    Ok(file)
}
fn incomplete(phase: &str) -> ApiError {
    let mut e = ApiError::new(
        ErrorCode::NeedsInput,
        "User review history is incomplete or corrupt; preserve and inspect",
    );
    e.details = serde_json::json!({"phase":phase,"publication_may_exist":true,"automatic_retry":false,"recovery":"preserve_and_inspect"});
    e
}
fn unknown(phase: &str) -> ApiError {
    let mut e = ApiError::new(
        ErrorCode::EngineFailed,
        "User review publication outcome is uncertain; query the same key",
    );
    e.details = serde_json::json!({"phase":phase,"delivery":"execution_unknown","publication_may_exist":true,"automatic_retry":false});
    e
}
fn io_error(e: std::io::Error) -> ApiError {
    crate::infrastructure::new_file::io_error(e)
}
#[cfg(test)]
mod process_tests;
#[cfg(test)]
mod tests;
