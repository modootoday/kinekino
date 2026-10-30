use crate::host_shutdown::{
    DependencyObservation, HostRunOutcome, IoPhase, TransportOutcome, dependency_deadline,
    finish_dependency, observe_dependency, pending_native, preserve_first_error, serve_transport,
};
#[cfg(not(feature = "validation-export-delivery-loss"))]
use kinekino_player::infrastructure::local_host::LocalHost;
use kinekino_player::{
    application::api::{ApiError, ErrorCode, PlayerSdk},
    assembly::native_playback::{
        StandaloneNativeAssembly, StandaloneNativeConfiguration, assemble_standalone,
    },
    infrastructure::{
        local_host::TransportOptions, mpv::NativePlaybackHost, subtitle_files::SubtitleFiles,
    },
};
use std::{io::Write, path::PathBuf};
#[cfg(feature = "validation-export-delivery-loss")]
#[path = "native_host/validation.rs"]
mod validation;
pub(crate) type Dependencies = (
    NativePlaybackHost,
    kinekino_player::application::skip_segments::host::SkipHostOwner,
);

struct Configuration {
    catalog: PathBuf,
    executable: PathBuf,
    sha256: String,
}
fn configuration(args: &[String]) -> Result<Configuration, ApiError> {
    let mut values: [Option<String>; 3] = [None, None, None];
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--serve-native" {
            index += 1;
            continue;
        }
        let slot = match args[index].as_str() {
            "--catalog-root" => 0,
            "--mpv-executable" => 1,
            "--mpv-sha256" => 2,
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
    let [Some(catalog), Some(executable), Some(sha256)] = values else {
        return Err(invalid());
    };
    let trusted = kinekino_player::application::launch::native_playback::NativePlaybackConfiguration::from_explicit(
        catalog.into(), executable.into(), sha256,
    )?;
    Ok(Configuration {
        catalog: trusted.catalog,
        executable: trusted.executable,
        sha256: trusted.sha256,
    })
}
fn invalid() -> ApiError {
    ApiError::new(
        ErrorCode::InvalidArgument,
        "Native host requires only --serve-native, --catalog-root ABS_DIR, --mpv-executable ABS_MPV_EXE and --mpv-sha256 SHA256",
    )
}
fn serve(args: &[String]) -> Result<HostRunOutcome<Dependencies>, ApiError> {
    #[cfg(feature = "validation-export-delivery-loss")]
    let (validation_args, validation_prepared) = validation::prepare(args)?;
    #[cfg(feature = "validation-export-delivery-loss")]
    let args = validation_args.as_slice();
    let configuration = configuration(args)?;
    let StandaloneNativeAssembly {
        host,
        skip,
        skip_owner,
        client,
    } = match assemble_standalone(
        || PlayerSdk::new(SubtitleFiles),
        StandaloneNativeConfiguration {
            catalog: &configuration.catalog,
            executable: &configuration.executable,
            sha256: &configuration.sha256,
        },
    ) {
        Ok(assembly) => assembly,
        Err(failure) => {
            let (error, host) = failure.into_parts();
            return match host {
                Some(host) => Ok(failed_export_setup(error, || host.shutdown())),
                None => Err(error),
            };
        }
    };
    #[cfg(not(feature = "validation-export-delivery-loss"))]
    let local_result = LocalHost::start(client, TransportOptions::default());
    #[cfg(feature = "validation-export-delivery-loss")]
    let local_result = validation::start(
        client,
        TransportOptions::default(),
        validation_prepared,
        host.engine().validation_export_counter_probe(),
    );
    let local = match local_result {
        Ok(local) => local,
        Err(error) => {
            skip.begin_close();
            return Ok(failed_export_setup(error, || host.shutdown()));
        }
    };
    #[cfg(feature = "validation-export-delivery-loss")]
    let (local, validation_report) = local;
    let ready = serde_json::json!({"event":"native_host_ready","endpoint":local.endpoint(),"scope":"current_user_same_privilege","engine_sha256":configuration.sha256,"ready_means":"transport_ready; no media engine starts before playback.open","source":"reverified stored episode resolution","playback_verified":false,"stop":"stdin_eof"});
    let outcome = serve_transport(
        local,
        (host, skip_owner),
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        &ready,
        |phase, _| match phase {
            IoPhase::Read => ApiError::new(ErrorCode::EngineFailed, "Native host input failed"),
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
        pending_native(host, skip_owner, deadline),
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
        writeln!(std::io::stdout().lock(), "{}", serde_json::json!({"event":"native_host_stopped","receipt":receipt,"host_lifetime_ended":true,"local_host_shutdown":crate::host_shutdown::LocalHostShutdownView::from(local_status)})).map_err(|_| output_failed())?;
        Ok(())
    })();
    #[cfg(feature = "validation-export-delivery-loss")]
    let result = validation::finish(validation_report, result);
    Ok(HostRunOutcome::Complete {
        result,
        local_host_shutdown: Some(local_status),
    })
}
fn output_failed() -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, "Native host output failed")
}
fn failed_export_setup<T: serde::Serialize>(
    error: ApiError,
    shutdown: impl FnOnce() -> Result<T, ApiError>,
) -> HostRunOutcome<Dependencies> {
    let cleanup = shutdown();
    let _ = writeln!(
        std::io::stderr().lock(),
        "{}",
        serde_json::json!({"event":"native_host_export_setup_shutdown","cleanup":&cleanup})
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
    #[cfg(not(feature = "validation-export-delivery-loss"))]
    #[test]
    fn normal_build_rejects_validation_options_without_starting_a_host() {
        let base = vec![
            "--serve-native".into(),
            "--catalog-root".into(),
            "D:/catalog".into(),
            "--mpv-executable".into(),
            "D:/mpv.exe".into(),
            "--mpv-sha256".into(),
            "a".repeat(64),
        ];
        assert!(configuration(&base).is_ok());
        let mut extra = base;
        extra.extend([
            "--validation-export-drop-request-id".into(),
            "validation-request".into(),
            "--validation-export-evidence".into(),
            "D:/proof.jsonl".into(),
        ]);
        assert!(configuration(&extra).is_err());
    }
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
