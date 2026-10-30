//! Real product module/callback/SDK assembly without a window, renderer or synthetic frame success.
use crate::application::api::ProposalClient;
#[path = "product_surface/return_viewing.rs"]
mod return_viewing;
#[path = "product_surface/slot_lifecycle.rs"]
mod slot_lifecycle;
use super::*;
use crate::assembly::meari::player_shell_host::StartRouteObservation;
use crate::presentation::web_ui_adapter::player_shell::document::product_frame::ProductFrameBlueprint;

#[test]
#[ignore = "Assigned original LocalHost slot; actual capture factory/export, no window/render"]
fn capture_caption_factory_and_original_export_reject_invalid_foreign_stale_retired_and_closed_inputs()
-> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        bind_product(host, rig, seed(&rig.client));
        open(host);
        let calls = requests(&rig.requests).len();
        crate::assembly::meari::viewer_host::capture_surface::caption_contract_tests::assert_original_surface(host);
        assert_eq!(requests(&rig.requests).len(), calls);
    })
}

#[test]
#[ignore = "Requires the assigned original LocalHost slot; no window/render"]
fn selected_project_caller_and_parent_archive_support_more_than_eight_edits()
-> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        let project = seed(&rig.client);
        let args = vec![
            "player".into(),
            "--native-viewer".into(),
            format!("--viewer-project={}", project.project.project_id),
            "--viewer-project-revision=1".into(),
        ];
        let selected = crate::assembly::meari::product_viewer_launch::prepare_project(
            crate::application::launch::desktop_launch_plan(&args)
                .unwrap()
                .project,
            rig.client.as_ref(),
            Some(&rig.root.join("projects")),
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            serde_json::to_value(&selected.stored).unwrap(),
            serde_json::to_value(&project).unwrap()
        );
        host.install_capture_evidence(selected.artifacts).unwrap();
        bind_product(host, rig, selected.stored);
        open(host);
        for index in 0..9 {
            text(host, &format!("Persisted edit {index}"));
            click(host, "onSave");
            ready(host);
            click(host, "onCaptureConsume");
            let status = host.captured_evidence_status().unwrap();
            assert_eq!(status.saved, index + 1);
            assert_eq!(status.pending, 0);
            assert!(status.first_error.is_none());
            assert!(
                host.take_captured_surface_acknowledgements()
                    .unwrap()
                    .is_empty()
            );
        }
        let captures = rig.root.join("projects/viewer-execution-evidence/captures");
        assert_eq!(std::fs::read_dir(captures).unwrap().count(), 9);
        let status = host.captured_evidence_status().unwrap();
        let files = crate::infrastructure::catalog::CatalogFiles::open(
            &rig.root.join("projects/viewer-execution-evidence"),
        )
        .unwrap();
        let loaded = crate::application::catalog::CatalogRawSource::load_raw(
            &files,
            &status.last_saved.unwrap().artifact_id,
        )
        .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&loaded.body).unwrap();
        assert_eq!(body["representation"], "native_ui_sdk_execution_diagnostic");
        assert_eq!(
            body["request"]["payload"]["anchor"]["project_revision_sha256"],
            project.revision_sha256
        );
        assert_immutable(rig, &project);
    })
}

