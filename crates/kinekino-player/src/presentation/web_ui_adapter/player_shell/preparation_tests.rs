use super::*;

#[test]
fn viewing_roots_share_the_closed_video_id_without_new_source_or_media_props() {
    use crate::presentation::web_ui_adapter::player_shell::document::{
        self, video::ViewingVideoId,
    };
    let catalog = RouteCatalog::bundled().unwrap();
    let mut state = ShellState::new();
    let mut prepared = request(&mut state, &catalog);
    let expected = meari::domain::Value::String(ViewingVideoId::Primary.as_str().into());
    assert_eq!(ViewingVideoId::Primary.as_str(), "viewing-primary-video");
    assert_eq!(
        prepared.document.props.get("videoSemanticId"),
        Some(&expected)
    );
    assert_eq!(prepared.document.inventory.len(), 9);
    let props = document::read_props(&labels());
    assert_eq!(props.get("videoSemanticId"), Some(&expected));
    document::enable_read_panel(&mut prepared.document, props)
        .ok()
        .unwrap();
    assert_eq!(prepared.document.inventory.len(), 16);
    assert!(
        prepared
            .document
            .inventory
            .keys()
            .any(|path| path.as_str() == "components/ui/range.tsx")
    );
    assert!(
        prepared
            .document
            .inventory
            .keys()
            .any(|path| path.as_str() == "viewing/proposal-chooser.tsx")
    );
    let chooser = document::proposal_chooser::component_reference();
    assert!(
        prepared
            .document
            .graph
            .modules
            .contains_key(&chooser.module)
    );
    assert!(matches!(
        prepared.document.props.get("proposalChooser"),
        Some(meari::domain::Value::NodeSlot(
            meari::domain::node_slots::NodeSlot::Empty
        ))
    ));
    assert!(
        prepared
            .document
            .inventory
            .keys()
            .any(|path| path.as_str() == "studio/cards/selection-rail.tsx")
    );
    assert_eq!(
        prepared.document.props.get("videoSemanticId"),
        Some(&expected)
    );
    assert!(!prepared.document.props.contains_key("src"));
    assert!(state.observed_route().is_none());
}

#[test]
fn retry_panel_prepares_local_references_without_sdk_command_payloads() {
    use crate::presentation::web_ui_adapter::player_shell::document::{
        self, enable_read_panel, read_props,
    };
    let mut state = ShellState::new();
    let mut prepared = request(&mut state, &RouteCatalog::bundled().unwrap());
    let props = read_props(&labels());
    for name in [
        "onRetryCapabilitiesAck",
        "onRetryEngineAck",
        "onRetryStatusAck",
        "onRefreshReadProjection",
        "onOpenJobs",
    ] {
        assert!(
            matches!(props.get(name), Some(meari::domain::Value::CallbackReference(reference)) if reference.binding == name)
        );
    }
    enable_read_panel(&mut prepared.document, props)
        .ok()
        .unwrap();
    assert_eq!(prepared.document.inventory.len(), 16);
    assert!(
        prepared
            .document
            .inventory
            .keys()
            .any(|path| path.as_str() == "viewing/proposal-chooser.tsx")
    );
    assert!(
        prepared
            .document
            .graph
            .modules
            .contains_key(&document::proposal_chooser::component_reference().module)
    );
    assert!(
        prepared
            .document
            .inventory
            .keys()
            .any(|path| path.as_str() == "studio/cards/selection-rail.tsx")
    );
}

#[test]
fn interactive_read_root_is_opt_in_and_retains_start_labels() {
    let catalog = RouteCatalog::bundled().unwrap();
    let mut state = ShellState::new();
    let mut prepared = request(&mut state, &catalog);
    let labels = labels();
    let props = crate::presentation::web_ui_adapter::player_shell::document::read_props(&labels);
    assert!(
        crate::presentation::web_ui_adapter::player_shell::document::enable_read_panel(
            &mut prepared.document,
            props
        )
        .is_ok()
    );
    assert!(
        prepared
            .document
            .inventory
            .keys()
            .any(|path| path.as_str() == "viewing/read-panel.tsx")
    );
    assert_eq!(
        prepared.document.props.get("connectionLabel"),
        Some(&meari::domain::Value::String(labels.connection))
    );
}
use crate::presentation::web_ui_adapter::player_shell::{ShellIntent, ShellPhase};

