//! UI projection fixtures, not actual native playback or user quality evidence.
#[test]
fn catalog_release_callback_uses_root_binding_and_preserves_nonterminal_guard() {
    let fixture = route_fixture();
    let (_bindings, input) = local_input(&fixture.owner.borrow(), "onReleaseCatalog");
    let before = fixture.owner.borrow().tickets().unwrap().len();
    fixture
        .owner
        .borrow_mut()
        .queue_local(&fixture.scope, &input)
        .unwrap();
    let drained = fixture
        .owner
        .borrow_mut()
        .drain_local(&fixture.scope)
        .unwrap();
    assert!(!drained.refreshed);
    assert!(fixture.owner.borrow().local_error().is_some());
    assert!(fixture.owner.borrow().local_queue_empty());
    assert_eq!(fixture.owner.borrow().tickets().unwrap().len(), before);
    finish(&fixture);
}
use super::*;
use crate::{
    application::playback_session::EpisodePlaybackObservation,
    domain::playback_control::PlaybackControl,
};

#[test]
fn actual_catalog_root_factory_and_owned_callbacks_accept_nonempty_rows_and_captured_values() {
    use crate::presentation::web_ui_adapter::player_shell::{
        RouteCatalog,
        catalog_browser::{self, CatalogBrowserView, CatalogRow},
        document::{self, StartLabels, product_frame::ProductFrameBlueprint},
    };
    use meari::domain::host_callbacks::CallbackSource;
    use meari_native::domain::widgets::Action;
    fn original_document_error(error: document::DocumentPreparationError) -> String {
        match error {
            document::DocumentPreparationError::Source(error) => format!("{error:?}"),
            document::DocumentPreparationError::UnknownRoute => "UnknownRoute".into(),
            document::DocumentPreparationError::SourceMissing => "SourceMissing".into(),
            document::DocumentPreparationError::SourceNotIntegrated => "SourceNotIntegrated".into(),
            document::DocumentPreparationError::InvalidFixedPath => "InvalidFixedPath".into(),
        }
    }
    let execution = execution();
    let owner = Rc::new(RefCell::new(ViewerCallbacks::new(
        execution.handle(),
        RequestContext::default(),
    )));
    let labels = StartLabels {
        connection: "not observed".into(),
        time: "not observed".into(),
    };
    let mut root = document::read_props(&labels);
    root.insert("diagnosticsExpanded".into(), Value::Bool(true));
    let series_key = format!("{}:0", "a".repeat(64));
    let episode_key = format!("{}:0", "b".repeat(64));
    let browse_episode_key = format!("{}:1", "b".repeat(64));
    root.extend(
        catalog_browser::props(CatalogBrowserView {
            series: vec![CatalogRow {
                key: series_key.clone(),
                title: "Graph series A".into(),
                disabled: false,
            }],
            episodes: vec![
                CatalogRow {
                    key: episode_key.clone(),
                    title: "Graph episode B".into(),
                    disabled: false,
                },
                CatalogRow {
                    key: browse_episode_key.clone(),
                    title: "Browse-only episode".into(),
                    disabled: false,
                },
            ],
            selected_series: String::new(),
            selected_episode: browse_episode_key.clone(),
            ..Default::default()
        })
        .unwrap(),
    );
    root.insert(
        "catalogCurrentEpisode".into(),
        Value::String(episode_key.clone()),
    );
    root.insert("nativeControlsDisabled".into(), Value::Bool(false));
    root.insert("nativeSeekRangeDisabled".into(), Value::Bool(false));
    root.insert("nativeVolumeRangeDisabled".into(), Value::Bool(false));
    root.insert(
        "nativeSeekDraft".into(),
        Value::String("00:00:00.000037".into()),
    );
    root.insert("nativeVolumeDraft".into(), Value::String("65".into()));
    let mut viewing =
        document::prepare_start_document(&RouteCatalog::bundled().unwrap(), "view.start", labels)
            .unwrap_or_else(|error| {
                panic!("prepare_start_document: {}", original_document_error(error))
            });
    document::enable_read_panel(&mut viewing, root.clone())
        .unwrap_or_else(|error| panic!("enable_read_panel: {}", original_document_error(error)));
    // Reuse the existing product-frame fixture's inert, callback-free Cards source.
    let card_path: BuildPath = "catalog-graph-card.tsx".to_owned().try_into().unwrap();
    let cards = BTreeMap::from([(
        card_path.clone(),
        "export function Card(){return <article/>;}".into(),
    )]);
    let card_graph = meari::parse_module_graph(&card_path, &cards).unwrap();
    let blueprint =
        ProductFrameBlueprint::prepare(viewing.graph, viewing.inventory, card_graph, cards)
            .unwrap();
    crate::presentation::web_ui_adapter::player_shell::window_caption::contract_fixture::assert_required_caption_fields(
        &root, |invalid| blueprint.mount_viewing(invalid, "caption-viewing-negative".into(), 1,
            std::num::NonZeroU64::new(1).unwrap()).is_ok());
    for mutation in 0..14 {
        let mut invalid = root.clone();
        match mutation {
            0 => {
                invalid.remove("nativePositionLabel");
            }
            1 => {
                invalid.insert("unknownViewingExtra".into(), Value::Bool(true));
            }
            2 => {
                invalid.insert(
                    "diagnosticsExpanded".into(),
                    Value::String("wrong type".into()),
                );
            }
            3 => {
                invalid.insert(
                    "onToggleDiagnostics".into(),
                    Value::CallbackReference(CallbackReference {
                        binding: "onToggleDiagnostics".into(),
                        signature: CallbackSignature::Text,
                    }),
                );
            }
            4 => {
                invalid.remove("nativeSeekRangeValue");
            }
            5 => {
                invalid.insert(
                    "nativeSeekRangeDisabled".into(),
                    Value::String("wrong type".into()),
                );
            }
            6 => {
                invalid.insert(
                    "onNativeSeekCommit".into(),
                    Value::CallbackReference(CallbackReference {
                        binding: "onNativeSeekCommit".into(),
                        signature: CallbackSignature::Action,
                    }),
                );
            }
            7 => {
                invalid.remove("viewingFullscreen");
            }
            8 => {
                invalid.insert(
                    "viewingFullscreen".into(),
                    Value::String("not an observed bool".into()),
                );
            }
            9 => {
                invalid.remove("nativeStatusSummary");
            }
            10 => {
                invalid.insert(
                    "nativeStatusError".into(),
                    Value::String("not a typed error flag".into()),
                );
            }
            11 => {
                invalid.insert("nativeStatusSummary".into(), Value::Bool(true));
            }
            12 => {
                invalid.remove("catalogCurrentEpisode");
            }
            _ => {
                invalid.insert("catalogConnectingEpisode".into(), Value::Bool(true));
            }
        }
        assert!(
            blueprint
                .mount_viewing(
                    invalid,
                    format!("invalid-viewing-contract-{mutation}"),
                    1,
                    std::num::NonZeroU64::new(1).unwrap()
                )
                .is_err()
        );
    }
    let program = blueprint
        .mount_viewing(
            root.clone(),
            "catalog-graph-original-factory".into(),
            1,
            std::num::NonZeroU64::new(1).unwrap(),
        )
        .unwrap();
    let projection = program.project().unwrap();
    let text = serde_json::to_string(&projection.component).unwrap();
    assert!(text.contains("Graph series A"));
    assert!(text.contains("Graph episode B"));
    let handlers = projection.callback_bindings();
    for (name, signature) in [
        ("onNativePlay", CallbackSignature::Action),
        ("onNativePause", CallbackSignature::Action),
        ("onToggleDiagnostics", CallbackSignature::Action),
        ("onToggleEpisodes", CallbackSignature::Action),
        ("onNativeSeek", CallbackSignature::Text),
        ("onNativeVolume", CallbackSignature::Text),
        ("onNativeSeekDraft", CallbackSignature::Text),
        ("onNativeVolumeDraft", CallbackSignature::Text),
        ("onNativeSeekRangeDraft", CallbackSignature::Text),
        ("onNativeVolumeRangeDraft", CallbackSignature::Text),
        ("onNativeSeekCommit", CallbackSignature::Text),
        ("onNativeVolumeCommit", CallbackSignature::Text),
        ("onCatalogAiring", CallbackSignature::Action),
        ("onCatalogCompleted", CallbackSignature::Action),
        ("onCatalogMovies", CallbackSignature::Action),
        ("onCatalogSeries", CallbackSignature::Text),
        ("onCatalogEpisode", CallbackSignature::Text),
        ("onCatalogCancel", CallbackSignature::Action),
        ("onPreparePrevious", CallbackSignature::Action),
        ("onPrepareNext", CallbackSignature::Action),
        ("onCancelTransitionJob", CallbackSignature::Action),
        ("onObserveTransition", CallbackSignature::Action),
        ("onReleaseCatalog", CallbackSignature::Action),
        ("onRetryCatalogAck", CallbackSignature::Action),
    ] {
        assert!(
            handlers
                .values()
                .any(|handler| handler.reference.binding == name
                    && handler.reference.signature == signature),
            "missing actual source callback {name}"
        );
    }
    for binding in [
        "onCatalogPoll",
        "onCatalogReadSeries",
        "onCatalogReadEpisodes",
        "onCatalogInspect",
        "onCatalogPlay",
        "onCommitTransition",
        "onReadTransitionJob",
        "onReleaseTransition",
    ] {
        assert!(
            !handlers
                .values()
                .any(|handler| handler.reference.binding == binding),
            "technical step must not be a final UI action"
        );
    }
    let seek_path = handlers
        .iter()
        .find(|(_, h)| h.reference.binding == "onNativeSeek")
        .map(|(path, handler)| {
            assert_eq!(handler.source, CallbackSource::CapturedText);
            assert_eq!(handler.captured_text.as_deref(), Some("00:00:00.000037"));
            path.clone()
        })
        .unwrap();
    let volume = handlers
        .values()
        .find(|handler| handler.reference.binding == "onNativeVolume")
        .unwrap();
    assert_eq!(volume.source, CallbackSource::CapturedText);
    assert_eq!(volume.captured_text.as_deref(), Some("65"));
    for name in ["onNativeSeekCommit", "onNativeVolumeCommit"] {
        let handler = handlers
            .values()
            .find(|handler| handler.reference.binding == name)
            .unwrap();
        assert_eq!(handler.reference.signature, CallbackSignature::Text);
        assert!(
            handler.captured_text.is_none(),
            "Range terminal cannot capture a manual text draft"
        );
    }
    for name in ["onCatalogSeries", "onCatalogEpisode"] {
        assert!(
            handlers
                .values()
                .any(|handler| handler.reference.binding == name
                    && handler.source == CallbackSource::SelectionValue)
        );
    }
    let exporting = owner.clone();
    let mut native = InteractiveDocument::from_module_program_with_callbacks(
        program,
        "",
        Viewport {
            width: 960,
            height: 640,
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
    let scope = native.callback_scope().unwrap().clone();
    for (current, connecting) in [(true, false), (false, true), (false, false), (true, false)] {
        root.insert(
            "catalogCurrentEpisode".into(),
            Value::String(if current {
                episode_key.clone()
            } else {
                String::new()
            }),
        );
        root.insert(
            "catalogConnectingEpisode".into(),
            Value::String(if connecting {
                episode_key.clone()
            } else {
                String::new()
            }),
        );
        root.insert(
            "catalogEpisodeConnectionLabel".into(),
            Value::String(
                if connecting {
                    "엔진 연결 확인 중"
                } else {
                    ""
                }
                .into(),
            ),
        );
        let props = blueprint.workspace_props(&native,
            crate::presentation::web_ui_adapter::player_shell::document::product_frame::WorkspaceRoute::Viewing,
            std::num::NonZeroU64::new(1).unwrap(), root.clone()).unwrap();
        let bindings = owner.borrow().bindings().unwrap();
        native
            .update_module_props_with_callbacks(props, bindings)
            .unwrap();
        let actual_row = native
            .elements()
            .iter()
            .find(|element| {
                element.attributes.get("aria-label").map(String::as_str) == Some("Graph episode B")
                    && element.attributes.get("role").map(String::as_str) == Some("radio")
            })
            .unwrap();
        assert_eq!(
            actual_row
                .attributes
                .get("aria-current")
                .map(String::as_str),
            Some(if current { "true" } else { "false" })
        );
        let browse_row = native
            .elements()
            .iter()
            .find(|element| {
                element.attributes.get("aria-label").map(String::as_str)
                    == Some("Browse-only episode")
                    && element.attributes.get("role").map(String::as_str) == Some("radio")
            })
            .unwrap();
        assert_eq!(
            browse_row
                .attributes
                .get("aria-current")
                .map(String::as_str),
            Some("false")
        );
        assert_eq!(
            browse_row
                .attributes
                .get("aria-checked")
                .map(String::as_str),
            Some("true")
        );
        assert_eq!(native.callback_scope(), Some(&scope));
    }
    // Exercise the actual product frame -> owned workspace -> ReadRoot -> imported Range.
    let original_handlers = native.module_projection().unwrap().callback_bindings();
    let text_paths = crate::assembly::meari::viewer_read_projection::range_feedback::display_paths(
        &native.module_projection().unwrap().component.root,
    )
    .unwrap();
    assert_eq!(text_paths.len(), 6);
    let mut feedback_root = root.clone();
    feedback_root.insert(
        "nativeSeekRangeValue".into(),
        Value::String("500000".into()),
    );
    feedback_root.insert(
        "nativeSeekPreviewLabel".into(),
        Value::String("00:00:00.050".into()),
    );
    feedback_root.insert("nativeVolumeRangeValue".into(), Value::String("655".into()));
    feedback_root.insert(
        "nativeVolumePreviewLabel".into(),
        Value::String("65.5%".into()),
    );
    let feedback = blueprint.workspace_props(&native,
        crate::presentation::web_ui_adapter::player_shell::document::product_frame::WorkspaceRoute::Viewing,
        std::num::NonZeroU64::new(1).unwrap(), feedback_root).unwrap();
    native
        .update_module_range_props_with_text(feedback, &text_paths)
        .unwrap();
    assert_eq!(native.callback_scope(), Some(&scope));
    assert_eq!(
        native.module_projection().unwrap().callback_bindings(),
        original_handlers
    );
    let internal_bindings = owner.borrow().bindings().unwrap();
    for binding in [
        "onCatalogPoll",
        "onCatalogReadSeries",
        "onCatalogReadEpisodes",
        "onCatalogInspect",
        "onCatalogPlay",
        "onCommitTransition",
        "onReadTransitionJob",
        "onReleaseTransition",
    ] {
        assert!(
            internal_bindings
                .resolve(&CallbackReference {
                    binding: binding.into(),
                    signature: CallbackSignature::Action
                })
                .is_ok(),
            "original continuation must remain owned"
        );
    }
    native
        .dispatch(Action::Activate { path: seek_path })
        .unwrap();
    let captured = native.drain_callbacks();
    assert_eq!(captured.len(), 1);
    assert_eq!(
        captured[0].invocation.arguments,
        CallbackArguments::Text("00:00:00.000037".into())
    );
    let series_path = native
        .elements()
        .iter()
        .find(|element| {
            element.attributes.get("aria-label").map(String::as_str) == Some("Graph series A")
                && element.attributes.get("role").map(String::as_str) == Some("radio")
        })
        .unwrap()
        .path
        .clone();
    native
        .dispatch(Action::Activate { path: series_path })
        .unwrap();
    let selected = native.drain_callbacks();
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0].invocation.arguments,
        CallbackArguments::Text(series_key)
    );
    let episode_path = native
        .elements()
        .iter()
        .find(|element| {
            element.attributes.get("aria-label").map(String::as_str) == Some("Graph episode B")
                && element.attributes.get("role").map(String::as_str) == Some("radio")
        })
        .unwrap()
        .path
        .clone();
    native
        .dispatch(Action::Activate { path: episode_path })
        .unwrap();
    let selected = native.drain_callbacks();
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0].invocation.arguments,
        CallbackArguments::Text(episode_key)
    );
    // DOM input/typed export only: no callback is dispatched into a new SDK job here.
    assert!(owner.borrow().tickets().unwrap().is_empty());
    finish(&RouteFixture {
        execution,
        owner,
        _document: native,
        scope,
    });
}

