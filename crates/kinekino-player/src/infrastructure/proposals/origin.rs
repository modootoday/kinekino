//! Identity metadata is issued only by an explicit live capture under the
//! existing writer. Opening or reading legacy proposals creates nothing.
use super::{ProposalFiles, paths, reservation::WriterReservation};
use crate::infrastructure::os_policy::{OpenOptionsPolicy, ShareMode};
use crate::{
    application::api::{ApiError, ErrorCode},
    domain::proposals::ProposalStoreOrigin,
    infrastructure::storage_directory_identity::StorageDirectoryIdentity,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

const NAME: &str = ".proposal-store-origin.v1.json";
const MAX_BYTES: u64 = 4096;
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OriginRecord {
    schema_version: String,
    origin: ProposalStoreOrigin,
    directory: StorageDirectoryIdentity,
}
pub(super) struct OriginSnapshot {
    record: OriginRecord,
    _file: File,
}
fn recover() -> ApiError {
    let mut error = ApiError::new(
        ErrorCode::NeedsInput,
        "Proposal origin metadata is incomplete, changed or belongs to another root",
    );
    error.remediation = "recover_proposal_store_origin".into();
    error
}
fn read(path: &Path) -> Result<OriginSnapshot, ApiError> {
    let mut file = paths::file(path)?;
    let mut raw = Vec::new();
    Read::by_ref(&mut file)
        .take(MAX_BYTES + 1)
        .read_to_end(&mut raw)
        .map_err(|_| recover())?;
    if raw.len() as u64 > MAX_BYTES {
        return Err(recover());
    }
    let record: OriginRecord = serde_json::from_slice(&raw).map_err(|_| recover())?;
    if record.schema_version != "proposal-store-root/1" {
        return Err(recover());
    }
    record.origin.validate().map_err(|_| recover())?;
    Ok(OriginSnapshot {
        record,
        _file: file,
    })
}
fn issue(directory: StorageDirectoryIdentity) -> Result<OriginRecord, ApiError> {
    let id = crate::infrastructure::os_policy::fresh_guid_hex().map_err(|_| {
        ApiError::new(
            ErrorCode::EngineFailed,
            "Proposal origin identity allocation failed",
        )
    })?;
    let origin = ProposalStoreOrigin::issued(id).map_err(|_| recover())?;
    Ok(OriginRecord {
        schema_version: "proposal-store-root/1".into(),
        origin,
        directory,
    })
}

/// A mutation may verify previously issued identity, but never issue a new one.
pub(super) fn existing_origin(
    store: &ProposalFiles,
    writer: &WriterReservation<'_>,
) -> Result<ProposalStoreOrigin, ApiError> {
    writer.check(store)?;
    let root = store._guards.last().ok_or_else(recover)?;
    let directory = StorageDirectoryIdentity::observe(root)?;
    if let Some(snapshot) = store.origin.get() {
        if snapshot.record.directory != directory {
            return Err(recover());
        }
        return Ok(snapshot.record.origin.clone());
    }
    let snapshot = read(&store.path.join(NAME)).map_err(|error| {
        if error.code == ErrorCode::NotFound {
            recover()
        } else {
            error
        }
    })?;
    if snapshot.record.directory != directory {
        return Err(recover());
    }
    store.origin.set(snapshot).map_err(|_| recover())?;
    Ok(store
        .origin
        .get()
        .ok_or_else(recover)?
        .record
        .origin
        .clone())
}
pub(super) fn capture_origin(
    store: &ProposalFiles,
    writer: &WriterReservation<'_>,
) -> Result<ProposalStoreOrigin, ApiError> {
    writer.check(store)?;
    // immutable_storage_paths::root opens reversed ancestors; its final held
    // directory is exactly the original root. No path reopen replaces it.
    let root = store._guards.last().ok_or_else(recover)?;
    let directory = StorageDirectoryIdentity::observe(root)?;
    if let Some(snapshot) = store.origin.get() {
        if snapshot.record.directory != directory {
            return Err(recover());
        }
        return Ok(snapshot.record.origin.clone());
    }
    let path = store.path.join(NAME);
    let snapshot = match read(&path) {
        Ok(snapshot) => snapshot,
        Err(error) if error.code == ErrorCode::NotFound => {
            let record = issue(directory)?;
            let raw = serde_json::to_vec(&record).map_err(|_| recover())?;
            if raw.len() as u64 > MAX_BYTES {
                return Err(recover());
            }
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .share(ShareMode::ReadOnly)
                .open(&path)
            {
                Ok(mut file) => {
                    file.write_all(&raw).map_err(|_| recover())?;
                    file.sync_all().map_err(|_| recover())?;
                    drop(file);
                    let snapshot = read(&path)?;
                    if snapshot.record != record {
                        return Err(recover());
                    }
                    snapshot
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => read(&path)?,
                Err(error) => return Err(super::io_error(error)),
            }
        }
        Err(error) => return Err(error),
    };
    // Re-query the SAME original directory after possible creation/read; a
    // copied metadata record never grants this store the old root's identity.
    if snapshot.record.directory != StorageDirectoryIdentity::observe(root)? {
        return Err(recover());
    }
    store.origin.set(snapshot).map_err(|_| {
        ApiError::new(
            ErrorCode::Internal,
            "Proposal origin was concurrently initialized",
        )
    })?;
    Ok(store
        .origin
        .get()
        .ok_or_else(recover)?
        .record
        .origin
        .clone())
}

#[cfg(test)]
mod tests;
