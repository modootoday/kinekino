//! Model port tests: ordering/identity only, never native cleanup or frame proof.
use super::*;
use crate::application::route_media::{RouteMediaError, RouteMediaState};
use std::cell::{Cell, RefCell};

pub(crate) struct TestPort {
    state: RefCell<RouteMediaState<PluginScope>>,
    pub(crate) retire_ready: Cell<bool>,
    pub(crate) viewing_ready: Cell<bool>,
    pub(crate) begins: Cell<usize>,
    fail: RefCell<Option<RouteMediaError>>,
}
impl TestPort {
    pub(crate) fn new(scope: PluginScope) -> Self {
        Self {
            state: RefCell::new(RouteMediaState::new(scope)),
            retire_ready: Cell::new(false),
            viewing_ready: Cell::new(false),
            begins: Cell::new(0),
            fail: RefCell::new(None),
        }
    }
}
impl RouteMediaPort<PluginScope> for TestPort {
    fn begin_retire(
        &self,
        owner: &PluginScope,
    ) -> Result<RetireTicket<PluginScope>, RouteMediaError> {
        self.begins.set(self.begins.get() + 1);
        self.state.borrow_mut().begin_retire(owner)
    }
    fn poll_retire(
        &self,
        ticket: &RetireTicket<PluginScope>,
    ) -> Result<RouteMediaProgress, RouteMediaError> {
        let mut state = self.state.borrow_mut();
        if let Some(error) = self.fail.borrow().clone() {
            state.fail_retire(ticket, error)?;
        }
        if self.retire_ready.get() {
            state.complete_retire(ticket)?;
        }
        state.poll_retire(ticket)
    }
    fn cancel_retire(&self, ticket: &RetireTicket<PluginScope>) -> Result<(), RouteMediaError> {
        self.state.borrow_mut().cancel_retire(ticket)
    }
    fn begin_viewing(
        &self,
        owner: &PluginScope,
        retired: &RetireTicket<PluginScope>,
    ) -> Result<ViewingTicket<PluginScope>, RouteMediaError> {
        self.state.borrow_mut().begin_viewing(owner, retired)
    }
    fn poll_viewing(
        &self,
        ticket: &ViewingTicket<PluginScope>,
    ) -> Result<RouteMediaProgress, RouteMediaError> {
        let mut state = self.state.borrow_mut();
        if self.viewing_ready.get() {
            state.complete_viewing(ticket)?;
        }
        state.poll_viewing(ticket)
    }
}
fn scope() -> PluginScope {
    let component = meari::parse(
        "type Props={};export function Scope(props:Props){return <main/>;}",
        meari::application::Language::Tsx,
    )
    .unwrap();
    let program = meari::application::state_program::StateProgram::mount(
        component,
        "native-route-gate-model".into(),
        1,
    )
    .unwrap();
    PluginScope {
        instance: program.snapshot().unwrap().instance,
        source_generation: 1,
    }
}
#[test]
fn same_ticket_pending_complete_and_fresh_viewing_generation_are_distinct() {
    let scope = scope();
    let port = Rc::new(TestPort::new(scope.clone()));
    let mut gate = NativeRouteGate::default();
    gate.install(port.clone()).unwrap();
    assert!(!gate.retire(&scope).unwrap());
    let original = gate.retirement.as_ref().unwrap().clone();
    assert!(!gate.retire(&scope).unwrap());
    assert_eq!(port.begins.get(), 1);
    assert!(gate.restore_viewing(&scope).is_err());
    port.retire_ready.set(true);
    assert!(gate.retire(&scope).unwrap());
    let mut foreign_scope = scope.clone();
    foreign_scope.source_generation += 1;
    assert!(gate.retire(&foreign_scope).is_err());
    assert!(gate.restore_viewing(&foreign_scope).is_err());
    assert!(!gate.restore_viewing(&scope).unwrap());
    assert!(
        gate.retire(&scope).is_err(),
        "Pending fresh registration cannot admit another Studio entry"
    );
    assert!(
        port.poll_retire(&original).is_err(),
        "Old ticket cannot authorize a fresh registration"
    );
    let foreign = TestPort::new(scope.clone());
    assert!(foreign.poll_retire(&original).is_err());
    port.viewing_ready.set(true);
    assert!(gate.restore_viewing(&scope).unwrap());
    let fresh = gate.viewing.as_ref().unwrap().generation();
    assert!(fresh > original.generation());
    port.retire_ready.set(false);
    assert!(!gate.retire(&scope).unwrap());
    assert_eq!(port.begins.get(), 2);
    assert_ne!(
        gate.retirement.as_ref().unwrap().generation(),
        original.generation()
    );
}
#[test]
fn native_failure_retains_first_error_and_ticket_without_implicit_headless_fallback() {
    let scope = scope();
    assert!(NativeRouteGate::default().retire(&scope).is_err());
    let port = Rc::new(TestPort::new(scope.clone()));
    let mut gate = NativeRouteGate::default();
    gate.install(port.clone()).unwrap();
    assert!(!gate.retire(&scope).unwrap());
    let generation = gate.retirement.as_ref().unwrap().generation();
    let original = Arc::new(ApiError::new(
        ErrorCode::NeedsInput,
        "Synthetic native cleanup failure",
    ));
    *port.fail.borrow_mut() = Some(original.clone());
    assert!(Arc::ptr_eq(&gate.retire(&scope).unwrap_err(), &original));
    *port.fail.borrow_mut() = None;
    port.retire_ready.set(true);
    assert!(Arc::ptr_eq(&gate.retire(&scope).unwrap_err(), &original));
    assert_eq!(gate.retirement.as_ref().unwrap().generation(), generation);
    assert!(gate.declare_headless().is_err());
    assert_eq!(port.begins.get(), 1);
}
