//! Native5 and explicit Native8 codec profiles share the same owned SDK execution path.
mod port;
mod registration;
#[cfg(all(test, windows))]
mod tests;
use super::proposal_manifest::{
    async_proposal_manifest, output_review_proposal_manifest, raw_review_proposal_manifest,
};
use crate::application::{
    ai_provenance::ImportHistoricalProposal,
    api::{ApiError, RequestContext},
    proposals::output_review::{
        InspectOutputReviewTarget, ListOutputDecisions, OutputDecisionHistory,
        OutputReviewInspection, RecordOutputDecision, StoredOutputDecision,
    },
    proposals::{AcceptProposal, CreateProposal, EditProposal, InspectProposal},
    sdk_execution::ExecutionHandle,
};
use kinekino_ui_plugins::infrastructure::typed_async_sdk_plugin::{
    AsyncCodecBudget, AsyncSdkPump, TypedAsyncSdkPlugin, async_binding,
};
use meari::application::plugins::*;
pub use registration::AsyncProposalRegistration;
use std::{collections::BTreeMap, rc::Rc};

pub fn adapter<C>(
    execution: ExecutionHandle,
    budget: AsyncCodecBudget,
    context: C,
) -> Result<(TypedAsyncSdkPlugin, AsyncSdkPump), PluginError>
where
    C: Fn(&PluginTicket) -> Result<RequestContext, ApiError> + 'static,
{
    adapter_profile(execution, budget, context, false, false)
}
/// Opt-in composition only; the existing registration continues to request Native5.
pub fn output_review_adapter<C>(
    execution: ExecutionHandle,
    budget: AsyncCodecBudget,
    context: C,
) -> Result<(TypedAsyncSdkPlugin, AsyncSdkPump), PluginError>
where
    C: Fn(&PluginTicket) -> Result<RequestContext, ApiError> + 'static,
{
    adapter_profile(execution, budget, context, true, false)
}
pub fn raw_review_adapter<C>(
    execution: ExecutionHandle,
    budget: AsyncCodecBudget,
    context: C,
) -> Result<(TypedAsyncSdkPlugin, AsyncSdkPump), PluginError>
where
    C: Fn(&PluginTicket) -> Result<RequestContext, ApiError> + 'static,
{
    adapter_profile(execution, budget, context, true, true)
}
fn adapter_profile<C>(
    execution: ExecutionHandle,
    budget: AsyncCodecBudget,
    context: C,
    output_reviews: bool,
    raw_reviews: bool,
) -> Result<(TypedAsyncSdkPlugin, AsyncSdkPump), PluginError>
where
    C: Fn(&PluginTicket) -> Result<RequestContext, ApiError> + 'static,
{
    let execution = std::sync::Arc::new(execution.attach().map_err(|_| {
        PluginError::new(
            PluginErrorCode::Busy,
            "SDK worker is attached or initial loading remains unacknowledged",
        )
    })?);
    let manifest = if raw_reviews {
        raw_review_proposal_manifest()
    } else if output_reviews {
        output_review_proposal_manifest()
    } else {
        async_proposal_manifest()
    };
    let context = Rc::new(context);
    let wire_limit = budget.max_terminal_wire_bytes;
    macro_rules! bind {
        ($name:literal,$input:ty,$output:ty) => {{
            let descriptor = manifest.capabilities.get($name).cloned().ok_or_else(|| {
                PluginError::new(
                    PluginErrorCode::MissingCapability,
                    "Async proposal descriptor missing",
                )
            })?;
            async_binding(
                descriptor.clone(),
                port::SdkPort::<$input, $output, C>::new(
                    execution.clone(),
                    $name,
                    descriptor,
                    context.clone(),
                    wire_limit,
                ),
            )
        }};
    }
    let mut bindings = BTreeMap::from([
        (
            "proposals.create".into(),
            bind!(
                "proposals.create",
                CreateProposal,
                crate::domain::proposals::ProposalDocument
            ),
        ),
        (
            "proposals.inspect".into(),
            bind!(
                "proposals.inspect",
                InspectProposal,
                crate::domain::proposals::StoredProposal
            ),
        ),
        (
            "proposals.edit".into(),
            bind!(
                "proposals.edit",
                EditProposal,
                crate::domain::proposals::ProposalDocument
            ),
        ),
        (
            "proposals.accept".into(),
            bind!(
                "proposals.accept",
                AcceptProposal,
                crate::domain::proposals::AcceptanceReceipt
            ),
        ),
        (
            "proposals.import_historical".into(),
            bind!(
                "proposals.import_historical",
                ImportHistoricalProposal,
                crate::domain::proposals::StoredProposal
            ),
        ),
    ]);
    if output_reviews {
        bindings.insert(
            "proposals.inspect_output_review_target".into(),
            bind!(
                "proposals.inspect_output_review_target",
                InspectOutputReviewTarget,
                OutputReviewInspection
            ),
        );
        bindings.insert(
            "proposals.record_output_decision".into(),
            bind!(
                "proposals.record_output_decision",
                RecordOutputDecision,
                StoredOutputDecision
            ),
        );
        bindings.insert(
            "proposals.output_decisions".into(),
            bind!(
                "proposals.output_decisions",
                ListOutputDecisions,
                OutputDecisionHistory
            ),
        );
    }
    if raw_reviews {
        use crate::application::model_review::{
            InspectReviewPackage, ListReviewDecisions, ReadReviewTextAsset, ReviewPackageView,
            ReviewTextAsset, StoredDecision,
        };
        use crate::domain::model_review::RecordReviewDecision;
        bindings.extend([
            (
                "reviews.inspect".into(),
                bind!("reviews.inspect", InspectReviewPackage, ReviewPackageView),
            ),
            (
                "reviews.read_text_asset".into(),
                bind!(
                    "reviews.read_text_asset",
                    ReadReviewTextAsset,
                    ReviewTextAsset
                ),
            ),
            (
                "reviews.record_decision".into(),
                bind!(
                    "reviews.record_decision",
                    RecordReviewDecision,
                    StoredDecision
                ),
            ),
            (
                "reviews.decisions".into(),
                bind!(
                    "reviews.decisions",
                    ListReviewDecisions,
                    Vec<StoredDecision>
                ),
            ),
        ]);
    }
    TypedAsyncSdkPlugin::new(manifest, bindings, budget)
}
