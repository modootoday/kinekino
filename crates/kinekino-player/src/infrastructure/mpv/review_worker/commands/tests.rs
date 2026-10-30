//! Mailbox interleavings only; no real engine or proof authenticity claims.
use super::*;
use crate::application::review_playback::ReviewContentClass;
struct Fixture {
    reader: ReviewWorkerControl,
    requests: mpsc::Receiver<CommandRequest>,
    control: MpvControlHandle,
    wire: tokio::sync::mpsc::Receiver<serde_json::Value>,
    source: ReviewInputIdentity,
    gate: Gate,
}
fn fixture() -> Fixture {
    let source = ReviewInputIdentity {
        case: ReviewCase {
            package_sha256: "a".repeat(64),
            item_id: "item".into(),
            source_request_sha256: "b".repeat(64),
        },
        media: MediaReference {
            asset_id: "media".into(),
            sha256: "c".repeat(64),
        },
        bytes: 1,
        declared_media_type: "video/mp4".into(),
        classification: ReviewContentClass::IsoBmffHeader,
    };
    let (sender, requests) = mpsc::sync_channel(8);
    let (wire_sender, wire) = tokio::sync::mpsc::channel(8);
    let control = MpvControlHandle::new(wire_sender).unwrap();
    control.state.connected.store(true, Ordering::Release);
    let gate = Arc::new(Mutex::new(false));
    let reader = ReviewWorkerControl::new(
        sender,
        gate.clone(),
        source.case.clone(),
        source.media.clone(),
        control.instance_id(),
    )
    .unwrap();
    Fixture {
        reader,
        requests,
        control,
        wire,
        source,
        gate,
    }
}
fn runtime() -> Deadline {
    Deadline::new(
        Instant::now(),
        Duration::from_secs(10),
        Duration::from_secs(1),
    )
    .unwrap()
}
fn queued(fixture: &Fixture, instance: u64) -> mpsc::Receiver<Result<ControlReceipt, ApiError>> {
    let (reply, response) = mpsc::sync_channel(1);
    let mut binding = fixture.reader.binding.clone();
    binding.instance = instance;
    fixture
        .reader
        .sender
        .try_send(CommandRequest {
            binding,
            action: PlaybackControl::Pause,
            expires: Instant::now() + Duration::from_secs(2),
            phase: Arc::new(AtomicU8::new(QUEUED)),
            reply,
        })
        .unwrap();
    response
}
#[test]
fn queued_caller_timeout_prevents_late_ipc_even_after_worker_resumes() {
    let mut f = fixture();
    let failure = f
        .reader
        .submit_until(PlaybackControl::Pause, Instant::now())
        .unwrap_err();
    assert_eq!(failure.details["delivery"], "not_sent");
    let calls = std::cell::Cell::new(0);
    process_pending(
        &f.requests,
        &f.source,
        &f.control,
        runtime(),
        &f.gate,
        &|_| {
            calls.set(calls.get() + 1);
            Ok(())
        },
    );
    assert_eq!(calls.get(), 0);
    assert!(f.wire.try_recv().is_err());
}
#[test]
fn foreign_instance_and_revoked_proof_never_enter_ipc() {
    let mut f = fixture();
    let wrong = queued(&f, f.control.instance_id() + 1);
    process_pending(
        &f.requests,
        &f.source,
        &f.control,
        runtime(),
        &f.gate,
        &|_| panic!("foreign instance reached proof"),
    );
    assert_eq!(
        wrong.recv().unwrap().unwrap_err().code,
        ErrorCode::RevisionConflict
    );
    let revoked = queued(&f, f.control.instance_id());
    process_pending(
        &f.requests,
        &f.source,
        &f.control,
        runtime(),
        &f.gate,
        &|_| Err(ApiError::new(ErrorCode::RevisionConflict, "proof revoked")),
    );
    let failure = revoked.recv().unwrap().unwrap_err();
    assert_eq!(failure.code, ErrorCode::RevisionConflict);
    assert_eq!(failure.details["delivery"], "not_sent");
    assert!(f.wire.try_recv().is_err());
}
#[test]
fn close_after_proof_but_before_publication_rejects_the_original_request() {
    let mut f = fixture();
    let reply = queued(&f, f.control.instance_id());
    process_pending(
        &f.requests,
        &f.source,
        &f.control,
        runtime(),
        &f.gate,
        &|_| {
            *f.gate.lock().unwrap() = true;
            Ok(())
        },
    );
    assert_eq!(
        reply.recv().unwrap().unwrap_err().details["delivery"],
        "not_sent"
    );
    assert!(f.wire.try_recv().is_err());
}
#[test]
fn accepted_worker_request_enqueues_once_and_reports_only_unobserved_completion() {
    let mut f = fixture();
    let reply = queued(&f, f.control.instance_id());
    let checked = std::cell::Cell::new(0);
    process_pending(
        &f.requests,
        &f.source,
        &f.control,
        runtime(),
        &f.gate,
        &|_| {
            checked.set(checked.get() + 1);
            Ok(())
        },
    );
    let receipt = reply.recv().unwrap().unwrap();
    assert_eq!(receipt.engine_instance_id.0, f.control.instance_id());
    assert_eq!(
        receipt.completion,
        crate::application::playback_control::ControlCompletion::Unobserved
    );
    assert_eq!(checked.get(), 1);
    assert!(f.wire.try_recv().is_ok());
    assert!(f.wire.try_recv().is_err());
}
#[test]
fn missing_reply_after_publication_begins_is_uncertain_not_false_not_sent() {
    let f = fixture();
    let reader = f.reader.clone();
    let caller = thread::spawn(move || reader.submit(PlaybackControl::Pause));
    let request = f.requests.recv().unwrap();
    assert!(
        request
            .phase
            .compare_exchange(QUEUED, SENDING, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    );
    drop(request);
    let failure = caller.join().unwrap().unwrap_err();
    assert_eq!(failure.code, ErrorCode::EngineFailed);
    assert_eq!(failure.details["delivery"], "execution_unknown");
    assert_eq!(failure.details["automatic_retry"], false);
}
