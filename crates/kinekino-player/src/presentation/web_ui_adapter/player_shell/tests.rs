use super::super::{
    availability::*,
    routes::{CatalogError, RouteKind},
};
use super::*;
fn catalog() -> RouteCatalog {
    RouteCatalog::bundled().unwrap()
}
fn report(state: &mut ShellState, catalog: &RouteCatalog, id: &str) {
    let ticket = state
        .request_route(catalog, id, NavigationGuards::default())
        .unwrap();
    state
        .observe_route(
            &ticket,
            RouteHostObservation::Reported {
                route: catalog.get(id).unwrap().id().clone(),
            },
        )
        .unwrap();
}
#[test]
fn bundled_catalog_keeps_declared_counts_and_unknown_routes_absent() {
    let catalog = catalog();
    assert_eq!(
        catalog
            .iter()
            .filter(|route| route.kind() == RouteKind::Viewing)
            .count(),
        3
    );
    assert_eq!(
        catalog
            .iter()
            .filter(|route| route.kind() == RouteKind::Studio)
            .count(),
        20
    );
    assert_eq!(
        catalog
            .iter()
            .filter(|route| route.kind() == RouteKind::Studio && route.source().is_some())
            .count(),
        5
    );
    assert!(catalog.get("settings.general").is_none());
    assert!(catalog.get("unknown").is_none());
    assert_eq!(catalog.get("view.start").unwrap().path(), "/");
}
#[test]
fn catalog_rejects_duplicate_unknown_fields_and_runtime_success_claim() {
    let mut value: serde_json::Value =
        serde_json::from_str(include_str!("../../web_ui/routing/routes.json")).unwrap();
    let entry = value["view_routes"][0].clone();
    value["view_routes"].as_array_mut().unwrap().push(entry);
    assert!(matches!(
        RouteCatalog::parse(&value.to_string()),
        Err(CatalogError::DuplicateRoute)
    ));
    value["view_routes"].as_array_mut().unwrap().pop();
    value["status"] = "runtime-registered".into();
    assert!(matches!(
        RouteCatalog::parse(&value.to_string()),
        Err(CatalogError::NotDeclarationCatalog)
    ));
    value["status"] = "product-route-contract-not-runtime-registered".into();
    value["unexpected"] = true.into();
    assert!(matches!(
        RouteCatalog::parse(&value.to_string()),
        Err(CatalogError::InvalidJson)
    ));
}
#[test]
fn initial_and_requested_routes_are_not_host_observations() {
    let catalog = catalog();
    let mut state = ShellState::new();
    assert_eq!(state.phase(), ShellPhase::NoSelection);
    assert!(state.observed_route().is_none());
    let ticket = state
        .request_route(&catalog, "view.start", NavigationGuards::default())
        .unwrap();
    assert_eq!(state.desired_route().unwrap().as_str(), "view.start");
    assert!(state.observed_route().is_none());
    state.cancel_route(&ticket).unwrap();
    assert_eq!(state.phase(), ShellPhase::NoSelection);
    assert!(state.desired_route().is_none());
    assert_eq!(
        state
            .request_route(&catalog, "unknown", NavigationGuards::default())
            .unwrap_err(),
        NavigationError::UnknownRoute
    );
}
#[test]
fn dirty_pending_and_recovery_veto_do_not_change_original_screen() {
    let catalog = catalog();
    let mut state = ShellState::new();
    report(&mut state, &catalog, "view.start");
    for (guards, error) in [
        (
            NavigationGuards {
                dirty: true,
                ..Default::default()
            },
            NavigationError::UnsavedEdits,
        ),
        (
            NavigationGuards {
                pending: true,
                ..Default::default()
            },
            NavigationError::Busy,
        ),
        (
            NavigationGuards {
                recovery_blocked: true,
                ..Default::default()
            },
            NavigationError::RecoveryBlocked,
        ),
    ] {
        assert_eq!(
            state
                .request_route(&catalog, "studio.cards", guards)
                .unwrap_err(),
            error
        );
        assert_eq!(state.request_close(guards), Err(error));
        assert_eq!(state.observed_route().unwrap().as_str(), "view.start");
        assert!(state.desired_route().is_none());
    }
}
#[test]
fn failed_or_cancelled_route_preserves_original_and_late_observations_are_stale() {
    let catalog = catalog();
    let mut state = ShellState::new();
    report(&mut state, &catalog, "view.start");
    let ticket = state
        .request_route(&catalog, "studio.cards", NavigationGuards::default())
        .unwrap();
    state
        .observe_route(
            &ticket,
            RouteHostObservation::Failed {
                diagnostic: "source not integrated".into(),
            },
        )
        .unwrap();
    assert_eq!(state.observed_route().unwrap().as_str(), "view.start");
    assert_eq!(state.diagnostic(), Some("source not integrated"));
    assert_eq!(
        state.observe_route(
            &ticket,
            RouteHostObservation::Reported {
                route: catalog.get("studio.cards").unwrap().id().clone()
            }
        ),
        Err(NavigationError::StaleObservation)
    );
    let next = state
        .request_route(&catalog, "studio.jobs", NavigationGuards::default())
        .unwrap();
    state.cancel_route(&next).unwrap();
    assert_eq!(state.observed_route().unwrap().as_str(), "view.start");
    assert_eq!(
        state.cancel_route(&next),
        Err(NavigationError::StaleObservation)
    );
}
#[test]
fn foreign_and_wrong_route_observations_cannot_consume_pending_request() {
    let catalog = catalog();
    let mut first = ShellState::new();
    let mut second = ShellState::new();
    let ticket = first
        .request_route(&catalog, "view.start", NavigationGuards::default())
        .unwrap();
    let foreign = second
        .request_route(&catalog, "view.start", NavigationGuards::default())
        .unwrap();
    assert_eq!(ticket.generation(), foreign.generation());
    assert_eq!(
        first.cancel_route(&foreign),
        Err(NavigationError::ForeignObservation)
    );
    assert_eq!(
        first.observe_route(
            &ticket,
            RouteHostObservation::Reported {
                route: catalog.get("studio.jobs").unwrap().id().clone()
            }
        ),
        Err(NavigationError::RouteMismatch)
    );
    assert_eq!(first.desired_route().unwrap().as_str(), "view.start");
    assert!(first.observed_route().is_none());
    first
        .observe_route(
            &ticket,
            RouteHostObservation::Reported {
                route: catalog.get("view.start").unwrap().id().clone(),
            },
        )
        .unwrap();
    assert_eq!(first.observed_route().unwrap().as_str(), "view.start");
    assert_eq!(
        first.cancel_route(&ticket),
        Err(NavigationError::StaleObservation)
    );
}
#[test]
fn generation_exhaustion_preserves_screen_and_admits_no_new_intent() {
    let catalog = catalog();
    let mut state = ShellState::new();
    report(&mut state, &catalog, "view.start");
    state.generation = u64::MAX;
    assert_eq!(
        state
            .request_route(&catalog, "studio.jobs", NavigationGuards::default())
            .unwrap_err(),
        NavigationError::GenerationExhausted
    );
    assert!(state.desired_route().is_none());
    assert_eq!(state.observed_route().unwrap().as_str(), "view.start");
}
#[test]
fn support_dimensions_remain_distinct_and_source_is_not_a_mount() {
    let catalog = catalog();
    let route = catalog.get("view.start").unwrap();
    assert_eq!(
        availability(route, Default::default()),
        Availability::Unknown
    );
    let ready = AvailabilityObservation {
        source: SourceObservation::Integrated,
        capability: CapabilityObservation::Supported,
        backend: BackendObservation::Ready,
        mount: MountObservation::ReportedByHost,
        recovery_blocked: false,
    };
    for (observation, expected) in [
        (ready, Availability::Available),
        (
            AvailabilityObservation {
                source: SourceObservation::Unknown,
                ..ready
            },
            Availability::Unknown,
        ),
        (
            AvailabilityObservation {
                capability: CapabilityObservation::Unknown,
                ..ready
            },
            Availability::Unknown,
        ),
        (
            AvailabilityObservation {
                backend: BackendObservation::Unknown,
                ..ready
            },
            Availability::Unknown,
        ),
        (
            AvailabilityObservation {
                mount: MountObservation::Unknown,
                ..ready
            },
            Availability::Unknown,
        ),
        (
            AvailabilityObservation {
                source: SourceObservation::NotIntegrated,
                ..ready
            },
            Availability::SourceNotIntegrated,
        ),
        (
            AvailabilityObservation {
                capability: CapabilityObservation::Unsupported,
                ..ready
            },
            Availability::CapabilityUnsupported,
        ),
        (
            AvailabilityObservation {
                backend: BackendObservation::Unavailable,
                ..ready
            },
            Availability::BackendUnavailable,
        ),
        (
            AvailabilityObservation {
                mount: MountObservation::Unavailable,
                ..ready
            },
            Availability::HostUnavailable,
        ),
        (
            AvailabilityObservation {
                recovery_blocked: true,
                ..ready
            },
            Availability::RecoveryBlocked,
        ),
    ] {
        assert_eq!(availability(route, observation), expected);
    }
    assert_eq!(
        availability(catalog.get("view.watch").unwrap(), ready),
        Availability::SourceMissing
    );
}
#[test]
fn close_and_panel_requests_do_not_create_native_completion_or_routes() {
    let mut state = ShellState::new();
    state.request_panel(ShellPanel::Settings).unwrap();
    assert_eq!(state.intent(), Some(ShellIntent::SettingsRequested));
    assert!(state.observed_route().is_none());
    state.request_close(Default::default()).unwrap();
    assert_eq!(state.intent(), Some(ShellIntent::CloseRequested));
    assert_eq!(state.phase(), ShellPhase::NoSelection);
    assert_eq!(
        state
            .request_route(&catalog(), "view.start", Default::default())
            .unwrap_err(),
        NavigationError::Busy
    );
    assert_eq!(
        state.request_panel(ShellPanel::Episodes),
        Err(NavigationError::Busy)
    );
    state.dismiss_close_request();
    assert_eq!(state.phase(), ShellPhase::NoSelection);
    assert_eq!(state.intent(), None);
}
