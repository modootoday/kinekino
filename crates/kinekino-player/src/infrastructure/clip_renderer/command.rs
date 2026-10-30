use super::*;
mod video;
use crate::domain::clip_document::FitMode;
use std::{ffi::OsString, path::Path};
fn seconds(micros: i128) -> String {
    let n = micros.unsigned_abs();
    format!(
        "{}{}.{:06}",
        if micros < 0 { "-" } else { "" },
        n / 1_000_000,
        n % 1_000_000
    )
}
pub(super) fn arguments(
    input: &Path,
    demuxer: &str,
    plan: &ManualClipPlan,
) -> Result<Vec<OsString>, ApiError> {
    arguments_with_overlay(input, demuxer, plan, None)
}
pub(super) fn arguments_with_overlay(
    input: &Path,
    demuxer: &str,
    plan: &ManualClipPlan,
    overlay: Option<&super::subtitles::PreparedClipOverlay<'_, '_>>,
) -> Result<Vec<OsString>, ApiError> {
    if !["mov", "matroska"].contains(&demuxer) {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Unsupported clip source demuxer",
        ));
    }
    let mut args: Vec<OsString> = [
        "-hide_banner",
        "-nostdin",
        "-v",
        "error",
        "-filter_complex_threads",
        "1",
        "-threads",
        "2",
        "-max_alloc",
        "268435456",
        "-copyts",
        "-noautorotate",
        "-protocol_whitelist",
        "file",
        "-f",
        demuxer,
    ]
    .into_iter()
    .map(Into::into)
    .collect();
    if demuxer == "mov" {
        args.extend(["-enable_drefs", "0"].into_iter().map(Into::into));
    }
    args.extend([
        "-seek_timestamp".into(),
        "1".into(),
        "-noaccurate_seek".into(),
        "-ss".into(),
        seconds(plan.input_seek_start_us).into(),
        "-i".into(),
        input.as_os_str().to_owned(),
    ]);
    if let Some(overlay) = overlay {
        overlay.append_inputs(&mut args);
    }
    let mut graph = if overlay.is_some() {
        video::graph_with_overlay(plan, overlay)?
    } else {
        video::graph(plan)?
    };
    if let Some(audio) = &plan.audio {
        graph.push_str(&format!(
            ";[0:{}]atrim=start={}:end={},asetpts=PTS-STARTPTS,volume={}.{:02}[a]",
            audio.stream_index,
            seconds(audio.absolute_start_us),
            seconds(audio.absolute_end_us),
            if audio.muted {
                0
            } else {
                audio.volume_percent / 100
            },
            if audio.muted {
                0
            } else {
                audio.volume_percent % 100
            }
        ));
    }
    args.extend([
        "-filter_complex".into(),
        graph.into(),
        "-map".into(),
        "[v]".into(),
    ]);
    if plan.audio.is_some() {
        args.extend(
            [
                "-map",
                "[a]",
                "-c:a",
                "aac",
                "-b:a",
                "192k",
                "-threads:a",
                "1",
            ]
            .into_iter()
            .map(Into::into),
        );
    } else {
        args.push("-an".into());
    }
    args.extend(
        [
            "-sn",
            "-dn",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            "18",
            "-pix_fmt",
            "yuv420p",
            "-threads:v",
            "2",
            "-color_primaries",
            "bt709",
            "-color_trc",
            "bt709",
            "-colorspace",
            "bt709",
            "-color_range",
            "tv",
            "-map_metadata",
            "-1",
            "-map_chapters",
            "-1",
            "-t",
        ]
        .into_iter()
        .map(Into::into),
    );
    args.push(seconds(i128::from(plan.duration_us)).into());
    args.extend(
        [
            "-use_editlist",
            "1",
            "-avoid_negative_ts",
            "disabled",
            "-f",
            "mp4",
            "-fd",
            "1",
            "-blocksize",
            "65536",
            "fd:",
        ]
        .into_iter()
        .map(Into::into),
    );
    Ok(args)
}
