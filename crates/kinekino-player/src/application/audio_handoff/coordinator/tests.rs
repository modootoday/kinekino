use super::*;

struct Ports {
    native: NativePlayback,
    calls: Vec<&'static str>,
    lease: bool,
    fail_pause: bool,
    fail_stop: bool,
    fail_restore: bool,
}
impl Default for Ports {
    fn default() -> Self {
        Self {
            native: NativePlayback {
                connected: true,
                time: Some(2.),
                duration: Some(1420.),
                width: Some(1920),
                height: Some(1080),
                volume: Some(80.),
                muted: Some(true),
                ..Default::default()
            },
            calls: vec![],
            lease: false,
            fail_pause: false,
            fail_stop: false,
            fail_restore: false,
        }
    }
}
fn failure() -> ApiError {
    ApiError::new(ErrorCode::EngineFailed, "Injected adapter failure")
}
fn web() -> WebSilenceObservation {
    WebSilenceObservation {
        document_generation: DecimalU64(11),
        observation_sequence: DecimalU64(3),
        paused: None,
        muted: None,
        view_muted: Some(true),
        confirmed_for_engine_instance_id: Some(DecimalU64(7)),
    }
}
impl AudioHandoffPorts for Ports {
    fn native_snapshot(&mut self) -> Result<(DecimalU64, NativePlayback), ApiError> {
        Ok((DecimalU64(7), self.native.clone()))
    }
    fn submit_native(
        &mut self,
        _: DecimalU64,
        control: PlaybackControl,
    ) -> Result<ControlReceipt, ApiError> {
        self.calls.push(match control {
            PlaybackControl::Play | PlaybackControl::Pause => "pause",
            PlaybackControl::SetVolume { .. } => "volume",
            PlaybackControl::SetMuted { .. } => "mute",
            _ => panic!("Unexpected control"),
        });
        // Queue admission deliberately does not update observations.
        Ok(ControlReceipt::enqueued(7))
    }
    fn silence_web(
        &mut self,
        _: DecimalU64,
        _: DecimalU64,
        _: DecimalU64,
        _: bool,
    ) -> Result<WebAudioSilenceReceipt, ApiError> {
        self.calls.push("silence");
        self.lease = true;
        Ok(WebAudioSilenceReceipt {
            lease_id: DecimalU64(1),
            previous_muted: false,
            observation: web(),
        })
    }
    fn pause_web(&mut self, _: DecimalU64) -> Result<(), ApiError> {
        self.calls.push("web_pause");
        if self.fail_pause {
            Err(failure())
        } else {
            Ok(())
        }
    }
    fn active_web_lease(&self) -> Result<Option<DecimalU64>, ApiError> {
        Ok(self.lease.then_some(DecimalU64(1)))
    }
    fn observe_web(
        &mut self,
        _: DecimalU64,
        _: DecimalU64,
    ) -> Result<WebSilenceObservation, ApiError> {
        Ok(web())
    }
    fn record_web_user_mute(&mut self, _: bool) -> Result<(), ApiError> {
        self.calls.push("user_mute");
        Ok(())
    }
    fn stop_native(&mut self, engine: DecimalU64) -> Result<StoppedNativeEngine, ApiError> {
        self.calls.push("stop");
        if self.fail_stop {
            return Err(failure());
        }
        StoppedNativeEngine::observed(engine, true, true, true, true)
    }
    fn restore_web(&mut self, _: DecimalU64, _: &StoppedNativeEngine) -> Result<(), ApiError> {
        self.calls.push("restore");
        if self.fail_restore {
            return Err(failure());
        }
        self.lease = false;
        Ok(())
    }
}
fn coordinator(now: Instant) -> (AudioHandoffCoordinator, AudioHandoffIntents) {
    let prepared = PreparedAudioHandoff::local_with_web_view(
        DecimalU64(11),
        true,
        AudioIntent::new(80., false).unwrap(),
        false,
        now,
    )
    .unwrap();
    let intents = AudioHandoffIntents::default();
    intents
        .bind_for_handoff(DecimalU64(7), prepared.requested())
        .unwrap();
    (
        prepared.bind(DecimalU64(7), intents.clone()).unwrap(),
        intents,
    )
}

