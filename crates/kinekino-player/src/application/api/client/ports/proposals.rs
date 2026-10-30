//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Proposals, cue edits, subtitle exports and review media.
pub trait ProposalClient: ApplicationClient {
    fn edit_subtitle_cues(
        &self,
        context: RequestContext,
        input: crate::application::proposals::cue_edit::EditCues,
    ) -> Result<crate::domain::proposals::ProposalDocument, ApiError> {
        review_call(
            self,
            context,
            crate::application::proposals::cue_edit::COMMAND,
            input,
        )
    }
    fn export_subtitle_range(
        &self,
        context: RequestContext,
        payload: crate::application::subtitles::export_range::ExportSubtitleRange,
    ) -> Result<ConversionResult, ApiError> {
        review_call(self, context, "subtitles.export_range", payload)
    }
    fn list_proposals(
        &self,
        context: RequestContext,
        payload: crate::application::proposals::list::ListProposals,
    ) -> Result<crate::application::proposals::list::ProposalListPage, ApiError> {
        review_call(self, context, "proposals.list", payload)
    }
    fn create_proposal(
        &self,
        context: RequestContext,
        payload: crate::application::proposals::CreateProposal,
    ) -> Result<crate::domain::proposals::ProposalDocument, ApiError> {
        review_call(self, context, "proposals.create", payload)
    }
    fn inspect_proposal(
        &self,
        context: RequestContext,
        payload: crate::application::proposals::InspectProposal,
    ) -> Result<crate::application::proposals::StoredProposal, ApiError> {
        review_call(self, context, "proposals.inspect", payload)
    }
    fn import_historical_proposal(
        &self,
        context: RequestContext,
        payload: crate::application::ai_provenance::ImportHistoricalProposal,
    ) -> Result<crate::application::proposals::StoredProposal, ApiError> {
        review_call(self, context, "proposals.import_historical", payload)
    }
    fn inspect_proposal_source_context(
        &self,
        context: RequestContext,
        payload: crate::application::proposals::InspectProposal,
    ) -> Result<crate::application::proposals::source_context::SourceContextInspection, ApiError>
    {
        let result: crate::application::proposals::source_context::SourceContextInspection =
            review_call(
                self,
                context,
                crate::application::proposals::source_context::COMMAND,
                &payload,
            )?;
        result.validate()?;
        if result.state.document.proposal_id != payload.proposal_id {
            return Err(ApiError::new(
                ErrorCode::EngineFailed,
                "Source context response belongs to a different proposal",
            ));
        }
        Ok(result)
    }
    fn prepare_review_media(
        &self,
        context: RequestContext,
        payload: &crate::application::review_source::media_api::MediaSelectionRequest,
    ) -> Result<crate::application::review_source::media_api::MediaPreparationReceipt, ApiError>
    {
        let result: crate::application::review_source::media_api::MediaPreparationReceipt =
            review_call(
                self,
                context,
                crate::application::review_source::media_api::PREPARE,
                payload,
            )?;
        result.validate(payload)?;
        Ok(result)
    }
    fn inspect_review_media(
        &self,
        context: RequestContext,
        payload: &crate::application::review_source::media_api::MediaSelectionRequest,
    ) -> Result<crate::application::review_source::media_api::MediaPreparationReceipt, ApiError>
    {
        let result: crate::application::review_source::media_api::MediaPreparationReceipt =
            review_call(
                self,
                context,
                crate::application::review_source::media_api::INSPECT,
                payload,
            )?;
        result.validate(payload)?;
        Ok(result)
    }
    fn edit_proposal(
        &self,
        context: RequestContext,
        payload: crate::application::proposals::EditProposal,
    ) -> Result<crate::domain::proposals::ProposalDocument, ApiError> {
        review_call(self, context, "proposals.edit", payload)
    }
    fn edit_captured_proposal(
        &self,
        context: RequestContext,
        payload: crate::application::proposals::guarded::EditCapturedProposal,
    ) -> Result<crate::application::proposals::guarded::CaptureMutationResult, ApiError> {
        review_call(self, context, "proposals.edit_capture", payload)
    }
    fn accept_captured_proposal(
        &self,
        context: RequestContext,
        payload: crate::application::proposals::guarded::AcceptCapturedProposal,
    ) -> Result<crate::application::proposals::guarded::CaptureMutationResult, ApiError> {
        review_call(self, context, "proposals.accept_capture", payload)
    }
    fn accept_proposal(
        &self,
        context: RequestContext,
        payload: crate::application::proposals::AcceptProposal,
    ) -> Result<crate::domain::proposals::AcceptanceReceipt, ApiError> {
        review_call(self, context, "proposals.accept", payload)
    }
    fn export_accepted_subtitles(
        &self,
        context: RequestContext,
        input: crate::application::proposals::subtitle_export::ExportAcceptedSubtitles,
    ) -> Result<crate::application::proposals::subtitle_export::ExportedAcceptedSubtitles, ApiError>
    {
        review_call(
            self,
            context,
            crate::application::proposals::subtitle_export::COMMAND,
            input,
        )
    }

    fn convert_subtitles(
        &self,
        context: RequestContext,
        payload: ConvertSubtitles,
    ) -> Result<ConversionResult, ApiError> {
        let mut request = Request::new("subtitles.convert");
        request.context = context;
        request.payload = serde_json::to_value(payload).map_err(|_| invalid_response())?;
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
}
