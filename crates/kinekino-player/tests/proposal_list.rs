#![cfg(windows)]
//! Written acceptance cases. Actual CLI cases require a separate allocated gate.
use kinekino_player::application::api::ProposalClient;
#[path = "support/proposal_list_fixture.rs"]
mod fixture;
#[path = "../src/infrastructure/immutable_storage_paths.rs"]
pub mod immutable_storage_paths;
#[path = "support/owned_process_evidence.rs"]
pub mod owned_process_evidence;
use fixture::{CliFixture, StoreFixture};
use kinekino_player::{
    application::{
        api::{ApplicationClient, ErrorCode, Request, RequestContext},
        proposals::{ProposalStore, list::*},
    },
    domain::{api_values::DecimalU64, proposals::ProposalTask},
};
use serde_json::json;
fn hash(bytes: &[u8]) -> String {
    kinekino_player::infrastructure::content_digest::sha256(
        &mut std::io::Cursor::new(bytes),
        bytes.len() as u64,
    )
    .unwrap()
    .1
}
#[test]
fn strict_request_rejects_null_unknown_limit_and_cursor_before_store_read() {
    kinekino_platform::install();
    for value in [
        json!({"task":null}),
        json!({"cursor":null}),
        json!({"limit":0}),
        json!({"limit":201}),
        json!({"task":"other"}),
        json!({"cursor":""}),
        json!({"path":"C:/outside"}),
    ] {
        assert_eq!(
            ListProposals::decode(value).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(ListProposals::decode(json!({})).unwrap().page_limit(), 50);
    assert_eq!(
        ListProposals::decode(json!({"limit":200}))
            .unwrap()
            .page_limit(),
        200
    );
}
#[test]
fn ordinary_store_lists_sorted_filtered_repair_metadata_without_origin_grant() {
    kinekino_platform::install();
    let f = StoreFixture::new();
    assert!(f.page(json!({})).rows.is_empty());
    f.seed("z-highlight", ProposalTask::Highlights, false);
    f.seed("a-card", ProposalTask::Cards, false);
    f.seed("m-repair", ProposalTask::Correction, true);
    let page = f.page(json!({}));
    assert_eq!(
        page.rows
            .iter()
            .map(|r| r.proposal_id.as_str())
            .collect::<Vec<_>>(),
        vec!["a-card", "m-repair", "z-highlight"]
    );
    assert_eq!(page.rows[1].typed_item_count, 0);
    assert_eq!(page.rows[1].draft_revision, DecimalU64(1));
    assert!(page.next_cursor.is_none());
    assert!(!f.origin_path().exists());
    let value = serde_json::to_value(&page).unwrap();
    assert_eq!(value["current_evidence"], "not_checked");
    assert_eq!(value["selection_authority"], "none");
    assert!(value["rows"][0].get("draft").is_none());
    assert!(value["rows"][0].get("origin").is_none());
    assert_eq!(
        f.page(json!({"task":"highlights"})).rows[0].proposal_id,
        "z-highlight"
    );
}
#[test]
fn continuation_binds_query_dataset_and_full_snapshot_including_filtered_out_mutation() {
    kinekino_platform::install();
    let f = StoreFixture::new();
    for id in ["a", "b", "c"] {
        f.seed(id, ProposalTask::Cards, false)
    }
    f.seed("z", ProposalTask::Highlights, false);
    let first = f.page(json!({"limit":1,"task":"cards"}));
    let cursor = first.next_cursor.unwrap();
    assert_eq!(
        f.page(json!({"limit":1,"task":"cards","cursor":cursor}))
            .rows[0]
            .proposal_id,
        "b"
    );
    assert_eq!(
        f.request(json!({"limit":2,"task":"cards","cursor":cursor}))
            .error
            .unwrap()
            .code,
        ErrorCode::InvalidArgument
    );
    let foreign = StoreFixture::new();
    foreign.seed("a", ProposalTask::Cards, false);
    assert_eq!(
        foreign
            .request(json!({"limit":1,"task":"cards","cursor":cursor}))
            .error
            .unwrap()
            .code,
        ErrorCode::SourceExpired
    );
    f.edit("z");
    assert_eq!(
        f.request(json!({"limit":1,"task":"cards","cursor":cursor}))
            .error
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
}
#[test]
fn corruption_outside_filter_is_not_skipped_and_writer_is_released_on_error() {
    kinekino_platform::install();
    let f = StoreFixture::new();
    f.seed("a", ProposalTask::Cards, false);
    f.seed("z", ProposalTask::Highlights, false);
    let revision = f.root.join("p-z/r-1");
    let file = std::fs::read_dir(&revision)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let original = std::fs::read(&file).unwrap();
    std::fs::write(&file, b"{").unwrap();
    assert_eq!(
        f.request(json!({"task":"cards"})).error.unwrap().code,
        ErrorCode::NeedsInput
    );
    // The failed scan must release the original writer, not leave a new reader owner.
    std::fs::write(&file, &original).unwrap();
    assert_eq!(f.page(json!({})).rows.len(), 2);
}
#[test]
fn writer_busy_fails_without_wait_and_same_store_recovers_after_release() {
    kinekino_platform::install();
    let f = StoreFixture::new();
    f.seed("a", ProposalTask::Cards, false);
    let held = immutable_storage_paths::lock(&f.root.join("writer.lock")).unwrap();
    assert_eq!(
        f.request(json!({})).error.unwrap().code,
        ErrorCode::RevisionConflict
    );
    drop(held);
    assert_eq!(f.page(json!({})).rows.len(), 1);
}
#[test]
fn malformed_cursor_is_rejected_before_original_writer_admission() {
    kinekino_platform::install();
    let f = StoreFixture::new();
    let held = immutable_storage_paths::lock(&f.root.join("writer.lock")).unwrap();
    assert_eq!(
        f.request(json!({"cursor":"!not-base64"}))
            .error
            .unwrap()
            .code,
        ErrorCode::InvalidArgument
    );
    drop(held);
    assert!(f.page(json!({})).rows.is_empty());
}
#[test]
fn create_and_accept_each_invalidate_an_earlier_page() {
    kinekino_platform::install();
    let f = StoreFixture::new();
    for id in ["a", "b"] {
        f.seed(id, ProposalTask::Cards, false)
    }
    let old = f.page(json!({"limit":1})).next_cursor.unwrap();
    f.seed("c", ProposalTask::Cards, false);
    assert_eq!(
        f.request(json!({"limit":1,"cursor":old}))
            .error
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
    let old = f.page(json!({"limit":1})).next_cursor.unwrap();
    f.client()
        .accept_proposal(
            RequestContext {
                expected_revision: Some("1".into()),
                ..RequestContext::default()
            },
            kinekino_player::application::proposals::AcceptProposal {
                proposal_id: "b".into(),
                target_source_revision: DecimalU64(1),
            },
        )
        .unwrap();
    assert_eq!(
        f.request(json!({"limit":1,"cursor":old}))
            .error
            .unwrap()
            .code,
        ErrorCode::RevisionConflict
    );
}
#[test]
fn unsupported_optional_list_does_not_remove_existing_inspect_support() {
    kinekino_platform::install();
    struct Legacy;
    impl ProposalStore for Legacy {
        fn load(
            &self,
            _: &str,
        ) -> Result<
            kinekino_player::domain::proposals::StoredProposal,
            kinekino_player::application::api::ApiError,
        > {
            Err(kinekino_player::application::api::ApiError::new(
                ErrorCode::NotFound,
                "Absent",
            ))
        }
        fn publish(
            &self,
            _: Option<u64>,
            _: &kinekino_player::domain::proposals::StoredProposal,
        ) -> Result<(), kinekino_player::application::api::ApiError> {
            unreachable!()
        }
    }
    let sdk = kinekino_player::application::api::PlayerSdk::new(
        kinekino_player::infrastructure::subtitle_files::SubtitleFiles,
    )
    .with_proposal_store(Legacy);
    let caps = sdk.execute(Request::new("capabilities")).data.unwrap();
    assert_eq!(caps["proposals"]["backend_available"], true);
    assert_eq!(caps["proposals"]["listing"]["available"], false);
    assert_eq!(
        sdk.list_proposals(RequestContext::default(), ListProposals::default())
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    let mut request = Request::new("schema");
    request.payload = json!({"command":"proposals.inspect"});
    assert_eq!(sdk.execute(request).data.unwrap()["status"], "supported");
}
#[test]
fn queued_cancel_and_running_request_keep_original_terminal_ack_and_join() {
    kinekino_platform::install();
    use kinekino_player::application::{
        api::{ApiError, PlayerSdk, Response},
        sdk_execution::*,
    };
    use kinekino_player::infrastructure::subtitle_files::SubtitleFiles;
    use std::{
        panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
        sync::{Arc, Condvar, Mutex, mpsc},
        time::{Duration, Instant},
    };
    struct Client {
        inner: PlayerSdk<SubtitleFiles>,
        gate: (Mutex<bool>, Condvar),
        entered: mpsc::Sender<()>,
    }
    impl Client {
        fn release(&self) {
            if let Ok(mut ready) = self.gate.0.lock() {
                *ready = true;
                self.gate.1.notify_all();
            }
        }
    }
    impl ApplicationClient for Client {
        fn execute(&self, request: Request) -> Response {
            self.entered.send(()).unwrap();
            let mut ready = self.gate.0.lock().unwrap();
            while !*ready {
                ready = self.gate.1.wait(ready).unwrap();
            }
            drop(ready);
            self.inner.execute(request)
        }
    }
    struct Notify;
    impl CompletionNotifier for Notify {
        fn notify(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }
    let f = StoreFixture::new();
    let (tx, rx) = mpsc::channel();
    let client = Arc::new(Client {
        inner: f.client(),
        gate: (Mutex::new(false), Condvar::new()),
        entered: tx,
    });
    let owner = ExecutionOwner::start(
        client.clone(),
        ExecutionBudget {
            max_operations: 2,
            max_request_bytes: 4096,
            max_total_request_bytes: 16384,
            max_terminal_bytes: 512 * 1024,
            max_total_terminal_bytes: 1024 * 1024,
        },
        Arc::new(Notify),
    )
    .unwrap();
    let handle = owner.handle();
    let action = catch_unwind(AssertUnwindSafe(|| {
        let running = handle.submit(Request::new("proposals.list")).unwrap();
        rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let queued = handle.submit(Request::new("proposals.list")).unwrap();
        assert_eq!(
            handle.cancel(&queued).unwrap(),
            CancelDisposition::QueuedCancelled
        );
        assert_eq!(
            handle.cancel(&running).unwrap(),
            CancelDisposition::Requested
        );
        assert!(!matches!(
            owner.poll_close().unwrap(),
            ExecutionCloseProgress::Closed(_)
        ));
        client.release();
        let cutoff = Instant::now() + Duration::from_secs(5);
        let terminal = loop {
            if let ExecutionSnapshot::Terminal(terminal) = handle.peek(&running).unwrap() {
                break terminal;
            }
            assert!(Instant::now() < cutoff);
            std::thread::sleep(Duration::from_millis(2));
        };
        assert!(terminal.cancellation_requested);
        assert!(!terminal.queued_cancelled);
        assert!(terminal.response.ok);
        let ExecutionSnapshot::Terminal(queued_terminal) = handle.peek(&queued).unwrap() else {
            panic!("Queued terminal required")
        };
        assert!(queued_terminal.queued_cancelled);
        assert_eq!(
            queued_terminal.response.error.as_ref().unwrap().code,
            ErrorCode::Cancelled
        );
        assert!(
            rx.try_recv().is_err(),
            "Queued request must not enter the SDK scan"
        );
        assert!(
            !matches!(
                owner.poll_close().unwrap(),
                ExecutionCloseProgress::Closed(_)
            ),
            "Original terminals still require ACK"
        );
    }));
    // Even a failed assertion releases the SAME barrier and drains original IDs.
    client.release();
    let cleanup = catch_unwind(AssertUnwindSafe(|| {
        let cutoff = Instant::now() + Duration::from_secs(5);
        for id in handle.retained_operations().unwrap() {
            loop {
                if matches!(handle.peek(&id).unwrap(), ExecutionSnapshot::Terminal(_)) {
                    break;
                }
                assert!(Instant::now() < cutoff);
                std::thread::sleep(Duration::from_millis(2));
            }
            handle.acknowledge(&id).unwrap();
        }
        loop {
            if matches!(
                owner.poll_close().unwrap(),
                ExecutionCloseProgress::Closed(_)
            ) {
                break;
            }
            assert!(Instant::now() < cutoff);
            std::thread::sleep(Duration::from_millis(2));
        }
        drop(immutable_storage_paths::lock(&f.root.join("writer.lock")).unwrap());
    }));
    if cleanup.is_err() {
        Box::leak(Box::new(owner));
    }
    if let Err(error) = action {
        resume_unwind(error)
    }
    if let Err(error) = cleanup {
        resume_unwind(error)
    }
}

#[test]
#[ignore = "Requires central actual CLI slot, producer pins and fresh evidence"]
fn actual_cli_pages_across_processes_and_exact_inspect_match_typed_client() {
    kinekino_platform::install();
    let f = CliFixture::new("pages");
    for id in ["c", "a", "b"] {
        f.store.seed(id, ProposalTask::Cards, false)
    }
    let first = f.call("proposals.list", json!({"limit":2}));
    assert_eq!(first["ok"], true);
    let typed = f.store.page(json!({"limit":2}));
    assert_eq!(first["data"], serde_json::to_value(typed).unwrap());
    let next = f.call(
        "proposals.list",
        json!({"limit":2,"cursor":first["data"]["next_cursor"]}),
    );
    assert_eq!(next["data"]["rows"][0]["proposal_id"], "c");
    assert!(next["data"]["next_cursor"].is_null());
    let inspected = f.call("proposals.inspect", json!({"proposal_id":"c"}));
    assert_eq!(
        inspected["data"]["document"]["draft_revision"],
        next["data"]["rows"][0]["draft_revision"]
    );
    assert!(!f.store.origin_path().exists());
}
#[test]
#[ignore = "Requires central actual CLI slot, producer pins and fresh evidence"]
fn actual_cli_mutation_and_foreign_dataset_reject_original_cursor() {
    kinekino_platform::install();
    let f = CliFixture::new("stale");
    for id in ["a", "b"] {
        f.store.seed(id, ProposalTask::Cards, false)
    }
    let first = f.call("proposals.list", json!({"limit":1}));
    let cursor = first["data"]["next_cursor"].clone();
    let other = StoreFixture::at(f.evidence.join("other"));
    other.seed("a", ProposalTask::Cards, false);
    assert_eq!(
        f.call_at(
            "proposals.list",
            json!({"limit":1,"cursor":cursor}),
            &other.root
        )["error"]["code"],
        "source_expired"
    );
    f.store.edit("b");
    assert_eq!(
        f.call("proposals.list", json!({"limit":1,"cursor":cursor}))["error"]["code"],
        "revision_conflict"
    );
}
#[test]
#[ignore = "Requires central actual CLI slot, producer pins and fresh evidence"]
fn actual_cli_empty_busy_corrupt_and_missing_root_preserve_failure_contracts() {
    kinekino_platform::install();
    let f = CliFixture::new("errors");
    assert_eq!(
        f.call("proposals.list", json!({}))["data"]["rows"],
        json!([])
    );
    f.store.seed("a", ProposalTask::Cards, false);
    let guard = immutable_storage_paths::lock(&f.store.root.join("writer.lock")).unwrap();
    assert_eq!(
        f.call("proposals.list", json!({}))["error"]["code"],
        "revision_conflict"
    );
    drop(guard);
    assert_eq!(f.call("proposals.list", json!({}))["ok"], true);
    std::fs::create_dir(f.store.root.join("unexpected")).unwrap();
    assert_eq!(
        f.call("proposals.list", json!({}))["error"]["code"],
        "needs_input"
    );
    assert!(f.store.root.join("unexpected").exists());
    assert_eq!(
        f.call_at("proposals.list", json!({}), &f.evidence.join("absent"))["error"]["code"],
        "not_found"
    );
    assert!(!f.evidence.join("absent").exists());
}
#[test]
#[ignore = "Requires central actual CLI slot and allocated 4096-entry filesystem boundary fixture"]
fn actual_cli_root_scan_limit_is_not_just_a_page_limit() {
    kinekino_platform::install();
    let f = CliFixture::new("root-bound");
    // The existing writer entry makes these 4096 directories exceed4096 root entries.
    drop(immutable_storage_paths::lock(&f.store.root.join("writer.lock")).unwrap());
    for i in 0..4096 {
        std::fs::create_dir(f.store.root.join(format!("p-item-{i:04}"))).unwrap();
    }
    assert_eq!(
        f.call("proposals.list", json!({"limit":1}))["error"]["code"],
        "resource_exhausted"
    );
}
#[test]
#[ignore = "Requires central filesystem slot and supported Windows reparse fixture privileges"]
fn reparse_proposal_is_denied_without_reading_or_modifying_external_source() {
    kinekino_platform::install();
    let f = StoreFixture::new();
    let outside = f.root.with_file_name(format!(
        "{}-outside",
        f.root.file_name().unwrap().to_string_lossy()
    ));
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("sentinel"), b"outside unchanged").unwrap();
    std::os::windows::fs::symlink_dir(&outside, f.root.join("p-escape")).unwrap();
    assert_eq!(
        f.request(json!({})).error.unwrap().code,
        ErrorCode::PermissionDenied
    );
    assert_eq!(
        std::fs::read(outside.join("sentinel")).unwrap(),
        b"outside unchanged"
    );
}
mod infrastructure {
    pub(crate) use kinekino_player::infrastructure::os_policy;
}
