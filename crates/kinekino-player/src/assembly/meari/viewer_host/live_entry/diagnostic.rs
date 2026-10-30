//! Opt-in bounded phase metadata. Never serializes a selection, URL, result or error message.
use super::super::mount_proposal::{ProductProposalMountFailure, ProductProposalMountPartial};
use super::{LiveEntryPhase, LiveOpening, ViewerHost, ViewerHostError};
use std::{io::Write, sync::Mutex};

const MAX_RECORDS: u64 = 64;
const MAX_BYTES: usize = 768;
#[derive(Default)]
struct Trace {
    sequence: u64,
    last: Option<(u64, Option<LiveEntryPhase>, Option<String>)>,
    first_error_seen: bool,
}
static TRACE: Mutex<Trace> = Mutex::new(Trace {
    sequence: 0,
    last: None,
    first_error_seen: false,
});
fn kind(error: &ViewerHostError) -> String {
    match error {
        ViewerHostError::ReadConsumerRetry(_) => "ReadConsumerRetry".into(),
        ViewerHostError::Callback(error) => format!("Callback::{:?}", error.code),
        ViewerHostError::UnsupportedCallback => "UnsupportedCallback".into(),
        ViewerHostError::ForeignCheckpoint => "ForeignCheckpoint".into(),
        ViewerHostError::Navigation(_) => "Navigation".into(),
        ViewerHostError::Sdk(error) => format!("Sdk::{:?}", error.code),
        ViewerHostError::Close(_) => "Close".into(),
        ViewerHostError::Card(_) => "Card".into(),
    }
}
fn mount_reason(message: &str) -> &'static str {
    match message {
        "Proposal provider differs from original closed product graph" => "ClosedGraphMismatch",
        "Original proposal mount turn differs" => "OriginalTurnMismatch",
        "Commit(\"node slot requires exact declared component props\")" => "ExactComponentProps",
        _ => "Other",
    }
}
fn mount_metadata(failure: &ProductProposalMountFailure) -> (&'static str, &'static str) {
    let stage = match &failure.partial {
        None => "Prepare",
        Some(ProductProposalMountPartial::Session(_)) => "BindSession",
        Some(ProductProposalMountPartial::Mount(failure)) => match &failure.error {
            crate::presentation::web_ui_adapter::proposals::window_bridge::PanelBridgeError::Commit(_) => "Commit",
            _ => "MountBridge",
        },
    };
    (stage, mount_reason(&failure.diagnostic))
}
fn record(
    generation: u64,
    phase: Option<LiveEntryPhase>,
    error: Option<&ViewerHostError>,
    mount: Option<&ProductProposalMountFailure>,
) {
    if !crate::infrastructure::diagnostics::enabled() {
        return;
    }
    let Ok(mut trace) = TRACE.lock() else { return };
    let signature = (generation, phase, error.map(kind));
    if trace.sequence >= MAX_RECORDS || trace.last.as_ref() == Some(&signature) {
        return;
    }
    let first_error = signature.2.is_some() && !trace.first_error_seen;
    trace.first_error_seen |= signature.2.is_some();
    trace.sequence += 1;
    let record = serde_json::json!({
        "kind":"product-live-entry-phase",
        "sequence":trace.sequence,
        "generation":generation,
        "phase":phase.map(|value|format!("{value:?}")),
        "error_kind":signature.2,
        "first_error":first_error,
        "mount_stage":mount.map(mount_metadata).map(|value| value.0),
        "mount_reason":mount.map(mount_metadata).map(|value| value.1),
    });
    trace.last = Some(signature);
    drop(trace);
    if let Ok(mut bytes) = serde_json::to_vec(&record)
        && bytes.len() < MAX_BYTES
    {
        bytes.push(b'\n');
        // Diagnostic failure cannot replace the original result or change a retry decision.
        let _ = std::io::stderr().lock().write_all(&bytes);
    }
}
pub(super) fn opening(opening: &LiveOpening, error: Option<&ViewerHostError>) {
    record(
        opening.ticket.generation(),
        Some(opening.phase),
        error.or(opening.error.as_ref()),
        opening.mount_failure.as_deref(),
    );
}
pub(super) fn host(host: &ViewerHost, error: Option<&ViewerHostError>) {
    if !crate::infrastructure::diagnostics::enabled() {
        return;
    }
    if let Ok(cards) = host.cards.try_borrow() {
        match &cards.live_opening {
            Some(entry) => opening(entry, error),
            None => record(0, None, error, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn route_trace_error_kind_never_contains_original_message_or_url() {
        let original =
            ViewerHostError::Sdk(std::sync::Arc::new(crate::application::api::ApiError::new(
                crate::application::api::ErrorCode::Unsupported,
                "PRIVATE_CANDIDATE https://private.invalid/source",
            )));
        assert_eq!(kind(&original), "Sdk::Unsupported");
        assert!(!kind(&original).contains("PRIVATE_CANDIDATE"));
        assert!(!kind(&original).contains("private.invalid"));
    }
    #[test]
    fn mount_reason_matches_only_known_static_contracts() {
        assert_eq!(
            mount_reason("Commit(\"node slot requires exact declared component props\")"),
            "ExactComponentProps"
        );
        assert_eq!(
            mount_reason("PRIVATE_CANDIDATE https://private.invalid/source"),
            "Other"
        );
        assert_eq!(
            mount_reason("Commit(\"node slot requires exact declared component props\") PRIVATE"),
            "Other"
        );
    }
}
