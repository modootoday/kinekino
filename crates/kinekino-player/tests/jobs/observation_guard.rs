//! Test-only ownership cleanup and the concrete provider adapter's busy admission contract.
use kinekino_player::application::{
    api::{ApiError, ErrorCode},
    browser_provider::{BrowserProviderOwner, BrowserProviderPort, BrowserTicket},
};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

pub(super) fn browser_call<T>(deadline: Instant, call: impl FnMut() -> Result<T, ApiError>) -> T {
    browser_result(deadline, call)
        .unwrap_or_else(|error| panic!("Browser observation call failed: {error:?}"))
}

fn browser_result<T>(
    deadline: Instant,
    mut call: impl FnMut() -> Result<T, ApiError>,
) -> Result<T, ApiError> {
    loop {
        match call() {
            // The concrete adapter uses ResourceExhausted for both busy and limits.
            // Only this observed busy diagnostic is safe to poll again without new work.
            Err(error)
                if error.code == ErrorCode::ResourceExhausted
                    && error.message == "Browser state is busy"
                    && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(5))
            }
            result => return result,
        }
    }
}

pub(super) struct OwnedObservation {
    owner: Arc<dyn BrowserProviderOwner>,
    port: Arc<dyn BrowserProviderPort>,
    pub ticket: Option<BrowserTicket>,
    evidence: PathBuf,
    pub finished: bool,
}
impl OwnedObservation {
    pub fn new(
        owner: Arc<dyn BrowserProviderOwner>,
        port: Arc<dyn BrowserProviderPort>,
        evidence: PathBuf,
    ) -> Self {
        Self {
            owner,
            port,
            ticket: None,
            evidence,
            finished: false,
        }
    }
}
impl Drop for OwnedObservation {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        let cancel = self.ticket.as_ref().map(|ticket| {
            browser_result(Instant::now() + Duration::from_secs(2), || {
                self.port.cancel(ticket)
            })
            .map(|value| format!("{value:?}"))
        });
        let deadline = Instant::now() + Duration::from_secs(10);
        let shutdown = browser_result(deadline, || self.owner.request_shutdown());
        let stopped = loop {
            let snapshot = browser_result(deadline, || self.owner.shutdown_snapshot());
            if !matches!(&snapshot, Ok(state) if !state.worker_exited) || Instant::now() >= deadline
            {
                break snapshot;
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        let observation = match stopped {
            Ok(state) => {
                json!({"worker_exited":state.worker_exited,"host_state":format!("{:?}",state.state)})
            }
            Err(error) => json!({"worker_exited":null,"error":error}),
        };
        if let Ok(bytes) = serde_json::to_vec_pretty(
            &json!({"cleanup_scope":"test-owned SDK provider only","cancel":cancel,"shutdown_request":shutdown,"observation":observation}),
        ) {
            let _ = fs::write(&self.evidence, bytes);
        }
    }
}

#[test]
fn busy_wait_does_not_retry_limits_or_other_errors() {
    kinekino_platform::install();
    let mut calls = 0;
    let observed = browser_result(Instant::now() + Duration::from_secs(1), || {
        calls += 1;
        if calls == 1 {
            Err(ApiError::new(
                ErrorCode::ResourceExhausted,
                "Browser state is busy",
            ))
        } else {
            Ok(7)
        }
    })
    .unwrap();
    assert_eq!(observed, 7);
    assert_eq!(calls, 2);
    for (code, message) in [
        (
            ErrorCode::ResourceExhausted,
            "Browser ticket or active-work limit reached",
        ),
        (ErrorCode::EngineFailed, "failed"),
    ] {
        let mut calls = 0;
        let error = browser_result(Instant::now() + Duration::from_secs(1), || {
            calls += 1;
            Err::<(), _>(ApiError::new(code, message))
        })
        .unwrap_err();
        assert_eq!(error.code, code);
        assert_eq!(calls, 1);
    }
}

#[test]
fn panic_cleanup_requests_only_the_owned_provider_and_records_observation() {
    kinekino_platform::install();
    use super::synthetic_browser::SyntheticBrowser;
    let root = std::env::temp_dir().join(format!(
        "player-observation-guard-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let evidence = root.join("cleanup.json");
    let provider = Arc::new(SyntheticBrowser::default());
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = OwnedObservation::new(provider.clone(), provider.clone(), evidence.clone());
        panic!("controlled observation fixture panic");
    }));
    assert!(outcome.is_err());
    let receipt: serde_json::Value = serde_json::from_slice(&fs::read(&evidence).unwrap()).unwrap();
    assert_eq!(receipt["observation"]["worker_exited"], true);
    // Synthetic owner proves guard wiring, not native COM/OS shutdown.
    assert_eq!(receipt["observation"]["host_state"], "Stopped");
    fs::remove_file(&evidence).unwrap();
    fs::remove_dir(&root).unwrap();
}
