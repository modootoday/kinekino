//! Explicit launcher assembly only; no OS window, default-player or video acceptance.
use super::*;
use crate::application::api::ProposalClient;
use crate::{
    application::{
        api::{ApiError, PlayerSdk, RequestContext},
        launch::native_proposal::{
            NATIVE_PROPOSAL_SCHEMA_VERSION, NativeProposalSelection, NativeProposalTarget,
        },
        proposals::CreateProposal,
        sdk_execution::{CompletionNotifier, ExecutionCloseProgress, ExecutionOwner},
    },
    domain::{api_values::DecimalU64, product::PRODUCT, proposals::*},
    infrastructure::proposals::ProposalFiles,
};
use meari_native::infrastructure::native_host::{HostRetry, HostTurnReason};
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

struct Signal(mpsc::Sender<()>);
impl CompletionNotifier for Signal {
    fn notify(&self) -> Result<(), ApiError> {
        let _ = self.0.send(());
        Ok(())
    }
}

fn execution(client: ApplicationClientHandle) -> (ExecutionOwner, mpsc::Receiver<()>) {
    let (sender, receiver) = mpsc::channel();
    let owner = ExecutionOwner::start(
        client,
        ExecutionBudget {
            max_operations: 8,
            max_request_bytes: 256 * 1024,
            max_total_request_bytes: 8 * 256 * 1024,
            max_terminal_bytes: 256 * 1024,
            max_total_terminal_bytes: 8 * 256 * 1024,
        },
        Arc::new(Signal(sender)),
    )
    .unwrap();
    (owner, receiver)
}

fn config() -> ProductWindowConfig {
    let styles = crate::infrastructure::meari::product_styles::prepare_product_styles().unwrap();
    ProductWindowConfig {
        labels: ProductWindowLabels::korean(),
        visuals: ProductWindowVisuals {
            css: styles.css,
            assets: Some(styles.assets),
        },
        viewport: Viewport {
            width: 1180,
            height: 760,
            dark: true,
        },
        limits: HostScheduleLimits::new(Duration::from_millis(2), Duration::from_millis(250), 1024)
            .unwrap(),
        worker_exit_delay: Duration::from_millis(5),
        observer: Box::new(|_| panic!("Launcher assembly test must not open an OS window")),
    }
}

