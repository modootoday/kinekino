use serde_json::{Value, json};

pub const COMMANDS: &[&str] = &[
    "asset_artifacts.inspect",
    "asset_sources.export",
    "asset_sources.inspect",
    "asset_projects.create",
    "asset_projects.inspect",
    "asset_projects.save_revision",
    "clip_projects.create",
    "clip_sources.inspect",
    "clip_projects.inspect",
    "clip_projects.save_revision",
    "job_batches.create",
    "job_batches.inspect",
    "job_batches.submit",
    "job_batches.refresh",
    "job_batches.retry_failed",
    "card_projects.create",
    "card_projects.inspect",
    "card_projects.save_revision",
    crate::application::review_source::media_api::PREPARE,
    crate::application::review_source::media_api::INSPECT,
    "reviews.playback.open",
    "reviews.playback.status",
    "reviews.playback.close",
    "reviews.playback.control",
    "reviews.association.inspect",
    "reviews.association.create",
    "reviews.association.revise",
    "reviews.range_draft.read",
    "reviews.range_draft.save",
    crate::application::proposals::card_export::COMMAND,
    "version",
    "capabilities",
    "schema",
    "catalog.search",
    "catalog.read_raw",
    "episodes.list",
    "episodes.inspect_resolution",
    "episodes.resolve",
    "playback.open",
    "playback.status",
    "playback.audio_handoff",
    "playback.play",
    "playback.pause",
    "playback.seek",
    "playback.engine",
    "playback.set_volume",
    "playback.set_muted",
    "playback.set_rate",
    "playback.select_track",
    "playback.previous",
    "playback.next",
    "playback.transition.observe",
    "playback.transition.commit",
    "playback.transition.release",
    "playback.close",
    "segments.detect",
    "segments.open",
    "segments.inspect",
    "segments.evaluate",
    "segments.skip",
    crate::application::skip_segments::host::SKIP_EXPORT,
    crate::application::skip_segments::host::EXPORT_LIST,
    crate::application::skip_segments::host::EXPORT_PEEK,
    crate::application::skip_segments::host::EXPORT_CONSUME,
    "segments.override",
    "segments.close",
    "segments.edit",
    "segments.accept_skip",
    "segments.set_skip_policy",
    "subtitles.transcribe",
    "subtitles.diarize",
    "subtitles.convert",
    "subtitles.export_range",
    "subtitles.from_transcript",
    "subtitles.speakers",
    "subtitles.align_speakers",
    "subtitles.edit_cues",
    "subtitles.edit_speakers",
    "production.render_short",
    "production.download_media",
    "production.capture_frame",
    "production.export_snapshot",
    "production.export_cards",
    "production.segment_subject",
    "production.build_assets",
    "production.package_projection",
    "models.list",
    "models.inspect",
    "models.install",
    "models.verify",
    "models.remove",
    "jobs.get",
    "jobs.submit",
    "jobs.list",
    "jobs.watch",
    "jobs.cancel",
    "jobs.retry",
    "projects.create",
    "projects.inspect",
    "projects.save_revision",
    "projects.reconnect_source",
    "projects.export",
    "reviews.inspect",
    "reviews.record_decision",
    "reviews.decisions",
    "reviews.read_text_asset",
    "proposals.create",
    "proposals.list",
    "proposals.import_historical",
    "proposals.inspect",
    crate::application::proposals::source_context::COMMAND,
    "proposals.edit",
    "proposals.accept",
    crate::application::proposals::subtitle_export::COMMAND,
    "proposals.inspect_output_review_target",
    "proposals.record_output_decision",
    "proposals.output_decisions",
];

pub fn supported_with_reviews(
    command: &str,
    files: bool,
    frames: bool,
    playback: bool,
    control: bool,
    reviews: bool,
) -> bool {
    supported_with_control(command, files, frames, playback, control)
        || crate::application::model_review::is_command(command) && reviews
}
pub fn capabilities_with_reviews(
    files: bool,
    frame: crate::application::frame_capture::FrameCaptureCapability,
    playback: bool,
    control: bool,
    reviews: bool,
) -> Value {
    let mut value = capabilities_with_control(files, frame, playback, control);
    if let Some(commands) = value["commands"].as_array_mut() {
        for command in commands {
            if command["command"]
                .as_str()
                .is_some_and(crate::application::model_review::is_command)
            {
                command["status"] = json!(if reviews { "supported" } else { "unsupported" });
            }
        }
    }
    value["reviews"] = json!({"backend_available":reviews,"package_version":"1","content_identity":"sha256_and_bytes","verdicts":["usable","needs_changes","deferred"],"scope":"sample_only","automatic_application_enabled":false,"immutable_decisions":true,"text_assets":{"max_bytes":crate::application::model_review::MAX_TEXT_ASSET_BYTES,"encoding":"strict_utf8","media_types":["text/*","application/json"],"package_references_only":true}});
    value
}

