use super::*;

fn snapshot() -> NativePlayback {
    NativePlayback {
        connected: true,
        time: Some(2.),
        duration: Some(1420.),
        width: Some(1920),
        height: Some(1080),
        volume: Some(80.),
        muted: Some(true),
        ..Default::default()
    }
}
fn web(generation: u64, paused: Option<bool>, muted: Option<bool>) -> WebSilenceObservation {
    WebSilenceObservation {
        document_generation: DecimalU64(generation),
        observation_sequence: DecimalU64(2),
        paused,
        muted,
        view_muted: None,
        confirmed_for_engine_instance_id: None,
    }
}
fn confirmed_web(generation: u64) -> WebSilenceObservation {
    WebSilenceObservation {
        document_generation: DecimalU64(generation),
        observation_sequence: DecimalU64(2),
        paused: None,
        muted: None,
        view_muted: Some(true),
        confirmed_for_engine_instance_id: Some(DecimalU64(7)),
    }
}
fn start(now: Instant, muted: bool) -> NativeAudioHandoff {
    NativeAudioHandoff::begin(
        DecimalU64(7),
        AudioOrigin::Web {
            document_generation: DecimalU64(11),
            observed_sequence: DecimalU64(1),
            expected_duration_seconds: 1420.,
        },
        AudioIntent::new(80., muted).unwrap(),
        now,
    )
    .unwrap()
}
fn reach_submission(handoff: &mut NativeAudioHandoff, native: &NativePlayback, now: Instant) {
    assert_eq!(
        handoff.observe(DecimalU64(7), native, None, now).unwrap(),
        Some(AudioHandoffAction::SilenceWeb {
            expected_document_generation: DecimalU64(11),
            after_observation_sequence: DecimalU64(1),
            document_bound: true,
        })
    );
    let silent = confirmed_web(11);
    assert!(matches!(
        handoff
            .observe(DecimalU64(7), native, Some(&silent), now)
            .unwrap(),
        Some(AudioHandoffAction::ApplyNativeAudio { .. })
    ));
}

#[test]
fn web_pause_request_never_releases_native_audio_without_matching_silence_observation() {
    let now = Instant::now();
    let mut handoff = start(now, false);
    let native = snapshot();
    assert!(matches!(
        handoff.observe(DecimalU64(7), &native, None, now).unwrap(),
        Some(AudioHandoffAction::SilenceWeb { .. })
    ));
    assert_eq!(handoff.phase(), AudioHandoffPhase::AwaitingWebSilence);
    let mut old_silence = confirmed_web(11);
    old_silence.observation_sequence = DecimalU64(1);
    assert_eq!(
        handoff
            .observe(DecimalU64(7), &native, Some(&old_silence), now)
            .unwrap(),
        None
    );
    for sample in [
        None,
        Some(web(11, None, None)),
        Some(web(11, Some(false), Some(false))),
        Some(web(11, Some(true), Some(true))),
    ] {
        assert_eq!(
            handoff
                .observe(DecimalU64(7), &native, sample.as_ref(), now)
                .unwrap(),
            None
        );
    }
    assert_eq!(handoff.phase(), AudioHandoffPhase::AwaitingWebSilence);
    let silent = confirmed_web(11);
    assert_eq!(
        handoff
            .observe(DecimalU64(7), &native, Some(&silent), now)
            .unwrap(),
        Some(AudioHandoffAction::ApplyNativeAudio {
            expected_engine_instance_id: DecimalU64(7),
            plan_id: DecimalU64(1),
            intent: AudioIntent::new(80., false).unwrap(),
            paused: None,
        })
    );
    assert_eq!(
        handoff
            .observe(DecimalU64(7), &native, Some(&silent), now)
            .unwrap(),
        None
    );
}

#[test]
fn enqueue_receipts_and_echoed_matching_values_do_not_mark_audio_ready() {
    let now = Instant::now();
    let mut handoff = start(now, false);
    let mut native = snapshot();
    reach_submission(&mut handoff, &native, now);
    native.muted = Some(false);
    handoff.observe(DecimalU64(7), &native, None, now).unwrap();
    assert_eq!(handoff.phase(), AudioHandoffPhase::AwaitingNativeSubmission);
    let receipt = ControlReceipt::enqueued(7);
    handoff
        .native_audio_submitted(DecimalU64(1), &receipt, &receipt, now)
        .unwrap();
    assert_eq!(handoff.phase(), AudioHandoffPhase::AwaitingNativeAudio);
    native.muted = None;
    handoff.observe(DecimalU64(7), &native, None, now).unwrap();
    assert_eq!(handoff.phase(), AudioHandoffPhase::AwaitingNativeAudio);
    native.muted = Some(false);
    native.volume = None;
    handoff.observe(DecimalU64(7), &native, None, now).unwrap();
    assert_eq!(handoff.phase(), AudioHandoffPhase::AwaitingNativeAudio);
    native.volume = Some(80.);
    handoff.observe(DecimalU64(7), &native, None, now).unwrap();
    assert_eq!(handoff.phase(), AudioHandoffPhase::Ready);
}

