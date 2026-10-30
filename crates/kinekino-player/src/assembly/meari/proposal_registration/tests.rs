use super::*;
use crate::application::api::PlayerSdk;
use meari::{
    application::{Language, state_program::StateProgram},
    domain::build::{ContentHash, TargetKind},
};
use std::sync::Arc;
struct Fixture {
    _program: StateProgram,
    scope: PluginScope,
    registry: Rc<RefCell<PluginRegistry>>,
    client: ApplicationClientHandle,
    context: Rc<RefCell<RequestContext>>,
}
fn fixture() -> Fixture {
    let component = meari::parse(
        "type Props={};export function Scope(props:Props){return <main/>;}",
        Language::Tsx,
    )
    .unwrap();
    let program = StateProgram::mount(component, "sdk-registration-test".into(), 1).unwrap();
    let scope = PluginScope {
        instance: program.snapshot().unwrap().instance,
        source_generation: 1,
    };
    let registry = Rc::new(RefCell::new(
        PluginRegistry::new(
            ContentHash::of(b"sdk-registration-test"),
            TargetKind::Native,
            scope.clone(),
            8,
        )
        .unwrap(),
    ));
    Fixture {
        _program: program,
        scope,
        registry,
        client: Arc::new(PlayerSdk::default()),
        context: Rc::new(RefCell::new(RequestContext::default())),
    }
}
#[test]
fn cleanup_failure_retains_registration_until_explicit_retry() {
    let f = fixture();
    let mut lease = ProposalSdkRegistration::register(
        f.client.clone(),
        f.registry.clone(),
        &f.scope,
        f.context.clone(),
    )
    .unwrap();
    let handle = lease.handle().unwrap().clone();
    let borrowed = f.registry.borrow();
    assert_eq!(lease.close().unwrap_err().code, PluginErrorCode::Busy);
    assert_eq!(lease.handle().unwrap(), &handle);
    drop(borrowed);
    lease.close().unwrap();
    lease.close().unwrap();
    assert_eq!(lease.handle().unwrap_err().code, PluginErrorCode::Stale);
    assert_eq!(
        f.registry.borrow().manifest(&handle).unwrap_err().code,
        PluginErrorCode::Stale
    );
}
#[test]
fn different_document_scope_is_rejected_without_hidden_registration() {
    let f = fixture();
    let mut wrong = f.scope.clone();
    wrong.source_generation += 1;
    let error = ProposalSdkRegistration::register(
        f.client.clone(),
        f.registry.clone(),
        &wrong,
        f.context.clone(),
    )
    .err()
    .unwrap();
    assert_eq!(error.code, PluginErrorCode::Stale);
    let mut correct =
        ProposalSdkRegistration::register(f.client, f.registry, &f.scope, f.context).unwrap();
    correct.close().unwrap();
}

#[test]
fn ticket_context_preserves_explicit_options_and_borrow_conflicts_are_reported() {
    use crate::application::api::{ApplicationClient, Request, Response};
    use meari::application::plugins::{PluginDispatch, PluginPayload};
    use std::sync::Mutex;
    struct ObservedClient(Arc<Mutex<Vec<Request>>>);
    impl ApplicationClient for ObservedClient {
        fn execute(&self, request: Request) -> Response {
            self.0.lock().unwrap().push(request.clone());
            PlayerSdk::default().execute(request)
        }
    }
    let f = fixture();
    let requests = Arc::new(Mutex::new(Vec::new()));
    *f.context.borrow_mut() = RequestContext {
        request_id: "명시-문맥".into(),
        expected_revision: Some("9007199254740993".into()),
        idempotency_key: Some("explicit-key".into()),
        timeout_ms: Some(100),
        ..Default::default()
    };
    let original = serde_json::to_value(&*f.context.borrow()).unwrap();
    let mut lease = ProposalSdkRegistration::register(
        Arc::new(ObservedClient(requests.clone())),
        f.registry.clone(),
        &f.scope,
        f.context.clone(),
    )
    .unwrap();
    let payload = || PluginPayload {
        schema: proposal_manifest().capabilities["proposals.inspect"]
            .request_schema
            .clone(),
        value: serde_json::json!({"proposal_id":"missing"}),
    };
    let borrowed = f.context.borrow_mut();
    let result = f
        .registry
        .borrow_mut()
        .dispatch(lease.handle().unwrap(), "proposals.inspect", payload())
        .unwrap();
    assert!(matches!(result.outcome, PluginDispatch::Immediate(Err(_))));
    assert!(requests.lock().unwrap().is_empty());
    drop(borrowed);
    for _ in 0..2 {
        f.registry
            .borrow_mut()
            .dispatch(lease.handle().unwrap(), "proposals.inspect", payload())
            .unwrap();
    }
    let recorded = requests.lock().unwrap();
    assert_eq!(recorded.len(), 2);
    assert_ne!(
        recorded[0].context.request_id,
        recorded[1].context.request_id
    );
    for request in recorded.iter() {
        assert!(request.context.request_id.starts_with("ui-"));
        let mut observed = serde_json::to_value(&request.context).unwrap();
        observed["request_id"] = original["request_id"].clone();
        assert_eq!(
            observed, original,
            "Explicit CAS/idempotency/timeout options must not be stripped"
        );
    }
    drop(recorded);
    lease.close().unwrap();
}

mod context_validation;
