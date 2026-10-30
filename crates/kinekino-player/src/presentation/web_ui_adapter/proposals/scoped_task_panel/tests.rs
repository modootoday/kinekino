use super::*;
use crate::application::api::ProposalClient;
use kinekino_player::{
    application::proposals::{CreateProposal, InspectProposal},
    assembly::meari::proposal_registration::ProposalSdkRegistration,
    domain::{api_values::DecimalU64, proposals::*},
};
use meari::{
    application::state_program::StateProgram,
    domain::host_callbacks::{CallbackArguments, CallbackReference, CallbackSignature},
};
use sha2::{Digest, Sha256};
use std::cell::Cell;

#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Highlight,
    Cue,
    Document,
}

/// Real SDK/store fixture reused from the original Card suite; no synthetic save response.
pub(crate) fn fixture(kind: Kind) -> (StateProgram, ScopedPanelInput, StoredProposal) {
    let (frame, mut input, original) = super::super::scoped_panel::tests::fixture();
    let mut source = original.document.source;
    let (id, payload, target) = match kind {
        Kind::Highlight => (
            "scoped-highlight",
            ProposalPayload::Highlights {
                highlights: vec![
                    Highlight {
                        title: "first".into(),
                        source_refs: vec!["cue-a".into()],
                    },
                    Highlight {
                        title: "second title".into(),
                        source_refs: vec!["cue-a".into()],
                    },
                ],
            },
            BootstrapTarget::Highlight { index: 1 },
        ),
        Kind::Cue => {
            let SourceSnapshot::Cue(cues) = &mut source else {
                panic!("cue fixture")
            };
            cues.cues.push(SourceCue {
                cue_id: "cue-b".into(),
                text: "둘째 원문".into(),
                start_us: None,
                end_us: None,
            });
            (
                "scoped-correction",
                ProposalPayload::Correction {
                    edits: vec![
                        Correction {
                            cue_id: "cue-a".into(),
                            text: "첫 수정".into(),
                        },
                        Correction {
                            cue_id: "cue-b".into(),
                            text: "둘째 수정".into(),
                        },
                    ],
                },
                BootstrapTarget::CueCorrection {
                    index: 1,
                    cue_id: "cue-b".into(),
                },
            )
        }
        Kind::Document => {
            let content = concat!(
                "Edit this ASR draft. Correct punctuation only. Do not guess unclear words. ",
                "JSON only: {\"proposed\":string,\"uncertain_words\":[string]}. Under100words. ",
                "/no_think\n[\"원문 하나\",\"원문 둘\"]"
            );
            let asset =
                serde_json::json!({"messages":[{"role":"user","content":content}]}).to_string();
            let document = DocumentSourceSnapshot::from_asset(
                "scoped-document-source".into(),
                DecimalU64(9_007_199_254_740_993),
                format!("{:x}", Sha256::digest(asset.as_bytes())),
                asset,
            )
            .unwrap();
            let first = document.text_reference("/0").unwrap();
            let second = document.text_reference("/1").unwrap();
            source = SourceSnapshot::Document(document);
            (
                "scoped-document",
                ProposalPayload::DocumentCorrection {
                    edits: vec![
                        DocumentCorrection {
                            source_ref: first,
                            text: "문서 수정 하나".into(),
                        },
                        DocumentCorrection {
                            source_ref: second.clone(),
                            text: "문서 수정 둘".into(),
                        },
                    ],
                },
                BootstrapTarget::DocumentCorrection {
                    index: 1,
                    source_ref: second,
                },
            )
        }
    };
    let document = input
        .client
        .create_proposal(
            RequestContext::default(),
            CreateProposal {
                proposal_id: id.into(),
                source,
                generated: GeneratedResult {
                    task: payload.task(),
                    raw_text: serde_json::to_string(&payload).unwrap(),
                },
            },
        )
        .unwrap();
    let stored = input
        .client
        .inspect_proposal(
            RequestContext::default(),
            InspectProposal {
                proposal_id: id.into(),
            },
        )
        .unwrap();
    input.selection = BootstrapSelection {
        proposal_id: document.proposal_id,
        document_revision: document.draft_revision,
        source_id: document.source.source_id().into(),
        source_revision: document.source.source_revision(),
        source_sha256: document.source.source_sha256().into(),
        target,
    };
    (frame, input, stored)
}

