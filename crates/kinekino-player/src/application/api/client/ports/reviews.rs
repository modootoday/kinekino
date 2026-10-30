//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Review and output decisions.
pub trait ReviewClient: ApplicationClient {
    fn inspect_output_review_target(
        &self,
        context: RequestContext,
        payload: crate::application::proposals::output_review::InspectOutputReviewTarget,
    ) -> Result<crate::application::proposals::output_review::OutputReviewInspection, ApiError>
    {
        review_call(
            self,
            context,
            "proposals.inspect_output_review_target",
            payload,
        )
    }
    fn record_output_decision(
        &self,
        context: RequestContext,
        payload: crate::application::proposals::output_review::RecordOutputDecision,
    ) -> Result<crate::application::proposals::output_review::StoredOutputDecision, ApiError> {
        review_call(self, context, "proposals.record_output_decision", payload)
    }
    fn output_decisions(
        &self,
        context: RequestContext,
        payload: crate::application::proposals::output_review::ListOutputDecisions,
    ) -> Result<crate::application::proposals::output_review::OutputDecisionHistory, ApiError> {
        review_call(self, context, "proposals.output_decisions", payload)
    }
    fn inspect_review(
        &self,
        context: RequestContext,
        payload: crate::application::model_review::InspectReviewPackage,
    ) -> Result<crate::application::model_review::ReviewPackageView, ApiError> {
        review_call(self, context, "reviews.inspect", payload)
    }
    fn record_review_decision(
        &self,
        context: RequestContext,
        payload: crate::domain::model_review::RecordReviewDecision,
    ) -> Result<crate::application::model_review::StoredDecision, ApiError> {
        review_call(self, context, "reviews.record_decision", payload)
    }
    fn review_decisions(
        &self,
        context: RequestContext,
        payload: crate::application::model_review::ListReviewDecisions,
    ) -> Result<Vec<crate::application::model_review::StoredDecision>, ApiError> {
        review_call(self, context, "reviews.decisions", payload)
    }
}