pub fn supported(command: &str, files_available: bool, frame_available: bool) -> bool {
    supported_with_playback(command, files_available, frame_available, false)
}
pub fn supported_with_playback(
    command: &str,
    files_available: bool,
    frame_available: bool,
    playback_available: bool,
) -> bool {
    supported_with_control(
        command,
        files_available,
        frame_available,
        playback_available,
        false,
    )
}
pub fn supported_with_control(
    command: &str,
    files_available: bool,
    frame_available: bool,
    playback_available: bool,
    control_available: bool,
) -> bool {
    matches!(command, "version" | "capabilities" | "schema")
        || matches!(
            command,
            "subtitles.convert" | "subtitles.export_range" | "subtitles.from_transcript"
        ) && files_available
        || command == "production.capture_frame" && frame_available
        || command == "playback.status" && playback_available
        || crate::application::playback_control::is_command(command) && control_available
}
pub fn capabilities(
    files_available: bool,
    frame: crate::application::frame_capture::FrameCaptureCapability,
) -> Value {
    capabilities_with_playback(files_available, frame, false)
}
pub fn capabilities_with_playback(
    files_available: bool,
    frame: crate::application::frame_capture::FrameCaptureCapability,
    playback_available: bool,
) -> Value {
    capabilities_with_control(files_available, frame, playback_available, false)
}
pub fn capabilities_with_control(
    files_available: bool,
    frame: crate::application::frame_capture::FrameCaptureCapability,
    playback_available: bool,
    control_available: bool,
) -> Value {
    let supported = |command: &str, files, frames| {
        supported_with_control(
            command,
            files,
            frames,
            playback_available,
            control_available,
        )
    };
    json!({"runtime":"in_process", "local_host_transport":"unsupported", "subtitles":{"formats":["srt","vtt"],"encoding":"utf-8","max_input_bytes":crate::application::subtitles::storage::MAX_SUBTITLE_BYTES,"overwrite":false,"loss_confirmation_required":true,"ass":"output_only","ocr":"unsupported"},"frame_capture":{"dependency":frame,"formats":["mp4","mkv","webm"],"output":"png","max_media_bytes":crate::domain::frame_capture::MAX_MEDIA_BYTES,"max_pixels":crate::domain::frame_capture::MAX_FRAME_PIXELS,"max_timeout_ms":120000,"selection":"first_source_pts_at_or_after_request","network":false,"overwrite":false,"hdr":"unsupported"}, "commands": COMMANDS.iter().map(|command| json!({"command":command,"status":if supported(command,files_available,frame.available) {"supported"} else {"unsupported"}})).collect::<Vec<_>>()})
}

/// Renders the README command table from [`COMMANDS`] (grouped by namespace).
pub fn readme_command_table() -> String {
    let mut groups: Vec<(&str, Vec<&str>)> = Vec::new();
    for command in COMMANDS {
        let (group, action) = command.split_once('.').unwrap_or((command, ""));
        let action = if action.is_empty() { group } else { action };
        match groups.iter_mut().find(|(name, _)| *name == group) {
            Some((_, actions)) => actions.push(action),
            None => groups.push((group, vec![action])),
        }
    }
    let mut out = String::from("| Group | Commands |\n| --- | --- |\n");
    for (group, actions) in groups {
        let list: Vec<String> = actions.iter().map(|a| format!("`{a}`")).collect();
        out.push_str(&format!("| `{group}` | {} |\n", list.join(", ")));
    }
    out
}

#[cfg(test)]
mod readme_drift_tests {
    #[test]
    fn readme_command_block_matches_capabilities() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../README.md");
        let readme = std::fs::read_to_string(path).expect("README.md");
        let begin = "<!-- kinekino:commands:begin -->\n";
        let end = "<!-- kinekino:commands:end -->";
        let start = readme.find(begin).expect("begin marker") + begin.len();
        let stop = readme.find(end).expect("end marker");
        let expected = super::readme_command_table();
        assert_eq!(
            &readme[start..stop],
            expected,
            "README command table drifted; replace the block with:\n{expected}"
        );
    }
}