pub(crate) fn correction_labels(input: &ScopedPanelInput) -> CorrectionLabels {
    CorrectionLabels {
        panel: input.labels.panel.clone(),
        source_count: "원문".into(),
        unselected_source: "선택 필요".into(),
    }
}
pub(crate) struct Lease {
    sdk: ProposalSdkRegistration,
}
impl ParentProposalLease for Lease {
    fn registration(&self) -> Result<&RegistrationHandle, PluginError> {
        self.sdk.handle()
    }
    fn close(&mut self) -> Result<(), PluginError> {
        self.sdk.close()
    }
}
pub(crate) fn register(
    client: ApplicationClientHandle,
    registry: Rc<RefCell<PluginRegistry>>,
    scope: &PluginScope,
    context: Rc<RefCell<RequestContext>>,
) -> Result<Lease, PluginError> {
    ProposalSdkRegistration::register(client, registry, scope, context).map(|sdk| Lease { sdk })
}
fn bind(kind: Kind) -> (ApplicationClientHandle, ScopedPanel<Lease>) {
    let (_, input, stored) = fixture(kind);
    let labels = if matches!(kind, Kind::Cue | Kind::Document) {
        Some(correction_labels(&input))
    } else {
        None
    };
    let client = input.client.clone();
    let panel = bind_task_panel(input, stored, labels, register)
        .unwrap_or_else(|failure| panic!("{:?}", failure.cause));
    (client, panel)
}

#[test]
fn explicit_highlight_keeps_original_refs_generated_text_and_second_title() {
    let (_, input, stored) = fixture(Kind::Highlight);
    let original = serde_json::to_value(&stored).unwrap();
    let scope = input.scope.clone();
    let mut panel =
        bind_task_panel(input, stored, None, register).unwrap_or_else(|_| panic!("highlight bind"));
    assert_eq!(panel.session.review().scope().view, scope);
    assert_eq!(
        panel.initial_props()["title"],
        Value::String("second title".into())
    );
    assert_eq!(
        serde_json::to_value(panel.session.review().stored()).unwrap(),
        original
    );
    assert_eq!(panel.metadata.selection.item_index, Some(1));
    assert!(panel.initial_props().contains_key("sourceText"));
    assert!(panel.initial_props().contains_key("generatedRaw"));
    panel.unbind().unwrap();
}

#[test]
fn correction_uses_exact_second_cue_and_document_reference() {
    for kind in [Kind::Cue, Kind::Document] {
        let (_, input, stored) = fixture(kind);
        let source = serde_json::to_value(&stored.document.source).unwrap();
        let expected = match kind {
            Kind::Cue => "둘째 수정",
            Kind::Document => "문서 수정 둘",
            _ => unreachable!(),
        };
        let labels = correction_labels(&input);
        let mut panel = bind_task_panel(input, stored, Some(labels), register)
            .unwrap_or_else(|failure| panic!("{:?}", failure.cause));
        assert_eq!(
            panel.initial_props()["workingText"],
            Value::String(expected.into())
        );
        assert_eq!(
            serde_json::to_value(&panel.session.review().stored().document.source).unwrap(),
            source
        );
        assert_eq!(panel.metadata.selection.item_index, Some(1));
        panel.unbind().unwrap();
    }
}

#[test]
fn missing_labels_wrong_cue_ref_or_stale_identity_never_registers() {
    for variant in 0..5 {
        let (_, mut input, stored) = fixture(if variant == 2 {
            Kind::Document
        } else {
            Kind::Cue
        });
        let labels = if variant == 0 {
            None
        } else {
            Some(correction_labels(&input))
        };
        match variant {
            1 => {
                let BootstrapTarget::CueCorrection { cue_id, .. } = &mut input.selection.target
                else {
                    unreachable!()
                };
                *cue_id = "cue-a".into();
            }
            2 => {
                let BootstrapTarget::DocumentCorrection { source_ref, .. } =
                    &mut input.selection.target
                else {
                    unreachable!()
                };
                source_ref.inner_pointer = "/0".into();
            }
            3 => input.selection.source_sha256 = "b".repeat(64),
            4 => input.selection.document_revision.0 += 1,
            _ => {}
        }
        let calls = Cell::new(0);
        let result = bind_task_panel::<Lease>(input, stored, labels, |_, _, _, _| {
            calls.set(calls.get() + 1);
            panic!("Invalid input registered")
        });
        assert!(result.is_err());
        assert_eq!(calls.get(), 0);
    }
}

#[test]
fn foreign_scope_and_unsupported_repair_keep_registry_without_parent() {
    for repair in [false, true] {
        let (_, mut input, stored) = fixture(Kind::Document);
        let registry = input.registry.clone();
        if repair {
            input.selection.target = BootstrapTarget::DocumentRepair { source_ref: None };
        } else {
            input.scope.source_generation += 1;
        }
        let failure = match bind_task_panel::<Lease>(input, stored, None, |_, _, _, _| {
            panic!("No registration")
        }) {
            Err(failure) => failure,
            Ok(_) => panic!("Must reject"),
        };
        assert!(failure.parent.is_none() && failure.session.is_none());
        assert!(Rc::ptr_eq(&failure.registry, &registry));
        if repair {
            assert!(matches!(failure.cause, ScopedPanelCause::UnsupportedRepair));
        }
    }
}