#[test]
fn actual_owned_start_screen_is_removed_and_restored_without_replacing_video_or_submitting_sdk() {
    use crate::application::api::{ApplicationClient, Request, Response};
    use crate::presentation::web_ui_adapter::player_shell::{
        RouteCatalog,
        document::{
            self, StartLabels,
            product_frame::{ProductFrameBlueprint, WorkspaceRoute},
        },
    };
    use meari_native::domain::widgets::Action;
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct CountingClient(Arc<AtomicUsize>);
    impl ApplicationClient for CountingClient {
        fn execute(&self, request: Request) -> Response {
            self.0.fetch_add(1, Ordering::SeqCst);
            PlayerSdk::default().execute(request)
        }
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let execution = ExecutionOwner::start(
        Arc::new(CountingClient(calls.clone())),
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
    let owner = Rc::new(RefCell::new(ViewerCallbacks::new(
        execution.handle(),
        RequestContext::default(),
    )));
    let labels = StartLabels {
        connection: "not observed".into(),
        time: "not observed".into(),
    };
    let mut root = document::read_props(&labels);
    root.insert("diagnosticsExpanded".into(), Value::Bool(true));
    root.insert("nativeControlsDisabled".into(), Value::Bool(false));
    root.insert(
        "nativeSeekDraft".into(),
        Value::String("00:00:00.000037".into()),
    );
    let mut viewing =
        document::prepare_start_document(&RouteCatalog::bundled().unwrap(), "view.start", labels)
            .unwrap_or_else(|_| panic!("original viewing graph must prepare"));
    document::enable_read_panel(&mut viewing, root.clone())
        .unwrap_or_else(|_| panic!("original interactive graph must prepare"));
    // Same inert Cards fixture as the existing actual catalog Root graph test.
    let card_path: BuildPath = "catalog-graph-card.tsx".to_owned().try_into().unwrap();
    let cards = BTreeMap::from([(
        card_path.clone(),
        "export function Card(){return <article/>;}".into(),
    )]);
    let card_graph = meari::parse_module_graph(&card_path, &cards).unwrap();
    let blueprint =
        ProductFrameBlueprint::prepare(viewing.graph, viewing.inventory, card_graph, cards)
            .unwrap();
    let generation = std::num::NonZeroU64::new(1).unwrap();
    let program = blueprint
        .mount_viewing(
            root.clone(),
            "start-screen-original-factory".into(),
            1,
            generation,
        )
        .unwrap();
    let exporting = owner.clone();
    let mut native = InteractiveDocument::from_module_program_with_callbacks(
        program,
        "",
        Viewport {
            width: 960,
            height: 640,
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
    let scope = native.callback_scope().unwrap().clone();
    let initial = native.module_projection().unwrap();
    let original_callbacks = initial.callback_bindings();
    let original_video = native
        .elements()
        .iter()
        .find(|element| element.dom_tag == "video")
        .unwrap()
        .clone();
    let semantic_id = match root.get("videoSemanticId").unwrap() {
        Value::String(id) => id.clone(),
        _ => panic!("original semantic video id must be text"),
    };
    assert_eq!(original_video.id, semantic_id);
    let original_video_node = native
        .renderer
        .document
        .get_element_by_id(&semantic_id)
        .unwrap();
    for closed in [true, false, true, false] {
        root.insert("episodePanelClosed".into(), Value::Bool(closed));
        root.insert(
            "viewingRootClass".into(),
            Value::String(
                if closed {
                    "aw-viewing-read-root aw-viewing-panel-closed"
                } else {
                    "aw-viewing-read-root"
                }
                .into(),
            ),
        );
        root.insert(
            "episodePanelClass".into(),
            Value::String(
                if closed {
                    "aw-episode-panel aw-episodes-closed"
                } else {
                    "aw-episode-panel"
                }
                .into(),
            ),
        );
        root.insert(
            "automaticError".into(),
            Value::String("original-status-outside-episode-panel".into()),
        );
        let props = blueprint
            .workspace_props(&native, WorkspaceRoute::Viewing, generation, root.clone())
            .unwrap();
        let bindings = owner.borrow().bindings().unwrap();
        native
            .update_module_props_with_callbacks(props, bindings)
            .unwrap();
        assert_eq!(
            native.renderer.document.get_element_by_id(&semantic_id),
            Some(original_video_node)
        );
        assert_eq!(native.callback_scope(), Some(&scope));
        let source = serde_json::to_string(&native.module_projection().unwrap().component).unwrap();
        assert!(source.contains("재생 상태와 작업"));
        assert!(source.contains("original-status-outside-episode-panel"));
        assert!(
            native
                .module_projection()
                .unwrap()
                .callback_bindings()
                .values()
                .any(|handler| handler.reference.binding == "onCancelTransitionJob")
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
    let mut removed_paths = Vec::new();
    fn chrome_hidden(node: &meari::domain::Node) -> Option<bool> {
        use meari::domain::NodeKind;
        if let NodeKind::Element {
            attributes,
            children,
            ..
        } = &node.kind
        {
            if attributes.iter().any(|attribute| {
                attribute.name == "className"
                    && attribute.value == Value::String("aw-viewing-chrome".into())
            }) {
                return Some(attributes.iter().any(|attribute| {
                    attribute.name == "hidden" && attribute.value == Value::Bool(true)
                }));
            }
            return children.iter().find_map(chrome_hidden);
        }
        None
    }
    for fullscreen in [true, false, true, false] {
        root.insert("viewingFullscreen".into(), Value::Bool(fullscreen));
        root.insert(
            "viewingFrameClass".into(),
            Value::String(
                if fullscreen {
                    "aw-player-shell aw-player-fullscreen"
                } else {
                    "aw-player-shell"
                }
                .into(),
            ),
        );
        let props = blueprint
            .workspace_props(&native, WorkspaceRoute::Viewing, generation, root.clone())
            .unwrap();
        let bindings = owner.borrow().bindings().unwrap();
        native
            .update_module_props_with_callbacks(props, bindings)
            .unwrap();
        assert_eq!(
            chrome_hidden(&native.module_projection().unwrap().component.root),
            Some(fullscreen)
        );
        assert_eq!(
            native.renderer.document.get_element_by_id(&semantic_id),
            Some(original_video_node)
        );
        assert_eq!(native.callback_scope(), Some(&scope));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
    fn nonhidden_text(node: &meari::domain::Node, text: &str, hidden: bool) -> bool {
        use meari::domain::NodeKind;
        match &node.kind {
            NodeKind::Text(Value::String(value)) => !hidden && value == text,
            NodeKind::Element {
                attributes,
                children,
                ..
            } => {
                let hidden = hidden
                    || attributes.iter().any(|attribute| {
                        attribute.name == "hidden" && attribute.value == Value::Bool(true)
                    });
                children
                    .iter()
                    .any(|child| nonhidden_text(child, text, hidden))
            }
            _ => false,
        }
    }
    let status_baseline = root.clone();
    for (error, uncertain, summary) in [
        (false, false, "unknown-visible"),
        (true, false, "typed-failure-visible"),
        (true, true, "uncertain-visible"),
    ] {
        root.insert("diagnosticsExpanded".into(), Value::Bool(false));
        root.insert("episodePanelClosed".into(), Value::Bool(true));
        root.insert(
            "viewingRootClass".into(),
            Value::String("aw-viewing-read-root aw-viewing-panel-closed".into()),
        );
        root.insert(
            "episodePanelClass".into(),
            Value::String("aw-episode-panel aw-episodes-closed".into()),
        );
        root.insert("nativeStatusSummary".into(), Value::String(summary.into()));
        root.insert("nativeStatusError".into(), Value::Bool(error));
        root.insert("nativeStatusUncertain".into(), Value::Bool(uncertain));
        root.insert("localInputErrorPresent".into(), Value::Bool(true));
        root.insert(
            "localInputError".into(),
            Value::String("local-input-visible".into()),
        );
        let props = blueprint
            .workspace_props(&native, WorkspaceRoute::Viewing, generation, root.clone())
            .unwrap();
        let bindings = owner.borrow().bindings().unwrap();
        native
            .update_module_props_with_callbacks(props, bindings)
            .unwrap();
        let projection = native.module_projection().unwrap();
        assert!(nonhidden_text(&projection.component.root, summary, false));
        assert!(nonhidden_text(
            &projection.component.root,
            "local-input-visible",
            false
        ));
        for name in ["onReadEngine", "onRetryNativeAck", "onCancelTransitionJob"] {
            assert!(
                projection
                    .callback_bindings()
                    .values()
                    .any(|handler| handler.reference.binding == name)
            );
        }
        assert_eq!(
            native.renderer.document.get_element_by_id(&semantic_id),
            Some(original_video_node)
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
    root = status_baseline;
    let props = blueprint
        .workspace_props(&native, WorkspaceRoute::Viewing, generation, root.clone())
        .unwrap();
    let bindings = owner.borrow().bindings().unwrap();
    native
        .update_module_props_with_callbacks(props, bindings)
        .unwrap();
    for (step, observed) in [false, true, false].into_iter().enumerate() {
        if step != 0 {
            root.insert("nativeObservationPresent".into(), Value::Bool(observed));
            let props = blueprint
                .workspace_props(&native, WorkspaceRoute::Viewing, generation, root.clone())
                .unwrap();
            let bindings = owner.borrow().bindings().unwrap();
            native
                .update_module_props_with_callbacks(props, bindings)
                .unwrap();
        }
        assert_eq!(native.callback_scope(), Some(&scope));
        let guides: Vec<_> = native
            .elements()
            .iter()
            .filter(|element| {
                element.attributes.get("aria-label").map(String::as_str) == Some("시청 시작")
            })
            .collect();
        if observed {
            assert!(guides.is_empty());
            assert!(!removed_paths.is_empty());
            assert!(removed_paths.iter().all(|path| {
                native
                    .elements()
                    .iter()
                    .all(|element| &element.path != path)
            }));
            assert!(!native.elements().iter().any(|element| {
                element.attributes.get("class").is_some_and(|class| {
                    class
                        .split_whitespace()
                        .any(|name| name == "aw-start-content")
                })
            }));
        } else {
            assert_eq!(guides.len(), 1);
            assert_eq!(guides[0].dom_tag, "section");
            let prefix = format!("{}/", guides[0].path);
            removed_paths = native
                .elements()
                .iter()
                .filter(|element| {
                    element.path == guides[0].path || element.path.starts_with(&prefix)
                })
                .map(|element| element.path.clone())
                .collect();
            assert!(
                removed_paths.len() > 1,
                "observe the actual start subtree, not just a placeholder"
            );
        }
        let videos: Vec<_> = native
            .elements()
            .iter()
            .filter(|element| element.dom_tag == "video")
            .collect();
        assert_eq!(videos.len(), 1);
        assert_eq!(videos[0].id, semantic_id);
        assert_eq!(videos[0].path, original_video.path);
        assert!(!videos[0].attributes.contains_key("src"));
        let projection = native.module_projection().unwrap();
        let callbacks = projection.callback_bindings();
        assert_eq!(callbacks.len(), original_callbacks.len());
        for original in original_callbacks.values() {
            assert!(
                callbacks.values().any(|current| {
                    current.reference == original.reference
                        && current.source == original.source
                        && current.captured_text == original.captured_text
                }),
                "original callback contract must survive step {step}"
            );
        }
        let seek_path = callbacks
            .iter()
            .find(|(_, handler)| handler.reference.binding == "onNativeSeek")
            .map(|(path, _)| path.clone())
            .unwrap();
        native
            .dispatch(Action::Activate { path: seek_path })
            .unwrap();
        let pending = native.drain_callbacks();
        assert_eq!(pending.len(), 1);
        assert_eq!(
            pending[0].invocation.arguments,
            CallbackArguments::Text("00:00:00.000037".into())
        );
        // Exercise original DOM/export capture only; never dispatch into the SDK owner.
        assert!(owner.borrow().tickets().unwrap().is_empty());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
    finish(&RouteFixture {
        execution,
        owner,
        _document: native,
        scope,
    });
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

fn observed(id: u64) -> EpisodePlaybackObservation {
    serde_json::from_value(serde_json::json!({
        "engine_instance_id":id.to_string(), "resolution_artifact_id":"a".repeat(64),
        "playback":{"origin":"native_observation","published_age_ms":"0","connected":true,
            "paused":true,"idle":false,"position_us":"10","duration_us":"100",
            "volume_percent":50.0,"muted":false,"video_size":null,"audio_channels":null,"has_error":false},
        "diagnostics":null,"options":null,
    })).unwrap()
}
#[test]
fn range_draft_changes_only_preview_and_terminal_keeps_exact_original_sdk_payload() {
    use crate::assembly::meari::viewer_read_projection::native_playback::{
        NativePlaybackProjection, typed_payload,
    };
    let mut projection = NativePlaybackProjection::default();
    projection.observe(observed(7));
    assert!(projection.range_control(true, "500000").is_err());
    projection.set_range_draft(true, "500000").unwrap();
    assert_eq!(
        projection.props()["nativePositionUs"],
        Value::String("10".into())
    );
    let payload = typed_payload(&projection.range_control(true, "500000").unwrap());
    assert_eq!(payload["engine_instance_id"], serde_json::json!("7"));
    assert_eq!(payload["position_us"], serde_json::json!("50"));
    assert!(projection.range_control(true, "500001").is_err());
    projection.set_range_draft(false, "655").unwrap();
    assert_eq!(
        projection.props()["nativeVolumePercent"],
        Value::String("50".into())
    );
    let payload = typed_payload(&projection.range_control(false, "655").unwrap());
    assert_eq!(payload["volume_percent"], serde_json::json!(65.5));
    assert_eq!(payload["engine_instance_id"], serde_json::json!("7"));
}
#[test]
fn basic_native_status_retains_first_typed_failure_and_separates_unknown_from_error() {
    use crate::assembly::meari::viewer_read_projection::native_playback::NativePlaybackProjection;
    let mut projection = NativePlaybackProjection::default();
    assert_eq!(projection.props()["nativeStatusError"], Value::Bool(false));
    assert_eq!(
        projection.props()["nativeStatusUncertain"],
        Value::Bool(false)
    );
    projection.control_failed("original failure".into(), false);
    projection.control_failed("later unknown acceptance".into(), true);
    assert_eq!(
        projection.props()["nativeStatusSummary"],
        Value::String("original failure".into())
    );
    assert_eq!(projection.props()["nativeStatusError"], Value::Bool(true));
    assert_eq!(
        projection.props()["nativeStatusUncertain"],
        Value::Bool(true)
    );
    projection.observe(observed(7));
    assert_eq!(projection.props()["nativeStatusError"], Value::Bool(false));
    let mut error_observation = observed(7);
    error_observation.playback.has_error = true;
    projection.observe(error_observation);
    assert_eq!(projection.props()["nativeStatusError"], Value::Bool(true));
    assert_eq!(
        projection.props()["nativeStatusUncertain"],
        Value::Bool(false)
    );
}
#[test]
fn controls_idle_requires_actual_playing_and_rejects_pause_error_disconnected_or_unknown() {
    use crate::assembly::meari::viewer_read_projection::native_playback::NativePlaybackProjection;
    let mut projection = NativePlaybackProjection::default();
    assert!(!projection.permits_controls_idle());
    projection.observe(observed(7));
    assert!(!projection.permits_controls_idle());
    let mut playing = observed(7);
    playing.playback.paused = Some(false);
    projection.observe(playing.clone());
    assert!(projection.permits_controls_idle());
    projection.control_failed("original failure".into(), true);
    assert!(!projection.permits_controls_idle());
    for mutation in 0..4 {
        let mut state = playing.clone();
        match mutation {
            0 => state.playback.paused = None,
            1 => state.playback.idle = None,
            2 => state.playback.has_error = true,
            _ => state.playback.connected = false,
        }
        projection.observe(state);
        assert!(!projection.permits_controls_idle());
    }
}
#[test]
fn local_range_input_error_publishes_original_typed_diagnostic_without_sdk_submission() {
    let fixture = route_fixture();
    install_observation(&fixture.owner.borrow(), 7);
    let bindings = fixture.owner.borrow().bindings().unwrap();
    let input = PendingCallback {
        handle: bindings
            .resolve(&CallbackReference {
                binding: "onNativeSeekRangeDraft".into(),
                signature: CallbackSignature::Text,
            })
            .unwrap(),
        invocation: CallbackInvocation {
            prop: "onNativeSeekRangeDraft".into(),
            arguments: CallbackArguments::Text("invalid-coordinate".into()),
            span: Span::default(),
        },
    };
    fixture
        .owner
        .borrow_mut()
        .queue_local(&fixture.scope, &input)
        .unwrap();
    fixture
        .owner
        .borrow_mut()
        .drain_local(&fixture.scope)
        .unwrap();
    let owner = fixture.owner.borrow();
    let original = owner.local_error().unwrap();
    assert_eq!(original.code, PluginErrorCode::InvalidPayload);
    let props = owner.local_props();
    assert_eq!(props["localInputErrorPresent"], Value::Bool(true));
    assert_eq!(
        props["localInputError"],
        Value::String(format!(
            "최근 입력 오류 · {:?}: {}",
            original.code, original.message
        ))
    );
    assert!(owner.tickets().unwrap().is_empty());
    drop(owner);
    finish(&fixture);
}
#[test]
fn range_terminal_rejects_original_context_changes_and_disabled_admission_without_sdk_ticket() {
    use crate::assembly::meari::viewer_read_projection::native_playback::NativePlaybackProjection;
    for mutation in 0..4 {
        let mut projection = NativePlaybackProjection::default();
        projection.observe(observed(7));
        projection.set_range_draft(true, "500000").unwrap();
        let mut next = observed(7);
        match mutation {
            0 => next.engine_instance_id.0 = 8,
            1 => next.resolution_artifact_id = "b".repeat(64),
            2 => next.playback.duration_us = Some(crate::domain::api_values::DecimalU64(101)),
            _ => next.playback.connected = false,
        }
        projection.observe(next);
        assert!(projection.range_control(true, "500000").is_err());
    }
    let fixture = route_fixture();
    install_observation(&fixture.owner.borrow(), 7);
    let before = fixture.owner.borrow().tickets().unwrap().len();
    let owner = fixture.owner.borrow();
    let mut projection = owner.projection.borrow_mut();
    projection.native_admission.set(false);
    let props = projection.props().unwrap();
    for key in [
        "nativeControlsDisabled",
        "nativeSeekRangeDisabled",
        "nativeVolumeRangeDisabled",
    ] {
        assert_eq!(props.get(key), Some(&Value::Bool(true)), "{key}");
    }
    assert!(projection.set_native_range_draft(true, "500000").is_err());
    drop(projection);
    assert_eq!(owner.tickets().unwrap().len(), before);
    drop(owner);
    finish(&fixture);
}
#[test]
fn range_feedback_requires_original_context_and_never_falls_back_after_retained_failure() {
    use crate::assembly::meari::viewer_read_projection::range_feedback::RangeFeedback;
    use meari_native::infrastructure::native_host::HostStamp;
    let fixture = route_fixture();
    let stamp = HostStamp {
        host_id: 1,
        owner: fixture.scope.clone(),
        definition_revision: 1,
        document_revision: 1,
        turn_sequence: 1,
    };
    let original = BTreeMap::from([
        ("nativeSeekRangeDisabled".into(), Value::Bool(false)),
        ("nativeVolumeRangeDisabled".into(), Value::Bool(false)),
        ("nativeSeekRangeValue".into(), Value::String("0".into())),
        ("nativeInstanceId".into(), Value::String("7".into())),
        ("nativeResolutionId".into(), Value::String("a".repeat(64))),
        ("nativeDurationUs".into(), Value::String("100".into())),
        (
            "nativeSeekDraft".into(),
            Value::String("00:00:00.000037".into()),
        ),
    ]);
    let mut feedback = RangeFeedback::default();
    assert!(!feedback.retained(&stamp, &original).unwrap());
    feedback.committed(stamp.clone(), original.clone());
    let mut draft = original.clone();
    draft.insert(
        "nativeSeekRangeValue".into(),
        Value::String("500000".into()),
    );
    assert!(feedback.retained(&stamp, &draft).unwrap());
    for key in [
        "nativeInstanceId",
        "nativeResolutionId",
        "nativeDurationUs",
        "nativeSeekDraft",
    ] {
        let mut changed = draft.clone();
        changed.insert(key.into(), Value::String("changed".into()));
        assert!(!feedback.retained(&stamp, &changed).unwrap(), "{key}");
    }
    let mut disabled = draft.clone();
    disabled.insert("nativeSeekRangeDisabled".into(), Value::Bool(true));
    assert!(!feedback.retained(&stamp, &disabled).unwrap());
    let mut foreign = stamp.clone();
    foreign.host_id += 1;
    assert!(!feedback.retained(&foreign, &draft).unwrap());
    feedback.failed("original retained error".into());
    feedback.failed("secondary error".into());
    assert_eq!(
        feedback.retained(&stamp, &draft),
        Err("original retained error".into())
    );
    assert_eq!(
        feedback.retained(&foreign, &disabled),
        Err("original retained error".into())
    );
    finish(&fixture);
}
#[test]
fn original_range_draft_callback_submits_zero_tickets_then_terminal_submits_one_control() {
    let fixture = route_fixture();
    install_observation(&fixture.owner.borrow(), 7);
    let bindings = fixture.owner.borrow().bindings().unwrap();
    let pending = |name: &str, value: &str| PendingCallback {
        handle: bindings
            .resolve(&CallbackReference {
                binding: name.into(),
                signature: CallbackSignature::Text,
            })
            .unwrap(),
        invocation: CallbackInvocation {
            prop: name.into(),
            arguments: CallbackArguments::Text(value.into()),
            span: Span::default(),
        },
    };
    let draft = pending("onNativeSeekRangeDraft", "500000");
    for value in ["100000", "200000", "300000", "400000"] {
        fixture
            .owner
            .borrow_mut()
            .queue_local(&fixture.scope, &pending("onNativeSeekRangeDraft", value))
            .unwrap();
    }
    assert!(
        matches!(fixture.owner.borrow_mut().queue_local(&fixture.scope, &draft),
        Err(meari::application::host_callbacks::LocalDispatchError::Invalid(error)) if error.code == PluginErrorCode::Busy)
    );
    let progress = fixture
        .owner
        .borrow_mut()
        .drain_local(&fixture.scope)
        .unwrap();
    assert!(
        !progress.refreshed,
        "Draft feedback must not reset automatic SDK refresh policy"
    );
    fixture
        .owner
        .borrow_mut()
        .queue_local(&fixture.scope, &draft)
        .unwrap();
    let progress = fixture
        .owner
        .borrow_mut()
        .drain_local(&fixture.scope)
        .unwrap();
    assert!(!progress.refreshed);
    assert!(fixture.owner.borrow().local_error().is_none());
    assert!(fixture.owner.borrow().tickets().unwrap().is_empty());
    assert_eq!(
        fixture
            .owner
            .borrow()
            .projection
            .borrow()
            .native
            .borrow()
            .props()["nativePositionUs"],
        Value::String("10".into())
    );
    let receipt = fixture
        .owner
        .borrow()
        .dispatch(&fixture.scope, &pending("onNativeSeekCommit", "500000"))
        .unwrap();
    assert_eq!(receipt.ticket.capability(), "playback.seek");
    let status = fixture
        .owner
        .borrow()
        .projection
        .borrow()
        .native
        .borrow()
        .props();
    assert_eq!(status["nativeStatusError"], Value::Bool(false));
    assert_eq!(
        status["nativeStatusSummary"],
        Value::String("명령 처리 결과를 확인 중입니다. 같은 요청을 다시 보내지 마세요.".into())
    );
    assert_eq!(fixture.owner.borrow().tickets().unwrap().len(), 1);
    assert!(
        fixture
            .owner
            .borrow()
            .dispatch(&fixture.scope, &pending("onNativeSeekCommit", "500000"))
            .is_err()
    );
    assert_eq!(fixture.owner.borrow().tickets().unwrap().len(), 1);
    let deadline = Instant::now() + Duration::from_secs(2);
    while fixture
        .owner
        .borrow()
        .tickets()
        .unwrap()
        .contains(&receipt.ticket)
    {
        fixture.owner.borrow_mut().consume_reads().unwrap();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(fixture.owner.borrow().tickets().unwrap().is_empty());
    assert!(matches!(
        fixture
            .owner
            .borrow()
            .projection
            .borrow()
            .retained_event(&receipt.ticket)
            .unwrap()
            .event,
        PluginEventKind::Failed(_)
    ));
    finish(&fixture);
}
#[test]
fn human_native_labels_use_only_connected_observations_and_leave_raw_sdk_units_intact() {
    let mut projection = crate::assembly::meari::viewer_read_projection::native_playback::NativePlaybackProjection::default();
    assert_eq!(
        projection.props()["nativePositionLabel"],
        Value::String("미확인".into())
    );
    assert_eq!(
        projection.props()["nativeVolumeLabel"],
        Value::String("미확인".into())
    );
    let mut value = observed(7);
    value.playback.position_us = Some(crate::domain::api_values::DecimalU64(3_723_456_789));
    value.playback.volume_percent = Some(51.5);
    projection.observe(value.clone());
    assert_eq!(
        projection.props()["nativePositionLabel"],
        Value::String("01:02:03.456".into())
    );
    assert_eq!(
        projection.props()["nativePositionUs"],
        Value::String("3723456789".into())
    );
    assert_eq!(
        projection.props()["nativeVolumeLabel"],
        Value::String("51.5%".into())
    );
    projection.set_draft(true, "00:00:01.250".into());
    projection.observe(value.clone());
    assert_eq!(
        projection.props()["nativeSeekDraft"],
        Value::String("00:00:01.250".into())
    );
    let mut next_instance = value.clone();
    next_instance.engine_instance_id = crate::domain::api_values::DecimalU64(8);
    projection.observe(next_instance);
    assert_eq!(
        projection.props()["nativeSeekDraft"],
        Value::String(String::new())
    );
    value.playback.connected = false;
    projection.observe(value);
    assert_eq!(
        projection.props()["nativePositionLabel"],
        Value::String("미확인".into())
    );
    assert_eq!(
        projection.props()["nativeDurationLabel"],
        Value::String("미확인".into())
    );
    assert_eq!(
        projection.props()["nativeVolumeLabel"],
        Value::String("미확인".into())
    );
}
#[test]
fn human_seek_uses_exact_microseconds_and_original_instance_without_submitting_invalid_text() {
    use crate::assembly::meari::viewer_read_projection::native_playback::{
        NativePlaybackProjection, typed_payload,
    };
    let mut projection = NativePlaybackProjection::default();
    projection.observe(observed(7));
    let input = projection
        .control(PlaybackControl::Seek {
            position_us: crate::domain::api_values::DecimalU64(
                crate::presentation::playback_time::parse_seek_time("00:00:00.000037").unwrap(),
            ),
        })
        .unwrap();
    let payload = typed_payload(&input);
    assert_eq!(payload["position_us"], serde_json::json!("37"));
    assert_eq!(payload["engine_instance_id"], serde_json::json!("7"));
    let fixture = route_fixture();
    install_observation(&fixture.owner.borrow(), 7);
    let bindings = fixture.owner.borrow().bindings().unwrap();
    let handle = bindings
        .resolve(&CallbackReference {
            binding: "onNativeSeek".into(),
            signature: CallbackSignature::Text,
        })
        .unwrap();
    let before = fixture.owner.borrow().tickets().unwrap().len();
    for value in ["37", "00:60:00", "5124095576:01:49.551616"] {
        let pending = PendingCallback {
            handle: handle.clone(),
            invocation: CallbackInvocation {
                prop: "onNativeSeek".into(),
                arguments: CallbackArguments::Text(value.into()),
                span: Span::default(),
            },
        };
        assert!(
            fixture
                .owner
                .borrow()
                .dispatch(&fixture.scope, &pending)
                .is_err()
        );
        assert_eq!(fixture.owner.borrow().tickets().unwrap().len(), before);
    }
    finish(&fixture);
}
fn install_observation(owner: &ViewerCallbacks, id: u64) {
    let projection = owner.projection.borrow();
    projection.native.borrow_mut().observe(observed(id));
    projection.native_admission.set(true);
}
fn finish(fixture: &RouteFixture) {
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
#[test]
fn sdk_failed_native_ticket_retires_after_ack_then_new_observation_allows_new_instance() {
    let fixture = route_fixture();
    install_observation(&fixture.owner.borrow(), 7);
    let (_bindings, play) = local_input(&fixture.owner.borrow(), "onNativePlay");
    let receipt = fixture
        .owner
        .borrow()
        .dispatch(&fixture.scope, &play)
        .unwrap();
    assert!(
        fixture
            .owner
            .borrow()
            .projection
            .borrow()
            .native
            .borrow()
            .control(PlaybackControl::Play)
            .is_err()
    );
    let deadline = Instant::now() + Duration::from_secs(2);
    while fixture
        .owner
        .borrow()
        .tickets()
        .unwrap()
        .contains(&receipt.ticket)
    {
        fixture.owner.borrow_mut().consume_reads().unwrap();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    // This fixture's SDK has no native backend. The real SDK error must be retained.
    assert!(matches!(
        fixture
            .owner
            .borrow()
            .projection
            .borrow()
            .retained_event(&receipt.ticket)
            .unwrap()
            .event,
        PluginEventKind::Failed(_)
    ));
    install_observation(&fixture.owner.borrow(), 8);
    assert_eq!(
        fixture
            .owner
            .borrow()
            .projection
            .borrow()
            .native
            .borrow()
            .control(PlaybackControl::Pause)
            .unwrap()
            .engine_instance_id
            .0,
        8
    );
    finish(&fixture);
}
#[test]
fn late_a_control_receipt_does_not_change_observed_b_state() {
    let fixture = route_fixture();
    install_observation(&fixture.owner.borrow(), 11);
    let (_bindings, play) = local_input(&fixture.owner.borrow(), "onNativePlay");
    let receipt = fixture
        .owner
        .borrow()
        .dispatch(&fixture.scope, &play)
        .unwrap();
    install_observation(&fixture.owner.borrow(), 12);
    {
        let owner = fixture.owner.borrow();
        let projection = owner.projection.borrow();
        let mut native = projection.native.borrow_mut();
        native
            .receipt(
                &receipt.ticket,
                crate::application::playback_control::ControlReceipt::enqueued(11),
            )
            .unwrap();
        assert_eq!(
            native
                .control(PlaybackControl::Pause)
                .unwrap()
                .engine_instance_id
                .0,
            12
        );
        assert_eq!(
            native.props()["nativePositionUs"],
            Value::String("10".into())
        );
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    while fixture
        .owner
        .borrow()
        .tickets()
        .unwrap()
        .contains(&receipt.ticket)
    {
        fixture.owner.borrow_mut().consume_reads().unwrap();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    finish(&fixture);
}

#[test]
fn projection_only_ack_failure_keeps_original_ticket_until_same_terminal_is_acked() {
    use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::TerminalAcceptance;
    let fixture = route_fixture();
    install_observation(&fixture.owner.borrow(), 21);
    let (_bindings, play) = local_input(&fixture.owner.borrow(), "onNativePlay");
    let receipt = fixture
        .owner
        .borrow()
        .dispatch(&fixture.scope, &play)
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let accepted = loop {
        if let Some(accepted) = fixture
            .owner
            .borrow_mut()
            .accept_terminal(&receipt.ticket)
            .unwrap()
        {
            break accepted;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    };
    assert!(accepted.owner_ack.is_ok());
    // Synthetic UI envelope fault only. Actual SDK ACK above succeeded; this test does not
    // claim a transport ACK failure, cancellation, or native engine execution.
    fixture
        .owner
        .borrow()
        .projection
        .borrow_mut()
        .retain(TerminalAcceptance {
            event: accepted.event.clone(),
            owner_ack: Err(PluginError::new(
                PluginErrorCode::Busy,
                "synthetic projection ACK failure",
            )),
        });
    install_observation(&fixture.owner.borrow(), 22);
    assert!(
        fixture
            .owner
            .borrow()
            .projection
            .borrow()
            .native
            .borrow()
            .control(PlaybackControl::Play)
            .is_err()
    );
    fixture
        .owner
        .borrow()
        .projection
        .borrow_mut()
        .retain(accepted);
    install_observation(&fixture.owner.borrow(), 22);
    assert_eq!(
        fixture
            .owner
            .borrow()
            .projection
            .borrow()
            .native
            .borrow()
            .control(PlaybackControl::Play)
            .unwrap()
            .engine_instance_id
            .0,
        22
    );
    assert!(fixture.owner.borrow().tickets().unwrap().is_empty());
    finish(&fixture);
}
