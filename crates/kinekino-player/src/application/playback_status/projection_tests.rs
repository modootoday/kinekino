use super::{projection::engine_micros, *};
use crate::application::{
    native_playback::NativePlayback, playback_session::test_guarded_observation,
};
use crate::domain::api_values::DecimalU64;

#[test]
fn shared_engine_micros_preserves_floor_unknown_and_checked_range() {
    for value in [
        None,
        Some(-1.),
        Some(f64::NAN),
        Some(f64::INFINITY),
        Some(f64::NEG_INFINITY),
        Some(20_000_000_000_000.),
    ] {
        assert_eq!(engine_micros(value), None);
    }
    assert_eq!(engine_micros(Some(0.)), Some(DecimalU64(0)));
    assert_eq!(engine_micros(Some(1.23456789)), Some(DecimalU64(1_234_567)));
    assert_eq!(
        engine_micros(Some(10_000_000_000_000.)),
        Some(DecimalU64(10_000_000_000_000_000_000))
    );
    assert_eq!(engine_micros(Some(u64::MAX as f64 / 1_000_000.)), None);
}

#[test]
fn publisher_and_issuer_view_share_status_without_reinterpreting_publication_age() {
    let state = NativePlayback {
        connected: true,
        time: Some(1.23456789),
        duration: Some(20.),
        paused: true,
        pause_observed: true,
        idle: false,
        idle_observed: true,
        volume: Some(0.),
        muted: Some(true),
        width: Some(1920),
        height: Some(1080),
        audio_channels: Some(2),
        error: Some("synthetic observed failure".into()),
        ..Default::default()
    };
    let (publisher, reader) = native_status_channel();
    publisher.publish(&state).unwrap();
    let mut published = reader.status().unwrap();
    let (view, _guard) = test_guarded_observation(state)
        .into_guarded_parts()
        .unwrap();
    published.published_age_ms = DecimalU64(0);
    assert_eq!(
        serde_json::to_value(published).unwrap(),
        serde_json::to_value(view.playback()).unwrap()
    );
    assert_eq!(view.playback().position_us, Some(DecimalU64(1_234_567)));
    assert!(view.playback().has_error);
    publisher.clear().unwrap();
    assert!(reader.status().is_err());
    assert_eq!(view.playback().position_us, Some(DecimalU64(1_234_567)));
}

#[test]
fn unknown_flags_and_invalid_media_dimensions_remain_unknown() {
    let state = NativePlayback {
        paused: false,
        idle: false,
        width: Some(u64::MAX),
        height: Some(1080),
        audio_channels: Some(0),
        volume: Some(f64::NAN),
        ..Default::default()
    };
    let projected = project_native_status(&state);
    assert_eq!(projected.paused, None);
    assert_eq!(projected.idle, None);
    assert!(projected.video_size.is_none());
    assert_eq!(projected.audio_channels, None);
    assert_eq!(projected.volume_percent, None);
}
