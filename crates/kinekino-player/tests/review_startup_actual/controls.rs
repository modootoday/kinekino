//! Product review host controls: delivery receipts and observed state stay separate.
use super::*;
fn wait(
    endpoint: &str,
    root: &Path,
    label: &str,
    session: &Value,
    condition: impl Fn(&Value) -> bool,
) -> Value {
    wait_bounded(
        endpoint,
        root,
        label,
        session,
        Duration::from_secs(5),
        condition,
    )
}
fn wait_bounded(
    endpoint: &str,
    root: &Path,
    label: &str,
    session: &Value,
    budget: Duration,
    condition: impl Fn(&Value) -> bool,
) -> Value {
    let start = Instant::now();
    let cutoff = start + budget;
    for poll in 0.. {
        let value = call(
            endpoint,
            root,
            &format!("{label}-{poll}"),
            &request("reviews.playback.status", json!({"session":session})),
            true,
        );
        assert_eq!(value["data"]["observation"]["has_error"], false, "{value}");
        if condition(&value["data"]["observation"]) {
            assert!(
                Instant::now() < cutoff,
                "observation arrived outside explicit budget: {value}"
            );
            write_new(
                &root.join(format!("{label}-WAIT.json")),
                &json!({"budget_ms":budget.as_millis(),"elapsed_ms":start.elapsed().as_millis(),"observed":true,"last":value}),
            );
            return value;
        }
        assert!(
            Instant::now() < cutoff,
            "expected control observation absent: {value}"
        );
        thread::sleep(Duration::from_millis(20));
    }
    unreachable!()
}
fn control(
    endpoint: &str,
    root: &Path,
    label: &str,
    session: &Value,
    instance: &str,
    intent: Value,
    success: bool,
) -> Value {
    let value = call(
        endpoint,
        root,
        label,
        &request(
            "reviews.playback.control",
            json!({"session":session,"engine_instance_id":instance,"control":intent}),
        ),
        success,
    );
    if success {
        assert_eq!(value["data"]["receipt"]["engine_instance_id"], instance);
        assert_eq!(value["data"]["receipt"]["completion"], "unobserved");
    } else {
        assert_eq!(value["error"]["details"]["delivery"], "not_sent");
    }
    value
}
pub(super) fn exercise(
    endpoint: &str,
    root: &Path,
    session: &Value,
    connected: &Value,
    open: &Request,
    window_start: u64,
    outside_window: u64,
) -> Value {
    let instance = connected["data"]["observation"]["engine_instance_id"]
        .as_str()
        .unwrap();
    let wrong = instance
        .parse::<u64>()
        .unwrap()
        .checked_add(1)
        .unwrap()
        .to_string();
    assert_eq!(
        control(
            endpoint,
            root,
            "control-wrong-instance",
            session,
            &wrong,
            json!({"action":"pause"}),
            false
        )["error"]["code"],
        "revision_conflict"
    );
    control(
        endpoint,
        root,
        "control-pause",
        session,
        instance,
        json!({"action":"pause"}),
        true,
    );
    wait(endpoint, root, "paused", session, |observation| {
        observation["paused"] == true
    });
    assert_eq!(
        control(
            endpoint,
            root,
            "play-before-seek",
            session,
            instance,
            json!({"action":"play"}),
            false
        )["error"]["code"],
        "needs_input"
    );
    assert_eq!(
        control(
            endpoint,
            root,
            "seek-outside-window",
            session,
            instance,
            json!({"action":"seek","position_us":outside_window.to_string()}),
            false
        )["error"]["code"],
        "needs_input"
    );
    control(
        endpoint,
        root,
        "seek-window-start",
        session,
        instance,
        json!({"action":"seek","position_us":window_start.to_string()}),
        true,
    );
    wait(endpoint, root, "window-start", session, |observation| {
        observation["paused"] == true
            && observation["position_us"]
                .as_str()
                .and_then(|v| v.parse::<u64>().ok())
                .is_some_and(|v| v.abs_diff(window_start) <= 100_000)
    });
    control(
        endpoint,
        root,
        "control-volume",
        session,
        instance,
        json!({"action":"set_volume","volume_percent":33.0}),
        true,
    );
    control(
        endpoint,
        root,
        "control-muted",
        session,
        instance,
        json!({"action":"set_muted","muted":true}),
        true,
    );
    wait(endpoint, root, "volume-muted", session, |observation| {
        observation["volume_percent"].as_f64() == Some(33.0) && observation["muted"] == true
    });
    control(
        endpoint,
        root,
        "control-play",
        session,
        instance,
        json!({"action":"play"}),
        true,
    );
    wait(endpoint, root, "playing", session, |observation| {
        observation["paused"] == false
    });
    control(
        endpoint,
        root,
        "control-pause-again",
        session,
        instance,
        json!({"action":"pause"}),
        true,
    );
    wait(endpoint, root, "paused-again", session, |observation| {
        observation["paused"] == true
    });
    let mut foreign = session.clone();
    foreign["host_scope"] = json!("foreign-host");
    assert_eq!(
        call(
            endpoint,
            root,
            "close-foreign",
            &request("reviews.playback.close", json!({"session":foreign})),
            false
        )["error"]["code"],
        "revision_conflict"
    );
    let close = request("reviews.playback.close", json!({"session":session}));
    assert_eq!(
        call(endpoint, root, "close", &close, true)["data"]["close_requested"],
        true
    );
    assert_eq!(
        call(endpoint, root, "close-replay", &close, true)["data"]["close_requested"],
        true
    );
    let closed = wait(endpoint, root, "closed", session, |observation| {
        observation["owner_finished"] == true && observation["physical_closed"] == true
    });
    assert_eq!(closed["data"]["observation"]["family_empty"], true);
    assert_eq!(closed["data"]["observation"]["ipc_exited"], true);
    assert_eq!(
        control(
            endpoint,
            root,
            "control-after-close",
            session,
            instance,
            json!({"action":"pause"}),
            false
        )["error"]["code"],
        "not_found"
    );
    let mut reopen = open.clone();
    reopen.context.idempotency_key = Some("startup-controls-reopen".into());
    let reopened = call(endpoint, root, "reopen", &reopen, true);
    let next = reopened["data"]["session"].clone();
    assert_ne!(session, &next);
    wait_bounded(
        endpoint,
        root,
        "reopened",
        &next,
        Duration::from_secs(20),
        |observation| observation["connected"] == true,
    );
    write_new(
        &root.join("CONTROLS.json"),
        &json!({"closed":closed,"reopened":next,"control_completion":"unobserved_until_status","quality_approved":false}),
    );
    next
}
