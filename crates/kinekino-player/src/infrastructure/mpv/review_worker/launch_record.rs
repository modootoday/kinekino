//! Persist exact pre-spawn inputs while the original source/tool/Job guards are held.
use crate::{
    application::api::{ApiError, ErrorCode},
    infrastructure::{
        immutable_storage_paths, model_review::GuardedReviewMedia,
        review_runtime::RunningReviewCustody,
    },
};
use kinekino_process::infrastructure::family::{JobBudget, SpawnSpec};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};
pub(super) fn write(
    root: &Path,
    source: &GuardedReviewMedia,
    custody: &RunningReviewCustody,
    budget: JobBudget,
    spec: &SpawnSpec,
) -> Result<File, ApiError> {
    let path = root.join("INPUT.json");
    let arguments = spec
        .arguments
        .iter()
        .map(|value| {
            value.to_str().ok_or_else(|| {
                ApiError::new(
                    ErrorCode::Unsupported,
                    "Review launch evidence requires Unicode argv",
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let identity = source.identity();
    let body = serde_json::json!({"schema_version":1,"case":identity.case,"media":identity.media,"bytes":identity.bytes,"declared_media_type":identity.declared_media_type,"classification":format!("{:?}",identity.classification),"source_path":source.path(),"tool_path":custody.tool.path(),"tool_sha256":custody.tool.sha256(),"cwd":spec.cwd,"arguments":arguments,"job_committed_bytes":budget.max_committed_bytes,"affinity_mask":budget.affinity_mask,"cpu_threads":custody.profile.cpu_threads,"startup_timeout_ms":custody.profile.startup_timeout_ms,"playback_ready":false,"gui_rendering":false,"environment":"inherit; fixed no-config/no-scripts argv"});
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(error)?;
    file.write_all(&serde_json::to_vec_pretty(&body).map_err(|_| {
        ApiError::new(
            ErrorCode::EngineFailed,
            "Review launch record serialization failed",
        )
    })?)
    .map_err(error)?;
    file.sync_all().map_err(error)?;
    drop(file);
    immutable_storage_paths::file(&path)
}
fn error(error: std::io::Error) -> ApiError {
    ApiError::new(
        ErrorCode::EngineFailed,
        format!("Review launch record failed: {error}"),
    )
}
pub(super) fn terminal(root: &Path, state: &super::State) -> Result<(), ApiError> {
    let snapshot = state
        .lock()
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Review terminal state poisoned"))?
        .clone();
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join("PROCESS.json"))
        .map_err(error)?;
    file.write_all(&serde_json::to_vec_pretty(&snapshot).map_err(|_| {
        ApiError::new(
            ErrorCode::EngineFailed,
            "Review terminal serialization failed",
        )
    })?)
    .map_err(error)?;
    file.sync_all().map_err(error)
}
