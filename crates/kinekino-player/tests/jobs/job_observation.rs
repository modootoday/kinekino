//! Job observation helpers shared by the provider and non-provider suites; they only read job
//! state and never cancel or replace the original job.
#[path = "provider/observation_trace.rs"]
mod observation_trace;
use kinekino_player::application::{
    api::{ApplicationClient, JobClient, RequestContext},
    jobs::{GetJob, JobId, JobRecord, JobState},
};
use serde_json::json;
use std::time::{Duration, Instant};

pub(super) fn terminal(client: &dyn ApplicationClient, id: &JobId) -> JobRecord {
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(45))
        .expect("Job observation deadline overflow");
    terminal_until(client, id, deadline)
}
pub(super) fn try_terminal_until(
    endpoint: &str,
    id: &JobId,
    original: kinekino_process::domain::deadline::Deadline,
    cap: Duration,
    root: &std::path::Path,
) -> Result<JobRecord, Box<super::kinekino_process::CliFailure>> {
    use super::kinekino_process::{CliFailure, phase_deadline, sdk_failure, transport_at};
    let failed = |stage, error| CliFailure::not_spawned(stage, error, root);
    let deadline =
        phase_deadline(original, cap).map_err(|error| failed("job observation budget", error))?;
    let mut trace = observation_trace::ObservationTrace::new();
    let result = (|| {
        loop {
            let client = transport_at(endpoint, deadline)
                .map_err(|error| failed("job transport budget", error))?;
            trace.begin_read();
            let job = match client.get_job(RequestContext::default(), GetJob { job_id: id.clone() })
            {
                Ok(job) => {
                    trace.observed(job.state);
                    job
                }
                Err(error) => {
                    trace.failed(&error);
                    return Err(sdk_failure("job observation", error, root));
                }
            };
            if deadline.operation_remaining(Instant::now()).is_zero() {
                return Err(failed(
                    "job observation deadline",
                    std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "Original job observation cutoff exceeded",
                    ),
                ));
            }
            if job.state.terminal() {
                return Ok(job);
            }
            std::thread::sleep(
                Duration::from_millis(100).min(deadline.operation_remaining(Instant::now())),
            );
        }
    })();
    trace.finish(result, root, id)
}
// Observation timeout does not cancel the original job or replace its owner.
pub(super) fn terminal_until(
    client: &dyn ApplicationClient,
    id: &JobId,
    deadline: Instant,
) -> JobRecord {
    loop {
        let job = client
            .get_job(RequestContext::default(), GetJob { job_id: id.clone() })
            .unwrap();
        if job.state.terminal() {
            return job;
        }
        assert!(
            Instant::now() < deadline,
            "browser job did not reach terminal state"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}