struct LostEvidenceReceipt {
    inner: Arc<dyn crate::application::catalog::CatalogArtifacts>,
    first: std::sync::atomic::AtomicBool,
    attempts: std::sync::Mutex<Vec<(Vec<u8>, crate::application::catalog::CaptureMetadata)>>,
}
#[test]
#[ignore = "Requires original LocalHost; parent close and existing artifact-port fault only"]
fn parent_close_retains_acknowledged_original_until_storage_retry_completes()
-> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        let archive = Arc::new(LostEvidenceReceipt {
            inner: crate::infrastructure::meari::capture_evidence::open(&rig.root.join("projects"))
                .unwrap(),
            first: std::sync::atomic::AtomicBool::new(true),
            attempts: std::sync::Mutex::new(Vec::new()),
        });
        host.install_capture_evidence(archive.clone()).unwrap();
        bind_product(host, rig, seed(&rig.client));
        open(host);
        text(host, "Retained through original close");
        click(host, "onSave");
        ready(host);
        // Explicit trusted consumer decision; parent close must finish ACK and persist its carrier.
        host.capture_adapter_mut()
            .unwrap()
            .consume_execution_terminal()
            .unwrap();
        let end = Instant::now() + Duration::from_secs(5);
        loop {
            if host.poll_unrun_shutdown().is_err() {
                break;
            }
            assert!(
                Instant::now() < end,
                "Expected original storage failure before completion"
            );
        }
        assert_eq!(host.captured_evidence_status().unwrap().pending, 1);
        assert_eq!(archive.attempts.lock().unwrap().len(), 1);
        assert!(host.poll_unrun_shutdown().is_err());
        assert_eq!(archive.attempts.lock().unwrap().len(), 1);
        host.retry_captured_evidence().unwrap();
        assert_eq!(host.captured_evidence_status().unwrap().pending, 0);
        assert_eq!(host.captured_evidence_status().unwrap().saved, 1);
        loop {
            if matches!(
                host.poll_unrun_shutdown().unwrap(),
                crate::assembly::meari::viewer_host::ViewerCloseProgress::Complete { .. }
            ) {
                break;
            }
            assert!(
                Instant::now() < end,
                "Same original owner did not complete after publication"
            );
        }
        let attempts = archive.attempts.lock().unwrap().clone();
        assert_eq!(attempts[0].0, attempts[1].0);
    })
}
impl crate::application::catalog::CatalogArtifacts for LostEvidenceReceipt {
    fn save(
        &self,
        bytes: &[u8],
        metadata: &crate::application::catalog::CaptureMetadata,
    ) -> Result<crate::application::catalog::RawArtifact, ApiError> {
        self.attempts
            .lock()
            .unwrap()
            .push((bytes.to_vec(), metadata.clone()));
        let published = self.inner.save(bytes, metadata)?;
        if self.first.swap(false, Ordering::SeqCst) {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Injected loss of evidence publication receipt",
            ));
        }
        Ok(published)
    }
}
#[test]
#[ignore = "Requires the assigned original LocalHost slot; existing artifact-port fault only"]
fn parent_archive_retry_preserves_original_bytes_without_sdk_resubmission()
-> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        let project = seed(&rig.client);
        let archive = Arc::new(LostEvidenceReceipt {
            inner: crate::infrastructure::meari::capture_evidence::open(&rig.root.join("projects"))
                .unwrap(),
            first: std::sync::atomic::AtomicBool::new(true),
            attempts: std::sync::Mutex::new(Vec::new()),
        });
        host.install_capture_evidence(archive.clone()).unwrap();
        bind_product(host, rig, project);
        open(host);
        text(host, "Backend and immutable evidence commit once");
        click(host, "onSave");
        ready(host);
        click(host, "onCaptureConsume");
        let failed = host.captured_evidence_status().unwrap();
        assert_eq!(failed.saved, 0);
        assert_eq!(failed.pending, 1);
        assert!(failed.first_error.is_some());
        let calls = requests(&rig.requests).len();
        for _ in 0..2 {
            host.after_dispatch(turn(host, HostTurnReason::Completion))
                .unwrap();
        }
        assert_eq!(archive.attempts.lock().unwrap().len(), 1);
        assert!(
            !host
                .can_close(
                    &host.document.as_ref().unwrap().stamp().unwrap(),
                    CloseCause::User
                )
                .unwrap()
        );
        click(host, "onCaptureRetry");
        let saved = host.captured_evidence_status().unwrap();
        assert_eq!(saved.saved, 1);
        assert_eq!(saved.pending, 0);
        assert!(saved.first_error.is_some());
        assert_eq!(requests(&rig.requests).len(), calls);
        let attempts = archive.attempts.lock().unwrap().clone();
        assert_eq!(attempts.len(), 2);
        assert_eq!(attempts[0].0, attempts[1].0);
        assert_eq!(
            serde_json::to_value(&attempts[0].1).unwrap(),
            serde_json::to_value(&attempts[1].1).unwrap()
        );
        assert_eq!(
            std::fs::read_dir(rig.root.join("projects/viewer-execution-evidence/captures"))
                .unwrap()
                .count(),
            1
        );
    })
}

