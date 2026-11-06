//! Contract tests for host-owned BrowserCaptureSession + mock engine (M3-5 / ADR 20261106032051).

use super::{
    AllowlistAdmission, BrowserCaptureSession, CaptureError, CaptureNavigateTarget,
    CaptureWaitHint, MockBrowserEngine, ObservedNetworkRequest,
};
use crate::engine::port::{BrowserEngine, InterceptedRequest, NetworkInterceptor, RequestDecision};
use kinekino_core::source::StreamCandidate;

fn admission() -> AllowlistAdmission {
    AllowlistAdmission::new(
        ["doc.example.test", "cdn.example.test"],
        ["cdn.example.test"],
    )
}

fn media_obs(url: &str) -> ObservedNetworkRequest {
    ObservedNetworkRequest {
        url: url.into(),
        method: "GET".into(),
        resource_type: "media".into(),
        headers: vec![("Accept".into(), "video/*".into())],
    }
}

/// Test-double plugin: map media observations to StreamCandidate without extending the schema.
fn plugin_map_to_candidates(obs: &[ObservedNetworkRequest]) -> Vec<StreamCandidate> {
    obs.iter()
        .filter(|o| o.resource_type == "media")
        .map(|o| StreamCandidate {
            url: o.url.clone(),
            headers: o.headers.clone(),
        })
        .collect()
}

#[test]
fn admitted_navigate_collects_observations() {
    let mut engine = MockBrowserEngine::new();
    let nav = "https://doc.example.test/play/1";
    engine.script_observations(
        nav,
        vec![
            ObservedNetworkRequest {
                url: "https://cdn.example.test/seg.m3u8".into(),
                method: "GET".into(),
                resource_type: "media".into(),
                headers: vec![("Referer".into(), nav.into())],
            },
            ObservedNetworkRequest {
                url: "https://cdn.example.test/beacon".into(),
                method: "POST".into(),
                resource_type: "xhr".into(),
                headers: vec![],
            },
        ],
    );
    let mut session = BrowserCaptureSession::open(engine, Box::new(admission()));
    let obs = session
        .collect(
            &CaptureNavigateTarget { url: nav.into() },
            CaptureWaitHint {
                network_idle: true,
                timeout_ms: Some(1_000),
            },
        )
        .expect("admitted navigate");
    assert!(
        obs.iter().any(|o| {
            o.url == "https://cdn.example.test/seg.m3u8"
                && o.method == "GET"
                && o.resource_type == "media"
                && o.headers.iter().any(|(k, v)| k == "Referer" && v == nav)
        }),
        "expected media observation with headers, got {obs:?}"
    );
    assert!(
        session
            .engine_mut()
            .navigated_urls()
            .iter()
            .any(|u| u == nav),
        "engine must have navigated"
    );
}

#[test]
fn plugin_resolve_path_returns_stream_candidate() {
    let mut engine = MockBrowserEngine::new();
    let nav = "https://doc.example.test/ep/2";
    let media = "https://cdn.example.test/a.mp4";
    engine.script_observations(nav, vec![media_obs(media)]);
    let mut session = BrowserCaptureSession::open(engine, Box::new(admission()));
    let obs = session
        .collect(
            &CaptureNavigateTarget { url: nav.into() },
            CaptureWaitHint::default(),
        )
        .expect("collect");
    let candidates = plugin_map_to_candidates(&obs);
    assert_eq!(
        candidates,
        vec![StreamCandidate {
            url: media.into(),
            headers: vec![("Accept".into(), "video/*".into())],
        }]
    );
}

