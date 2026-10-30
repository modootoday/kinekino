#![cfg(windows)]
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "support/local_host_cli.rs"]
pub mod cli_fixture;
use kinekino_player::{
    application::{
        api::{
            ApiError, ApplicationClientHandle, PlayerSdk, Request, RequestContext,
            SkipSegmentsClient,
        },
        playback_control::ControlReceipt,
        playback_session::ControlEpisodePlayback,
        playback_status::{PlaybackStatus, PlaybackStatusOrigin},
        skip_segments::{SkipBinding, SkipObservation, SkipPlaybackPort, host::*},
    },
    domain::{api_values::DecimalU64, skip_segments::Seekability},
    infrastructure::local_host::{LocalHost, TransportOptions},
};
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct Port {
    calls: AtomicUsize,
    known: bool,
}
impl SkipPlaybackPort for Port {
    fn observe(&self) -> Result<SkipObservation, ApiError> {
        Ok(SkipObservation {
            binding: binding(),
            seekability: if self.known {
                Seekability::Yes
            } else {
                Seekability::Unknown
            },
            observation_authorized: self.known,
            playback: PlaybackStatus {
                origin: PlaybackStatusOrigin::NativeObservation,
                published_age_ms: DecimalU64(0),
                connected: true,
                paused: Some(false),
                idle: Some(false),
                position_us: Some(DecimalU64(15_000_000)),
                duration_us: Some(DecimalU64(100_000_000)),
                volume_percent: None,
                muted: None,
                video_size: None,
                audio_channels: None,
                has_error: false,
            },
        })
    }
    fn submit(&self, input: ControlEpisodePlayback) -> Result<ControlReceipt, ApiError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(ControlReceipt::enqueued(input.engine_instance_id.0))
    }
}
fn binding() -> SkipBinding {
    SkipBinding {
        engine_instance_id: DecimalU64(7),
        resolution_artifact_id: "episode".into(),
    }
}
fn setup(
    generation: &str,
    known: bool,
) -> (ApplicationClientHandle, Arc<SkipHostRegistry>, Arc<Port>) {
    let port = Arc::new(Port {
        calls: AtomicUsize::new(0),
        known,
    });
    let registry =
        Arc::new(SkipHostRegistry::new(generation.into(), port.clone(), 1, known).unwrap());
    let sdk: ApplicationClientHandle =
        Arc::new(PlayerSdk::default().with_skip_host(registry.clone()));
    (sdk, registry, port)
}
fn call(endpoint: &str, command: &str, payload: Value, revision: Option<&str>) -> (i32, Value) {
    let mut request = Request::new(command);
    request.payload = payload;
    request.context.expected_revision = revision.map(str::to_owned);
    let result = if matches!(command, "schema" | "capabilities") {
        cli_fixture::invoke(endpoint, &request)
    } else {
        let mut runner = cli_fixture::CliRunner::start(endpoint, &request);
        let initial_progress = runner.progress();
        let polled = runner
            .poll()
            .map(|receipt| (receipt.code, receipt.response.clone()));
        eprintln!("CliProgress before_finish={initial_progress:?}");
        let result = {
            let receipt = runner.finish();
            eprintln!("{receipt:?}");
            (receipt.code, receipt.response.clone())
        };
        eprintln!("CliProgress after_finish={:?}", runner.progress());
        if let Some(polled) = polled {
            assert_eq!(polled, result, "Polled and joined CLI outcomes must agree");
        }
        result
    };
    if let Some(folder) = std::env::var_os("MANUAL_SKIP_EVIDENCE_DIR") {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::path::PathBuf::from(folder).join(format!(
            "request-{}-{}.json",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .unwrap();
        use std::io::Write;
        file.write_all(&serde_json::to_vec_pretty(&json!({"endpoint":endpoint,"request":request,"exit_code":result.0,"response":result.1,"fixture":"synthetic port; not native seek","cli_sha256":format!("{:x}",sha2::Sha256::digest(std::fs::read(crate::cli_exe::kinekino_cli_exe()).unwrap()))})).unwrap()).unwrap();
    }
    result
}
use sha2::Digest;

#[test]
fn fresh_cli_processes_share_only_the_same_live_host_owned_session() {
    kinekino_platform::install();
    let (sdk, registry, port) = setup("cli-host-one", true);
    let _owner = registry.owner();
    let host = LocalHost::start(sdk.clone(), TransportOptions::default()).unwrap();
    let endpoint = host.endpoint();
    let (code, opened) = call(
        endpoint,
        "segments.open",
        json!({"binding":binding()}),
        None,
    );
    assert_eq!(code, 0, "{opened}");
    let reference = opened["data"]["session"].clone();
    let (code, edited) = call(
        endpoint,
        "segments.edit",
        json!({"session":reference,"input":{"segment_id":"op","kind":"opening","in_us":"10000000","out_us":"20000000"}}),
        Some("1"),
    );
    assert_eq!(code, 0, "{edited}");
    assert_eq!(
        call(
            endpoint,
            "segments.accept_skip",
            json!({"session":reference,"input":{"segment_id":"op","annotation_revision":"1"}}),
            Some("2")
        )
        .0,
        0
    );
    assert_eq!(
        call(
            endpoint,
            "segments.set_skip_policy",
            json!({"session":reference,"input":{"policy":{"opening":"auto","ending":"manual"}}}),
            Some("3")
        )
        .0,
        0
    );
    let (code, read) = call(endpoint, "segments.inspect", reference.clone(), None);
    assert_eq!(code, 0);
    assert_eq!(read["data"]["revision"], "4");
    assert_eq!(read["data"]["durable"], false);
    let typed: SessionRef = serde_json::from_value(reference.clone()).unwrap();
    let direct = sdk
        .inspect_skip_session(RequestContext::default(), typed)
        .unwrap();
    assert_eq!(serde_json::to_value(direct).unwrap(), read["data"]);
    let (code, action) = call(
        endpoint,
        "segments.skip",
        json!({"session":reference,"input":{"segment_id":"op","intent":"automatic"}}),
        Some("4"),
    );
    assert_eq!(code, 0, "{action}");
    assert_eq!(
        action["data"]["result"]["receipt"]["completion"],
        "unobserved"
    );
    let (code, cancel) = call(
        endpoint,
        "segments.skip",
        json!({"session":reference,"input":{"segment_id":"op","intent":"automatic","cancelled":true}}),
        Some("5"),
    );
    assert_eq!(code, 0);
    assert_eq!(cancel["data"]["result"]["status"], "too_late_or_unobserved");
    assert_eq!(cancel["data"]["snapshot"]["revision"], "6");
    assert_eq!(
        call(
            endpoint,
            "segments.skip",
            json!({"session":reference,"input":{"segment_id":"op","intent":"automatic"}}),
            Some("6")
        )
        .0,
        0
    );
    assert_eq!(port.calls.load(Ordering::SeqCst), 1);
    let (code, reused) = call(
        endpoint,
        "segments.open",
        json!({"binding":binding()}),
        None,
    );
    assert_eq!(code, 0);
    assert_eq!(reused["data"]["reused"], true);
    assert_eq!(reused["data"]["session"], reference);
    assert_eq!(
        call(endpoint, "segments.close", reference.clone(), Some("6")).0,
        0
    );
    assert_eq!(call(endpoint, "segments.inspect", reference, None).0, 4);
}

#[test]
fn unknown_native_like_host_edits_state_but_never_advertises_or_submits_seek() {
    kinekino_platform::install();
    let (sdk, registry, port) = setup("unknown-host", false);
    let _owner = registry.owner();
    let host = LocalHost::start(sdk, TransportOptions::default()).unwrap();
    let endpoint = host.endpoint();
    let (code, caps) = call(endpoint, "capabilities", json!({}), None);
    assert_eq!(code, 0);
    assert_eq!(
        caps["data"]["skip_sessions"]["seek_submission_available"],
        false
    );
    assert_eq!(
        caps["data"]["skip_sessions"]["automatic_host_tick_supported"],
        false
    );
    let (_, schema) = call(endpoint, "schema", json!({"command":"segments.skip"}), None);
    assert_eq!(schema["data"]["status"], "unsupported");
    let (_, opened) = call(
        endpoint,
        "segments.open",
        json!({"binding":binding()}),
        None,
    );
    let reference = opened["data"]["session"].clone();
    assert_eq!(call(endpoint,"segments.edit",json!({"session":reference,"input":{"segment_id":"op","kind":"opening","in_us":"10000000","out_us":"20000000"}}),Some("1")).0,0);
    assert_eq!(
        call(
            endpoint,
            "segments.accept_skip",
            json!({"session":reference,"input":{"segment_id":"op","annotation_revision":"1"}}),
            Some("2")
        )
        .0,
        0
    );
    assert_eq!(
        call(
            endpoint,
            "segments.set_skip_policy",
            json!({"session":reference,"input":{"policy":{"opening":"auto","ending":"manual"}}}),
            Some("3")
        )
        .0,
        0
    );
    let (code, result) = call(
        endpoint,
        "segments.skip",
        json!({"session":reference,"input":{"segment_id":"op","intent":"manual"}}),
        Some("4"),
    );
    assert_eq!(code, 3, "{result}");
    assert_eq!(result["error"]["code"], "unsupported");
    assert_eq!(port.calls.load(Ordering::SeqCst), 0);
    let default = PlayerSdk::default().execute(Request::new("segments.open"));
    assert_eq!(default.error.unwrap().exit_code(), 3);
}

#[test]
fn explicit_close_and_new_host_generation_invalidate_old_sessions() {
    kinekino_platform::install();
    let (sdk, registry, _) = setup("old-generation", true);
    let owner = registry.owner();
    let host = LocalHost::start(sdk, TransportOptions::default()).unwrap();
    let (_, opened) = call(
        host.endpoint(),
        "segments.open",
        json!({"binding":binding()}),
        None,
    );
    let old = opened["data"]["session"].clone();
    drop(owner);
    assert_eq!(
        call(host.endpoint(), "segments.inspect", old.clone(), None).0,
        3
    );
    drop(host);
    let (sdk, registry, port) = setup("new-generation", true);
    let _owner = registry.owner();
    let next = LocalHost::start(sdk, TransportOptions::default()).unwrap();
    let (_, opened) = call(
        next.endpoint(),
        "segments.open",
        json!({"binding":binding()}),
        None,
    );
    assert_ne!(
        opened["data"]["session"]["skip_session_id"],
        old["skip_session_id"]
    );
    let (code, result) = call(next.endpoint(), "segments.inspect", old, None);
    assert_eq!(code, 4, "{result}");
    assert_eq!(result["error"]["code"], "not_found");
    assert_eq!(port.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn evaluate_and_override_wire_preserve_canonical_state_and_reject_stale_revision() {
    kinekino_platform::install();
    let (sdk, registry, port) = setup("evaluate-wire-host", true);
    let _owner = registry.owner();
    let host = LocalHost::start(sdk.clone(), TransportOptions::default()).unwrap();
    let endpoint = host.endpoint();
    let (code, opened) = call(
        endpoint,
        "segments.open",
        json!({"binding":binding()}),
        None,
    );
    assert_eq!(code, 0, "{opened}");
    let reference = opened["data"]["session"].clone();
    assert_eq!(call(endpoint, "segments.edit", json!({"session":reference,"input":{"segment_id":"op","kind":"opening","in_us":"10000000","out_us":"20000000"}}), Some("1")).0, 0);
    assert_eq!(
        call(
            endpoint,
            "segments.accept_skip",
            json!({"session":reference,"input":{"segment_id":"op","annotation_revision":"1"}}),
            Some("2")
        )
        .0,
        0
    );
    assert_eq!(
        call(
            endpoint,
            "segments.set_skip_policy",
            json!({"session":reference,"input":{"policy":{"opening":"auto","ending":"manual"}}}),
            Some("3")
        )
        .0,
        0
    );

    let evaluate = json!({"session":reference,"input":{"segment_id":"op","intent":"automatic"}});
    let (code, eligible) = call(endpoint, "segments.evaluate", evaluate.clone(), None);
    assert_eq!(code, 0, "{eligible}");
    assert_eq!(
        eligible["data"]["decision"],
        json!({"state":"eligible","target_us":"20000000"})
    );
    assert_eq!(
        eligible["data"]["snapshot"]["binding"],
        reference["binding"]
    );
    assert_eq!(eligible["data"]["snapshot"]["revision"], "4");
    let (code, overridden) = call(
        endpoint,
        "segments.override",
        json!({"session":reference,"input":{"action":"manual_seek"}}),
        Some("4"),
    );
    assert_eq!(code, 0, "{overridden}");
    assert_eq!(overridden["data"]["snapshot"]["revision"], "5");
    let (code, current) = call(endpoint, "segments.inspect", reference.clone(), None);
    assert_eq!(code, 0);
    assert_eq!(current["data"], overridden["data"]["snapshot"]);
    let (code, blocked) = call(endpoint, "segments.evaluate", evaluate.clone(), None);
    assert_eq!(code, 0);
    assert_eq!(
        blocked["data"]["decision"],
        json!({"state":"blocked","reason":"overridden"})
    );

    let (code, stale) = call(
        endpoint,
        "segments.override",
        json!({"session":reference,"input":{"action":"begin_new_pass"}}),
        Some("4"),
    );
    assert_eq!(code, 5, "{stale}");
    assert_eq!(stale["error"]["code"], "revision_conflict");
    let (code, unchanged) = call(endpoint, "segments.inspect", reference.clone(), None);
    assert_eq!(code, 0);
    assert_eq!(unchanged["data"], current["data"]);
    let (code, new_pass) = call(
        endpoint,
        "segments.override",
        json!({"session":reference,"input":{"action":"begin_new_pass"}}),
        Some("5"),
    );
    assert_eq!(code, 0, "{new_pass}");
    assert_eq!(new_pass["data"]["snapshot"]["revision"], "6");
    assert_eq!(new_pass["data"]["snapshot"]["pass_id"], "2");
    let (code, eligible) = call(endpoint, "segments.evaluate", evaluate.clone(), None);
    assert_eq!(code, 0);
    assert_eq!(
        eligible["data"]["decision"],
        json!({"state":"eligible","target_us":"20000000"})
    );
    let (code, cancelled) = call(
        endpoint,
        "segments.override",
        json!({"session":reference,"input":{"action":"cancel","segment_id":"op"}}),
        Some("6"),
    );
    assert_eq!(code, 0, "{cancelled}");
    assert_eq!(cancelled["data"]["cancel_status"], "before_admission");
    assert_eq!(cancelled["data"]["snapshot"]["revision"], "7");
    let (code, blocked) = call(endpoint, "segments.evaluate", evaluate, None);
    assert_eq!(code, 0);
    assert_eq!(
        blocked["data"]["decision"],
        json!({"state":"blocked","reason":"overridden"})
    );
    let (code, final_state) = call(endpoint, "segments.inspect", reference.clone(), None);
    assert_eq!(code, 0);
    assert_eq!(final_state["data"], cancelled["data"]["snapshot"]);
    let direct = sdk
        .inspect_skip_session(
            RequestContext::default(),
            serde_json::from_value(reference).unwrap(),
        )
        .unwrap();
    assert_eq!(serde_json::to_value(direct).unwrap(), final_state["data"]);
    assert_eq!(port.calls.load(Ordering::SeqCst), 0);
}
