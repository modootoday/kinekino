use super::*;
use crate::domain::api_values::DecimalU64;

#[cfg(feature = "source-adapter-test-support")]
#[test]
fn replay_policy_panic_preserves_original_payload_and_shared_owner_lock() {
    use crate::application::{
        browser_network::{NetworkResponse, RequestHeaderEvidence},
        browser_provider::{MediaReplayPolicy, fixture_work_for_request},
    };
    struct Panics(Arc<AtomicBool>);
    impl MediaReplayPolicy for Panics {
        fn authorize(
            &self,
            _: &NetworkResponse,
            _: &RequestHeaderEvidence,
        ) -> Result<(), ApiError> {
            std::panic::panic_any(self.0.clone());
        }
    }
    let (original_work, stream, ticket) =
        fixture_work_for_request("https://example.invalid/original");
    let (handle, _receiver) = handle();
    handle.shared.lock().unwrap().works.push(original_work);
    let payload = Arc::new(AtomicBool::new(false));
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = handle.issue_media_replay(stream, &Panics(payload.clone()));
    }))
    .unwrap_err();
    assert!(Arc::ptr_eq(
        caught.downcast_ref::<Arc<AtomicBool>>().unwrap(),
        &payload
    ));
    assert!(!handle.shared.is_poisoned());
    assert_eq!(handle.state().unwrap().works.len(), 1);
    assert!(handle.cancel(&ticket).is_ok());
    assert!(handle.poll(&ticket).is_ok());
}

fn handle() -> (Handle, std::sync::mpsc::Receiver<Command>) {
    let (commands, receiver) = mpsc::sync_channel(MAX_COMMANDS);
    let shared = Arc::new(Mutex::new(Shared {
        works: vec![],
        next_generation: 1,
        host: BrowserHostSnapshot {
            state: BrowserHostState::Ready,
            worker_exited: false,
            failure: None,
        },
    }));
    (
        Handle {
            shared,
            commands,
            shutdown: Arc::new(AtomicBool::new(false)),
            session_id: "fixture-browser".into(),
        },
        receiver,
    )
}
fn request() -> ValidatedBrowserRequest {
    ValidatedBrowserRequest::new(BrowserRequest {
        request_id: "fixture-request".into(),
        operation: BrowserOperation::ReadCatalog,
        url: "https://catalog.example.test/방영중".into(),
        timeout_ms: DecimalU64(1000),
        blocking_enabled: true,
    })
    .unwrap()
}

