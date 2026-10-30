//! Explicit startup adapter; settings select inputs, actual producers supply authority.
#[path = "review_host/settings.rs"]
mod settings;
use kinekino_player::{
    application::{
        api::{ApiError, ErrorCode, PlayerSdk, Response},
        review_source::MutationControl,
    },
    assembly::review_playback::{self, IncompleteTimelineCleanup, ReviewStdioOutcome},
    infrastructure::timeline_evidence::{
        production::PendingTimelineProcess, publication::PublishedTimelineResolver,
    },
};
use serde_json::json;
use std::{io::Write, sync::Arc};
struct Control;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
struct Retain;
impl IncompleteTimelineCleanup for Retain {
    fn incomplete(&mut self, error: &ApiError, pending: &mut PendingTimelineProcess<'_, '_>) -> ! {
        let _ = writeln!(
            std::io::stderr().lock(),
            "{}",
            json!({"error":error,"cleanup_complete":false,"output_root":pending.output_root(),"cleanup_error":pending.cleanup_error()})
        );
        // Original borrowed source/tool/session/Job remain alive at abnormal exit.
        std::process::exit(i32::from(error.exit_code()));
    }
}
fn serve(args: &[String]) -> Result<u8, ApiError> {
    let settings = settings::read(args)?;
    let transport = settings.transport_options()?;
    let (preparation, tools, runtime, max_history, _config_guard) = settings.build()?;
    let token = review_playback::prepare_timeline(preparation, &tools, &Control, &mut Retain)?;
    let ready_case = token.case().clone();
    let ready_evidence = token.reference().clone();
    let resolver = Arc::new(PublishedTimelineResolver::new(token)?);
    let assembly = review_playback::assemble_sdk_with_evidence(
        PlayerSdk::default(),
        runtime,
        resolver,
        true,
        max_history,
    )?;
    let host = match review_playback::start_transport(assembly.client, assembly.sessions, transport)
    {
        Ok(host) => host,
        Err(failure) => {
            // No requests or media owners exist before the ready record.
            let cleanup = failure
                .sessions
                .begin_close()
                .and_then(|()| failure.sessions.observe_shutdown());
            if !cleanup.as_ref().is_ok_and(|state| state.complete) {
                let _ = writeln!(
                    std::io::stderr().lock(),
                    "{}",
                    json!({"error":failure.error,"startup_cleanup_complete":false,"cleanup":format!("{cleanup:?}")})
                );
                std::process::exit(i32::from(failure.error.exit_code()));
            }
            return Err(failure.error);
        }
    };
    let ready = json!({"host":"review","endpoint":host.endpoint(),"pid":std::process::id(),"case":ready_case,"evidence":ready_evidence,"new_model_execution":false,"quality_approved":false});
    match review_playback::serve_stdio(
        host,
        &ready,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
    ) {
        ReviewStdioOutcome::Complete(done) => {
            if let Some(error) = done.io_error.or(done.shutdown.first_error) {
                return Err(error);
            }
            Ok(0)
        }
        ReviewStdioOutcome::Incomplete {
            mut pending,
            io_error,
        } => {
            let observation = pending.poll();
            let fallback = ApiError::new(
                ErrorCode::Timeout,
                "Review host physical cleanup incomplete",
            );
            let error = io_error
                .as_ref()
                .or(observation.first_error)
                .unwrap_or(&fallback);
            let _ = writeln!(
                std::io::stderr().lock(),
                "{}",
                json!({"error":error,"cleanup_complete":false,"observation":format!("{observation:?}")})
            );
            std::process::exit(i32::from(error.exit_code()));
        }
    }
}
pub(super) fn run(args: &[String]) -> u8 {
    match serve(args) {
        Ok(code) => code,
        Err(error) => {
            let code = error.exit_code();
            let response = Response::failure("review-host".into(), error);
            if let Ok(record) = serde_json::to_string(&response)
                && writeln!(std::io::stdout().lock(), "{record}").is_err()
            {
                let _ = writeln!(std::io::stderr().lock(), "{record}");
            }
            code
        }
    }
}
