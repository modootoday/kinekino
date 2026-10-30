use super::*;
use crate::{
    application::api::PlayerSdk,
    assembly::meari::{player_shell_startup::ShellInputs, viewer_cleanup::ViewerNoChild},
    presentation::web_ui_adapter::player_shell::{
        NavigationGuards, RouteCatalog, RouteTicket, ShellPhase, ShellState,
    },
};
use meari_native::{
    domain::Viewport,
    infrastructure::{document::InteractiveDocument, native_host::*},
};
use std::{collections::BTreeMap, sync::Arc, time::Duration};

struct SharedHost {
    route: Option<Rc<RefCell<StartRouteObservation>>>,
}
impl NativeWindowHost for SharedHost {
    type Error = std::io::Error;
    type CloseCheckpoint = ();
    fn admit_callback(
        &mut self,
        _: &HostStamp,
        _: &meari::application::host_callbacks::PendingCallback,
    ) -> IntentAdmission<Self::Error> {
        IntentAdmission::Rejected(std::io::Error::other("Fixture exports no callbacks"))
    }
    fn after_dispatch(&mut self, _: HostTurn) -> Result<HostEffects, Self::Error> {
        Ok(HostEffects::default())
    }
    fn close(&mut self, _: CloseRequest<()>) -> CloseDecision<(), Self::Error> {
        CloseDecision::HostCleanupComplete { checkpoint: () }
    }
}
fn prepared() -> (PreparedShellDocument<SharedHost>, RouteTicket) {
    let catalog = RouteCatalog::bundled().unwrap();
    let mut state = ShellState::new();
    let ticket = state
        .request_route(&catalog, "view.start", NavigationGuards::default())
        .unwrap();
    let document = InteractiveDocument::new(
        meari::parse(
            "type Props={};export function Frame(props:Props){return <main/>;}",
            meari::application::Language::Tsx,
        )
        .unwrap(),
        "",
        Viewport {
            width: 160,
            height: 80,
            dark: true,
        },
    )
    .unwrap();
    (
        PreparedShellDocument {
            inputs: ShellInputs {
                client: Arc::new(PlayerSdk::default()),
                host_owner: SharedHost { route: None },
            },
            state,
            ticket: ticket.clone(),
            document: NativeDocumentHandle::new(document, 1).unwrap(),
            style_hashes: BTreeMap::new(),
        },
        ticket,
    )
}
fn config() -> ShellHostConfig {
    ShellHostConfig {
        options: WindowOptions::default(),
        children: LifecycleLimits::new(1, Duration::from_secs(1)).unwrap(),
        schedule: HostScheduleLimits::new(
            Duration::from_millis(2),
            Duration::from_millis(250),
            100,
        )
        .unwrap(),
    }
}

#[test]
fn construction_shares_the_original_state_and_original_ticket_without_runtime_attach() {
    let (prepared, ticket) = prepared();
    let document_id = prepared.document.id();
    let calls = Rc::new(Cell::new(0));
    let invoked = calls.clone();
    let controller: ShellHostController<SharedHost, ViewerNoChild> =
        ShellHostController::new_with_route_binding(prepared, config(), move |host, route| {
            invoked.set(invoked.get() + 1);
            host.route = Some(route);
        });
    assert_eq!(calls.get(), 1);
    assert!(controller.runtime.is_none());
    assert_eq!(
        controller.pending_document_for_test().unwrap().id(),
        document_id
    );
    let shared = controller.pending_owner().unwrap().route.as_ref().unwrap();
    assert!(Rc::ptr_eq(shared, &controller.route));
    assert_eq!(shared.borrow().state.phase(), ShellPhase::AwaitingHost);
    shared.borrow_mut().state.cancel_route(&ticket).unwrap();
    assert_eq!(
        controller.route.borrow().state.phase(),
        ShellPhase::NoSelection
    );
    assert!(controller.route.borrow().state.observed_route().is_none());
}

#[test]
fn legacy_constructor_retains_pending_owner_and_original_route_without_opt_in_binding() {
    let (prepared, _) = prepared();
    let document_id = prepared.document.id();
    let controller: ShellHostController<SharedHost, ViewerNoChild> =
        ShellHostController::new(prepared, config());
    assert!(controller.pending_owner().unwrap().route.is_none());
    assert_eq!(
        controller.pending_document_for_test().unwrap().id(),
        document_id
    );
    assert_eq!(
        controller.route.borrow().state.phase(),
        ShellPhase::AwaitingHost
    );
    assert!(controller.runtime.is_none());
}
