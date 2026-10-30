use super::*;
use crate::{
    domain::{api_values::DecimalU64, projects::*, proposals::*},
    presentation::web_ui_adapter::proposals::card_selection_queue::CARD_SELECTION_BINDING,
};
use meari_native::{domain::widgets::Action, infrastructure::native_host::IntentAdmission};
use std::num::{NonZeroU64, NonZeroUsize};

fn project() -> ProjectRevision {
    let draft = ProposalPayload::Cards {
        cards: vec![
            Card {
                front: "first".into(),
                back: "first back".into(),
                source_refs: vec!["cue-a".into()],
            },
            Card {
                front: "second".into(),
                back: "second back".into(),
                source_refs: vec!["cue-a".into()],
            },
        ],
    };
    let source = SourceSnapshot::Cue(CueSourceSnapshot {
        source_id: "selected-source".into(),
        source_revision: DecimalU64(3),
        source_sha256: "a".repeat(64),
        cues: vec![SourceCue {
            cue_id: "cue-a".into(),
            text: "Selected source".into(),
            start_us: None,
            end_us: None,
        }],
    });
    let captured = StoredProposal {
        document: ProposalDocument {
            schema_version: proposal_document_version(&source, None).into(),
            proposal_id: "selected-proposal".into(),
            source,
            generated: GeneratedResult {
                task: ProposalTask::Cards,
                raw_text: serde_json::to_string(&draft).unwrap(),
            },
            draft_revision: DecimalU64(1),
            state: DraftState::Editable,
            draft: Some(draft),
            import_error: None,
        },
        acceptance: None,
        historical: None,
    };
    let mut project = ProjectRevision {
        schema_version: "1".into(),
        project_id: "selected-project".into(),
        revision: DecimalU64(1),
        parent_revision: None,
        title: "Selected project".into(),
        watching: None,
        proposals: vec![ProjectProposalSnapshot {
            capture_sha256: canonical_sha256(&captured).unwrap(),
            captured,
            working_draft: None,
            working_state: WorkingState::Absent,
            proposal_origin: None,
        }],
        work_state: ProjectWorkState::default(),
    };
    project.work_state = project.derived_work_state();
    project.validate().unwrap();
    project
}

#[test]
fn real_viewing_export_rejects_capture_edit_before_queue_or_sdk_admission() {
    struct RecordingSdk {
        inner: ApplicationClientHandle,
        calls: Arc<std::sync::Mutex<Vec<String>>>,
    }
    impl crate::application::api::ApplicationClient for RecordingSdk {
        fn execute(
            &self,
            request: crate::application::api::Request,
        ) -> crate::application::api::Response {
            self.calls.lock().unwrap().push(request.command.clone());
            self.inner.execute(request)
        }
    }
    let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
    let execution = ExecutionOwner::start(
        Arc::new(RecordingSdk {
            inner: Arc::new(PlayerSdk::default()),
            calls: calls.clone(),
        }),
        ExecutionBudget {
            max_operations: 8,
            max_request_bytes: 262144,
            max_total_request_bytes: 8 * 262144,
            max_terminal_bytes: 262144,
            max_total_terminal_bytes: 8 * 262144,
        },
        Arc::new(Notify),
    )
    .unwrap();
    let captured = project();
    let original_capture = serde_json::to_vec(&captured).unwrap();
    let owner = Rc::new(RefCell::new(ViewerCallbacks::new(
        execution.handle(),
        RequestContext::default(),
    )));
    owner
        .borrow_mut()
        .configure_card_choices(
            captured.clone(),
            NonZeroU64::new(1).unwrap(),
            NonZeroUsize::new(1).unwrap(),
        )
        .unwrap();
    let entry: BuildPath = "select.tsx".to_owned().try_into().unwrap();
    let source = "type Props={onSelect:(key:string)=>void;rowKey:string};export function Select(props:Props){return <button onClick={() => props.onSelect(props.rowKey)}>Select</button>;}";
    let graph =
        meari::parse_module_graph(&entry, &BTreeMap::from([(entry.clone(), source.into())]))
            .unwrap();
    let program = ModuleProgram::mount(
        graph,
        BTreeMap::from([
            ("rowKey".into(), Value::String("project-card-1-0-1".into())),
            (
                "onSelect".into(),
                Value::CallbackReference(CallbackReference {
                    binding: CARD_SELECTION_BINDING.into(),
                    signature: CallbackSignature::Text,
                }),
            ),
        ]),
        "viewing-selection".into(),
        1,
    )
    .unwrap();
    let exporting = owner.clone();
    let mut document = InteractiveDocument::from_module_program_with_callbacks(
        program,
        "",
        Viewport {
            width: 160,
            height: 80,
            dark: true,
        },
        None,
        1,
        move |scope| {
            exporting
                .borrow_mut()
                .export(scope)
                .map_err(|error| error.message)
        },
    )
    .unwrap();
    let scope = document.callback_scope().unwrap().clone();
    let button = document
        .elements()
        .iter()
        .find(|e| e.dom_tag == "button")
        .unwrap()
        .path
        .clone();
    document
        .dispatch(Action::Activate {
            path: button.clone(),
        })
        .unwrap();
    document
        .dispatch(Action::Activate { path: button })
        .unwrap();
    let events = document.drain_callbacks();
    assert_eq!(events.len(), 2);
    let queue = owner.borrow().card_selection_queue().unwrap();
    let borrowed_queue = queue.borrow_mut();
    for event in &events {
        assert!(
            matches!(owner.borrow_mut().queue_card_selection(&scope, event),
            Ok(Some(IntentAdmission::Rejected(error))) if error.code == PluginErrorCode::PermissionDenied)
        );
    }
    drop(borrowed_queue);
    assert!(queue.borrow().is_empty());
    assert_eq!(document.callback_scope(), Some(&scope));
    assert_eq!(serde_json::to_vec(&captured).unwrap(), original_capture);
    assert!(calls.lock().unwrap().is_empty());
    assert!(execution.handle().retained_operations().unwrap().is_empty());
    assert!(execution.handle().attach().is_err());
    assert!(
        owner
            .borrow_mut()
            .prepare_route_release(&scope)
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        owner.borrow_mut().queue_card_selection(&scope, &events[0]),
        Err(meari::application::host_callbacks::LocalDispatchError::Invalid(_))
    ));
    let next = execution.handle().attach().unwrap();
    next.release().unwrap();
    close(&execution);
}

#[test]
fn legacy_viewing_owner_has_no_card_binding_and_reconfiguration_cannot_replace_active_export() {
    let fixture = route_fixture();
    assert!(fixture.owner.borrow().card_selection_queue().is_none());
    let (_, pending) = local_input(&fixture.owner.borrow(), "onReadCapabilities");
    assert!(matches!(
        fixture
            .owner
            .borrow_mut()
            .queue_card_selection(&fixture.scope, &pending),
        Ok(None)
    ));
    assert_eq!(
        fixture
            .owner
            .borrow_mut()
            .configure_card_choices(
                project(),
                NonZeroU64::new(1).unwrap(),
                NonZeroUsize::new(1).unwrap()
            )
            .unwrap_err()
            .code,
        PluginErrorCode::Stale
    );
    assert!(
        fixture
            .owner
            .borrow_mut()
            .prepare_route_release(&fixture.scope)
            .unwrap()
            .is_none()
    );
    close(&fixture.execution);
}
