use crate::host_shutdown::{
    DependencyObservation, HostRunOutcome, IoPhase, TransportOutcome, dependency_deadline,
    failed_dependency, finish_dependency, observe_dependency, pending_provider, serve_transport,
};
use kinekino_player::{
    application::api::{ApiError, ErrorCode, PlayerSdk},
    infrastructure::{
        browser_provider_jobs::RetainingProviderJobsHost,
        local_host::{LocalHost, TransportOptions},
        subtitle_files::SubtitleFiles,
    },
};
use std::{io::Write, path::PathBuf, sync::Arc};

fn input(args: &[String]) -> Result<[PathBuf; 3], ApiError> {
    let mut roots = [None, None, None];
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--serve-provider" {
            index += 1;
            continue;
        }
        let selected = match args[index].as_str() {
            "--job-root" => 0,
            "--catalog-root" => 1,
            "--browser-profile-root" => 2,
            _ => {
                return Err(ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Provider host accepts only its mode and three explicit root options",
                ));
            }
        };
        index += 1;
        let value = args
            .get(index)
            .filter(|v| !v.starts_with("--"))
            .ok_or_else(|| {
                ApiError::new(ErrorCode::InvalidArgument, "Provider root requires a value")
            })?;
        if roots[selected].replace(PathBuf::from(value)).is_some() {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Duplicate provider root",
            ));
        }
        index += 1;
    }
    let [Some(job), Some(catalog), Some(profile)] = roots else {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Provider host requires --job-root, --catalog-root and --browser-profile-root",
        ));
    };
    Ok([job, catalog, profile])
}
fn serve(args: &[String]) -> Result<HostRunOutcome<RetainingProviderJobsHost>, ApiError> {
    let [job, catalog, profile] = input(args)?;
    let host = match RetainingProviderJobsHost::start_retaining(&job, &catalog, &profile) {
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
    let jobs = host.jobs();
    let operations = jobs.capability().operations;
    let client = Arc::new(
        PlayerSdk::new(SubtitleFiles)
            .with_jobs(jobs)
            .with_catalog(host.catalog()),
    );
    let local = match LocalHost::start(client, TransportOptions::default()) {
        Ok(local) => local,
        Err(error) => {
            let deadline = dependency_deadline();
            return Ok(failed_dependency(
                pending_provider(host, deadline),
                error,
                deadline,
            ));
        }
    };
    let ready = serde_json::json!({"event":"provider_host_ready","endpoint":local.endpoint(),"scope":"current_user_same_privilege","operations":operations,"experimental_operations":["catalog.search","episodes.list","episodes.resolve"],"episode_resolution_scope":kinekino_player::application::episode_resolve::RESOLUTION_SCOPE,"browser_engine_ready":false,"ready_means":"transport_ready; engine/task readiness is observed separately","source_kind":"request_descriptor","capture_kind":"dom_snapshot","site_verified":false,"stop":"stdin_eof"});
    let outcome = serve_transport(
        local,
        host,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        &ready,
        |phase, _| {
            ApiError::new(
                ErrorCode::EngineFailed,
                match phase {
                    IoPhase::Read => "Provider host input failed",
                    IoPhase::ReadyWrite | IoPhase::ReadyFlush => "Provider ready output failed",
                },
            )
        },
        || {},
    );
    let transport = match outcome {
        TransportOutcome::Incomplete(owner) => return Ok(HostRunOutcome::Incomplete(owner)),
        TransportOutcome::Ready(ready) => ready,
    };
    let (host, first_error, local_status) = transport.into_parts();
    let deadline = dependency_deadline();
    let completion = match observe_dependency(
        pending_provider(host, deadline),
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
        writeln!(
        std::io::stdout().lock(),
        "{}",
        serde_json::json!({"event":"provider_host_stopped","receipt":receipt,"local_host_shutdown":crate::host_shutdown::LocalHostShutdownView::from(local_status)})
    )
    .map_err(|_| ApiError::new(ErrorCode::EngineFailed, "Provider stopped output failed"))?;
        Ok(())
    })();
    Ok(HostRunOutcome::Complete {
        result,
        local_host_shutdown: Some(local_status),
    })
}
pub(crate) fn run(args: &[String]) -> HostRunOutcome<RetainingProviderJobsHost> {
    match serve(args) {
        Ok(outcome) => outcome,
        Err(error) => HostRunOutcome::Complete {
            result: Err(error),
            local_host_shutdown: None,
        },
    }
}
