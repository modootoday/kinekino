//! Decode bounded original observations; these data never become native/SDK authority.
use kinekino_player::application::ui_observation::{
    DispatchObservation, FINAL_MAX_BYTES, IDENTITY_TEXT_MAX_BYTES, JoinObservation,
    NATIVE_READY_KIND, NativeRoutedFinalV2, NativeRoutedFrameReadyV1, READY_MAX_BYTES,
    VIEWER_OUTCOME_KIND,
};
use serde::Deserialize;
use std::{collections::BTreeMap, io};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope<T> {
    kind: String,
    data: T,
    #[serde(rename = "elapsed_ms")]
    _elapsed_ms: u128,
    #[serde(rename = "elapsed_scope")]
    _elapsed_scope: String,
}
#[derive(Deserialize)]
struct FinalData {
    native_routed: NativeRoutedFinalV2,
    #[serde(flatten)]
    _compatible: BTreeMap<String, serde_json::Value>,
}
fn complete_line(bytes: &[u8], maximum: usize) -> io::Result<&[u8]> {
    let line = bytes
        .strip_suffix(b"\n")
        .ok_or_else(|| io::Error::other("Partial or missing original observation line"))?;
    if line.len() > maximum || line.contains(&b'\n') {
        return Err(io::Error::other(
            "Original observation line exceeds its bound",
        ));
    }
    Ok(line)
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8], maximum: usize) -> io::Result<T> {
    serde_json::from_slice(complete_line(bytes, maximum)?).map_err(io::Error::other)
}
pub(super) fn identity_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= IDENTITY_TEXT_MAX_BYTES
        && !value.chars().any(char::is_control)
}
pub(super) fn event_kind(bytes: &[u8]) -> io::Result<String> {
    let envelope: Envelope<serde_json::Value> = decode(bytes, FINAL_MAX_BYTES)?;
    Ok(envelope.kind)
}
pub(super) fn ready(bytes: &[u8]) -> io::Result<NativeRoutedFrameReadyV1> {
    let envelope: Envelope<NativeRoutedFrameReadyV1> = decode(bytes, READY_MAX_BYTES)?;
    let data = envelope.data;
    if envelope.kind != NATIVE_READY_KIND
        || data.schema_version != 1
        || !identity_text(&data.document.instance_document)
        || !identity_text(&data.frame.context_id)
        || data.child_id.0 == 0
        || data.route_generation.0 == 0
    {
        return Err(io::Error::other(
            "Original routed activation observation is invalid",
        ));
    }
    Ok(data)
}
pub(super) fn final_report(
    bytes: &[u8],
    expected_completed: u64,
) -> io::Result<NativeRoutedFinalV2> {
    let envelope: Envelope<FinalData> = decode(bytes, FINAL_MAX_BYTES)?;
    let data = envelope.data.native_routed;
    let provider_jobs = data.provider.jobs_receipt.as_ref();
    let valid = envelope.kind == VIEWER_OUTCOME_KIND
        && data.schema_version == 2
        && data.native_closed == Some(true)
        && data.viewer_workers_quiescent == Some(true)
        && data.execution.worker_stopped == Some(true)
        && data
            .execution
            .unacknowledged
            .is_some_and(|count| count.0 == 0)
        && matches!(data.local_host.pipe, JoinObservation::Joined)
        && matches!(data.local_host.sdk, JoinObservation::Joined)
        && matches!(
            data.local_host.dispatch,
            DispatchObservation::Idle | DispatchObservation::Completed
        )
        && data.transition.configured
        && data.transition.admission_closed == Some(true)
        && data.transition.ended == Some(true)
        && data.transition.error.is_none()
        && data.transition.remaining_job_id.is_none()
        && data.provider.configured
        && data.provider.completed == Some(true)
        && data.provider.pending == Some(false)
        && data.provider.browser_worker_exited == Some(true)
        && data.provider.first_error.is_none()
        && data.provider.secondary_errors.is_empty()
        && data.provider.secondary_overflow == Some(false)
        && provider_jobs.is_some()
        && data.dispatcher.joined == Some(true)
        && data.dispatcher.worker_panicked == Some(false)
        && data.dispatcher.faults.is_some_and(|faults| faults.0 == 0)
        && data.cleanup_complete
        && data.first_error.is_none()
        && data.diagnostic_write_error.is_none();
    if !valid {
        return Err(io::Error::other(
            "Original configured-owner cleanup is incomplete or faulted; preserve raw report",
        ));
    }
    if !provider_jobs.is_some_and(|jobs| {
        jobs.completed_total.0 == expected_completed
            && jobs.cancelled_total.0 == 0
            && jobs.interrupted_total.0 == 0
    }) {
        let observed = provider_jobs
            .map(|jobs| {
                format!(
                    "completed={}, cancelled={}, interrupted={}",
                    jobs.completed_total.0, jobs.cancelled_total.0, jobs.interrupted_total.0
                )
            })
            .unwrap_or_else(|| "missing receipt".into());
        return Err(io::Error::other(format!(
            "Original provider job outcomes do not match expected completed={expected_completed}; observed {observed}; native cleanup is complete; preserve raw report"
        )));
    }
    Ok(data)
}

#[cfg(test)]
#[path = "observation/tests.rs"]
mod tests;
