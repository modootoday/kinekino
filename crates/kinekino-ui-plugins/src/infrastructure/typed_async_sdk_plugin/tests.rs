use super::*;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use meari::{
    application::{Language, plugins::*, state_program::StateProgram},
    domain::build::{ContentHash, TargetKind},
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
    sync::Arc,
};

#[derive(Default)]
struct Probe {
    tickets: Vec<PluginTicket>,
    terminal: Option<Arc<Result<String, ApiError>>>,
    starts: usize,
    acks: usize,
    cancels: usize,
    shutdowns: usize,
    fail_start: bool,
    fail_ack: bool,
    fail_shutdown: bool,
}
struct Port(Rc<RefCell<Probe>>);
impl AsyncSdkPort<String, String> for Port {
    fn start(&mut self, ticket: &PluginTicket, _: String) -> Result<(), PluginError> {
        let mut probe = self.0.borrow_mut();
        if probe.fail_start {
            return Err(PluginError::new(
                PluginErrorCode::Busy,
                "pre-admission full",
            ));
        }
        probe.starts += 1;
        probe.tickets.push(ticket.clone());
        Ok(())
    }
    fn peek_terminal(
        &mut self,
        ticket: &PluginTicket,
    ) -> Result<Option<Arc<Result<String, ApiError>>>, PluginError> {
        let probe = self.0.borrow();
        assert!(probe.tickets.contains(ticket));
        Ok(probe.terminal.clone())
    }
    fn acknowledge(&mut self, ticket: &PluginTicket) -> Result<(), PluginError> {
        let mut probe = self.0.borrow_mut();
        assert!(probe.tickets.contains(ticket));
        if probe.fail_ack {
            return Err(PluginError::new(PluginErrorCode::Busy, "ACK retry"));
        }
        probe.acks += 1;
        Ok(())
    }
    fn cancel(&mut self, ticket: &PluginTicket) -> Result<CancelReceipt, PluginError> {
        let mut probe = self.0.borrow_mut();
        assert!(probe.tickets.contains(ticket));
        probe.cancels += 1;
        Ok(CancelReceipt::Requested)
    }
    fn shutdown(&mut self) -> Result<(), PluginError> {
        let mut probe = self.0.borrow_mut();
        probe.shutdowns += 1;
        if probe.fail_shutdown {
            return Err(PluginError::new(PluginErrorCode::Busy, "owner not joined"));
        }
        Ok(())
    }
}
fn hash(text: &str) -> ContentHash {
    ContentHash::of(text.as_bytes())
}
fn descriptor() -> CapabilityDescriptor {
    CapabilityDescriptor {
        targets: BTreeSet::from([TargetKind::Native]),
        request_schema: hash("request"),
        reply_schema: hash("reply"),
        error_schema: Some(hash("error")),
        event_schema: None,
    }
}
fn fixture(
    probe: Rc<RefCell<Probe>>,
    limit: usize,
) -> (PluginRegistry, RegistrationHandle, AsyncSdkPump) {
    fixture_with_wire(probe, limit, 1024 * 1024)
}
fn fixture_with_wire(
    probe: Rc<RefCell<Probe>>,
    limit: usize,
    wire: usize,
) -> (PluginRegistry, RegistrationHandle, AsyncSdkPump) {
    let program = StateProgram::mount(
        meari::parse(
            "type Props={};export function Scope(props:Props){return <main/>;}",
            Language::Tsx,
        )
        .unwrap(),
        "async-sdk-test".into(),
        1,
    )
    .unwrap();
    let mut registry = PluginRegistry::new(
        hash("registry"),
        TargetKind::Native,
        PluginScope {
            instance: program.snapshot().unwrap().instance,
            source_generation: 1,
        },
        8,
    )
    .unwrap();
    let manifest = PluginManifest {
        api_version: PLUGIN_API_VERSION,
        id: "test.async-sdk".into(),
        version: "1".into(),
        contract: hash("contract"),
        provider: hash("provider"),
        capabilities: BTreeMap::from([("test.run".into(), descriptor())]),
        assets: BTreeMap::new(),
    };
    let (adapter, pump) = TypedAsyncSdkPlugin::new(
        manifest,
        BTreeMap::from([("test.run".into(), async_binding(descriptor(), Port(probe)))]),
        AsyncCodecBudget {
            max_pending: limit,
            max_terminal_wire_bytes: wire,
            max_total_codec_wire_bytes: limit * wire * 4,
        },
    )
    .unwrap();
    let handle = registry
        .register(Box::new(adapter), BTreeSet::from(["test.run".into()]))
        .unwrap();
    (registry, handle, pump)
}
fn dispatch(registry: &mut PluginRegistry, handle: &RegistrationHandle) -> DispatchReceipt {
    registry
        .dispatch(
            handle,
            "test.run",
            PluginPayload::encode(hash("request"), &"input").unwrap(),
        )
        .unwrap()
}
#[test]
fn early_completion_is_retained_until_registry_acceptance_then_acknowledged() {
    let probe = Rc::new(RefCell::new(Probe {
        terminal: Some(Arc::new(Ok("already finished".into()))),
        ..Default::default()
    }));
    let (mut registry, handle, pump) = fixture(probe.clone(), 2);
    let receipt = dispatch(&mut registry, &handle);
    assert_eq!(receipt.outcome, PluginDispatch::Pending);
    assert_eq!(registry.pending_count(), 1);
    assert_eq!(probe.borrow().acks, 0);
    let result = pump
        .accept(&mut registry, &receipt.ticket)
        .unwrap()
        .unwrap();
    assert!(result.owner_ack.is_ok());
    assert!(matches!(result.event.event, PluginEventKind::Completed(_)));
    assert_eq!(registry.pending_count(), 0);
    assert_eq!(probe.borrow().starts, 1);
    assert_eq!(probe.borrow().acks, 1);
    assert!(pump.accept(&mut registry, &receipt.ticket).is_err());
}
#[test]
fn acknowledgement_failure_retries_without_duplicate_registry_delivery() {
    let probe = Rc::new(RefCell::new(Probe {
        terminal: Some(Arc::new(Ok("done".into()))),
        fail_ack: true,
        ..Default::default()
    }));
    let (mut registry, handle, pump) = fixture(probe.clone(), 2);
    let receipt = dispatch(&mut registry, &handle);
    let result = pump
        .accept(&mut registry, &receipt.ticket)
        .unwrap()
        .unwrap();
    assert!(result.owner_ack.is_err());
    assert_eq!(registry.pending_count(), 0);
    assert_eq!(
        registry.unregister(&handle).unwrap_err().code,
        PluginErrorCode::Busy
    );
    probe.borrow_mut().fail_ack = false;
    assert!(
        pump.accept(&mut registry, &receipt.ticket)
            .unwrap()
            .unwrap()
            .owner_ack
            .is_ok()
    );
    assert_eq!(probe.borrow().starts, 1);
    assert_eq!(probe.borrow().acks, 1);
    registry.unregister(&handle).unwrap();
}
#[test]
fn requested_cancel_keeps_real_completed_result_and_actual_ticket() {
    let probe = Rc::new(RefCell::new(Probe::default()));
    let (mut registry, handle, pump) = fixture(probe.clone(), 2);
    let receipt = dispatch(&mut registry, &handle);
    assert_eq!(
        registry.cancel(&receipt.ticket).unwrap(),
        CancelReceipt::Requested
    );
    assert!(
        pump.accept(&mut registry, &receipt.ticket)
            .unwrap()
            .is_none()
    );
    probe.borrow_mut().terminal = Some(Arc::new(Ok("published before cancel".into())));
    let result = pump
        .accept(&mut registry, &receipt.ticket)
        .unwrap()
        .unwrap();
    assert!(result.event.cancellation_requested);
    assert!(matches!(result.event.event, PluginEventKind::Completed(_)));
    assert_eq!(probe.borrow().cancels, 1);
}
#[test]
fn queued_cancel_is_typed_canonical_error_without_cloning_api_error() {
    let probe = Rc::new(RefCell::new(Probe::default()));
    let (mut registry, handle, pump) = fixture(probe.clone(), 2);
    let receipt = dispatch(&mut registry, &handle);
    registry.cancel(&receipt.ticket).unwrap();
    probe.borrow_mut().terminal = Some(Arc::new(Err(ApiError::new(
        ErrorCode::Cancelled,
        "removed before claim",
    ))));
    let result = pump
        .accept(&mut registry, &receipt.ticket)
        .unwrap()
        .unwrap();
    let PluginEventKind::Failed(error) = result.event.event else {
        panic!("canonical error expected")
    };
    assert_eq!(
        error
            .details
            .unwrap()
            .decode::<ApiError>(&hash("error"))
            .unwrap()
            .code,
        ErrorCode::Cancelled
    );
    assert_eq!(probe.borrow().acks, 1);
}
#[test]
fn oversized_terminal_keeps_pending_original_and_blocks_normal_close() {
    let original = Arc::new(Ok("x".repeat(1024 * 1024 + 1)));
    let probe = Rc::new(RefCell::new(Probe {
        terminal: Some(original.clone()),
        ..Default::default()
    }));
    let (mut registry, handle, pump) = fixture(probe.clone(), 2);
    let receipt = dispatch(&mut registry, &handle);
    assert_eq!(
        pump.accept(&mut registry, &receipt.ticket)
            .err()
            .unwrap()
            .code,
        PluginErrorCode::InvalidPayload
    );
    assert_eq!(registry.pending_count(), 1);
    assert_eq!(probe.borrow().acks, 0);
    assert!(Arc::ptr_eq(
        probe.borrow().terminal.as_ref().unwrap(),
        &original
    ));
    assert_eq!(
        registry.unregister(&handle).unwrap_err().code,
        PluginErrorCode::Busy
    );
    assert_eq!(probe.borrow().shutdowns, 0);
}
#[test]
fn tracking_capacity_and_failed_admission_never_start_hidden_work() {
    let probe = Rc::new(RefCell::new(Probe::default()));
    let (mut registry, handle, _pump) = fixture(probe.clone(), 1);
    dispatch(&mut registry, &handle);
    assert_eq!(
        registry
            .dispatch(
                &handle,
                "test.run",
                PluginPayload::encode(hash("request"), &"input").unwrap()
            )
            .unwrap_err()
            .code,
        PluginErrorCode::Busy
    );
    assert_eq!(probe.borrow().starts, 1);
    let probe = Rc::new(RefCell::new(Probe {
        fail_start: true,
        ..Default::default()
    }));
    let (mut registry, handle, pump) = fixture(probe.clone(), 1);
    assert!(
        registry
            .dispatch(
                &handle,
                "test.run",
                PluginPayload::encode(hash("request"), &"input").unwrap()
            )
            .is_err()
    );
    assert_eq!(probe.borrow().starts, 0);
    assert_eq!(registry.pending_count(), 0);
    assert!(pump.tickets().unwrap().is_empty());
}
#[test]
fn shutdown_delegation_is_retried_without_restarting_sdk() {
    let probe = Rc::new(RefCell::new(Probe {
        fail_shutdown: true,
        ..Default::default()
    }));
    let (mut registry, handle, _) = fixture(probe.clone(), 1);
    assert_eq!(
        registry.unregister(&handle).unwrap_err().code,
        PluginErrorCode::Busy
    );
    probe.borrow_mut().fail_shutdown = false;
    registry.unregister(&handle).unwrap();
    assert_eq!(probe.borrow().shutdowns, 2);
    assert_eq!(probe.borrow().starts, 0);
}

