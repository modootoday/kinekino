use super::*;
use crate::application::playback_control::ControlAcceptance;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
struct Queue {
    calls: AtomicUsize,
    fail: bool,
}
impl PlaybackControlPort for Queue {
    fn submit(&self, _: PlaybackControl) -> Result<ControlReceipt, ApiError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.fail {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Injected queue failure",
            ));
        }
        Ok(ControlReceipt::enqueued(7))
    }
}
fn requested() -> RequestedStartupState {
    RequestedStartupState {
        audio: AudioIntent::new(80., false).unwrap(),
        paused: true,
    }
}
fn bound() -> AudioHandoffIntents {
    let intents = AudioHandoffIntents::default();
    intents
        .bind_for_handoff(DecimalU64(7), requested())
        .unwrap();
    intents
}

#[test]
fn deferred_settings_coalesce_without_native_submission_but_seek_reaches_queue() {
    let intents = bound();
    let queue = Queue::default();
    for control in [
        PlaybackControl::SetMuted { muted: true },
        PlaybackControl::SetVolume { volume_percent: 0. },
        PlaybackControl::Play,
    ] {
        assert_eq!(
            intents
                .submit(&queue, DecimalU64(7), control)
                .unwrap()
                .acceptance,
            ControlAcceptance::DeferredUntilHandoff
        );
    }
    assert_eq!(queue.calls.load(Ordering::SeqCst), 0);
    let current = intents.requested(DecimalU64(7)).unwrap();
    assert!(current.audio.muted());
    assert_eq!(current.audio.volume_percent(), 0.);
    assert!(!current.paused);
    assert_eq!(
        intents
            .submit(
                &queue,
                DecimalU64(7),
                PlaybackControl::Seek {
                    position_us: DecimalU64(0)
                }
            )
            .unwrap()
            .acceptance,
        ControlAcceptance::Accepted
    );
    assert_eq!(queue.calls.load(Ordering::SeqCst), 1);
}

#[test]
fn same_value_aba_invalidates_captured_plan_without_running_it() {
    let intents = bound();
    let queue = Queue::default();
    for volume in [12., 80.] {
        intents
            .submit(
                &queue,
                DecimalU64(7),
                PlaybackControl::SetVolume {
                    volume_percent: volume,
                },
            )
            .unwrap();
    }
    let submitted = intents
        .perform_plan(
            DecimalU64(7),
            DecimalU64(0),
            requested().audio,
            Some(true),
            || -> Result<(), ApiError> { panic!("Stale plan ran") },
        )
        .unwrap();
    assert!(submitted.is_none());
    assert_eq!(queue.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn old_cleanup_cannot_end_clear_or_publish_over_new_instance() {
    let intents = bound();
    intents
        .bind_for_handoff(DecimalU64(8), requested())
        .unwrap();
    assert!(intents.end_expected(DecimalU64(7)).is_err());
    assert!(intents.clear_expected(DecimalU64(7)).is_err());
    assert!(
        intents
            .publish_handoff(
                DecimalU64(7),
                serde_json::json!({"ready":true}),
                Instant::now()
            )
            .is_err()
    );
    assert!(intents.pending_requested(DecimalU64(8)).unwrap().is_some());
}

#[test]
fn closed_newer_lifetime_rejects_old_publication_and_cleanup() {
    let intents = bound();
    intents
        .bind_for_handoff(DecimalU64(8), requested())
        .unwrap();
    intents.end_expected(DecimalU64(8)).unwrap();
    intents.clear_expected(DecimalU64(8)).unwrap();
    intents
        .publish_handoff(
            DecimalU64(8),
            serde_json::json!({"engine_instance_id":"8","lifetime_status":"closed"}),
            Instant::now(),
        )
        .unwrap();
    for result in [
        intents.end_expected(DecimalU64(7)),
        intents.clear_expected(DecimalU64(7)),
        intents.publish_handoff(
            DecimalU64(7),
            serde_json::json!({"ready":true}),
            Instant::now(),
        ),
    ] {
        assert_eq!(result.unwrap_err().code, ErrorCode::RevisionConflict);
    }
    assert_eq!(
        intents.reported_handoff().unwrap()["engine_instance_id"],
        "8"
    );
}

#[test]
fn failed_queue_admission_does_not_become_user_preference() {
    let intents = AudioHandoffIntents::default();
    intents.bind_ready(DecimalU64(7)).unwrap();
    let queue = Queue {
        fail: true,
        ..Default::default()
    };
    assert!(
        intents
            .submit(
                &queue,
                DecimalU64(7),
                PlaybackControl::SetMuted { muted: true }
            )
            .is_err()
    );
    assert!(intents.explicit_user_mute().unwrap().is_none());
    intents
        .bind_for_handoff(DecimalU64(8), requested())
        .unwrap();
    assert!(!intents.requested(DecimalU64(8)).unwrap().audio.muted());
}

#[test]
fn ending_rejects_admission_and_counter_exhaustion_has_no_side_effect() {
    let intents = bound();
    let queue = Queue::default();
    intents.0.lock().unwrap().revision = u64::MAX;
    assert_eq!(
        intents
            .submit(&queue, DecimalU64(7), PlaybackControl::Play)
            .unwrap_err()
            .code,
        ErrorCode::ResourceExhausted
    );
    assert!(intents.requested(DecimalU64(7)).unwrap().paused);
    intents.end_expected(DecimalU64(7)).unwrap();
    assert_eq!(
        intents
            .submit(&queue, DecimalU64(7), PlaybackControl::Pause)
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    assert_eq!(queue.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn concurrent_requests_preserve_independent_mute_and_volume_intents() {
    let intents = bound();
    let queue = Arc::new(Queue::default());
    std::thread::scope(|threads| {
        for control in [
            PlaybackControl::SetMuted { muted: true },
            PlaybackControl::SetVolume {
                volume_percent: 13.25,
            },
        ] {
            let intents = intents.clone();
            let queue = queue.clone();
            threads.spawn(move || {
                intents
                    .submit(queue.as_ref(), DecimalU64(7), control)
                    .unwrap()
            });
        }
    });
    let current = intents.requested(DecimalU64(7)).unwrap();
    assert!(current.audio.muted());
    assert_eq!(current.audio.volume_percent(), 13.25);
    assert_eq!(queue.calls.load(Ordering::SeqCst), 0);
}
