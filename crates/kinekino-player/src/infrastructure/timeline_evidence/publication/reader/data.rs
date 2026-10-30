use super::*;
use crate::{
    domain::media_timeline::TimeBase,
    infrastructure::{
        frame_capture,
        timeline_evidence::{audio, video},
    },
};
pub(super) fn verify(c: &Context<'_>) -> Result<(), ApiError> {
    let snapshot = &c.token.snapshot;
    let movie = reference(&c.input.inspect(&EvidenceReference {
        asset_id: snapshot.media.asset_id.clone(),
        sha256: snapshot.media.sha256.clone(),
    })?);
    if c.input.inspect(&movie)?.bytes.0 != c.token.source_identity.bytes {
        return Err(conflict("Source movie bytes differ"));
    }
    use crate::application::review_playback::ReviewContentClassifier;
    let prefix = c.input.read_prefix(&movie, 65536)?;
    let class = crate::infrastructure::review_content_classification::LocalReviewContentClassifier
        .classify(&prefix)?;
    if class != c.token.source_identity.classification
        || c.input.inspect(&movie)?.media_type != c.token.source_identity.declared_media_type
    {
        return Err(invalid("Live source container differs"));
    }
    let raw_video = c.bytes("video", "stdout.raw", 12 * 1024 * 1024)?;
    let (frames, base, index, cadence) = video::parse_live(
        std::str::from_utf8(&raw_video).map_err(|_| invalid("Video UTF8 required"))?,
        &class,
    )?;
    if frames != snapshot.video_frames
        || cadence != snapshot.video_cadence
        || base != snapshot.time_base
        || index != c.manifest.video_stream_index
    {
        return Err(conflict(
            "Actual video frames differ from publisher snapshot",
        ));
    }
    let raw_audio = c.bytes(
        "original-pcm",
        "stderr.raw",
        c.token.budget.max_output_bytes,
    )?;
    let frames = audio::parse_live(
        std::str::from_utf8(&raw_audio).map_err(|_| invalid("Audio UTF8 required"))?,
    )?;
    if frames != snapshot.pcm.frames {
        return Err(conflict("Actual source audio clock differs"));
    }
    let full = c.bytes("original-pcm", "stdout.raw", 64 * 1024 * 1024)?;
    let replay = c.bytes("replay-pcm", "stdout.raw", 64 * 1024 * 1024)?;
    let clip = c.bytes("replay-pcm", "clip.s16le", 64 * 1024 * 1024)?;
    let samples = frames
        .iter()
        .map(|frame| u64::from(frame.samples))
        .sum::<u64>();
    let start = snapshot
        .pcm
        .clip_start_sample
        .checked_mul(2)
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| invalid("PCM clip offset overflow"))?;
    let end = start
        .checked_add(clip.len())
        .ok_or_else(|| invalid("PCM clip endpoint overflow"))?;
    if full != replay
        || full.len() as u64 != samples * 2
        || clip.len() as u64 != snapshot.pcm.clip_samples * 2
        || full.get(start..end) != Some(clip.as_slice())
        || c.asset("original-pcm", "stdout.raw")?.sha256 != snapshot.pcm.original_data_sha256
        || c.asset("replay-pcm", "stdout.raw")?.sha256 != snapshot.pcm.replay_data_sha256
        || c.asset("replay-pcm", "clip.s16le")?.sha256 != snapshot.pcm.clip_data_sha256
    {
        return Err(conflict("Exact source/replay/clip PCM bytes differ"));
    }
    let replay_audio = c.bytes("replay-pcm", "stderr.raw", 12 * 1024 * 1024)?;
    let replay_frames = audio::parse(
        std::str::from_utf8(&replay_audio).map_err(|_| invalid("Replay clock UTF8 required"))?,
    )?;
    if replay_frames
        .iter()
        .map(|frame| u64::from(frame.samples))
        .sum::<u64>()
        != samples
    {
        return Err(conflict("Replay clock sample count differs"));
    }
    for (role, pts, png_sha, metadata_sha) in [
        (
            "origin",
            snapshot.origin.pts,
            &c.asset("origin", "stdout.raw")?.sha256,
            Some(&snapshot.origin.observation_sha256),
        ),
        (
            "start",
            snapshot.start_capture.actual_pts,
            &snapshot.start_capture.png_sha256,
            None,
        ),
        (
            "end",
            snapshot.end_capture.actual_pts,
            &snapshot.end_capture.png_sha256,
            None,
        ),
    ] {
        if &c.asset(role, "stdout.raw")?.sha256 != png_sha {
            return Err(conflict("Capture PNG identity differs"));
        }
        frame_capture::validate_png(&c.bytes(role, "stdout.raw", 64 * 1024 * 1024)?)?;
        if let Some(sha) = metadata_sha
            && sha != &c.asset(role, "stderr.raw")?.sha256
        {
            return Err(conflict("Decoded origin observation differs"));
        }
        let metadata = c.bytes(role, "stderr.raw", 12 * 1024 * 1024)?;
        let (actual, base, _) = frame_capture::selected_pts(&metadata)?;
        if actual.parse::<i64>().ok() != Some(pts)
            || TimeBase::parse(&base).ok() != Some(snapshot.time_base)
        {
            return Err(conflict("Actual decoded capture PTS/time base differs"));
        }
    }
    Ok(())
}
