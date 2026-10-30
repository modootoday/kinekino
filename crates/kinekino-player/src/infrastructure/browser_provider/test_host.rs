//! Controlled in-process owner facts. This fixture starts no COM/window/thread.
use super::*;

pub(crate) struct BrowserFixture {
    handle: Arc<Handle>,
}
impl BrowserFixture {
    pub(crate) fn shutdown_requested(&self) -> bool {
        self.handle.shutdown.load(Ordering::Acquire)
    }
    pub(crate) fn report_exit(&self, failure: Option<ApiError>) {
        let mut shared = self.handle.shared.lock().unwrap();
        shared.host.worker_exited = true;
        shared.host.state = if failure.is_some() {
            BrowserHostState::Failed
        } else {
            BrowserHostState::Stopped
        };
        shared.host.failure = failure.map(Arc::new);
    }
}
pub(crate) fn host_fixture() -> (BrowserHost, BrowserFixture) {
    let (commands, _receiver) = mpsc::sync_channel(MAX_COMMANDS);
    let handle = Arc::new(Handle {
        shared: Arc::new(Mutex::new(Shared {
            works: vec![],
            next_generation: 1,
            host: BrowserHostSnapshot {
                state: BrowserHostState::Ready,
                worker_exited: false,
                failure: None,
            },
        })),
        commands,
        shutdown: Arc::new(AtomicBool::new(false)),
        session_id: "fixture-provider-startup".into(),
    });
    (
        BrowserHost {
            handle: Arc::clone(&handle),
        },
        BrowserFixture { handle },
    )
}