#[test]
#[ignore = "Requires the assigned original LocalHost slot; no window/render"]
fn product_legacy_card_choice_stays_disabled_without_sdk_mutation()
-> Result<(), Box<GuardedFailure>> {
    with_rig(|host, rig| {
        let mut legacy = seed(&rig.client).project;
        legacy.project_id = "legacy-product-card".into();
        legacy.schema_version = "1".into();
        legacy.proposals[0].proposal_origin = None;
        let stored = ProjectFiles::open(&rig.root.join("projects"))
            .unwrap()
            .publish(None, &legacy)
            .unwrap();
        bind_product(host, rig, stored);
        let handle = host.document.as_ref().unwrap().clone();
        let path = captured_radio_path(&handle);
        assert!(
            handle
                .try_with(|document| document
                    .elements()
                    .iter()
                    .find(|element| element.path == path)
                    .unwrap()
                    .attributes
                    .contains_key("disabled"))
                .unwrap()
        );
        let before = requests(&rig.requests).len();
        assert!(!handle.dispatch_action(Action::Activate { path }).unwrap());
        let (pending, _) = handle
            .try_reproject(&handle.stamp().unwrap(), |document| {
                Ok::<_, String>(document.drain_callbacks())
            })
            .unwrap();
        assert!(pending.is_empty());
        assert!(host.card_selection_queue().unwrap().borrow().is_empty());
        assert!(host.captured_surface_receipt().is_none());
        assert_eq!(requests(&rig.requests).len(), before);
    })
}

#[test]
#[ignore = "Requires the assigned original LocalHost slot; no window/render"]
fn product_stale_error_preserves_local_text_and_original_capture() -> Result<(), Box<GuardedFailure>>
{
    with_rig(|host, rig| {
        let project = seed(&rig.client);
        bind_product(host, rig, project.clone());
        open(host);
        rig.client
            .edit_captured_proposal(
                RequestContext::default(),
                EditCapturedProposal {
                    anchor: anchor(&project),
                    expected: None,
                    draft: draft("External editor"),
                },
            )
            .unwrap();
        text(host, "Unsaved product text");
        click(host, "onSave");
        ready(host);
        click(host, "onCaptureConsume");
        {
            let capture = host.capture_adapter_mut().unwrap();
            assert_eq!(capture.display().state.document.draft_revision.0, 1);
            assert_eq!(
                capture.display().sdk_error.unwrap().code,
                ErrorCode::RevisionConflict
            );
        }
        let handle = host.document.as_ref().unwrap().clone();
        assert!(
            !host
                .can_close(&handle.stamp().unwrap(), CloseCause::User)
                .unwrap()
        );
        let ProposalPayload::Cards { cards } = host.captured_local_card_draft().unwrap() else {
            panic!("Card buffer");
        };
        assert_eq!(cards[0].front, "Unsaved product text");
        click(host, "onCaptureRevert");
        assert!(
            host.can_close(&handle.stamp().unwrap(), CloseCause::User)
                .unwrap()
        );
        assert_immutable(rig, &project);
    })
}