fn fixture() -> (
    PathBuf,
    ApplicationClientHandle,
    NativeProposalLaunchRequest,
) {
    let root = std::env::temp_dir().join(format!(
        "kinekino-launch-frame-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let client: ApplicationClientHandle =
        Arc::new(PlayerSdk::default().with_proposal_store(ProposalFiles::open(&root).unwrap()));
    let payload = ProposalPayload::Cards {
        cards: vec![Card {
            front: "명시 카드 日本語".into(),
            back: "명시 답안".into(),
            source_refs: vec!["cue-a".into()],
        }],
    };
    let document = client
        .create_proposal(
            RequestContext::default(),
            CreateProposal {
                proposal_id: "explicit-launch-proposal".into(),
                source: SourceSnapshot::Cue(CueSourceSnapshot {
                    source_id: "explicit-launch-source".into(),
                    source_revision: DecimalU64(7),
                    source_sha256: "a".repeat(64),
                    cues: vec![SourceCue {
                        cue_id: "cue-a".into(),
                        text: "명시 원문".into(),
                        start_us: None,
                        end_us: None,
                    }],
                }),
                generated: GeneratedResult {
                    task: payload.task(),
                    raw_text: serde_json::to_string(&payload).unwrap(),
                },
            },
        )
        .unwrap();
    let request = NativeProposalLaunchRequest {
        schema_version: NATIVE_PROPOSAL_SCHEMA_VERSION.into(),
        context: RequestContext::default(),
        selection: NativeProposalSelection {
            proposal_id: document.proposal_id,
            document_revision: document.draft_revision,
            source_id: document.source.source_id().into(),
            source_revision: document.source.source_revision(),
            source_sha256: document.source.source_sha256().into(),
            target: NativeProposalTarget::Card { index: 0 },
        },
    };
    (root, client, request)
}

fn cleanup(owners: ProductWindowOwners, expected: NativeCleanupState) {
    let mut cleanup = OwnedProductCleanup::new(
        owners,
        CleanupLimits::new(Duration::from_secs(20), 10000, Duration::from_millis(2)).unwrap(),
    );
    loop {
        match cleanup.poll() {
            CleanupStatus::SdkQuiescent { native } => {
                assert_eq!(native, expected);
                break;
            }
            CleanupStatus::Pending(HostRetry::After(delay)) => std::thread::sleep(delay),
            other => panic!("Launcher owner did not clean up: {other:?}"),
        }
    }
    assert!(
        matches!(cleanup.evidence.last_sdk, Some(ExecutionCloseProgress::Closed(receipt)) if receipt.worker_stopped && receipt.unacknowledged == 0)
    );
}

#[test]
fn explicit_launcher_mounts_original_frame_inspector_and_selected_sdk_editor() {
    let (root, client, request) = fixture();
    let (owner, receiver) = execution(client.clone());
    let mut startup = prepare_launch_window(client, owner, request, config())
        .unwrap_or_else(|_| panic!("Explicit launcher assembly failed"));
    let deadline = Instant::now() + Duration::from_secs(20);
    let runtime = loop {
        if let Some(runtime) = startup.prepare_document().unwrap() {
            break runtime;
        }
        assert!(
            startup.mount_failure.is_none(),
            "Original frame mount failed"
        );
        assert!(startup.bridge_failure.is_none(), "Original bridge failed");
        assert!(
            startup.initial_failure.is_none(),
            "Explicit SDK inspect failed"
        );
        assert!(Instant::now() < deadline);
        let _ = receiver.recv_timeout(Duration::from_millis(50));
    };
    runtime
        .borrow_mut()
        .after_dispatch(HostTurnReason::Completion)
        .unwrap();
    let handle = runtime.borrow().document();
    let text = handle
        .try_with(|document| {
            document
                .elements()
                .iter()
                .flat_map(|element| &element.text_runs)
                .map(|run| run.text.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap();
    assert!(text.contains(PRODUCT.display_name));
    assert!(text.contains("스튜디오"));
    assert!(text.contains("명시 원문"));
    assert!(
        !runtime
            .borrow()
            .host()
            .frame_projection()
            .unwrap()
            .inspector_state()
            .unwrap()
            .projection()
            .drawer_open
    );
    let bindings = handle
        .try_with(|document| document.module_projection().unwrap().callback_bindings())
        .unwrap();
    assert!(
        bindings
            .values()
            .any(|callback| callback.reference.binding == "outputReviewOnInspect")
    );
    drop(bindings);
    drop(handle);
    drop(runtime);
    cleanup(
        ProductWindowOwners::Headless(Box::new(startup)),
        NativeCleanupState::HeadlessClosed,
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn explicit_launcher_configuration_failure_retains_original_request_and_execution_owner() {
    let (root, client, request) = fixture();
    let original = serde_json::to_value(&request).unwrap();
    let (owner, _receiver) = execution(client.clone());
    let mut invalid = config();
    invalid.worker_exit_delay = Duration::ZERO;
    let failure = match prepare_launch_window(client, owner, request, invalid) {
        Err(failure) => failure,
        Ok(_) => panic!("Invalid launcher configuration was accepted"),
    };
    let ProductWindowBuildFailure::Input(input) = &failure else {
        panic!("Original input failure was not retained");
    };
    assert_eq!(serde_json::to_value(&input.request).unwrap(), original);
    assert_eq!(input.config.worker_exit_delay, Duration::ZERO);
    cleanup(
        ProductWindowOwners::BuildFailure(Box::new(failure)),
        NativeCleanupState::NoDocument,
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "Explicit one-loop Native startup authority slot; no OS view is created"]
fn sdk_start_failure_reuses_loop_and_unrun_ui_failure_keeps_authority() {
    let client: ApplicationClientHandle = Arc::new(PlayerSdk::default());
    let limits =
        HostScheduleLimits::new(Duration::from_millis(2), Duration::from_millis(250), 100).unwrap();
    let mut budget = ExecutionBudget {
        max_operations: 8,
        max_request_bytes: 262144,
        max_total_request_bytes: 8 * 262144,
        max_terminal_bytes: 262144,
        max_total_terminal_bytes: 8 * 262144,
    };
    budget.max_operations = 0;
    let failure = prepare_proposal_execution(client.clone(), budget, limits)
        .err()
        .expect("SDK budget rejection");
    assert!(matches!(
        failure.cause,
        super::super::proposal_window_startup::ProposalExecutionStartError::Sdk(_)
    ));
    assert!(failure.loop_owner.is_some() && failure.startup.is_some());
    assert!(Arc::ptr_eq(&failure.client, &client));
    budget.max_operations = 8;
    let (owner, startup, execution) = failure.retry_execution(budget).unwrap().into_parts();
    let request = NativeProposalLaunchRequest {
        schema_version: "1".into(),
        context: Default::default(),
        selection: NativeProposalSelection {
            proposal_id: "p".into(),
            document_revision: crate::domain::api_values::DecimalU64(1),
            source_id: "s".into(),
            source_revision: crate::domain::api_values::DecimalU64(1),
            source_sha256: "a".repeat(64),
            target: crate::application::launch::native_proposal::NativeProposalTarget::Card {
                index: 1,
            },
        },
    };
    let failure = prepare_launch_window(
        client,
        execution,
        request,
        ProductWindowConfig {
            labels: ProductWindowLabels::korean(),
            visuals: ProductWindowVisuals {
                css: String::new(),
                assets: None,
            },
            viewport: Viewport {
                width: 1180,
                height: 760,
                dark: true,
            },
            limits,
            worker_exit_delay: Duration::ZERO,
            observer: Box::new(|_| Ok(())),
        },
    )
    .err()
    .expect("invalid trusted configuration");
    let outcome = finish_unrun_product(failure, owner, startup, Vec::new());
    assert!(!outcome.succeeded());
    assert_eq!(outcome.report()["unrun_native_retained"], true);
    let ProductNativeOutcome::Finished {
        cleanup,
        unrun_native,
        ..
    } = &outcome
    else {
        panic!("retained unrun outcome")
    };
    assert!(unrun_native.is_some());
    assert_eq!(
        cleanup.status,
        CleanupStatus::SdkQuiescent {
            native: NativeCleanupState::NoDocument
        }
    );
    let ProductWindowOwners::BuildFailure(failure) = &cleanup.owners else {
        panic!("original failure")
    };
    let ProductWindowBuildFailure::Input(failure) = failure.as_ref() else {
        panic!("original input")
    };
    assert!(
        failure
            .error
            .message
            .contains("outside Native schedule limits")
    );
}
