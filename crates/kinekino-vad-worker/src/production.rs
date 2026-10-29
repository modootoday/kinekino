//! Conditional composition of existing admission/reader/engine ports, no issuer.
mod release_file;
use crate::{
    application::bind_static_worker,
    entry::WorkerLaunch,
    staged_files::{WorkerArtifactFiles, WorkerRequestFiles},
};
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use kinekino_vad::protocol::application::ValidatedWorkerRequest;
use kinekino_vad::{
    application::{AnalysisControl, ProfileAdmission, VadEngine},
    domain::{ArtifactPin, VadProfile},
};
use std::{
    io::{Read, Write},
    path::Path,
};

/// Borrowed existing trusted composition inputs. Constructing this struct grants
/// nothing: the original admission is called and actual bytes remain verified.
/// Never supply these expectations by copying worker argv/request/sidecar JSON.
pub struct ReleaseInputs<'a> {
    pub profile: &'a VadProfile,
    pub manifest: &'a ArtifactPin,
    pub admission: &'a dyn ProfileAdmission,
}
pub fn run_released_request(
    request: &ValidatedWorkerRequest,
    root: &Path,
    actual_executable: &Path,
    release: &ReleaseInputs<'_>,
    engine: &dyn VadEngine,
    control: &dyn AnalysisControl,
) -> Result<Vec<u8>, ApiError> {
    // Existing issuer must authorize BEFORE opening any release/model sidecar.
    release.admission.authorize(release.profile)?;
    let original = release_file::OriginalRelease::open(root, release.manifest, control)?;
    if original.release().profile() != release.profile {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Actual manifest differs from independently approved release",
        ));
    }
    bind_static_worker(
        release.profile,
        request.expected().profile(),
        original.release().profile().runtime(),
    )?;
    let reader = WorkerArtifactFiles::open(
        root,
        actual_executable,
        request.expected().descriptor(),
        release.profile,
    )?;
    // Original P0 rechecks admission and actual model/license/executable/WAV bytes.
    // Original manifest guard and reader remain live until engine/result returns.
    crate::run::run_request(request, release.admission, &reader, engine, control)
}
pub fn serve_released_request(
    reader: impl Read,
    files: &WorkerRequestFiles,
    launch: &WorkerLaunch,
    actual_executable: &Path,
    writer: impl Write,
    release: &ReleaseInputs<'_>,
    engine: &dyn VadEngine,
) -> Result<(), ApiError> {
    if &launch.manifest != release.manifest {
        return Err(ApiError::new(
            ErrorCode::SourceExpired,
            "Launch pin differs from independently approved original manifest",
        ));
    }
    let root = launch
        .request
        .parent()
        .ok_or_else(|| ApiError::new(ErrorCode::InvalidArgument, "Worker release root absent"))?;
    crate::control::serve_controlled_request(reader, files, launch, writer, |request, control| {
        run_released_request(request, root, actual_executable, release, engine, control)
    })
}

#[cfg(feature = "native-ort")]
pub fn serve_native_request<V: crate::infrastructure::LoadedRuntimeAdmission>(
    reader: impl Read,
    files: &WorkerRequestFiles,
    launch: &WorkerLaunch,
    actual_executable: &Path,
    writer: impl Write,
    release: &ReleaseInputs<'_>,
    original_runtime: V,
) -> Result<(), ApiError> {
    let engine = crate::infrastructure::OrtVadEngine::new(original_runtime);
    serve_released_request(
        reader,
        files,
        launch,
        actual_executable,
        writer,
        release,
        &engine,
    )
}
