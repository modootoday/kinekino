//! Classifies initial media readiness without accepting foreign or failed observations.
use super::{DecimalU64, EpisodePlaybackObservation, OpenedEpisodePlayback};

pub(super) fn observe(
    sample: &EpisodePlaybackObservation,
    opened: &OpenedEpisodePlayback,
    target: DecimalU64,
    tolerance: u64,
    connected_seen: &mut bool,
) -> Result<bool, &'static str> {
    if sample.engine_instance_id != opened.engine_instance_id
        || sample.resolution_artifact_id != opened.resolution_artifact_id
    {
        return Err("Original binding differs");
    }
    if sample.playback.has_error {
        return Err("Original media reports an error");
    }
    if !sample.playback.connected {
        return if *connected_seen {
            Err("Original media disconnected after connection")
        } else {
            Ok(false)
        };
    }
    *connected_seen = true;
    Ok(matches!(
        (sample.playback.position_us, sample.playback.duration_us),
        (Some(position), Some(duration))
            if sample.playback.paused == Some(true)
                && position.0.checked_add(tolerance).is_some_and(|value| value < target.0)
                && target < duration
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn observations() -> (EpisodePlaybackObservation, OpenedEpisodePlayback) {
        let artifact = "a".repeat(64);
        let sample = serde_json::from_value(json!({
            "engine_instance_id":"1", "resolution_artifact_id":artifact,
            "diagnostics":null,"options":null,
            "playback":{"origin":"native_observation","published_age_ms":"0",
                "connected":false,"paused":null,"idle":null,"position_us":null,
                "duration_us":null,"volume_percent":null,"muted":null,
                "video_size":null,"audio_channels":null,"has_error":false}
        }))
        .unwrap();
        let opened = serde_json::from_value(json!({
            "engine_instance_id":"1","resolution_artifact_id":artifact,
            "completion":"unobserved"
        }))
        .unwrap();
        (sample, opened)
    }

    #[test]
    fn pending_connection_can_become_ready_only_with_all_original_conditions() {
        kinekino_platform::install();
        let (mut sample, opened) = observations();
        let mut seen = false;
        assert_eq!(
            observe(&sample, &opened, DecimalU64(5_000_000), 250_000, &mut seen),
            Ok(false)
        );
        assert!(!seen);
        sample.playback.connected = true;
        sample.playback.paused = Some(true);
        sample.playback.position_us = Some(DecimalU64(0));
        assert_eq!(
            observe(&sample, &opened, DecimalU64(5_000_000), 250_000, &mut seen),
            Ok(false)
        );
        assert!(seen);
        sample.playback.duration_us = Some(DecimalU64(10_000_000));
        assert_eq!(
            observe(&sample, &opened, DecimalU64(5_000_000), 250_000, &mut seen),
            Ok(true)
        );
        sample.playback.connected = false;
        assert_eq!(
            observe(&sample, &opened, DecimalU64(5_000_000), 250_000, &mut seen),
            Err("Original media disconnected after connection")
        );
    }

    #[test]
    fn foreign_binding_and_explicit_error_fail_even_before_connection() {
        kinekino_platform::install();
        let (sample, opened) = observations();
        for foreign_engine in [true, false] {
            let mut changed = sample.clone();
            if foreign_engine {
                changed.engine_instance_id = DecimalU64(2);
            } else {
                changed.resolution_artifact_id = "b".repeat(64);
            }
            assert_eq!(
                observe(
                    &changed,
                    &opened,
                    DecimalU64(5_000_000),
                    250_000,
                    &mut false
                ),
                Err("Original binding differs")
            );
        }
        let mut failed = sample;
        failed.playback.has_error = true;
        assert_eq!(
            observe(&failed, &opened, DecimalU64(5_000_000), 250_000, &mut false),
            Err("Original media reports an error")
        );
        failed.playback.connected = true;
        assert_eq!(
            observe(&failed, &opened, DecimalU64(5_000_000), 250_000, &mut true),
            Err("Original media reports an error")
        );
    }

    #[test]
    fn connection_alone_does_not_relax_paused_target_or_overflow_boundaries() {
        kinekino_platform::install();
        let (mut sample, opened) = observations();
        sample.playback.connected = true;
        sample.playback.paused = Some(true);
        sample.playback.position_us = Some(DecimalU64(0));
        sample.playback.duration_us = Some(DecimalU64(10_000_000));
        for (paused, position, duration) in [
            (None, 0, 10_000_000),
            (Some(false), 0, 10_000_000),
            (Some(true), 4_750_000, 10_000_000),
            (Some(true), 0, 5_000_000),
            (Some(true), u64::MAX, 10_000_000),
        ] {
            sample.playback.paused = paused;
            sample.playback.position_us = Some(DecimalU64(position));
            sample.playback.duration_us = Some(DecimalU64(duration));
            assert_eq!(
                observe(&sample, &opened, DecimalU64(5_000_000), 250_000, &mut false),
                Ok(false)
            );
        }
    }
}
