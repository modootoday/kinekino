//! Contract fixtures; no platform launch or proof authenticity is claimed here.
use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
#[derive(Default)]
struct Harness {
    state: Mutex<OwnerObservation>,
    starts: AtomicUsize,
    closes: AtomicUsize,
    drops: AtomicUsize,
    deny: AtomicBool,
    fail_observe: AtomicBool,
    foreign: AtomicBool,
    case_leases: AtomicUsize,
    control_available: AtomicBool,
    controls: AtomicUsize,
    bad_receipt: AtomicBool,
    host_observe: AtomicBool,
    fail_close: AtomicBool,
}
struct Launcher(Arc<Harness>);
struct CaseHold {
    case: ReviewCase,
    harness: Arc<Harness>,
}
impl crate::application::review_source::CaseLease for CaseHold {
    fn case(&self) -> &ReviewCase {
        &self.case
    }
    fn check_current(&self) -> Result<(), ApiError> {
        if self.harness.deny.load(Ordering::SeqCst) {
            Err(ApiError::new(ErrorCode::NotFound, "case denied"))
        } else {
            Ok(())
        }
    }
}
impl Drop for CaseHold {
    fn drop(&mut self) {
        self.harness.case_leases.fetch_sub(1, Ordering::SeqCst);
    }
}
struct Owner {
    selection: Selection,
    harness: Arc<Harness>,
}
impl SessionLauncher for Launcher {
    fn controls_available(&self) -> bool {
        self.0.control_available.load(Ordering::SeqCst)
    }
    fn retain_case(
        &self,
        case: &ReviewCase,
    ) -> Result<Box<dyn crate::application::review_source::CaseLease + '_>, ApiError> {
        if self.0.deny.load(Ordering::SeqCst) {
            Err(ApiError::new(ErrorCode::NotFound, "case denied"))
        } else {
            self.0.case_leases.fetch_add(1, Ordering::SeqCst);
            Ok(Box::new(CaseHold {
                case: case.clone(),
                harness: self.0.clone(),
            }))
        }
    }
    fn start(&self, selection: &Selection) -> Result<Box<dyn SessionOwner>, ApiError> {
        assert!(self.0.case_leases.load(Ordering::SeqCst) > 0);
        self.0.starts.fetch_add(1, Ordering::SeqCst);
        *self.0.state.lock().unwrap() = OwnerObservation::default();
        let mut selection = selection.clone();
        if self.0.foreign.load(Ordering::SeqCst) {
            selection.association_id = "foreign".into();
        }
        Ok(Box::new(Owner {
            selection,
            harness: self.0.clone(),
        }))
    }
}
impl SessionOwner for Owner {
    fn submit(&self, instance: DecimalU64, _: PlaybackControl) -> Result<ControlReceipt, ApiError> {
        assert!(self.harness.case_leases.load(Ordering::SeqCst) > 0);
        self.harness.controls.fetch_add(1, Ordering::SeqCst);
        Ok(ControlReceipt::enqueued(
            if self.harness.bad_receipt.load(Ordering::SeqCst) {
                instance.0 + 1
            } else {
                instance.0
            },
        ))
    }
    fn selection(&self) -> &Selection {
        &self.selection
    }
    fn observe(&self) -> Result<OwnerObservation, ApiError> {
        assert!(
            self.harness.case_leases.load(Ordering::SeqCst) > 0
                || self.harness.host_observe.load(Ordering::SeqCst)
        );
        if self.harness.fail_observe.load(Ordering::SeqCst) {
            Err(ApiError::new(ErrorCode::EngineFailed, "observation lost"))
        } else {
            Ok(self.harness.state.lock().unwrap().clone())
        }
    }
    fn request_close(&self) -> Result<(), ApiError> {
        self.harness.closes.fetch_add(1, Ordering::SeqCst);
        if self.harness.fail_close.load(Ordering::SeqCst) {
            return Err(ApiError::new(ErrorCode::EngineFailed, "close request lost"));
        }
        self.harness.state.lock().unwrap().closing = true;
        Ok(())
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        self.harness.drops.fetch_add(1, Ordering::SeqCst);
    }
}
struct Control(bool);
mod host_shutdown;
impl MutationControl for Control {
    fn checkpoint(&self) -> Result<(), ApiError> {
        if self.0 {
            Err(ApiError::new(ErrorCode::Cancelled, "cancelled"))
        } else {
            Ok(())
        }
    }
}
fn selection() -> Selection {
    Selection {
        case: ReviewCase {
            package_sha256: "a".repeat(64),
            item_id: "item".into(),
            source_request_sha256: "b".repeat(64),
        },
        association_id: "association".into(),
        association_revision: DecimalU64(2),
    }
}
fn fixture(capacity: usize) -> (Arc<Harness>, ReviewPlaybackSessions) {
    let harness = Arc::new(Harness::default());
    let sessions =
        ReviewPlaybackSessions::new("host-scope-a".into(), capacity, Launcher(harness.clone()))
            .unwrap();
    (harness, sessions)
}
fn finish(harness: &Harness) {
    *harness.state.lock().unwrap() = OwnerObservation {
        process_created: true,
        physical_closed: true,
        family_empty: true,
        ipc_exited: true,
        owner_finished: true,
        ..OwnerObservation::default()
    };
}
#[test]
fn open_replay_and_close_preserve_original_owner_until_all_physical_gates() {
    let (h, sessions) = fixture(3);
    let first = sessions.open(selection(), "open", &Control(false)).unwrap();
    assert_eq!(
        sessions.open(selection(), "open", &Control(true)).unwrap(),
        first
    );
    let mut changed = selection();
    changed.association_revision = DecimalU64(3);
    assert_eq!(
        sessions
            .open(changed, "open", &Control(false))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(h.starts.load(Ordering::SeqCst), 1);
    let close = sessions.close(&first.session).unwrap();
    assert!(close.close_requested && !close.observation.physical_closed);
    sessions.close(&first.session).unwrap();
    assert_eq!(h.closes.load(Ordering::SeqCst), 1);
    for state in [
        OwnerObservation {
            physical_closed: true,
            ..OwnerObservation::default()
        },
        OwnerObservation {
            process_created: true,
            physical_closed: true,
            owner_finished: true,
            ipc_exited: true,
            ..OwnerObservation::default()
        },
        OwnerObservation {
            process_created: true,
            physical_closed: true,
            owner_finished: true,
            family_empty: true,
            ..OwnerObservation::default()
        },
    ] {
        *h.state.lock().unwrap() = state;
        assert_eq!(
            sessions
                .open(selection(), "second", &Control(false))
                .unwrap_err()
                .code,
            ErrorCode::RevisionConflict
        );
    }
    finish(&h);
    let closed = sessions.status(&first.session).unwrap();
    assert!(closed.observation.quiescent());
    let second = sessions
        .open(selection(), "second", &Control(false))
        .unwrap();
    assert_ne!(first.session.ordinal, second.session.ordinal);
    assert_eq!(
        sessions.open(selection(), "open", &Control(true)).unwrap(),
        closed
    );
    assert_eq!(h.starts.load(Ordering::SeqCst), 2);
    let mut foreign = first.session.clone();
    foreign.host_scope = "old-host".into();
    assert_eq!(
        sessions.close(&foreign).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let mut foreign = first.session.clone();
    foreign.selection.association_revision = DecimalU64(9);
    assert_eq!(
        sessions.status(&foreign).unwrap_err().code,
        ErrorCode::RevisionConflict
    );
}
#[test]
fn cancellation_capacity_and_case_authority_do_not_erase_replay() {
    let (h, sessions) = fixture(1);
    assert_eq!(
        sessions
            .open(selection(), "open", &Control(true))
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    assert_eq!(h.starts.load(Ordering::SeqCst), 0);
    let first = sessions.open(selection(), "open", &Control(false)).unwrap();
    finish(&h);
    assert_eq!(
        sessions
            .open(selection(), "other", &Control(false))
            .unwrap_err()
            .code,
        ErrorCode::ResourceExhausted
    );
    assert!(
        sessions
            .open(selection(), "open", &Control(true))
            .unwrap()
            .observation
            .quiescent()
    );
    h.deny.store(true, Ordering::SeqCst);
    assert_eq!(
        sessions
            .open(selection(), "open", &Control(false))
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    assert_eq!(
        sessions.close(&first.session).unwrap_err().code,
        ErrorCode::NotFound
    );
    assert_eq!(h.starts.load(Ordering::SeqCst), 1);
}
#[test]
fn post_start_observation_failure_retains_the_same_owner_for_recovery() {
    let (h, sessions) = fixture(2);
    h.fail_observe.store(true, Ordering::SeqCst);
    let uncertain = sessions
        .open(selection(), "open", &Control(false))
        .unwrap_err();
    assert_eq!(uncertain.details["owner_retained"], true);
    assert_eq!(uncertain.details["automatic_retry"], false);
    assert_eq!(uncertain.details["delivery"], "execution_unknown");
    assert_eq!(
        sessions
            .open(selection(), "open", &Control(false))
            .unwrap_err()
            .code,
        ErrorCode::EngineFailed
    );
    assert_eq!(
        sessions
            .open(selection(), "open", &Control(false))
            .unwrap_err()
            .code,
        ErrorCode::EngineFailed
    );
    assert_eq!(h.starts.load(Ordering::SeqCst), 1);
    assert_eq!(h.drops.load(Ordering::SeqCst), 0);
    h.fail_observe.store(false, Ordering::SeqCst);
    let recovered = sessions.open(selection(), "open", &Control(false)).unwrap();
    sessions.close(&recovered.session).unwrap();
    assert_eq!(h.starts.load(Ordering::SeqCst), 1);
}
#[test]
fn foreign_runtime_owner_is_closed_and_retained_instead_of_replaced() {
    let (h, sessions) = fixture(2);
    h.foreign.store(true, Ordering::SeqCst);
    assert_eq!(
        sessions
            .open(selection(), "open", &Control(false))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(
        sessions
            .open(selection(), "other", &Control(false))
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(h.starts.load(Ordering::SeqCst), 1);
    assert_eq!(h.drops.load(Ordering::SeqCst), 0);
    drop(sessions);
    assert!(h.closes.load(Ordering::SeqCst) >= 1);
    assert_eq!(h.drops.load(Ordering::SeqCst), 1);
}
#[test]
fn concurrent_same_key_opens_create_one_owner() {
    let (h, sessions) = fixture(2);
    let sessions = Arc::new(sessions);
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let sessions = sessions.clone();
            std::thread::spawn(move || {
                sessions
                    .open(selection(), "same", &Control(false))
                    .unwrap()
                    .session
            })
        })
        .collect();
    let refs: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    assert!(refs.iter().all(|session| session == &refs[0]));
    assert_eq!(h.starts.load(Ordering::SeqCst), 1);
}
#[test]
fn session_control_checks_ref_instance_close_and_keeps_uncertain_receipts() {
    let (h, sessions) = fixture(2);
    h.control_available.store(true, Ordering::SeqCst);
    let opened = sessions.open(selection(), "open", &Control(false)).unwrap();
    assert_eq!(
        sessions
            .control(&opened.session, DecimalU64(7), PlaybackControl::Pause)
            .unwrap_err()
            .code,
        ErrorCode::NeedsInput
    );
    {
        let mut state = h.state.lock().unwrap();
        state.connected = true;
        state.engine_instance_id = Some(DecimalU64(7));
    }
    let mut foreign = opened.session.clone();
    foreign.selection.association_revision = DecimalU64(3);
    assert_eq!(
        sessions
            .control(&foreign, DecimalU64(7), PlaybackControl::Pause)
            .unwrap_err()
            .details["delivery"],
        "not_sent"
    );
    assert_eq!(
        sessions
            .control(&opened.session, DecimalU64(8), PlaybackControl::Pause)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(h.controls.load(Ordering::SeqCst), 0);
    assert_eq!(
        sessions
            .control(&opened.session, DecimalU64(7), PlaybackControl::Pause)
            .unwrap()
            .engine_instance_id,
        DecimalU64(7)
    );
    h.bad_receipt.store(true, Ordering::SeqCst);
    let uncertain = sessions
        .control(&opened.session, DecimalU64(7), PlaybackControl::Pause)
        .unwrap_err();
    assert_eq!(uncertain.details["delivery"], "execution_unknown");
    assert_eq!(uncertain.details["automatic_retry"], false);
    sessions.close(&opened.session).unwrap();
    assert_eq!(
        sessions
            .control(&opened.session, DecimalU64(7), PlaybackControl::Pause)
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
    assert_eq!(h.controls.load(Ordering::SeqCst), 2);
    assert_eq!(h.starts.load(Ordering::SeqCst), 1);
}
#[test]
fn control_wire_advertises_only_configured_controls_and_rejects_mutation_context() {
    use crate::application::api::{PlayerSdk, Request};
    use serde_json::json;
    let (h, sessions) = fixture(2);
    h.control_available.store(true, Ordering::SeqCst);
    let opened = sessions.open(selection(), "open", &Control(false)).unwrap();
    {
        let mut state = h.state.lock().unwrap();
        state.connected = true;
        state.engine_instance_id = Some(DecimalU64(7));
    }
    let sdk = PlayerSdk::default().with_review_playback_sessions(Arc::new(sessions));
    let mut request = Request::new("reviews.playback.control");
    request.payload = json!({"session":opened.session,"engine_instance_id":"7","control":{"action":"set_muted","muted":true}});
    let mut invalid = request.clone();
    invalid.context.idempotency_key = Some("not-a-replay-key".into());
    assert_eq!(
        sdk.execute(invalid).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
    let result = sdk.execute(request).data.unwrap();
    assert_eq!(result["receipt"]["completion"], "unobserved");
    let caps = sdk.execute(Request::new("capabilities")).data.unwrap();
    assert_eq!(caps["review_playback"]["control_available"], true);
    let mut schema = Request::new("schema");
    schema.payload = json!({"command":"reviews.playback.control"});
    let schema = sdk.execute(schema).data.unwrap();
    assert_eq!(schema["status"], "supported");
    assert!(
        schema["request"]["properties"]["payload"]["properties"]["control"]["oneOf"].is_array()
    );
}
#[test]
fn sdk_wire_context_capability_and_builders_preserve_the_same_session_service() {
    use crate::application::{
        api::{PlayerSdk, Request},
        frame_capture::UnavailableFrameCapture,
        model_review::UnavailableReviewStore,
        playback_control::UnavailablePlaybackControl,
        playback_status::UnavailablePlaybackStatus,
    };
    use serde_json::json;
    let (h, sessions) = fixture(3);
    let sdk = PlayerSdk::default()
        .with_review_playback_sessions(Arc::new(sessions))
        .with_frame_capture(UnavailableFrameCapture)
        .with_playback_status(UnavailablePlaybackStatus)
        .with_playback_control(UnavailablePlaybackControl)
        .with_review_store(UnavailableReviewStore);
    let mut open = Request::new("reviews.playback.open");
    open.payload = json!({"selection":selection()});
    assert_eq!(
        sdk.execute(open.clone()).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
    open.context.idempotency_key = Some("wire-open".into());
    for timeout in [Some(0), Some(30001)] {
        let mut invalid = open.clone();
        invalid.context.timeout_ms = timeout;
        assert_eq!(
            sdk.execute(invalid).error.unwrap().code,
            ErrorCode::InvalidArgument
        );
    }
    let mut invalid = open.clone();
    invalid.payload["executable"] = json!("untrusted.exe");
    assert_eq!(
        sdk.execute(invalid).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
    let first = sdk.execute(open.clone());
    assert!(first.ok, "{:?}", first.error);
    let first = first.data.unwrap();
    assert_eq!(sdk.execute(open).data.unwrap()["session"], first["session"]);
    assert_eq!(h.starts.load(Ordering::SeqCst), 1);
    let caps = sdk.execute(Request::new("capabilities")).data.unwrap();
    assert_eq!(caps["review_playback"]["available"], true);
    assert_eq!(caps["review_playback"]["max_history"], 3);
    for command in [
        "reviews.playback.open",
        "reviews.playback.status",
        "reviews.playback.close",
    ] {
        let mut request = Request::new("schema");
        request.payload = json!({"command":command});
        let schema = sdk.execute(request).data.unwrap();
        assert_eq!(schema["status"], "supported");
        assert_eq!(schema["frame_exact"], false);
        assert!(
            caps["commands"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["command"] == command && row["status"] == "supported")
        );
    }
    let mut close = Request::new("reviews.playback.close");
    close.payload = json!({"session":first["session"]});
    let mut invalid = close.clone();
    invalid.context.idempotency_key = Some("unneeded".into());
    assert_eq!(
        sdk.execute(invalid).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
    let mut invalid = close.clone();
    invalid.context.expected_revision = Some("2".into());
    assert_eq!(
        sdk.execute(invalid).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
    let closed = sdk.execute(close.clone()).data.unwrap();
    assert_eq!(closed["close_requested"], true);
    assert_eq!(closed["observation"]["physical_closed"], false);
    assert!(sdk.execute(close).ok);
    assert_eq!(h.closes.load(Ordering::SeqCst), 1);
    let mut status = Request::new("reviews.playback.status");
    status.payload = json!({"session":first["session"]});
    status.context.timeout_ms = Some(1);
    assert_eq!(
        sdk.execute(status.clone()).error.unwrap().code,
        ErrorCode::InvalidArgument
    );
    status.context.timeout_ms = None;
    assert!(sdk.execute(status).ok);
}
#[test]
fn default_sdk_session_commands_and_schema_both_report_unsupported() {
    use crate::application::api::{PlayerSdk, Request};
    let sdk = PlayerSdk::default();
    let caps = sdk.execute(Request::new("capabilities")).data.unwrap();
    assert_eq!(caps["review_playback"]["available"], false);
    for command in [
        "reviews.playback.open",
        "reviews.playback.status",
        "reviews.playback.close",
    ] {
        assert_eq!(
            sdk.execute(Request::new(command)).error.unwrap().code,
            ErrorCode::Unsupported
        );
        let mut request = Request::new("schema");
        request.payload = serde_json::json!({"command":command});
        assert_eq!(sdk.execute(request).data.unwrap()["status"], "unsupported");
    }
}
