use super::ReceiveState;
use crate::application::playback_session::FileAttribution;
use serde_json::json;
use std::time::{Duration, Instant};

#[test]
fn raw_properties_never_create_a_guarded_barrier_generation() {
    let mut state = ReceiveState::default();
    assert_eq!(state.generations(), None);
    state.connect().unwrap();
    let first = state.generations().unwrap();
    state
        .receive(
            &json!({"event":"property-change","name":"seekable","data":true}),
            Instant::now(),
        )
        .unwrap();
    assert_eq!(state.generations(), Some(first));
    state
        .receive(&json!({"event":"start-file"}), Instant::now())
        .unwrap();
    let next = state.generations().unwrap();
    assert_eq!(first.0, next.0);
    assert!(next.1 > first.1);
    state.disconnect();
    assert_eq!(state.generations(), None);
}

#[test]
fn busy_receive_lookup_is_retryable_and_never_writes() {
    use crate::application::api::ErrorCode;
    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
    let control = super::super::control::MpvControlHandle::new(sender).unwrap();
    let _lock = control.state.native_observation.lock().unwrap();
    let error = control.native_receive_observation().unwrap_err();
    assert_eq!(error.code, ErrorCode::NeedsInput);
    assert!(error.retryable);
    assert!(matches!(
        receiver.try_recv(),
        Err(tokio::sync::mpsc::error::TryRecvError::Empty)
    ));
}

#[test]
fn poisoned_receive_lookup_fails_closed_without_legacy_queue_submission() {
    use crate::application::api::ErrorCode;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
    let control = super::super::control::MpvControlHandle::new(sender).unwrap();
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _lock = control.state.native_observation.lock().unwrap();
            panic!("fake receive-state poison");
        }))
        .is_err()
    );
    assert_eq!(
        control.native_receive_observation().unwrap_err().code,
        ErrorCode::Internal
    );
    assert!(matches!(
        receiver.try_recv(),
        Err(tokio::sync::mpsc::error::TryRecvError::Empty)
    ));
}
#[test]
fn reads_and_unrelated_ack_never_refresh_property_receive_age_or_sequence() {
    let start = Instant::now();
    let mut state = ReceiveState::default();
    state.connect().unwrap();
    state
        .receive(
            &json!({"event":"property-change","name":"time-pos","data":4}),
            start,
        )
        .unwrap();
    let first = state.report(start);
    state
        .receive(
            &json!({"request_id":1000,"error":"success"}),
            start + Duration::from_secs(1),
        )
        .unwrap();
    let later = state.report(start + Duration::from_secs(2));
    assert_eq!(
        later.position_seconds.receive_sequence,
        first.position_seconds.receive_sequence
    );
    assert_eq!(
        later.position_seconds.received_age,
        Some(Duration::from_secs(2))
    );
    let again = state.report(start + Duration::from_secs(3));
    assert_eq!(
        again.position_seconds.receive_sequence,
        first.position_seconds.receive_sequence
    );
    assert_eq!(
        again.position_seconds.received_age,
        Some(Duration::from_secs(3))
    );
}
#[test]
fn repeated_real_value_is_new_receive_but_never_file_attribution() {
    let start = Instant::now();
    let mut state = ReceiveState::default();
    state.connect().unwrap();
    let message = json!({"event":"property-change","name":"pause","data":false});
    state.receive(&message, start).unwrap();
    let first = state.report(start);
    state
        .receive(&message, start + Duration::from_secs(1))
        .unwrap();
    let later = state.report(start + Duration::from_secs(1));
    assert!(later.paused.receive_sequence > first.paused.receive_sequence);
    assert_eq!(later.paused.value, Some(false));
    assert_eq!(later.paused.received_age, Some(Duration::ZERO));
    assert_eq!(later.paused.file_attribution, FileAttribution::Unverified);
}
#[test]
fn missing_null_and_wrong_values_are_unknown_and_unobserved_seekable_stays_unknown() {
    let now = Instant::now();
    let mut state = ReceiveState::default();
    state.connect().unwrap();
    assert_eq!(state.report(now).seekable.value, None);
    for message in [
        json!({"event":"property-change","name":"time-pos","data":-1}),
        json!({"event":"property-change","name":"time-pos","data":null}),
        json!({"event":"property-change","name":"time-pos"}),
    ] {
        state.receive(&message, now).unwrap();
        assert_eq!(state.report(now).position_seconds.value, None);
    }
    state
        .receive(
            &json!({"event":"property-change","name":"seekable","data":"true"}),
            now,
        )
        .unwrap();
    assert_eq!(state.report(now).seekable.value, None);
    state
        .receive(
            &json!({"event":"property-change","name":"seekable","data":false}),
            now,
        )
        .unwrap();
    assert_eq!(state.report(now).seekable.value, Some(false));
}
#[test]
fn lifecycle_invalidation_does_not_relabel_old_sample_or_authorize_delayed_sample() {
    let now = Instant::now();
    let mut state = ReceiveState::default();
    state.connect().unwrap();
    let message = json!({"event":"property-change","name":"time-pos","data":5});
    state.receive(&message, now).unwrap();
    let old = state.report(now);
    state.receive(&json!({"event":"start-file"}), now).unwrap();
    let cleared = state.report(now);
    assert!(cleared.lifecycle_generation > old.lifecycle_generation);
    assert_eq!(cleared.position_seconds.value, None);
    assert_eq!(
        cleared.position_seconds.lifecycle_generation_at_receive,
        old.position_seconds.lifecycle_generation_at_receive
    );
    state.receive(&message, now).unwrap();
    let delayed = state.report(now);
    assert_eq!(
        delayed.position_seconds.lifecycle_generation_at_receive,
        Some(delayed.lifecycle_generation)
    );
    assert_eq!(
        delayed.position_seconds.file_attribution,
        FileAttribution::Unverified
    );
    state.receive(&json!({"event":"end-file"}), now).unwrap();
    assert_eq!(state.report(now).position_seconds.value, None);
}
#[test]
fn disconnect_reconnect_does_not_reuse_values_or_change_old_receive_timestamps() {
    let now = Instant::now();
    let mut state = ReceiveState::default();
    state.connect().unwrap();
    state
        .receive(
            &json!({"event":"property-change","name":"seekable","data":true}),
            now,
        )
        .unwrap();
    let old = state.report(now);
    state.disconnect();
    state
        .receive(
            &json!({"event":"property-change","name":"seekable","data":true}),
            now,
        )
        .unwrap();
    assert_eq!(state.report(now).seekable.value, None);
    state.connect().unwrap();
    let next = state.report(now + Duration::from_secs(1));
    assert!(next.connection_generation > old.connection_generation);
    assert_eq!(next.seekable.value, None);
    assert_eq!(next.seekable.received_age, Some(Duration::from_secs(1)));
}
#[test]
fn counter_exhaustion_invalidates_values_and_cannot_reconnect() {
    let mut state = ReceiveState::default();
    state.connect().unwrap();
    state.exhaust_sequence();
    assert!(
        state
            .receive(
                &json!({"event":"property-change","name":"seekable","data":true}),
                Instant::now()
            )
            .is_err()
    );
    let report = state.report(Instant::now());
    assert!(report.counter_exhausted);
    assert!(!report.connected);
    assert_eq!(report.seekable.value, None);
    assert!(state.connect().is_err());
}
