//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait SkipSegmentsClient: ApplicationClient {
    fn skip_segment_export(
        &self,
        context: RequestContext,
        input: SkipExportRequest,
    ) -> Result<SkipExportResult, ApiError> {
        review_call(self, context, SKIP_EXPORT, input)
    }
    fn list_skip_exports(
        &self,
        context: RequestContext,
        input: ListSkipExports,
    ) -> Result<crate::application::playback_session::v1::NativeExportListV1, ApiError> {
        review_call(self, context, EXPORT_LIST, input)
    }
    fn peek_skip_export(
        &self,
        context: RequestContext,
        input: ReadSkipExport,
    ) -> Result<crate::application::playback_session::v1::NativeExportResultV1, ApiError> {
        review_call(self, context, EXPORT_PEEK, input)
    }
    fn consume_skip_export(
        &self,
        context: RequestContext,
        input: ReadSkipExport,
    ) -> Result<crate::application::playback_session::v1::NativeExportResultV1, ApiError> {
        review_call(self, context, EXPORT_CONSUME, input)
    }
    fn open_skip_session(
        &self,
        context: RequestContext,
        input: OpenSkipSession,
    ) -> Result<SessionView, ApiError> {
        review_call(self, context, "segments.open", input)
    }
    fn inspect_skip_session(
        &self,
        context: RequestContext,
        input: SessionRef,
    ) -> Result<SkipSnapshot, ApiError> {
        review_call(self, context, "segments.inspect", input)
    }
    fn edit_skip_segment(
        &self,
        context: RequestContext,
        input: SessionRequest<EditInput>,
    ) -> Result<SkipSnapshot, ApiError> {
        review_call(self, context, "segments.edit", input)
    }
    fn verify_skip_segment(
        &self,
        context: RequestContext,
        input: SessionRequest<VerifyInput>,
    ) -> Result<SkipSnapshot, ApiError> {
        review_call(self, context, "segments.accept_skip", input)
    }
    fn set_skip_policy(
        &self,
        context: RequestContext,
        input: SessionRequest<PolicyInput>,
    ) -> Result<SkipSnapshot, ApiError> {
        review_call(self, context, "segments.set_skip_policy", input)
    }
    fn evaluate_skip_segment(
        &self,
        context: RequestContext,
        input: SessionRequest<EvaluateInput>,
    ) -> Result<EvaluationResult, ApiError> {
        review_call(self, context, "segments.evaluate", input)
    }
    fn skip_segment(
        &self,
        context: RequestContext,
        input: SessionRequest<SkipInput>,
    ) -> Result<ActionResult, ApiError> {
        review_call(self, context, "segments.skip", input)
    }
    fn override_skip_session(
        &self,
        context: RequestContext,
        input: SessionRequest<OverrideInput>,
    ) -> Result<OverrideResult, ApiError> {
        review_call(self, context, "segments.override", input)
    }
    fn close_skip_session(
        &self,
        context: RequestContext,
        input: SessionRef,
    ) -> Result<ClosedSkipSession, ApiError> {
        review_call(self, context, "segments.close", input)
    }
}