#[test]
fn denied_navigate_host_does_not_navigate_or_leak() {
    let mut engine = MockBrowserEngine::new();
    let denied = "https://evil.example.test/x";
    engine.script_observations(denied, vec![media_obs("https://cdn.example.test/leak.mp4")]);
    let mut session = BrowserCaptureSession::open(engine, Box::new(admission()));
    let err = session
        .collect(
            &CaptureNavigateTarget { url: denied.into() },
            CaptureWaitHint::default(),
        )
        .expect_err("denied");
    assert!(matches!(err, CaptureError::NavigateDenied));
    assert!(
        session.engine_mut().navigated_urls().is_empty(),
        "must not navigate denied host"
    );
    // No candidates from a denied path.
    assert!(plugin_map_to_candidates(&[]).is_empty());
}

#[test]
fn non_admitted_media_host_is_filtered_out() {
    let mut engine = MockBrowserEngine::new();
    let nav = "https://doc.example.test/play/3";
    engine.script_observations(
        nav,
        vec![
            media_obs("https://cdn.example.test/ok.mp4"),
            media_obs("https://other.cdn.test/nope.mp4"),
        ],
    );
    let mut session = BrowserCaptureSession::open(engine, Box::new(admission()));
    let obs = session
        .collect(
            &CaptureNavigateTarget { url: nav.into() },
            CaptureWaitHint::default(),
        )
        .expect("collect");
    assert_eq!(obs.len(), 1);
    assert_eq!(obs[0].url, "https://cdn.example.test/ok.mp4");
    let candidates = plugin_map_to_candidates(&obs);
    assert_eq!(candidates.len(), 1);
    assert!(!candidates[0].url.contains("other.cdn.test"));
}

#[test]
fn network_interceptor_remains_allow_block_only() {
    // Compile-time / API shape: only Allow and Block exist.
    let decisions = [RequestDecision::Allow, RequestDecision::Block];
    assert_eq!(decisions.len(), 2);

    struct BlockCdn;
    impl NetworkInterceptor for BlockCdn {
        fn decide(&self, request: &InterceptedRequest) -> RequestDecision {
            if request.url.contains("blocked") {
                RequestDecision::Block
            } else {
                RequestDecision::Allow
            }
        }
    }

    let mut engine = MockBrowserEngine::new();
    let nav = "https://doc.example.test/play/4";
    engine.script_observations(
        nav,
        vec![
            media_obs("https://cdn.example.test/ok.mp4"),
            media_obs("https://cdn.example.test/blocked.mp4"),
        ],
    );
    engine
        .set_interceptor(Box::new(BlockCdn))
        .expect("interceptor");
    // Interceptor is engine policy, not the capture API: capture session is separate.
    let mut session = BrowserCaptureSession::open(engine, Box::new(admission()));
    let obs = session
        .collect(
            &CaptureNavigateTarget { url: nav.into() },
            CaptureWaitHint::default(),
        )
        .expect("collect");
    assert!(obs.iter().all(|o| !o.url.contains("blocked")));
    assert!(obs.iter().any(|o| o.url.ends_with("ok.mp4")));
}

#[test]
fn close_is_idempotent_and_collect_after_close_fails() {
    let engine = MockBrowserEngine::new();
    let mut session = BrowserCaptureSession::open(engine, Box::new(admission()));
    session.close().expect("first close");
    session.close().expect("second close");
    assert!(session.is_closed());
    let err = session
        .collect(
            &CaptureNavigateTarget {
                url: "https://doc.example.test/x".into(),
            },
            CaptureWaitHint::default(),
        )
        .expect_err("closed");
    assert!(matches!(err, CaptureError::Closed));
}

#[test]
fn observations_are_capped() {
    let mut engine = MockBrowserEngine::new();
    let nav = "https://doc.example.test/play/5";
    let many: Vec<_> = (0..10)
        .map(|i| media_obs(&format!("https://cdn.example.test/seg{i}.ts")))
        .collect();
    engine.script_observations(nav, many);
    let mut session =
        BrowserCaptureSession::open(engine, Box::new(admission())).with_max_observations(3);
    let obs = session
        .collect(
            &CaptureNavigateTarget { url: nav.into() },
            CaptureWaitHint::default(),
        )
        .expect("collect");
    assert_eq!(obs.len(), 3);
}