#[test]
fn queue_full_does_not_admit_a_ticket_or_consume_a_generation() {
    let (handle, _receiver) = handle();
    let ticket = BrowserTicket::new("fixture-browser".into(), "old".into(), 1).unwrap();
    for _ in 0..MAX_COMMANDS {
        handle.send(Command::Cancel(ticket.clone())).unwrap();
    }
    assert_eq!(
        handle.try_start(request()).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    let state = handle.state().unwrap();
    assert!(state.works.is_empty());
    assert_eq!(state.next_generation, 1);
}

#[test]
fn cancellation_queue_failure_preserves_current_work_state() {
    let (handle, _receiver) = handle();
    let ticket = handle.try_start(request()).unwrap();
    for _ in 1..MAX_COMMANDS {
        handle.send(Command::Cancel(ticket.clone())).unwrap();
    }
    assert_eq!(
        handle.cancel(&ticket).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    assert_eq!(
        handle.poll(&ticket).unwrap().state,
        BrowserWorkState::Queued
    );
    assert!(handle.release(&ticket).is_err());
}
#[test]
fn terminal_work_cannot_release_before_original_native_capture_observation() {
    let (handle, receiver) = handle();
    let ticket = handle.try_start(request()).unwrap();
    let _ = receiver.try_recv().unwrap();
    {
        let mut state = handle.state().unwrap();
        let original = work(&mut state, &ticket).unwrap();
        assert!(original.record_native_capture_released().is_err());
        original.cancel();
        original.apply(&ticket, BrowserEvent::Cancelled);
    }
    assert_eq!(
        handle.release(&ticket).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    assert_eq!(
        handle.poll(&ticket).unwrap().state,
        BrowserWorkState::Cancelled
    );
    {
        let mut state = handle.state().unwrap();
        work(&mut state, &ticket)
            .unwrap()
            .record_native_capture_released()
            .unwrap();
    }
    handle.release(&ticket).unwrap();
    assert!(handle.poll(&ticket).is_err());
}

#[test]
fn activation_queue_failure_does_not_consume_the_once_only_admission() {
    let (handle, receiver) = handle();
    let request = ValidatedBrowserRequest::new(BrowserRequest {
        request_id: "activate-queue".into(),
        operation: BrowserOperation::ResolveEpisode,
        url: "https://catalog.example.test/e/fixture".into(),
        timeout_ms: DecimalU64(1000),
        blocking_enabled: true,
    })
    .unwrap();
    let ticket = handle.try_start(request).unwrap();
    let _ = receiver.try_recv().unwrap();
    {
        let mut state = handle.state().unwrap();
        let work = work(&mut state, &ticket).unwrap();
        work.apply(
            &ticket,
            BrowserEvent::Started {
                engine_version: "fixture".into(),
                blocking_installed: true,
            },
        );
        work.apply(
            &ticket,
            BrowserEvent::Navigated {
                url: "https://catalog.example.test/e/fixture".into(),
                document_generation: 1,
            },
        );
        work.apply(
            &ticket,
            BrowserEvent::NavigationIdentity {
                document_generation: 1,
                native_navigation_id: 42,
            },
        );
        work.apply(
            &ticket,
            BrowserEvent::NavigationCompleted(MainNavigationDiagnostic {
                observed_unix_ms: DecimalU64(1),
                document_generation: DecimalU64(1),
                native_navigation_id: DecimalU64(42),
                http_status_code: Some(200),
                is_success: true,
                web_error_status: 0,
            }),
        );
        work.apply(
            &ticket,
            BrowserEvent::DocumentReady(BrowserDocument {
                kind: CaptureKind::DomSnapshot,
                final_url: "https://catalog.example.test/e/fixture".into(),
                document_generation: DecimalU64(1),
                observed_unix_ms: DecimalU64(1),
                body: Arc::from(b"fixture".as_slice()),
                response: None,
            }),
        );
    }
    for _ in 0..MAX_COMMANDS {
        handle.send(Command::Cancel(ticket.clone())).unwrap();
    }
    assert_eq!(
        handle.activate_episode(&ticket, 1).unwrap_err().code,
        ErrorCode::ResourceExhausted
    );
    assert!(handle.poll(&ticket).unwrap().activation.is_none());
    assert!(matches!(receiver.try_recv().unwrap(), Command::Cancel(_)));
    handle.activate_episode(&ticket, 1).unwrap();
    assert_eq!(
        handle.poll(&ticket).unwrap().activation.unwrap().status,
        EpisodeActivationStatus::Enqueued
    );
}

#[test]
fn disconnected_worker_is_not_reported_as_temporary_queue_pressure() {
    let (handle, receiver) = handle();
    drop(receiver);
    assert_eq!(
        handle.try_start(request()).unwrap_err().code,
        ErrorCode::EngineFailed
    );
    assert!(handle.state().unwrap().works.is_empty());
}

#[test]
fn lock_contention_and_poison_are_distinct() {
    let (handle, _receiver) = handle();
    let guard = handle.shared.lock().unwrap();
    assert!(matches!(
        handle.state(),
        Err(ApiError {
            code: ErrorCode::ResourceExhausted,
            ..
        })
    ));
    drop(guard);
    let shared = handle.shared.clone();
    assert!(
        std::thread::spawn(move || {
            let _guard = shared.lock().unwrap();
            panic!("synthetic browser lock poison");
        })
        .join()
        .is_err()
    );
    assert!(matches!(
        handle.state(),
        Err(ApiError {
            code: ErrorCode::Internal,
            ..
        })
    ));
}

#[test]
fn truncated_or_multibyte_over_limit_dom_never_becomes_a_document() {
    let url = "https://catalog.example.test/방영중";
    for (html, truncated) in [
        ("partial".to_owned(), true),
        ("한".repeat(MAX_CAPTURE_BYTES / 3 + 1), false),
    ] {
        let json = serde_json::to_string(&(url, html, truncated)).unwrap();
        assert!(matches!(
            worker::decode_dom(&json, url.into(), 1),
            Err(ApiError {
                code: ErrorCode::ResourceExhausted,
                ..
            })
        ));
    }
    let json = serde_json::to_string(&(url, "x".repeat(MAX_CAPTURE_BYTES), false)).unwrap();
    assert_eq!(
        worker::decode_dom(&json, url.into(), 1)
            .unwrap()
            .unwrap()
            .body
            .len(),
        MAX_CAPTURE_BYTES
    );
}
