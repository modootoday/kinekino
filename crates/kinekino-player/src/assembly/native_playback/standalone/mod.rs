use super::{NativeAssemblyFailure, StandaloneNativeAssembly, StandaloneNativeConfiguration};
use crate::{
    application::{
        api::{ApiError, PlayerSdk},
        catalog::CatalogArchive,
        playback_session::EpisodePlaybackSessions,
        skip_segments::host::SkipHostRegistry,
    },
    infrastructure::{
        catalog::CatalogFiles, mpv::NativePlaybackHost, skip_session_identity::new_host_generation,
        subtitle_files::SubtitleFiles,
    },
};
use std::sync::Arc;

/// No media is opened here. The existing SDK owns all admission and result policies.
pub fn assemble_standalone(
    base: impl FnOnce() -> PlayerSdk<SubtitleFiles>,
    configuration: StandaloneNativeConfiguration<'_>,
) -> Result<StandaloneNativeAssembly, Box<NativeAssemblyFailure>> {
    assemble_using(base, configuration, new_host_generation)
}

fn assemble_using(
    base: impl FnOnce() -> PlayerSdk<SubtitleFiles>,
    configuration: StandaloneNativeConfiguration<'_>,
    generation: impl FnOnce() -> Result<String, ApiError>,
) -> Result<StandaloneNativeAssembly, Box<NativeAssemblyFailure>> {
    let files = Arc::new(
        CatalogFiles::open(configuration.catalog).map_err(NativeAssemblyFailure::before_native)?,
    );
    let host = NativePlaybackHost::start_trusted_standalone(
        configuration.executable,
        configuration.sha256,
        files.clone(),
    )
    .map_err(NativeAssemblyFailure::before_native)?;
    let catalog =
        CatalogArchive::new(files.clone(), files.clone()).with_episode_results(files.clone());
    let sessions = Arc::new(EpisodePlaybackSessions::new(files, Arc::new(host.engine())));
    let skip = match generation().and_then(|generation| {
        SkipHostRegistry::new_standalone_export_host(generation, sessions.clone(), 1)
    }) {
        Ok(skip) => Arc::new(skip),
        Err(error) => {
            return Err(Box::new(NativeAssemblyFailure {
                error,
                host: Some(host),
            }));
        }
    };
    let skip_owner = skip.owner();
    let client = Arc::new(
        base()
            .with_catalog(catalog)
            .with_shared_episode_playback(sessions)
            .with_skip_host(skip.clone()),
    );
    Ok(StandaloneNativeAssembly {
        host,
        skip,
        skip_owner,
        client,
    })
}

#[cfg(test)]
mod tests;
