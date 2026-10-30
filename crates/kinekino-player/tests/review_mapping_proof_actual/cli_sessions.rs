//! Actual normal CLI -> LocalHost -> common SDK session service, preserving each child receipt.
use kinekino_player::{
    application::{
        api::Request,
        review_playback::sessions::{ReviewPlaybackSessions, Selection, SessionRef, SessionView},
    },
    assembly::review_playback::ReviewTransportHost,
    domain::playback_control::PlaybackControl,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
fn call(endpoint: &str, output: &Path, label: &str, request: &Request, success: bool) -> Value {
    let directory = output.join(label);
    fs::create_dir(&directory).unwrap();
    super::write_new(
        &directory.join("request.json"),
        &serde_json::to_value(request).unwrap(),
    );
    let mut runner = super::cli::CliRunner::start(endpoint, request);
    let receipt = runner.finish_preserving(&directory);
    assert_eq!(receipt.code == 0, success, "{label}: {receipt:?}");
    assert_eq!(
        receipt.response["ok"], success,
        "{label}: {:?}",
        receipt.response
    );
    receipt.response.clone()
}
fn wait(sessions: &ReviewPlaybackSessions, session: &SessionRef, connected: bool) -> SessionView {
    let cutoff = Instant::now() + Duration::from_secs(15);
    loop {
        let view = sessions.status(session).unwrap();
        if connected {
            assert!(!view.observation.has_error, "{view:?}");
            if view.observation.connected && view.observation.engine_instance_id.is_some() {
                return view;
            }
        } else if view.observation.owner_finished && view.observation.physical_closed {
            return view;
        }
        assert!(
            Instant::now() < cutoff,
            "original session not ready/closed: {view:?}"
        );
        thread::sleep(Duration::from_millis(20));
    }
}
fn wait_observed(
    sessions: &ReviewPlaybackSessions,
    session: &SessionRef,
    condition: impl Fn(&SessionView) -> bool,
) -> SessionView {
    let cutoff = Instant::now() + Duration::from_secs(5);
    loop {
        let view = sessions.status(session).unwrap();
        assert!(!view.observation.has_error, "{view:?}");
        if condition(&view) {
            return view;
        }
        assert!(
            Instant::now() < cutoff,
            "session control was not observed: {view:?}"
        );
        thread::sleep(Duration::from_millis(20));
    }
}
fn control(
    endpoint: &str,
    output: &Path,
    label: &str,
    session: &SessionRef,
    instance: u64,
    intent: PlaybackControl,
    success: bool,
) -> Value {
    let mut request = Request::new("reviews.playback.control");
    request.payload =
        json!({"session":session,"engine_instance_id":instance.to_string(),"control":intent});
    let response = call(endpoint, output, label, &request, success);
    if success {
        assert_eq!(response["data"]["session"], json!(session));
        assert_eq!(
            response["data"]["receipt"]["engine_instance_id"],
            instance.to_string()
        );
        assert_eq!(response["data"]["receipt"]["completion"], "unobserved");
    } else {
        assert_eq!(response["error"]["details"]["delivery"], "not_sent");
    }
    response
}
pub fn verify(
    host: ReviewTransportHost,
    sessions: Arc<ReviewPlaybackSessions>,
    selection: Selection,
    output: &Path,
) -> Value {
    let cli = super::custody::Guard::acquire(Path::new(crate::cli_exe::kinekino_cli_exe()));
    super::write_new(&output.join("CLI-WITNESS.json"), &cli.evidence());
    let cli_witness = super::custody::Guard::acquire(&output.join("CLI-WITNESS.json"));
    let endpoint = host.endpoint();
    let caps = call(
        endpoint,
        output,
        "cli-capabilities",
        &Request::new("capabilities"),
        true,
    );
    assert_eq!(caps["data"]["review_playback"]["available"], true);
    let witness: Value =
        serde_json::from_slice(&fs::read(output.join("WITNESS.json")).unwrap()).unwrap();
    let mut create = Request::new("reviews.association.create");
    create.context.idempotency_key = Some("cli-configured-create".into());
    create.payload = json!({"case":selection.case,"association_id":"cli-configured","media":witness["media"],"note":"configured source and playback SDK technical validation"});
    let created = call(endpoint, output, "cli-source-create", &create, true);
    assert_eq!(
        created["data"]["receipt"]["body"]["association"]["revision"],
        "1"
    );
    let mut verify = Request::new("reviews.association.revise");
    verify.context.idempotency_key = Some("cli-configured-verify".into());
    verify.context.expected_revision = Some("1".into());
    verify.payload = create.payload.clone();
    verify.payload["timeline_evidence"] = witness["evidence"].clone();
    let verified = call(endpoint, output, "cli-source-verify", &verify, true);
    assert_eq!(
        verified["data"]["receipt"]["body"]["association"]["revision"],
        "2"
    );
    assert!(
        verified["data"]["receipt"]["body"]["association"]["timeline_status"]["verified"]
            .is_object()
    );
    assert_eq!(verified["data"]["playback_ready"], false);
    let replay = call(endpoint, output, "cli-source-verify-replay", &verify, true);
    assert_eq!(replay["data"]["receipt"], verified["data"]["receipt"]);
    verify.context.idempotency_key = Some("cli-configured-foreign-proof".into());
    verify.context.expected_revision = Some("2".into());
    verify.payload["timeline_evidence"]["asset_id"] = json!("foreign-proof");
    let denied = call(endpoint, output, "cli-source-foreign-proof", &verify, false);
    assert_eq!(denied["error"]["code"], "revision_conflict");
    let mut inspect = Request::new("reviews.association.inspect");
    inspect.payload = json!({"case":selection.case,"association_id":"cli-configured"});
    let inspected = call(endpoint, output, "cli-source-inspect", &inspect, true);
    assert_eq!(
        inspected["data"]["stored"]["association"],
        verified["data"]["receipt"]["body"]["association"]
    );
    let mut selection = selection;
    selection.association_id = "cli-configured".into();
    for (label, command) in [
        ("cli-schema-open", "reviews.playback.open"),
        ("cli-schema-status", "reviews.playback.status"),
        ("cli-schema-close", "reviews.playback.close"),
        ("cli-schema-control", "reviews.playback.control"),
    ] {
        let mut request = Request::new("schema");
        request.payload = json!({"command":command});
        assert_eq!(
            call(endpoint, output, label, &request, true)["data"]["status"],
            "supported"
        );
    }
    let mut open = Request::new("reviews.playback.open");
    open.context.idempotency_key = Some("cli-open".into());
    open.payload = json!({"selection":selection});
    let opened = call(endpoint, output, "cli-open", &open, true);
    let reference: SessionRef = serde_json::from_value(opened["data"]["session"].clone()).unwrap();
    assert_eq!(
        call(endpoint, output, "cli-open-replay", &open, true)["data"]["session"],
        opened["data"]["session"]
    );
    let mut invalid = open.clone();
    invalid.payload["executable"] = json!("untrusted.exe");
    assert_eq!(
        call(endpoint, output, "cli-invalid-path", &invalid, false)["error"]["code"],
        "invalid_argument"
    );
    let connected = wait(&sessions, &reference, true);
    let instance = connected.observation.engine_instance_id.unwrap().0;
    assert_eq!(
        control(
            endpoint,
            output,
            "cli-play-before-seek",
            &reference,
            instance,
            PlaybackControl::Play,
            false
        )["error"]["code"],
        "needs_input"
    );
    assert_eq!(
        control(
            endpoint,
            output,
            "cli-wrong-control-instance",
            &reference,
            instance + 1,
            PlaybackControl::Pause,
            false
        )["error"]["code"],
        "revision_conflict"
    );
    let mut foreign_control = reference.clone();
    foreign_control.selection.association_revision =
        kinekino_player::domain::api_values::DecimalU64(99);
    assert_eq!(
        control(
            endpoint,
            output,
            "cli-foreign-control-selection",
            &foreign_control,
            instance,
            PlaybackControl::Pause,
            false
        )["error"]["code"],
        "revision_conflict"
    );
    assert_eq!(
        control(
            endpoint,
            output,
            "cli-seek-outside-window",
            &reference,
            instance,
            PlaybackControl::Seek {
                position_us: kinekino_player::domain::api_values::DecimalU64(26_000_000)
            },
            false
        )["error"]["code"],
        "needs_input"
    );
    control(
        endpoint,
        output,
        "cli-control-seek-start",
        &reference,
        instance,
        PlaybackControl::Seek {
            position_us: kinekino_player::domain::api_values::DecimalU64(17_000_000),
        },
        true,
    );
    wait_observed(&sessions, &reference, |view| {
        view.observation
            .position_us
            .is_some_and(|v| v.0.abs_diff(17_000_000) <= 100_000)
            && view.observation.paused == Some(true)
    });
    control(
        endpoint,
        output,
        "cli-control-volume",
        &reference,
        instance,
        PlaybackControl::SetVolume {
            volume_percent: 33.0,
        },
        true,
    );
    control(
        endpoint,
        output,
        "cli-control-muted",
        &reference,
        instance,
        PlaybackControl::SetMuted { muted: true },
        true,
    );
    wait_observed(&sessions, &reference, |view| {
        view.observation.volume_percent == Some(33.0) && view.observation.muted == Some(true)
    });
    let mut status = Request::new("reviews.playback.status");
    status.payload = json!({"session":reference});
    let observed = call(endpoint, output, "cli-status-connected", &status, true);
    assert_eq!(observed["data"]["observation"]["connected"], true);
    let properties = call(
        endpoint,
        output,
        "cli-status-control-properties",
        &status,
        true,
    );
    assert_eq!(properties["data"]["observation"]["volume_percent"], 33.0);
    assert_eq!(properties["data"]["observation"]["muted"], true);
    control(
        endpoint,
        output,
        "cli-control-play",
        &reference,
        instance,
        PlaybackControl::Play,
        true,
    );
    wait_observed(&sessions, &reference, |view| {
        view.observation.paused == Some(false)
            && view
                .observation
                .position_us
                .is_some_and(|v| v.0 > 17_000_000)
    });
    control(
        endpoint,
        output,
        "cli-control-pause",
        &reference,
        instance,
        PlaybackControl::Pause,
        true,
    );
    wait_observed(&sessions, &reference, |view| {
        view.observation.paused == Some(true)
    });
    assert_eq!(
        call(endpoint, output, "cli-status-manual-paused", &status, true)["data"]["observation"]["paused"],
        true
    );
    control(
        endpoint,
        output,
        "cli-control-seek-near-end",
        &reference,
        instance,
        PlaybackControl::Seek {
            position_us: kinekino_player::domain::api_values::DecimalU64(25_250_000),
        },
        true,
    );
    wait_observed(&sessions, &reference, |view| {
        view.observation.paused == Some(true)
            && view
                .observation
                .position_us
                .is_some_and(|v| v.0.abs_diff(25_250_000) <= 100_000)
    });
    control(
        endpoint,
        output,
        "cli-control-play-to-end",
        &reference,
        instance,
        PlaybackControl::Play,
        true,
    );
    let boundary_paused = wait_observed(&sessions, &reference, |view| {
        view.observation.paused == Some(true)
            && view
                .observation
                .position_us
                .is_some_and(|v| v.0 >= 25_500_000)
    });
    let boundary = call(
        endpoint,
        output,
        "cli-status-boundary-paused",
        &status,
        true,
    );
    assert_eq!(boundary["data"]["observation"]["paused"], true);
    assert_eq!(
        control(
            endpoint,
            output,
            "cli-play-at-end",
            &reference,
            instance,
            PlaybackControl::Play,
            false
        )["error"]["code"],
        "needs_input"
    );
    let mut close = Request::new("reviews.playback.close");
    close.payload = status.payload.clone();
    let mut foreign = close.clone();
    foreign.payload["session"]["host_scope"] = json!("foreign-host");
    assert_eq!(
        call(endpoint, output, "cli-foreign-host", &foreign, false)["error"]["code"],
        "revision_conflict"
    );
    assert_eq!(
        call(endpoint, output, "cli-close", &close, true)["data"]["close_requested"],
        true
    );
    assert_eq!(
        call(endpoint, output, "cli-close-replay", &close, true)["data"]["close_requested"],
        true
    );
    let closed = wait(&sessions, &reference, false);
    assert!(
        closed.observation.family_empty
            && closed.observation.ipc_exited
            && !closed.observation.has_error
    );
    let closed_cli = call(endpoint, output, "cli-status-closed", &status, true);
    assert_eq!(
        control(
            endpoint,
            output,
            "cli-control-after-close",
            &reference,
            instance,
            PlaybackControl::Pause,
            false
        )["error"]["code"],
        "not_found"
    );
    assert_eq!(closed_cli["data"]["observation"]["owner_finished"], true);
    assert_eq!(closed_cli["data"]["observation"]["physical_closed"], true);
    assert_eq!(
        call(endpoint, output, "cli-open-after-closed", &open, true)["data"],
        closed_cli["data"]
    );
    open.context.idempotency_key = Some("cli-close-after-open".into());
    let pending = call(endpoint, output, "cli-starting-open", &open, true);
    let pending_ref: SessionRef =
        serde_json::from_value(pending["data"]["session"].clone()).unwrap();
    close.payload = json!({"session":pending_ref});
    assert_eq!(
        call(endpoint, output, "cli-starting-close", &close, true)["data"]["close_requested"],
        true
    );
    let starting_closed = wait(&sessions, &pending_ref, false);
    assert!(!starting_closed.observation.has_error);
    if starting_closed.observation.process_created {
        assert!(starting_closed.observation.family_empty && starting_closed.observation.ipc_exited);
    }
    status.payload = json!({"session":pending_ref});
    assert_eq!(
        call(
            endpoint,
            output,
            "cli-starting-closed-status",
            &status,
            true
        )["data"]["observation"]["physical_closed"],
        true
    );
    open.context.idempotency_key = Some("cli-unverified".into());
    open.payload["selection"]["association_revision"] = json!("1");
    let unverified = call(endpoint, output, "cli-unverified-open", &open, true);
    let unverified_ref = serde_json::from_value(unverified["data"]["session"].clone()).unwrap();
    let failed = wait(&sessions, &unverified_ref, false);
    assert!(!failed.observation.process_created && failed.observation.has_error);
    status.payload = json!({"session":unverified_ref});
    let failed_cli = call(endpoint, output, "cli-unverified-status", &status, true);
    assert_eq!(
        failed_cli["data"]["observation"]["failure"]["code"],
        "needs_input"
    );
    assert_eq!(failed_cli["data"]["observation"]["process_created"], false);
    cli.assert_unchanged();
    cli_witness.assert_unchanged();
    let cli_processes = fs::read_dir(output)
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.path().join("process.json").is_file())
        .count();
    assert_eq!(cli_processes, 40);
    let mut pending = host.begin_shutdown(Instant::now() + Duration::from_secs(10));
    let cutoff = Instant::now() + Duration::from_secs(10);
    let shutdown = loop {
        let observation = pending.poll();
        assert!(observation.first_error.is_none(), "{observation:?}");
        if observation.complete {
            assert!(observation.succeeded());
            break json!({"pipe":format!("{:?}",observation.transport.pipe),"sdk":format!("{:?}",observation.transport.sdk),"dispatch":format!("{:?}",observation.transport.dispatch),"sessions":observation.sessions,"complete":true,"first_error":null});
        }
        assert!(
            Instant::now() < cutoff,
            "transport/session shutdown incomplete: {observation:?}"
        );
        thread::sleep(Duration::from_millis(20));
    };
    super::write_new(&output.join("TRANSPORT-SHUTDOWN.json"), &shutdown);
    json!({"configured_source":{"verified":verified,"replay":replay,"foreign_proof":denied,"inspected":inspected},"connected":connected,"boundary_paused":boundary_paused,"boundary_pause_overshoot_us":boundary_paused.observation.position_us.unwrap().0-25_500_000,"closed":closed,"close_after_open":starting_closed,"unverified":failed_cli,"cli":cli.evidence(),"cli_processes":cli_processes,"transport_shutdown":shutdown,"transport_commands_registered":true,"source_scope":"sealed_sprite_only","frame_exact":false,"quality_approved":false})
}
