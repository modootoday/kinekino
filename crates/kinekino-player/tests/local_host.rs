#![cfg(windows)]
#[path = "support/cli_exe.rs"]
mod cli_exe;
#[path = "support/local_host_cli.rs"]
pub mod cli_fixture;
use cli_fixture::invoke;
#[path = "support/local_host_flight.rs"]
mod flight_fixture;
use kinekino_player::{
    application::{
        api::{
            ApiError, ApplicationClient, ApplicationClientHandle, ErrorCode, PlayerSdk, Request,
            Response,
        },
        local_host::MAX_REQUEST_BYTES,
        playback_control::{ControlReceipt, PlaybackControlPort},
    },
    domain::{local_endpoint::LocalEndpoint, playback_control::PlaybackControl},
    infrastructure::local_host::{
        EndpointClient, LocalHost, TransportDiagnostics, TransportOptions,
    },
};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::windows::named_pipe::ClientOptions,
};

struct Port(Arc<AtomicUsize>);
impl PlaybackControlPort for Port {
    fn submit(&self, _: PlaybackControl) -> Result<ControlReceipt, ApiError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(ControlReceipt::enqueued(77))
    }
}
#[test]
fn real_cli_matches_explicit_host_sdk_discovery_schemas_errors_and_control_receipts() {
    kinekino_platform::install();
    let calls = Arc::new(AtomicUsize::new(0));
    let sdk: ApplicationClientHandle =
        Arc::new(PlayerSdk::default().with_playback_control(Port(calls.clone())));
    let host = LocalHost::start(sdk.clone(), TransportOptions::default()).unwrap();
    for command in ["version", "capabilities", "playback.status", "models.list"] {
        let mut request = Request::new(command);
        request.context.request_id = "host-요청".into();
        let direct = sdk.execute(request.clone());
        let (code, remote) = invoke(host.endpoint(), &request);
        assert_eq!(
            code,
            direct.error.as_ref().map_or(0, |e| e.exit_code() as i32),
            "{command}: {remote}"
        );
        assert_eq!(remote, serde_json::to_value(direct).unwrap());
    }
    for command in kinekino_player::application::api::capabilities::COMMANDS {
        let mut request = Request::new("schema");
        request.payload = json!({"command":command});
        let (code, remote) = invoke(host.endpoint(), &request);
        assert_eq!(code, 0);
        assert_eq!(remote, serde_json::to_value(sdk.execute(request)).unwrap());
    }
    for action in [
        PlaybackControl::Play,
        PlaybackControl::Pause,
        PlaybackControl::Seek {
            position_us: kinekino_player::domain::api_values::DecimalU64(100),
        },
        PlaybackControl::SetVolume {
            volume_percent: 25.,
        },
    ] {
        let mut request = Request::new(action.command());
        request.payload = action.payload();
        let (code, response) = invoke(host.endpoint(), &request);
        assert_eq!(code, 0);
        assert_eq!(
            response["data"],
            json!({"acceptance":"accepted","completion":"unobserved","engine_instance_id":"77"})
        );
    }
    assert_eq!(calls.load(Ordering::SeqCst), 4);
}
#[test]
fn strict_endpoint_rejects_remote_paths_urls_programs_and_stale_hosts() {
    kinekino_platform::install();
    for endpoint in [
        "https://private/?token=secret",
        r"\\remote\pipe\secret",
        r"\\.\pipe\secret",
        "cmd.exe",
        "pipe:com.kinekino.player.v1.0.0000000000000001.00000000000000000000000000000000",
    ] {
        assert!(EndpointClient::new(endpoint, TransportOptions::default()).is_err());
    }
    let host =
        LocalHost::start(Arc::new(PlayerSdk::default()), TransportOptions::default()).unwrap();
    let endpoint = host.endpoint().to_owned();
    let (_, missing) = invoke(&endpoint, &Request::new("playback.play"));
    assert_eq!(missing["error"]["code"], "unsupported");
    drop(host);
    let (code, missing) = invoke(&endpoint, &Request::new("version"));
    assert_eq!(code, 4);
    assert_eq!(missing["error"]["details"]["delivery"], "not_sent");
    assert!(!missing.to_string().contains(&endpoint));
}
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
#[test]
fn malformed_oversized_and_truncated_frames_do_not_dispatch_and_host_recovers() {
    kinekino_platform::install();
    struct Counting(Arc<AtomicUsize>);
    impl ApplicationClient for Counting {
        fn execute(&self, request: Request) -> Response {
            self.0.fetch_add(1, Ordering::SeqCst);
            PlayerSdk::default().execute(request)
        }
    }
    let calls = Arc::new(AtomicUsize::new(0));
    let trace = TransportDiagnostics::default();
    let host = LocalHost::start_observed(
        Arc::new(Counting(calls.clone())),
        TransportOptions {
            // Margin for loaded desktops: the whole session (verify + read + reply) must fit.
            timeout: Duration::from_millis(750),
        },
        trace.clone(),
    )
    .unwrap();
    let path = pipe_path(&LocalEndpoint::parse(host.endpoint()).unwrap());
    runtime().block_on(async {
        for length in [0, MAX_REQUEST_BYTES as u32 + 1, u32::MAX] {
            let mut pipe = open(&path).await;
            pipe.write_u32_le(length).await.unwrap();
            let mut byte = [0];
            assert!(
                tokio::time::timeout(Duration::from_secs(1), pipe.read_exact(&mut byte))
                    .await
                    .unwrap()
                    .is_err()
            );
        }
        {
            let mut pipe = open(&path).await;
            pipe.write_u32_le(50).await.unwrap();
            pipe.write_all(b"partial").await.unwrap();
        }
        let mut pipe = open(&path).await;
        let malformed = br#"{"command":"version","token":"SECRET"}"#;
        pipe.write_u32_le(malformed.len() as u32).await.unwrap();
        pipe.write_all(malformed).await.unwrap();
        let traced = |error: std::io::Error| -> ! {
            panic!("malformed reply: {error}; trace={:?}", trace.snapshot())
        };
        let length = pipe.read_u32_le().await.unwrap_or_else(|e| traced(e));
        let mut bytes = vec![0; length as usize];
        pipe.read_exact(&mut bytes)
            .await
            .unwrap_or_else(|e| traced(e));
        let rotation = trace.snapshot().len();
        pipe.write_u8(6).await.unwrap();
        let response: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(response["error"]["code"], "invalid_argument");
        assert!(!String::from_utf8(bytes).unwrap().contains("SECRET"));
        assert!(
            trace.wait_for("next_instance_ready", rotation, Duration::from_secs(1)),
            "ACK/session rotation barrier missing: {:?}",
            trace.snapshot()
        );
    });
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let (code, response) = invoke(host.endpoint(), &Request::new("version"));
    eprintln!("malformed recovery trace: {:?}", trace.snapshot());
    assert_eq!(code, 0, "response={response}; trace={:?}", trace.snapshot());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
/// Named-pipe address the Windows IPC adapter derives from the endpoint id (ADR 0007).
fn pipe_path(endpoint: &LocalEndpoint) -> String {
    format!(r"\\.\pipe\{}", endpoint.id().name())
}
async fn open(path: &str) -> tokio::net::windows::named_pipe::NamedPipeClient {
    // The host serves one session at a time; a stalled partial frame holds it up to the timeout.
    for _ in 0..600 {
        if let Ok(pipe) = ClientOptions::new().open(path) {
            return pipe;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    panic!("fixture pipe unavailable")
}
#[test]
fn transport_timeout_and_shutdown_report_uncertainty_without_retrying_sdk() {
    kinekino_platform::install();
    let mut fixture = flight_fixture::Flight::idle(TransportOptions::default());
    let client = EndpointClient::new(
        fixture.host.as_ref().unwrap().endpoint(),
        TransportOptions {
            timeout: Duration::from_millis(50),
        },
    )
    .unwrap();
    let response = client.execute(Request::new("version"));
    assert_eq!(response.error.as_ref().unwrap().code, ErrorCode::Timeout);
    assert_eq!(
        response.error.unwrap().details["delivery"],
        "execution_unknown"
    );
    fixture.await_started_for(Duration::from_secs(1));
    let before = Instant::now();
    drop(fixture.host.take());
    assert!(before.elapsed() < Duration::from_secs(1));
    fixture.release_and_finish();
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 1);
}
#[test]
fn host_drop_cancels_an_idle_partial_client_without_waiting_for_deadline() {
    kinekino_platform::install();
    let host =
        LocalHost::start(Arc::new(PlayerSdk::default()), TransportOptions::default()).unwrap();
    let endpoint = LocalEndpoint::parse(host.endpoint()).unwrap();
    let rt = runtime();
    let mut pipe = rt.block_on(open(&pipe_path(&endpoint)));
    rt.block_on(pipe.write_u32_le(100)).unwrap();
    let start = Instant::now();
    drop(host);
    assert!(start.elapsed() < Duration::from_secs(1));
    let response = rt.block_on(async {
        let mut byte = [0];
        tokio::time::timeout(Duration::from_secs(1), pipe.read_exact(&mut byte))
            .await
            .unwrap()
    });
    assert!(response.is_err());
}

#[test]
fn real_cli_disconnect_during_execution_is_unknown_and_never_falls_back() {
    kinekino_platform::install();
    let mut fixture = flight_fixture::Flight::start();
    fixture.await_started();
    drop(fixture.host.take());
    let receipt = fixture.runner.as_mut().unwrap().finish();
    eprintln!(
        "disconnect {receipt:?}; trace={:?}",
        fixture.trace.snapshot()
    );
    let code = receipt.code;
    let response = receipt.response.clone();
    assert_eq!(code, 8);
    assert_eq!(
        response["error"]["details"]["delivery"],
        "execution_unknown"
    );
    assert_eq!(response["error"]["retryable"], false);
    fixture.release_and_finish();
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 1);
}

#[test]
fn overlarge_cli_input_is_rejected_before_host_dispatch() {
    kinekino_platform::install();
    let calls = Arc::new(AtomicUsize::new(0));
    let host = LocalHost::start(
        Arc::new(PlayerSdk::default().with_playback_control(Port(calls.clone()))),
        TransportOptions::default(),
    )
    .unwrap();
    let mut request = Request::new("playback.play");
    request.payload = json!({"oversized":"x".repeat(MAX_REQUEST_BYTES)});
    let (code, response) = invoke(host.endpoint(), &request);
    assert_eq!(code, 2);
    assert_eq!(response["error"]["code"], "invalid_argument");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn busy_executor_does_not_queue_or_replay_a_second_request_after_timeout() {
    kinekino_platform::install();
    let mut fixture = flight_fixture::Flight::idle(TransportOptions {
        timeout: Duration::from_millis(50),
    });
    let client = EndpointClient::new(
        fixture.host.as_ref().unwrap().endpoint(),
        TransportOptions::default(),
    )
    .unwrap();
    assert_eq!(
        client
            .execute(Request::new("version"))
            .error
            .unwrap()
            .details["delivery"],
        "execution_unknown"
    );
    let rejected = client.execute(Request::new("version")).error.unwrap();
    assert_eq!(rejected.code, ErrorCode::ResourceExhausted);
    assert_eq!(rejected.details["delivery"], "not_dispatched");
    assert!(fixture.host.as_ref().unwrap().is_running());
    drop(fixture.host.take());
    fixture.release_and_finish();
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 1);
}

#[test]
fn stalled_response_reader_and_overlarge_sdk_response_remain_bounded() {
    kinekino_platform::install();
    struct Large(usize);
    impl ApplicationClient for Large {
        fn execute(&self, request: Request) -> Response {
            let mut response = PlayerSdk::default().execute(request);
            response.data = Some(json!({"blob":"x".repeat(self.0)}));
            response
        }
    }
    let host = LocalHost::start(
        Arc::new(Large(MAX_REQUEST_BYTES)),
        TransportOptions {
            timeout: Duration::from_millis(50),
        },
    )
    .unwrap();
    let endpoint = LocalEndpoint::parse(host.endpoint()).unwrap();
    let rt = runtime();
    let mut pipe = rt.block_on(open(&pipe_path(&endpoint)));
    rt.block_on(async {
        let request = serde_json::to_vec(&Request::new("version")).unwrap();
        pipe.write_u32_le(request.len() as u32).await.unwrap();
        pipe.write_all(&request).await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
    });
    assert!(host.is_running());
    let before = Instant::now();
    drop(host);
    assert!(before.elapsed() < Duration::from_secs(1));
    drop(pipe);
    let host = LocalHost::start(
        Arc::new(Large(
            kinekino_player::application::local_host::MAX_RESPONSE_BYTES + 1,
        )),
        TransportOptions::default(),
    )
    .unwrap();
    let (code, response) = invoke(host.endpoint(), &Request::new("version"));
    assert_eq!(code, 7);
    assert_eq!(response["error"]["code"], "resource_exhausted");
    assert_eq!(
        response["error"]["details"]["delivery"],
        "execution_unknown"
    );
}