fn bind_product(
    host: &mut ViewerHost,
    rig: &Rig,
    project: StoredProjectRevision,
) -> Rc<RefCell<StartRouteObservation>> {
    let labels = StartLabels {
        connection: "Not observed".into(),
        time: "Not observed".into(),
    };
    let mut prepared = prepare_shell(
        ShellInputs {
            client: rig.client.clone(),
            host_owner: (),
        },
        labels,
    )
    .unwrap_or_else(|_| panic!("Product source preparation"));
    let callbacks = host.callbacks();
    callbacks
        .borrow_mut()
        .configure_captured_edit_choices(
            project,
            NonZeroU64::new(1).unwrap(),
            std::num::NonZeroUsize::new(2).unwrap(),
        )
        .unwrap();
    let mut props = read_props(&StartLabels {
        connection: "Not observed".into(),
        time: "Not observed".into(),
    });
    props.extend(callbacks.borrow().configured_card_props());
    enable_read_panel(&mut prepared.request.document, props.clone()).unwrap_or_else(|error| {
        match error {
            crate::presentation::web_ui_adapter::player_shell::document::DocumentPreparationError::Source(diagnostic) => panic!("Product read source: {diagnostic:?}"),
            _ => panic!("Fixed product read source must prepare"),
        }
    });
    let (entry, mut inventory) = product_modules_for_panel(BootstrapPanel::Card);
    let provider = meari::parse_module_graph(&entry, &inventory).unwrap();
    let (studio, _) = FrameComposition::new(
        crate::domain::product::PRODUCT.display_name.into(),
        String::new(),
    )
    .prepare(provider, &mut inventory)
    .unwrap();
    let frame = ProductFrameBlueprint::prepare(
        prepared.request.document.graph,
        prepared.request.document.inventory,
        studio,
        inventory,
    )
    .unwrap();
    let program = frame
        .mount_viewing(
            props.clone(),
            "guarded-product-surface".into(),
            1,
            NonZeroU64::new(1).unwrap(),
        )
        .unwrap();
    let document = InteractiveDocument::from_module_program_with_callbacks(
        program,
        "",
        Viewport {
            width: 1280,
            height: 720,
            dark: true,
        },
        None,
        1,
        move |scope| callbacks.borrow_mut().export(scope).map_err(|e| e.message),
    )
    .unwrap();
    let handle = NativeDocumentHandle::new(document, 1).unwrap();
    host.bind_product_document(handle, props, frame);
    // A cancelled initial ticket is deliberately unobserved: this test cannot grant frame success.
    let mut state = ShellState::new();
    let ticket = state
        .request_route(
            &RouteCatalog::bundled().unwrap(),
            "view.start",
            Default::default(),
        )
        .unwrap();
    state.cancel_route(&ticket).unwrap();
    let observer = Rc::new(RefCell::new(StartRouteObservation::new(state, ticket)));
    host.bind_route_observation(observer.clone());
    observer
}
fn turn(host: &ViewerHost, reason: HostTurnReason) -> HostTurn {
    HostTurn {
        stamp: host.document.as_ref().unwrap().stamp().unwrap(),
        reason,
        accepted_callbacks: 0,
        rejected_callbacks: 0,
        remaining_callbacks: 0,
    }
}
fn drain(host: &mut ViewerHost) {
    let handle = host.document.as_ref().unwrap().clone();
    let (pending, _) = handle
        .try_reproject(&handle.stamp().unwrap(), |document| {
            Ok::<_, String>(document.drain_callbacks())
        })
        .unwrap();
    let stamp = handle.stamp().unwrap();
    for event in pending {
        assert!(matches!(
            host.admit_callback(&stamp, &event),
            IntentAdmission::Accepted
        ));
    }
    host.after_dispatch(turn(host, HostTurnReason::Input))
        .unwrap();
}
fn click(host: &mut ViewerHost, binding: &str) {
    let handle = host.document.as_ref().unwrap().clone();
    let path = callback_path(&handle, binding);
    handle.dispatch_action(Action::Activate { path }).unwrap();
    drain(host);
}
fn captured_radio_path(handle: &NativeDocumentHandle) -> String {
    let owner = callback_path(handle, CAPTURED_EDIT_BINDING);
    handle
        .try_with(|document| {
            document
                .elements()
                .iter()
                .find(|element| {
                    element.path.starts_with(&format!("{owner}/"))
                        && element.attributes.get("role").map(String::as_str) == Some("radio")
                        && element
                            .attributes
                            .get("data-meari-radio-value")
                            .map(String::as_str)
                            == Some("project-card-1-0-0")
                })
                .expect("owned captured choice radio item")
                .path
                .clone()
        })
        .unwrap()
}
fn open(host: &mut ViewerHost) {
    let handle = host.document.as_ref().unwrap().clone();
    let path = captured_radio_path(&handle);
    assert!(handle.dispatch_action(Action::Activate { path }).unwrap());
    drain(host);
    let end = Instant::now() + Duration::from_secs(5);
    while host.captured_surface_receipt().is_none() {
        host.after_dispatch(turn(host, HostTurnReason::Completion))
            .unwrap();
        assert!(Instant::now() < end);
    }
}
fn text(host: &mut ViewerHost, value: &str) {
    let handle = host.document.as_ref().unwrap().clone();
    let path = callback_path(&handle, "onFrontChange");
    handle
        .try_reproject(&handle.stamp().unwrap(), |document| {
            document.set_input_value(&path, value)
        })
        .unwrap();
    drain(host);
}
fn ready(host: &mut ViewerHost) {
    terminal(host);
    host.after_dispatch(turn(host, HostTurnReason::Completion))
        .unwrap();
}

