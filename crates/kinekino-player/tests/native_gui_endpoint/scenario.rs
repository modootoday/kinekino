use super::{cli::Run, fixture::TestResult, native::Native};
use kinekino_player::domain::{api_values::DecimalU64, playback_control::PlaybackControl};
use serde_json::json;
fn require(condition: bool, message: &str) -> TestResult<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}
pub fn exercise(run: &mut Run, original: &Native) -> TestResult<()> {
    let capabilities = run.call("capabilities", "capabilities", json!({}))?;
    for command in [
        "playback.status",
        "playback.play",
        "playback.pause",
        "playback.seek",
        "playback.set_volume",
    ] {
        require(
            capabilities["commands"].as_array().is_some_and(|entries| {
                entries
                    .iter()
                    .any(|entry| entry["command"] == command && entry["status"] == "supported")
            }),
            "Owned GUI SDK capability absent",
        )?;
        let schema = run.call("schema", "schema", json!({"command":command}))?;
        require(
            schema["status"] == "supported",
            "Owned GUI schema does not support control",
        )?;
    }
    let target = if original.position_us + 2_000_000 < original.duration_us {
        original.position_us + 1_000_000
    } else {
        1_000_000
    };
    run.control(
        "seek",
        PlaybackControl::Seek {
            position_us: DecimalU64(target),
        },
    )?;
    run.wait(|state| {
        state.paused == Some(true)
            && state
                .position_us
                .is_some_and(|p| p.0.abs_diff(target) <= 100_000)
    })?;
    let sought = run.native("after-seek")?;
    require(
        sought.paused && sought.position_us.abs_diff(target) <= 100_000,
        "Independent native seek observation differs",
    )?;
    let volume = if (original.volume - 40.).abs() > 0.1 {
        40.
    } else {
        41.
    };
    run.control(
        "volume",
        PlaybackControl::SetVolume {
            volume_percent: volume,
        },
    )?;
    run.wait(|state| {
        state
            .volume_percent
            .is_some_and(|v| (v - volume).abs() < 0.1)
    })?;
    let changed = run.native("after-volume")?;
    require(
        (changed.volume - volume).abs() < 0.1 && changed.muted == original.muted,
        "Independent native volume/mute observation differs",
    )?;
    run.control("play", PlaybackControl::Play)?;
    run.wait(|state| {
        state.paused == Some(false)
            && state
                .position_us
                .is_some_and(|p| p.0 > sought.position_us + 150_000)
    })?;
    let playing = run.native("during-play")?;
    require(
        !playing.paused
            && playing.position_us > sought.position_us + 100_000
            && (playing.volume - volume).abs() < 0.1,
        "Independent native play/progress observation differs",
    )?;
    run.control("pause", PlaybackControl::Pause)?;
    run.wait(|state| state.paused == Some(true))?;
    let paused = run.native("after-pause")?;
    require(
        paused.paused,
        "Independent native pause observation differs",
    )?;
    std::thread::sleep(std::time::Duration::from_millis(150));
    let stable = run.native("pause-stability")?;
    require(
        stable.paused && stable.position_us.abs_diff(paused.position_us) <= 50_000,
        "Paused native position continued advancing",
    )?;
    Ok(())
}
pub fn restore(run: &mut Run, original: &Native) -> TestResult<()> {
    // Every mutation rechecks the owned source and both process generations.
    run.control("restore-pause", PlaybackControl::Pause)?;
    run.wait(|state| state.paused == Some(true))?;
    run.control(
        "restore-volume",
        PlaybackControl::SetVolume {
            volume_percent: original.volume,
        },
    )?;
    run.wait(|state| {
        state
            .volume_percent
            .is_some_and(|v| (v - original.volume).abs() < 0.1)
    })?;
    run.control(
        "restore-seek",
        PlaybackControl::Seek {
            position_us: DecimalU64(original.position_us),
        },
    )?;
    run.wait(|state| {
        state.paused == Some(true)
            && state
                .position_us
                .is_some_and(|p| p.0.abs_diff(original.position_us) <= 100_000)
    })?;
    let restored = run.native("restored")?;
    require(
        restored.paused
            && restored.position_us.abs_diff(original.position_us) <= 100_000
            && (restored.volume - original.volume).abs() < 0.1
            && restored.muted == original.muted,
        "Original owned native state was not restored",
    )?;
    Ok(())
}
