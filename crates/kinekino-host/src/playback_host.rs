//! Combined host CLI input/output and assembly.
#[cfg(test)]
use crate::host_shutdown::preserve_first_error;
use crate::host_shutdown::{
    DependencyObservation, HostRunOutcome, IoPhase, TransportOutcome, dependency_deadline,
    failed_dependency, finish_dependency, observe_dependency, pending_playback, pending_workspace,
    serve_transport,
};
use kinekino_player::{
    application::api::{ApiError, ErrorCode, PlayerSdk},
    infrastructure::{
        local_host::{LocalHost, TransportOptions},
        playback_workspace::{PlaybackWorkspaceConfiguration, RetainingPlaybackWorkspaceHost},
        subtitle_files::SubtitleFiles,
    },
};
use std::{io::Write, path::PathBuf, sync::Arc};
pub(crate) type Dependencies = (
    RetainingPlaybackWorkspaceHost,
    kinekino_player::application::skip_segments::host::SkipHostOwner,
);

struct Configuration {
    job_root: PathBuf,
    catalog_root: PathBuf,
    profile_root: PathBuf,
    executable: PathBuf,
    sha256: String,
}
fn configuration(args: &[String]) -> Result<Configuration, ApiError> {
    let mut values: [Option<String>; 5] = [None, None, None, None, None];
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--serve-playback" {
            index += 1;
            continue;
        }
        let slot = match args[index].as_str() {
            "--job-root" => 0,
            "--catalog-root" => 1,
            "--browser-profile-root" => 2,
            "--mpv-executable" => 3,
            "--mpv-sha256" => 4,
            _ => return Err(invalid()),
        };
        index += 1;
        let value = args
            .get(index)
            .filter(|value| !value.starts_with("--"))
            .ok_or_else(invalid)?;
        if values[slot].replace(value.clone()).is_some() {
            return Err(invalid());
        }
        index += 1;
    }
    let [
        Some(job),
        Some(catalog),
        Some(profile),
        Some(executable),
        Some(sha256),
    ] = values
    else {
        return Err(invalid());
    };
    let [job_root, catalog_root, profile_root, executable] =
        [job, catalog, profile, executable].map(PathBuf::from);
    if [&job_root, &catalog_root, &profile_root, &executable]
        .iter()
        .any(|path| !path.is_absolute())
    {
        return Err(invalid());
    }
    Ok(Configuration {
        job_root,
        catalog_root,
        profile_root,
        executable,
        sha256,
    })
}
fn serve(args: &[String]) -> Result<HostRunOutcome<Dependencies>, ApiError> {
    let config = configuration(args)?;
    let host = match RetainingPlaybackWorkspaceHost::start_retaining_trusted_standalone(
        PlaybackWorkspaceConfiguration {
            job_root: &config.job_root,
            catalog_root: &config.catalog_root,
            profile_root: &config.profile_root,
            mpv_executable: &config.executable,
            mpv_sha256: &config.sha256,
            parent: None,
        },
    ) {
        Ok(host) => host,
        Err(failure) => {
            let deadline = dependency_deadline();
            let (error, pending) = failure.into_shutdown(deadline);
            return Ok(match pending {
                Some(owner) => failed_dependency(owner, error, deadline),
                None => HostRunOutcome::Complete {
                    result: Err(error),
                    local_host_shutdown: None,
                },
            });
        }
    };
    let sessions = Arc::new(host.sessions());
    let skip = match (|| {
        kinekino_player::application::skip_segments::host::SkipHostRegistry::new_standalone_export_host(
            kinekino_player::infrastructure::skip_session_identity::new_host_generation()?,
            sessions.clone(),
            1,
        )
    })() {
        Ok(skip) => Arc::new(skip),
        Err(error) => {
            let deadline = dependency_deadline();
            return Ok(failed_dependency(
                pending_workspace(host, deadline),
                error,
                deadline,
            ));
        }
    };
    let _skip_owner = skip.owner();
    let sdk = match (|| {
        Ok::<_, ApiError>(
            PlayerSdk::new(SubtitleFiles)
                .with_jobs(host.jobs()?)
                .with_catalog(host.catalog()?)
                .with_shared_episode_playback(sessions)
                .with_skip_host(skip.clone())
                .with_episode_transitions(host.transition_service()?),
        )
    })() {
        Ok(sdk) => sdk,
        Err(error) => {
            skip.begin_close();
            let deadline = dependency_deadline();
            return Ok(failed_dependency(
                pending_playback(host, _skip_owner, deadline),
                error,
                deadline,
            ));
        }
    };
    let local = match LocalHost::start(Arc::new(sdk), TransportOptions::default()) {
        Ok(local) => local,
        Err(error) => {
            skip.begin_close();
            let deadline = dependency_deadline();
            return Ok(failed_dependency(
                pending_playback(host, _skip_owner, deadline),
                error,
                deadline,
            ));
        }
    };
    let ready = serde_json::json!({
        "event":"playback_host_ready","endpoint":local.endpoint(),
        "engine_sha256":config.sha256,"scope":"current_user_same_privilege",
        "ready_means":"transport_ready; browser/task/native media readiness observed separately",
        "playback_verified":false,"automatic_transition_commit":false,"stop":"stdin_eof"
    });
    let outcome = serve_transport(
        local,
        (host, _skip_owner),
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        &ready,
        |phase, _| match phase {
            IoPhase::Read => ApiError::new(ErrorCode::EngineFailed, "Playback host input failed"),
            IoPhase::ReadyWrite | IoPhase::ReadyFlush => output_failed(),
        },
        || skip.begin_close(),
    );
    let transport = match outcome {
        TransportOutcome::Incomplete(owner) => return Ok(HostRunOutcome::Incomplete(owner)),
        TransportOutcome::Ready(ready) => ready,
    };
    let ((host, skip_owner), first_error, local_status) = transport.into_parts();
    let deadline = dependency_deadline();
    let completion = match observe_dependency(
        pending_playback(host, skip_owner, deadline),
        first_error,
        Some(local_status),
        deadline,
    ) {
        DependencyObservation::Complete(result) => result,
        DependencyObservation::Incomplete(owner) => {
            return Ok(HostRunOutcome::DependencyIncomplete(owner));
        }
    };
    let result = (|| {
        let receipt = finish_dependency(completion)?;
        writeln!(std::io::stdout().lock(), "{}", serde_json::json!({"event":"playback_host_stopped","host_lifetime_ended":true,"receipt":receipt,"local_host_shutdown":crate::host_shutdown::LocalHostShutdownView::from(local_status)})).map_err(|_| output_failed())?;
        Ok(())
    })();
    Ok(HostRunOutcome::Complete {
        result,
        local_host_shutdown: Some(local_status),
    })
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Playback host requires only --serve-playback, separate absolute job/catalog/profile roots, absolute mpv executable and explicit SHA256",
    )
}
fn output_failed() -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, "Playback host output failed")
}
#[cfg(test)]
fn failed_export_setup<T: serde::Serialize>(
    error: ApiError,
    shutdown: impl FnOnce() -> Result<T, ApiError>,
) -> HostRunOutcome<Dependencies> {
    let cleanup = shutdown();
    let _ = writeln!(
        std::io::stderr().lock(),
        "{}",
        serde_json::json!({"event":"playback_host_export_setup_shutdown","cleanup":&cleanup})
    );
    HostRunOutcome::Complete {
        result: preserve_first_error(Some(error), cleanup).map(|_| ()),
        local_host_shutdown: None,
    }
}
pub(crate) fn run(args: &[String]) -> HostRunOutcome<Dependencies> {
    match serve(args) {
        Ok(outcome) => outcome,
        Err(error) => HostRunOutcome::Complete {
            result: Err(error),
            local_host_shutdown: None,
        },
    }
}
#[cfg(test)]
mod export_setup_tests {
    use super::*;
    #[test]
    fn failed_capture_observes_shutdown_once_preserves_first_error_and_never_reports_ready() {
        for fails in [false, true] {
            let calls = std::cell::Cell::new(0);
            let outcome = failed_export_setup(
                ApiError::new(ErrorCode::EngineFailed, "original issuer failure"),
                || {
                    calls.set(calls.get() + 1);
                    if fails {
                        Err(ApiError::new(
                            ErrorCode::Cancelled,
                            "secondary cleanup failure",
                        ))
                    } else {
                        Ok(serde_json::Value::Null)
                    }
                },
            );
            assert_eq!(calls.get(), 1);
            let HostRunOutcome::Complete {
                result: Err(error),
                local_host_shutdown: None,
            } = outcome
            else {
                panic!("capture failure cannot report a ready transport")
            };
            assert_eq!(error.code, ErrorCode::EngineFailed);
            assert_eq!(error.message, "original issuer failure");
        }
    }
}