#[test]
fn queue_acknowledgements_do_not_publish_readiness_before_actual_properties() {
    let now = Instant::now();
    let (mut handoff, intents) = coordinator(now);
    let mut ports = Ports::default();
    handoff.drive(&mut ports, now);
    assert_eq!(ports.calls, vec!["silence", "web_pause"]);
    handoff.drive(&mut ports, now);
    assert_eq!(
        ports.calls,
        vec!["silence", "web_pause", "pause", "volume", "mute"]
    );
    handoff.drive(&mut ports, now);
    assert!(!handoff.is_ready());
    ports.native.muted = Some(false);
    ports.native.pause_observed = true;
    handoff.drive(&mut ports, now);
    assert!(handoff.is_ready());
    assert_eq!(intents.reported_handoff().unwrap()["ready"], true);
    handoff.close(&mut ports);
    assert!(handoff.cleanup_complete());
    assert_eq!(&ports.calls[ports.calls.len() - 2..], &["stop", "restore"]);
}

#[test]
fn failed_stop_keeps_web_silent_and_explicit_retry_closes_before_restore() {
    let now = Instant::now();
    let (mut handoff, intents) = coordinator(now);
    let mut ports = Ports {
        fail_pause: true,
        fail_stop: true,
        ..Default::default()
    };
    handoff.drive(&mut ports, now);
    assert!(!handoff.cleanup_complete());
    assert!(ports.lease);
    assert_eq!(ports.calls, vec!["silence", "web_pause", "stop"]);
    assert!(intents.pending_requested(DecimalU64(7)).unwrap().is_none());
    handoff.drive(&mut ports, now);
    assert_eq!(
        ports.calls.len(),
        3,
        "Failure must not spin cleanup every frame"
    );
    ports.fail_stop = false;
    handoff.close(&mut ports);
    assert!(handoff.cleanup_complete());
    assert!(!ports.lease);
    assert_eq!(&ports.calls[3..], &["stop", "restore"]);
    assert!(
        handoff.failure().is_some(),
        "Original failure must survive successful cleanup"
    );
}

#[test]
fn failed_restore_retains_stop_proof_and_retries_only_restore() {
    let now = Instant::now();
    let (mut handoff, _) = coordinator(now);
    let mut ports = Ports::default();
    handoff.drive(&mut ports, now);
    ports.fail_restore = true;
    handoff.close(&mut ports);
    assert!(!handoff.cleanup_complete());
    assert!(ports.lease);
    ports.fail_restore = false;
    handoff.close(&mut ports);
    assert!(handoff.cleanup_complete());
    assert_eq!(
        ports.calls.iter().filter(|call| **call == "stop").count(),
        1
    );
    assert_eq!(
        ports
            .calls
            .iter()
            .filter(|call| **call == "restore")
            .count(),
        2
    );
}

#[test]
fn ready_ipc_disconnect_fails_and_keeps_guard_until_stop_proof() {
    let now = Instant::now();
    let (mut handoff, _) = coordinator(now);
    let mut ports = Ports::default();
    handoff.drive(&mut ports, now);
    handoff.drive(&mut ports, now);
    ports.native.muted = Some(false);
    ports.native.pause_observed = true;
    handoff.drive(&mut ports, now);
    assert!(handoff.is_ready());
    ports.native.connected = false;
    ports.fail_stop = true;
    handoff.drive(&mut ports, now);
    assert!(!handoff.is_ready());
    assert!(handoff.failure().is_some());
    assert!(ports.lease);
    assert!(!handoff.cleanup_complete());
    assert!(!ports.calls.contains(&"restore"));
    ports.fail_stop = false;
    handoff.close(&mut ports);
    assert!(handoff.cleanup_complete());
    assert!(!ports.lease);
    assert!(handoff.failure().is_some());
}
