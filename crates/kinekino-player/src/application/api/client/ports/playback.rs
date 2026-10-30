//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Playback control, snapshots, frame capture and subtitle conversion.
pub trait PlaybackClient: ApplicationClient {
    fn export_snapshot(
        &self,
        context: RequestContext,
        input: crate::application::snapshot_export::ExportSnapshot,
    ) -> Result<crate::application::snapshot_export::ExportedSnapshot, ApiError> {
        review_call(self, context, "production.export_snapshot", input)
    }
    fn playback_control(
        &self,
        context: RequestContext,
        control: crate::domain::playback_control::PlaybackControl,
    ) -> Result<crate::application::playback_control::ControlReceipt, ApiError> {
        let mut request = Request::new(control.command());
        request.context = context;
        request.payload = control.payload();
        let response = self.execute(request);
        if let Some(error) = response.error {
            return Err(error);
        }
        if !response.ok {
            return Err(invalid_response());
        }
        serde_json::from_value(response.data.ok_or_else(invalid_response)?)
            .map_err(|_| invalid_response())
    }
    fn playback_status(
        &self,
        context: RequestContext,
    ) -> Result<crate::application::playback_status::PlaybackStatus, ApiError> {
        let mut request = Request::new("playback.status");
        request.context = context;
        let response = self.execute(request);
        if let Some(error) = response.error {
            return Err(error);
        }
        if !response.ok {
            return Err(invalid_response());
        }
        serde_json::from_value(response.data.ok_or_else(invalid_response)?)
            .map_err(|_| invalid_response())
    }
    fn capture_frame(
        &self,
        context: RequestContext,
        payload: crate::domain::frame_capture::CaptureFrame,
    ) -> Result<crate::application::frame_capture::CapturedFrame, ApiError> {
        let response = self.execute(Request {
            context,
            command: "production.capture_frame".into(),
            payload: serde_json::to_value(payload).map_err(|_| invalid_response())?,
        });
        if let Some(error) = response.error {
            return Err(error);
        }
        if !response.ok {
            return Err(invalid_response());
        }
        serde_json::from_value(response.data.ok_or_else(invalid_response)?)
            .map_err(|_| invalid_response())
    }
}