#[test]
fn explicit_user_mute_is_preserved_and_ready_never_reapplies_audio_to_later_user_changes() {
    let now = Instant::now();
    let mut handoff = start(now, true);
    let mut native = snapshot();
    reach_submission(&mut handoff, &native, now);
    let receipt = ControlReceipt::enqueued(7);
    handoff
        .native_audio_submitted(DecimalU64(1), &receipt, &receipt, now)
        .unwrap();
    handoff.observe(DecimalU64(7), &native, None, now).unwrap();
    assert_eq!(handoff.phase(), AudioHandoffPhase::Ready);
    native.muted = Some(false);
    native.volume = Some(23.);
    assert_eq!(
        handoff
            .observe(DecimalU64(7), &native, None, now + HANDOFF_BUDGET)
            .unwrap(),
        None
    );
    assert_eq!(handoff.phase(), AudioHandoffPhase::Ready);
    assert_ne!(
        AudioIntent::new(0., false).unwrap(),
        AudioIntent::new(0., true).unwrap()
    );
}

#[test]
fn stale_native_web_and_receipt_identities_cannot_advance_handoff() {
    let now = Instant::now();
    let mut handoff = start(now, false);
    let native = snapshot();
    assert_eq!(
        handoff
            .observe(DecimalU64(6), &native, None, now)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(handoff.phase(), AudioHandoffPhase::AwaitingMedia);
    handoff.observe(DecimalU64(7), &native, None, now).unwrap();
    let stale = web(10, Some(true), Some(true));
    assert_eq!(
        handoff
            .observe(DecimalU64(7), &native, Some(&stale), now)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(handoff.phase(), AudioHandoffPhase::AwaitingWebSilence);
    handoff
        .observe(DecimalU64(7), &native, Some(&confirmed_web(11)), now)
        .unwrap();
    assert_eq!(
        handoff
            .native_audio_submitted(
                DecimalU64(1),
                &ControlReceipt::enqueued(7),
                &ControlReceipt::enqueued(6),
                now
            )
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(handoff.phase(), AudioHandoffPhase::AwaitingNativeSubmission);
}

#[test]
fn full_media_identity_and_observed_initial_mute_are_required_before_web_switch() {
    let now = Instant::now();
    let mut handoff = start(now, false);
    let mut native = snapshot();
    native.duration = Some(3.);
    assert_eq!(
        handoff.observe(DecimalU64(7), &native, None, now).unwrap(),
        None
    );
    native.duration = Some(1420.);
    native.muted = None;
    assert_eq!(
        handoff.observe(DecimalU64(7), &native, None, now).unwrap(),
        None
    );
    assert_eq!(handoff.phase(), AudioHandoffPhase::AwaitingMedia);
    native.muted = Some(false);
    assert_eq!(
        handoff
            .observe(DecimalU64(7), &native, None, now)
            .unwrap_err()
            .code,
        ErrorCode::EngineFailed
    );
    assert_eq!(handoff.phase(), AudioHandoffPhase::Failed);
}

#[test]
fn partial_submission_failure_preserves_the_original_error_and_never_retries() {
    let now = Instant::now();
    let mut handoff = start(now, false);
    reach_submission(&mut handoff, &snapshot(), now);
    let mut original = ApiError::new(ErrorCode::ResourceExhausted, "Synthetic full native queue");
    original.details = serde_json::json!({"volume_accepted":true,"mute_accepted":false});
    let returned = handoff.submission_failed(original);
    assert_eq!(returned.code, ErrorCode::ResourceExhausted);
    assert_eq!(returned.details["volume_accepted"], true);
    assert_eq!(
        handoff
            .observe(DecimalU64(7), &snapshot(), None, now)
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
}

#[test]
fn timeout_is_terminal_and_does_not_claim_audio_was_never_applied() {
    let now = Instant::now();
    let mut handoff = start(now, false);
    reach_submission(&mut handoff, &snapshot(), now);
    let receipt = ControlReceipt::enqueued(7);
    handoff
        .native_audio_submitted(DecimalU64(1), &receipt, &receipt, now)
        .unwrap();
    let error = handoff
        .observe(DecimalU64(7), &snapshot(), None, now + HANDOFF_BUDGET)
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::Timeout);
    assert!(error.details["native_audio_released"].is_null());
    assert_eq!(
        error.details["phase_before_timeout"],
        "awaiting_native_audio"
    );
    assert_eq!(handoff.phase(), AudioHandoffPhase::Failed);
    assert_eq!(
        handoff
            .observe(DecimalU64(7), &snapshot(), None, now + HANDOFF_BUDGET)
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
}

#[test]
fn local_media_uses_the_same_submission_and_observation_gate_without_web_pause() {
    let now = Instant::now();
    let mut handoff = NativeAudioHandoff::begin(
        DecimalU64(7),
        AudioOrigin::Local,
        AudioIntent::new(0., false).unwrap(),
        now,
    )
    .unwrap();
    let native = snapshot();
    assert!(matches!(
        handoff.observe(DecimalU64(7), &native, None, now).unwrap(),
        Some(AudioHandoffAction::ApplyNativeAudio { .. })
    ));
    assert_eq!(handoff.phase(), AudioHandoffPhase::AwaitingNativeSubmission);
}

#[test]
fn disconnect_after_preparation_never_becomes_ready() {
    let now = Instant::now();
    let mut handoff = start(now, false);
    let mut native = snapshot();
    reach_submission(&mut handoff, &native, now);
    native.connected = false;
    assert_eq!(
        handoff
            .observe(DecimalU64(7), &native, None, now)
            .unwrap_err()
            .code,
        ErrorCode::EngineFailed
    );
    assert_eq!(handoff.phase(), AudioHandoffPhase::Failed);
}

#[test]
fn invalid_volume_and_handoff_identities_are_rejected_without_hidden_effects() {
    for volume in [f64::NAN, f64::INFINITY, -0.1, 100.1] {
        assert!(AudioIntent::new(volume, false).is_err());
    }
    let intent = AudioIntent::new(80., false).unwrap();
    let now = Instant::now();
    assert!(NativeAudioHandoff::begin(DecimalU64(0), AudioOrigin::Local, intent, now).is_err());
    for duration in [0., -1., f64::NAN, f64::INFINITY] {
        assert!(
            NativeAudioHandoff::begin(
                DecimalU64(7),
                AudioOrigin::Web {
                    document_generation: DecimalU64(11),
                    observed_sequence: DecimalU64(1),
                    expected_duration_seconds: duration
                },
                intent,
                now
            )
            .is_err()
        );
    }
}

#[test]
fn newer_intent_uses_a_new_plan_and_late_receipts_cannot_acknowledge_it() {
    let now = Instant::now();
    let mut handoff = start(now, false);
    reach_submission(&mut handoff, &snapshot(), now);
    let intent = AudioIntent::new(25., true).unwrap();
    assert_eq!(
        handoff.replace_intent(intent, now).unwrap(),
        Some(AudioHandoffAction::ApplyNativeAudio {
            expected_engine_instance_id: DecimalU64(7),
            plan_id: DecimalU64(2),
            intent,
            paused: None,
        })
    );
    let receipt = ControlReceipt::enqueued(7);
    assert_eq!(
        handoff
            .native_audio_submitted(DecimalU64(1), &receipt, &receipt, now)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
    assert_eq!(handoff.phase(), AudioHandoffPhase::AwaitingNativeSubmission);
    handoff
        .native_audio_submitted(DecimalU64(2), &receipt, &receipt, now)
        .unwrap();
    let mut native = snapshot();
    native.volume = Some(25.);
    handoff.observe(DecimalU64(7), &native, None, now).unwrap();
    assert_eq!(handoff.phase(), AudioHandoffPhase::Ready);
    assert_eq!(
        handoff
            .replace_intent(AudioIntent::new(80., false).unwrap(), now)
            .unwrap_err()
            .code,
        ErrorCode::RevisionConflict
    );
}

#[test]
fn exhausted_plan_identity_never_wraps_or_emits_hidden_audio_commands() {
    let now = Instant::now();
    let mut handoff = start(now, false);
    reach_submission(&mut handoff, &snapshot(), now);
    handoff.plan_id = DecimalU64(u64::MAX);
    assert_eq!(
        handoff
            .replace_intent(AudioIntent::new(25., true).unwrap(), now)
            .unwrap_err()
            .code,
        ErrorCode::ResourceExhausted
    );
    assert_eq!(handoff.plan_id.0, u64::MAX);
    assert_eq!(handoff.phase(), AudioHandoffPhase::Failed);
}

#[test]
fn pause_submission_error_is_terminal_without_native_audio_release() {
    let now = Instant::now();
    let mut handoff = start(now, false);
    handoff
        .observe(DecimalU64(7), &snapshot(), None, now)
        .unwrap();
    let returned = handoff.submission_failed(ApiError::new(
        ErrorCode::EngineFailed,
        "Synthetic web pause submission failure",
    ));
    assert_eq!(returned.code, ErrorCode::EngineFailed);
    assert_eq!(handoff.phase(), AudioHandoffPhase::Failed);
    assert_eq!(
        handoff
            .observe(DecimalU64(7), &snapshot(), Some(&confirmed_web(11)), now)
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
}
