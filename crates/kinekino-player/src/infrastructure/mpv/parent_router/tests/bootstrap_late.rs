//! Actual SDK/ExecutionOwner and route ports; inert synthetic Ready is not window evidence.
use super::*;
use crate::{
    application::{
        api::ApplicationClientHandle,
        model_review::{ReviewSession, ReviewStore},
        sdk_execution::{
            CompletionNotifier, ExecutionBudget, ExecutionCloseProgress, ExecutionOwner,
            ExecutionSnapshot,
        },
    },
    assembly::native_playback::routed::{
        ReadyRoutedNative, RoutedSetupError, bootstrap::prepare_routed_client, ready_seal,
    },
    infrastructure::subtitle_files::SubtitleFiles,
};
#[path = "bootstrap_late/fixture.rs"]
mod fixture;
struct ReviewProbe(Arc<AtomicUsize>);
impl ReviewStore for ReviewProbe {
    fn open(&self, _: &str) -> Result<Box<dyn ReviewSession>, ApiError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err(ApiError::new(
            ErrorCode::NotFound,
            "Original review store reached",
        ))
    }
}
struct Notify;
impl CompletionNotifier for Notify {
    fn notify(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
struct SyntheticReady {
    router: Arc<ParentEngineRouter>,
    route: ParentRouteHandle,
}
impl ready_seal::Sealed for SyntheticReady {}
impl ReadyRoutedNative for SyntheticReady {
    type Error = ();
    fn with_ready_route<T>(
        &self,
        visit: impl FnOnce(&Arc<ParentEngineRouter>, &ParentRouteHandle) -> T,
    ) -> Result<T, Self::Error> {
        Ok(visit(&self.router, &self.route))
    }
}
fn caps(client: &ApplicationClientHandle) -> serde_json::Value {
    let response = client.execute(Request::new("capabilities"));
    assert!(response.ok);
    response.data.unwrap()
}
fn review() -> Request {
    let mut request = Request::new("reviews.inspect");
    request.payload = serde_json::json!({"package_sha256":"a".repeat(64)});
    request
}

#[test]
fn same_configured_client_and_execution_owner_work_before_and_after_trusted_activation() {
    let mut dispatcher = host(8);
    let calls = Arc::new(AtomicUsize::new(0));
    let bootstrap = prepare_routed_client(
        PlayerSdk::new(SubtitleFiles).with_review_store(ReviewProbe(calls.clone())),
        Arc::new(UnusedResults),
        dispatcher.handle(),
    );
    let before = bootstrap.client();
    let (same, activation) = bootstrap.into_parts();
    assert!(Arc::ptr_eq(&before, &same));
    assert_eq!(
        before.execute(review()).error.unwrap().message,
        "Original review store reached"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        before
            .execute(Request::new("segments.inspect"))
            .error
            .unwrap()
            .code,
        ErrorCode::NeedsInput
    );
    assert_eq!(caps(&before)["skip_exports"]["available"], false);
    let execution = ExecutionOwner::start(
        before.clone(),
        ExecutionBudget {
            max_operations: 4,
            max_request_bytes: 4096,
            max_total_request_bytes: 16384,
            max_terminal_bytes: 8192,
            max_total_terminal_bytes: 32768,
        },
        Arc::new(Notify),
    )
    .unwrap();
    let handle = execution.handle();
    let old_operation = handle.submit(Request::new("version")).unwrap();
    let deadline = Instant::now() + LIMIT;
    loop {
        if let ExecutionSnapshot::Terminal(result) = handle.peek(&old_operation).unwrap() {
            assert!(result.response.ok);
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    handle.acknowledge(&old_operation).unwrap();
    let router = activation.router();
    let engine = fixture::engine();
    let route = router.prepare(engine.clone()).unwrap();
    router.activate(&route).unwrap();
    let activated = activation
        .activate(SyntheticReady {
            router: router.clone(),
            route: route.clone(),
        })
        .unwrap_or_else(|_| panic!("Original activation failed"));
    assert!(Arc::ptr_eq(&before, &activated._retained_client));
    assert_eq!(caps(&before)["skip_exports"]["available"], true);
    assert_eq!(
        caps(&before)["skip_sessions"]["seek_submission_available"],
        false
    );
    let next_operation = handle.submit(review()).unwrap();
    let deadline = Instant::now() + LIMIT;
    loop {
        if let ExecutionSnapshot::Terminal(result) = handle.peek(&next_operation).unwrap() {
            assert_eq!(
                result.response.error.as_ref().unwrap().message,
                "Original review store reached"
            );
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    handle.acknowledge(&next_operation).unwrap();
    assert!(matches!(
        handle.peek(&old_operation).unwrap(),
        ExecutionSnapshot::Acknowledged
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    let _ = execution.close();
    let deadline = Instant::now() + LIMIT;
    loop {
        if let ExecutionCloseProgress::Closed(receipt) = execution.poll_close().unwrap() {
            assert!(receipt.worker_stopped);
            assert_eq!(receipt.unacknowledged, 0);
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    complete(&router, &route);
    drop(activated.ready);
    drop(activated._retained_read_owner);
    finish(&mut dispatcher);
}

#[test]
fn foreign_ready_preserves_pending_original_port_then_original_ready_installs_once() {
    let mut dispatcher = host(8);
    let bootstrap = prepare_routed_client(
        PlayerSdk::new(SubtitleFiles),
        Arc::new(UnusedResults),
        dispatcher.handle(),
    );
    let before = bootstrap.client();
    let (_, activation) = bootstrap.into_parts();
    let original_router = activation.router();
    let foreign_router = Arc::new(ParentEngineRouter::new(dispatcher.handle()));
    let foreign_engine = fixture::engine();
    let foreign_route = foreign_router.prepare(foreign_engine).unwrap();
    foreign_router.activate(&foreign_route).unwrap();
    let failure = match activation.activate(SyntheticReady {
        router: foreign_router.clone(),
        route: foreign_route.clone(),
    }) {
        Err(failure) => failure,
        Ok(_) => panic!("Foreign Ready installed"),
    };
    assert!(
        matches!(failure.error, RoutedSetupError::Api(ref error) if error.code == ErrorCode::RevisionConflict)
    );
    assert_eq!(caps(&before)["skip_exports"]["available"], false);
    assert!(Arc::ptr_eq(
        &failure._retained_ready.router,
        &foreign_router
    ));
    complete(&foreign_router, &foreign_route);
    let engine = fixture::engine();
    let route = original_router.prepare(engine).unwrap();
    original_router.activate(&route).unwrap();
    let activated = failure
        .activation
        .activate(SyntheticReady {
            router: original_router.clone(),
            route: route.clone(),
        })
        .unwrap_or_else(|_| panic!("Retained original activation failed"));
    assert!(Arc::ptr_eq(&before, &activated._retained_client));
    assert_eq!(caps(&before)["skip_exports"]["available"], true);
    complete(&original_router, &route);
    drop(activated);
    finish(&mut dispatcher);
}

#[test]
fn closed_pending_activation_returns_original_ready_and_blocks_native_without_losing_reviews() {
    let mut dispatcher = host(8);
    let calls = Arc::new(AtomicUsize::new(0));
    let bootstrap = prepare_routed_client(
        PlayerSdk::new(SubtitleFiles).with_review_store(ReviewProbe(calls.clone())),
        Arc::new(UnusedResults),
        dispatcher.handle(),
    );
    let client = bootstrap.client();
    let (_, activation) = bootstrap.into_parts();
    activation.close_pending().unwrap();
    let router = activation.router();
    let route = router.prepare(fixture::engine()).unwrap();
    router.activate(&route).unwrap();
    let failure = match activation.activate(SyntheticReady {
        router: router.clone(),
        route: route.clone(),
    }) {
        Err(failure) => failure,
        Ok(_) => panic!("Cancelled pending host activated"),
    };
    assert!(
        matches!(failure.error, RoutedSetupError::Api(ref error) if error.code == ErrorCode::Cancelled)
    );
    assert!(failure._retained_revocation_error.is_none());
    assert!(Arc::ptr_eq(&failure._retained_ready.router, &router));
    assert_eq!(
        router.observe_native_guard().unwrap_err().code,
        ErrorCode::Cancelled
    );
    assert_eq!(caps(&client)["skip_exports"]["available"], false);
    assert_eq!(
        client.execute(review()).error.unwrap().message,
        "Original review store reached"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    complete(&router, &route);
    finish(&mut dispatcher);
}