#[test]
#[ignore = "Requires the assigned original LocalHost slot; product graph only, no window/render"]
fn product_card_callbacks_edit_accept_retry_and_retain_original_ack_evidence()
-> Result<(), Box<GuardedFailure>> {
    let mut acknowledged = Vec::new();
    let result = with_rig(|host, rig| {
        let project = seed(&rig.client);
        let observer = bind_product(host, rig, project.clone());
        // Read controls are intentionally hidden until the owned diagnostics
        // toggle opens them; exercise the actual visible input path first.
        click(host, "onToggleDiagnostics");
        let handle = host.document.as_ref().unwrap().clone();
        let old_read = callback_path(&handle, "onReadStatus");
        handle
            .dispatch_action(Action::Activate { path: old_read })
            .unwrap();
        let (mut old_callbacks, _) = handle
            .try_reproject(&handle.stamp().unwrap(), |document| {
                Ok::<_, String>(document.drain_callbacks())
            })
            .unwrap();
        let old_callback = old_callbacks.pop().unwrap();
        open(host);
        let calls_before = requests(&rig.requests).len();
        assert!(matches!(
            host.admit_callback(&handle.stamp().unwrap(), &old_callback),
            IntentAdmission::Rejected(_)
        ));
        assert_eq!(requests(&rig.requests).len(), calls_before);
        assert!(observer.borrow().state.observed_route().is_none());
        let receipt = host.captured_surface_receipt().unwrap();
        assert_eq!(
            receipt.after().host_id,
            host.document.as_ref().unwrap().stamp().unwrap().host_id
        );
        text(host, "Actual product text");
        let stamp = host.document.as_ref().unwrap().stamp().unwrap();
        assert!(!host.can_close(&stamp, CloseCause::User).unwrap());
        assert_eq!(
            host.callbacks
                .borrow()
                .projection
                .borrow()
                .window_ui
                .props()["captionStatus"],
            meari::domain::Value::String(
                "원 작업의 사용자 닫기 보호 조건으로 창을 유지했습니다.".into()
            )
        );
        click(host, "onSave");
        ready(host);
        assert_eq!(
            host.capture_adapter_mut()
                .unwrap()
                .display()
                .state
                .document
                .draft_revision
                .0,
            1
        );
        assert!(
            !host
                .can_close(
                    &host.document.as_ref().unwrap().stamp().unwrap(),
                    CloseCause::User
                )
                .unwrap()
        );
        click(host, "onCaptureConsume");
        assert_eq!(
            host.capture_adapter_mut()
                .unwrap()
                .display()
                .state
                .document
                .draft_revision
                .0,
            2
        );
        click(host, "onAccept");
        ready(host);
        click(host, "onCaptureConsume");
        click(host, "onCaptureRetry");
        ready(host);
        click(host, "onCaptureConsume");
        acknowledged.extend(host.take_captured_surface_acknowledgements().unwrap());
        assert_eq!(acknowledged.len(), 3);
        assert_eq!(
            acknowledged[1].request.payload,
            acknowledged[2].request.payload
        );
        assert!(
            host.capture_adapter_mut()
                .unwrap()
                .display()
                .reused_acceptance
        );
        assert!(observer.borrow().state.observed_route().is_none());
        assert_immutable(rig, &project);
    });
    // Keep original request/terminal evidence alive after the same host has completed cleanup.
    if result.is_ok() {
        assert!(
            acknowledged
                .iter()
                .all(|e| e.terminal.as_ref().unwrap().response.ok)
        );
    }
    result
}

#[test]
#[ignore = "Requires the assigned original LocalHost slot; no window/render"]
fn product_decode_fault_keeps_original_terminal_until_explicit_result_discard()
-> Result<(), Box<GuardedFailure>> {
    let mut acknowledged = Vec::new();
    with_rig(|host, rig| {
        bind_product(host, rig, seed(&rig.client));
        open(host);
        text(host, "Backend commits this text");
        rig.fault_after_guard.store(2, Ordering::SeqCst);
        click(host, "onSave");
        ready(host);
        let raw = host
            .capture_adapter_mut()
            .unwrap()
            .execution_terminal()
            .unwrap()
            .clone();
        click(host, "onCaptureConsume");
        assert!(host.captured_surface_error().is_some());
        assert!(Arc::ptr_eq(
            &raw,
            host.capture_adapter_mut()
                .unwrap()
                .execution_terminal()
                .unwrap()
        ));
        assert_eq!(
            host.capture_adapter_mut()
                .unwrap()
                .display()
                .state
                .document
                .draft_revision
                .0,
            1
        );
        click(host, "onCaptureDiscard");
        acknowledged.extend(host.take_captured_surface_acknowledgements().unwrap());
        assert!(matches!(
            acknowledged[0].disposition,
            CaptureExecutionDisposition::Discarded { .. }
        ));
    })
}
