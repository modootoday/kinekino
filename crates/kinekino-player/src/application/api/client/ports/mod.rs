//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

mod authoring;
mod episodes;
mod job_batches;
mod jobs;
mod playback;
mod proposals;
mod reviews;
pub use {
    authoring::AssetClient, authoring::CardClient, authoring::ClipClient, authoring::ProjectClient,
    episodes::EpisodeClient, job_batches::JobBatchClient, jobs::JobClient,
    playback::PlaybackClient, proposals::ProposalClient, reviews::ReviewClient,
};

/// Transport core; typed operations live in the extension traits below,
/// each blanket-implemented for every `ApplicationClient`.
pub trait ApplicationClient: Send + Sync {
    fn execute(&self, request: Request) -> Response;
    /// Opt-in recovery of receipt persistence on the original execution worker.
    /// The default preserves existing controlled behavior and advertises no recovery.
    fn execute_recoverable(
        &self,
        request: Request,
        control: &(dyn crate::application::review_source::MutationControl + Sync),
        _recovery: &dyn crate::application::sdk_execution::ReceiptRecoveryControl,
    ) -> Response {
        self.execute_controlled(request, control)
    }
    /// Cooperative hook for explicitly participating operations. Legacy clients keep
    /// their actual response; a cancellation request alone never rewrites success.
    fn execute_controlled(
        &self,
        request: Request,
        _control: &(dyn crate::application::review_source::MutationControl + Sync),
    ) -> Response {
        self.execute(request)
    }
}
impl<T: ApplicationClient + ?Sized> AssetClient for T {}
impl<T: ApplicationClient + ?Sized> ClipClient for T {}
impl<T: ApplicationClient + ?Sized> CardClient for T {}
impl<T: ApplicationClient + ?Sized> JobBatchClient for T {}
impl<T: ApplicationClient + ?Sized> EpisodeClient for T {}
impl<T: ApplicationClient + ?Sized> JobClient for T {}
impl<T: ApplicationClient + ?Sized> ProposalClient for T {}
impl<T: ApplicationClient + ?Sized> ReviewClient for T {}
impl<T: ApplicationClient + ?Sized> ProjectClient for T {}
impl<T: ApplicationClient + ?Sized> PlaybackClient for T {}