#[test]
fn sdk_error_within_core_limit_can_exceed_failed_envelope_budget() {
    let original = Arc::new(Err(ApiError::new(ErrorCode::Internal, "x".repeat(600_000))));
    assert!(serde_json::to_vec(original.as_ref()).unwrap().len() < 1024 * 1024);
    let probe = Rc::new(RefCell::new(Probe {
        terminal: Some(original.clone()),
        ..Default::default()
    }));
    let (mut registry, handle, pump) = fixture(probe.clone(), 1);
    let receipt = dispatch(&mut registry, &handle);
    assert!(pump.accept(&mut registry, &receipt.ticket).is_err());
    assert_eq!(registry.pending_count(), 1);
    assert_eq!(probe.borrow().acks, 0);
    assert!(Arc::ptr_eq(
        probe.borrow().terminal.as_ref().unwrap(),
        &original
    ));
}
#[test]
fn caller_wire_budget_is_checked_before_ack() {
    let probe = Rc::new(RefCell::new(Probe {
        terminal: Some(Arc::new(Ok("x".repeat(300)))),
        ..Default::default()
    }));
    let (mut registry, handle, pump) = fixture_with_wire(probe.clone(), 1, 256);
    let receipt = dispatch(&mut registry, &handle);
    assert!(pump.accept(&mut registry, &receipt.ticket).is_err());
    assert_eq!(probe.borrow().acks, 0);
    assert_eq!(registry.pending_count(), 1);
}
#[test]
fn invalid_registry_reply_preserves_pending_and_owner_for_valid_drain() {
    let probe = Rc::new(RefCell::new(Probe {
        terminal: Some(Arc::new(Ok("actual".into()))),
        ..Default::default()
    }));
    let (mut registry, handle, pump) = fixture(probe.clone(), 1);
    let receipt = dispatch(&mut registry, &handle);
    let invalid = PluginPayload::encode(hash("reply"), &true).unwrap();
    assert!(
        registry
            .accept_event(&receipt.ticket, PluginEventKind::Completed(invalid))
            .is_err()
    );
    assert_eq!(registry.pending_count(), 1);
    assert_eq!(probe.borrow().acks, 0);
    assert!(
        pump.accept(&mut registry, &receipt.ticket)
            .unwrap()
            .unwrap()
            .owner_ack
            .is_ok()
    );
    assert_eq!(probe.borrow().starts, 1);
}