#[test]
fn card_compatibility_entry_still_rejects_a_highlight() {
    let (_, input, stored) = fixture(Kind::Highlight);
    let failure =
        match super::super::scoped_panel::bind_card_panel::<Lease>(input, stored, |_, _, _, _| {
            panic!("Card-only entry")
        }) {
            Err(failure) => failure,
            Ok(_) => panic!("Must reject"),
        };
    assert!(matches!(
        failure.cause,
        ScopedPanelCause::Selection(BootstrapError::TargetMismatch)
    ));
}

#[test]
fn save_then_separate_accept_use_original_sdk_and_preserve_source() {
    for kind in [Kind::Highlight, Kind::Cue, Kind::Document] {
        let (client, mut panel) = bind(kind);
        let id = panel.session.review().stored().document.proposal_id.clone();
        let source =
            serde_json::to_value(&panel.session.review().stored().document.source).unwrap();
        let export = panel.session.export_callbacks().unwrap();
        let binding = if matches!(kind, Kind::Highlight) {
            "onTitleChange"
        } else {
            "onDraftChange"
        };
        for (name, arguments, signature) in [
            (
                binding,
                CallbackArguments::Text("사람 수정".into()),
                CallbackSignature::Text,
            ),
            (
                "onSave",
                CallbackArguments::Action,
                CallbackSignature::Action,
            ),
            (
                "onAccept",
                CallbackArguments::Action,
                CallbackSignature::Action,
            ),
        ] {
            panel
                .session
                .dispatch_after_renderer_guard(
                    &export,
                    &CallbackReference {
                        binding: name.into(),
                        signature,
                    },
                    arguments,
                )
                .unwrap();
            let report = panel.session.drain_after_dispatch().unwrap();
            assert!(
                !report.sdk_error
                    && report.view_error.is_none()
                    && report.transport_error.is_none()
            );
            let reopened = client
                .inspect_proposal(
                    RequestContext::default(),
                    InspectProposal {
                        proposal_id: id.clone(),
                    },
                )
                .unwrap();
            assert_eq!(
                serde_json::to_value(&reopened.document.source).unwrap(),
                source
            );
            if name == "onSave" {
                assert!(reopened.acceptance.is_none());
            }
            if name == "onAccept" {
                assert!(reopened.acceptance.is_some());
            }
        }
        panel.unbind().unwrap();
        assert!(panel.parent.registration().is_err());
    }
}

#[test]
fn post_registration_handle_failure_retains_same_parent_until_unbind() {
    struct Broken {
        lease: Lease,
        busy: Rc<Cell<bool>>,
    }
    impl ParentProposalLease for Broken {
        fn registration(&self) -> Result<&RegistrationHandle, PluginError> {
            Err(PluginError::new(
                PluginErrorCode::Busy,
                "original handle fault",
            ))
        }
        fn close(&mut self) -> Result<(), PluginError> {
            if self.busy.get() {
                return Err(PluginError::new(
                    PluginErrorCode::Busy,
                    "original unbind pending",
                ));
            }
            self.lease.close()
        }
    }
    let (_, input, stored) = fixture(Kind::Highlight);
    let busy = Rc::new(Cell::new(true));
    let retained = busy.clone();
    let mut failure = match bind_task_panel(
        input,
        stored,
        None,
        move |client, registry, scope, context| {
            Ok(Broken {
                lease: register(client, registry, scope, context)?,
                busy: retained,
            })
        },
    ) {
        Err(failure) => failure,
        Ok(_) => panic!("Original handle fault"),
    };
    assert!(failure.parent.is_some() && failure.session.is_none());
    assert!(failure.retry_unbind().is_err());
    assert!(
        failure
            .parent
            .as_ref()
            .unwrap()
            .lease
            .registration()
            .is_ok()
    );
    busy.set(false);
    failure.retry_unbind().unwrap();
    assert!(
        failure
            .parent
            .as_ref()
            .unwrap()
            .lease
            .registration()
            .is_err()
    );
}

#[test]
fn session_binding_collision_retains_original_registration_for_explicit_cleanup() {
    let (_, mut input, stored) = fixture(Kind::Highlight);
    input
        .local_signatures
        .insert("onSave".into(), CallbackSignature::Action);
    let registry = input.registry.clone();
    let mut failure = match bind_task_panel(input, stored, None, register) {
        Err(failure) => failure,
        Ok(_) => panic!("Duplicate local binding must fail"),
    };
    assert!(matches!(&failure.cause, ScopedPanelCause::Assembly(_)));
    assert!(failure.parent.is_some());
    assert!(Rc::ptr_eq(&registry, &failure.registry));
    failure.retry_unbind().unwrap();
    assert!(failure.parent.as_ref().unwrap().registration().is_err());
}