fn labels() -> StartLabels {
    StartLabels {
        connection: "연결 관측 없음".into(),
        time: "관측 없음".into(),
    }
}
fn request(state: &mut ShellState, catalog: &RouteCatalog) -> StartRequest {
    match request_start(state, catalog, labels(), NavigationGuards::default()) {
        Ok(request) => request,
        Err(_) => panic!("fixed viewing declaration should prepare"),
    }
}

#[test]
fn preparation_does_not_observe_or_select_media() {
    let catalog = RouteCatalog::bundled().unwrap();
    let mut state = ShellState::new();
    assert_eq!(state.phase(), ShellPhase::NoSelection);
    let prepared = request(&mut state, &catalog);
    assert_eq!(prepared.document.route().as_str(), "view.start");
    assert_eq!(prepared.document.inventory.len(), 9);
    assert!(state.observed_route().is_none());
    assert_eq!(state.phase(), ShellPhase::AwaitingHost);
}

#[test]
fn missing_and_unintegrated_sources_are_distinct() {
    use super::super::document::prepare_start_document;
    let catalog = RouteCatalog::bundled().unwrap();
    assert!(matches!(
        prepare_start_document(&catalog, "view.watch", labels()),
        Err(DocumentPreparationError::SourceMissing)
    ));
    assert!(matches!(
        prepare_start_document(&catalog, "studio.jobs", labels()),
        Err(DocumentPreparationError::SourceNotIntegrated)
    ));
    assert!(matches!(
        prepare_start_document(&catalog, "viewing.start", labels()),
        Err(DocumentPreparationError::UnknownRoute)
    ));
}

#[test]
fn host_commit_is_explicit_and_wrong_route_is_rejected() {
    let catalog = RouteCatalog::bundled().unwrap();
    let mut state = ShellState::new();
    let request = request(&mut state, &catalog);
    let wrong = catalog.get("view.watch").unwrap().id().clone();
    assert_eq!(
        observe_start(
            &mut state,
            &request.ticket,
            RouteHostObservation::Reported { route: wrong }
        ),
        Err(NavigationError::RouteMismatch)
    );
    assert!(state.observed_route().is_none());
    assert!(
        observe_start(
            &mut state,
            &request.ticket,
            RouteHostObservation::Reported {
                route: request.document.route().clone()
            }
        )
        .is_ok()
    );
    assert_eq!(state.observed_route().unwrap().as_str(), "view.start");
}

#[test]
fn cancelled_ticket_cannot_commit_late() {
    let catalog = RouteCatalog::bundled().unwrap();
    let mut state = ShellState::new();
    let request = request(&mut state, &catalog);
    state.cancel_route(&request.ticket).unwrap();
    assert_eq!(
        observe_start(
            &mut state,
            &request.ticket,
            RouteHostObservation::Reported {
                route: request.document.route().clone()
            }
        ),
        Err(NavigationError::StaleObservation)
    );
    assert_eq!(state.phase(), ShellPhase::NoSelection);
}

#[test]
fn close_intent_blocks_preparation_commit_without_releasing_owner() {
    let catalog = RouteCatalog::bundled().unwrap();
    let mut state = ShellState::new();
    state.request_close(NavigationGuards::default()).unwrap();
    let result = request_start(&mut state, &catalog, labels(), NavigationGuards::default());
    let diagnostic = match &result {
        Err(StartPreparationError::Document(DocumentPreparationError::Source(error))) => {
            format!("source preparation failed before navigation: {error:?}")
        }
        Err(StartPreparationError::Document(_)) => {
            "document preparation failed before navigation".into()
        }
        Err(StartPreparationError::Navigation(error)) => format!("navigation: {error:?}"),
        Ok(_) => "unexpected start preparation admitted after close intent".into(),
    };
    assert!(
        matches!(
            result,
            Err(StartPreparationError::Navigation(NavigationError::Busy))
        ),
        "{diagnostic}"
    );
    assert_eq!(state.intent(), Some(ShellIntent::CloseRequested));
    assert!(state.observed_route().is_none());
}

#[test]
fn failed_host_observation_is_not_ready() {
    let catalog = RouteCatalog::bundled().unwrap();
    let mut state = ShellState::new();
    let request = request(&mut state, &catalog);
    assert!(
        observe_start(
            &mut state,
            &request.ticket,
            RouteHostObservation::Failed {
                diagnostic: "attach unobserved".into()
            }
        )
        .is_ok()
    );
    assert_eq!(state.phase(), ShellPhase::Failed);
    assert!(state.observed_route().is_none());
}
