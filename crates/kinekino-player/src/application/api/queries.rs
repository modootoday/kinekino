use super::{ApiError, ErrorCode, Request, capabilities};
use crate::application::subtitles::storage::SubtitleStorage;
use serde_json::{Value, json};

pub fn execute(
    request: &Request,
    storage: &impl SubtitleStorage,
    frame: &impl crate::application::frame_capture::FrameCapturePort,
) -> Result<Value, ApiError> {
    execute_with_playback(
        request,
        storage,
        frame,
        &crate::application::playback_status::UnavailablePlaybackStatus,
    )
}

pub fn execute_with_playback(
    request: &Request,
    storage: &impl SubtitleStorage,
    frame: &impl crate::application::frame_capture::FrameCapturePort,
    playback: &impl crate::application::playback_status::PlaybackStatusPort,
) -> Result<Value, ApiError> {
    execute_with_control(
        request,
        storage,
        frame,
        playback,
        &crate::application::playback_control::UnavailablePlaybackControl,
    )
}
pub fn execute_with_control(
    request: &Request,
    storage: &impl SubtitleStorage,
    frame: &impl crate::application::frame_capture::FrameCapturePort,
    playback: &impl crate::application::playback_status::PlaybackStatusPort,
    control: &impl crate::application::playback_control::PlaybackControlPort,
) -> Result<Value, ApiError> {
    execute_with_reviews(
        request,
        storage,
        frame,
        playback,
        control,
        &crate::application::model_review::UnavailableReviewStore,
    )
}
pub fn execute_with_reviews(
    request: &Request,
    storage: &impl SubtitleStorage,
    frame: &impl crate::application::frame_capture::FrameCapturePort,
    playback: &impl crate::application::playback_status::PlaybackStatusPort,
    control: &impl crate::application::playback_control::PlaybackControlPort,
    reviews: &impl crate::application::model_review::ReviewStore,
) -> Result<Value, ApiError> {
    execute_with_reviews_controlled(request, storage, frame, playback, control, reviews, None)
}

pub fn execute_with_reviews_controlled(
    request: &Request,
    storage: &impl SubtitleStorage,
    frame: &impl crate::application::frame_capture::FrameCapturePort,
    playback: &impl crate::application::playback_status::PlaybackStatusPort,
    control: &impl crate::application::playback_control::PlaybackControlPort,
    reviews: &impl crate::application::model_review::ReviewStore,
    execution_control: Option<&dyn crate::application::review_source::MutationControl>,
) -> Result<Value, ApiError> {
    execute_with_reviews_recoverable(
        request,
        storage,
        frame,
        playback,
        control,
        reviews,
        crate::application::frame_capture::FrameExecutionContext {
            control: execution_control,
            recovery: None,
        },
    )
}
pub fn execute_with_reviews_recoverable(
    request: &Request,
    storage: &impl SubtitleStorage,
    frame: &impl crate::application::frame_capture::FrameCapturePort,
    playback: &impl crate::application::playback_status::PlaybackStatusPort,
    control: &impl crate::application::playback_control::PlaybackControlPort,
    reviews: &impl crate::application::model_review::ReviewStore,
    execution: crate::application::frame_capture::FrameExecutionContext<'_>,
) -> Result<Value, ApiError> {
    if !capabilities::COMMANDS.contains(&request.command.as_str()) {
        return Err(ApiError::new(ErrorCode::InvalidArgument, "Unknown command"));
    }
    match request.command.as_str() {
        "playback.audio_handoff" => super::audio_handoff::execute(request, control),
        "version" | "capabilities"
            if request.payload.as_object().is_some_and(|v| !v.is_empty()) =>
        {
            Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "This query accepts no payload fields",
            ))
        }
        "version" => Ok(
            json!({"product":crate::domain::product::PRODUCT,"sdk_version":env!("CARGO_PKG_VERSION"),"api_version":"1","schema_version":"1"}),
        ),
        "capabilities" => {
            let mut value = super::audio_handoff::capabilities(
                capabilities::capabilities_with_reviews(
                    storage.available(),
                    frame.capability(),
                    playback.available(),
                    control.available(),
                    reviews.available(),
                ),
                control.audio_handoff_available(),
            );
            value["frame_capture"]["in_process_sdk_execution_cooperative_cancel"] =
                json!(frame.capability().available && frame.cooperative_cancel_supported());
            value["frame_capture"]["wire_cancel_ack"] = json!(false);
            value["frame_capture"]["sdk_execution_cancel_scope"] = json!("file_frame_adapter");
            value["frame_capture"]["cancel_after_publication_checkpoint"] =
                json!("actual_success_retained");
            Ok(value)
        }
        "schema" if request.payload["command"] == "playback.audio_handoff" => {
            super::audio_handoff::schema(request, control.audio_handoff_available())
        }
        "schema" => super::schema::schema(
            &request.payload,
            storage.available(),
            frame.capability().available,
            playback.available(),
            control.available(),
            reviews.available(),
        ),
        name if crate::application::model_review::is_command(name) && reviews.available() => {
            crate::application::model_review::execute(request, reviews)
        }
        name if crate::application::playback_control::is_command(name) && control.available() => {
            crate::application::playback_control::execute(request, control)
        }
        "playback.status" if playback.available() => {
            if request
                .payload
                .as_object()
                .is_none_or(|fields| !fields.is_empty())
            {
                return Err(ApiError::new(
                    ErrorCode::InvalidArgument,
                    "Playback status accepts an empty object payload only",
                ));
            }
            if request.context.expected_revision.is_some()
                || request.context.idempotency_key.is_some()
                || request.context.timeout_ms.is_some()
            {
                return Err(ApiError::new(
                    ErrorCode::Unsupported,
                    "Playback status does not support revision, idempotency or timeout options",
                ));
            }
            serde_json::to_value(playback.status()?)
                .map_err(|_| ApiError::new(ErrorCode::Internal, "Invalid playback observation"))
        }
        "production.capture_frame" if frame.capability().available => {
            if request.context.expected_revision.is_some()
                || request.context.idempotency_key.is_some()
            {
                return Err(ApiError::new(
                    ErrorCode::Unsupported,
                    "Capture revision and idempotency options are unavailable",
                ));
            }
            crate::application::frame_capture::execute_recoverable(
                frame,
                &request.payload,
                request.context.timeout_ms,
                execution,
            )
        }
        "subtitles.convert" | "subtitles.export_range" | "subtitles.from_transcript"
            if storage.available() =>
        {
            if request.context.expected_revision.is_some()
                || request.context.idempotency_key.is_some()
                || request.context.timeout_ms.is_some()
            {
                return Err(ApiError::new(
                    ErrorCode::Unsupported,
                    "Conversion does not support revision, idempotency or timeout context options",
                ));
            }
            if request.command == "subtitles.export_range" {
                crate::application::subtitles::export_range::execute(storage, &request.payload)
            } else if request.command == "subtitles.from_transcript" {
                crate::application::subtitles::from_transcript::execute(storage, &request.payload)
            } else {
                crate::application::subtitles::convert::execute(storage, &request.payload)
            }
        }
        _ => Err(ApiError::new(
            ErrorCode::Unsupported,
            "Command requires an unavailable application adapter",
        )),
    }
}
