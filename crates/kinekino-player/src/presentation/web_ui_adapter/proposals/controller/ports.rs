//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

/// Composition supplies the existing typed Meari plugin host, not another SDK policy layer.
pub trait ReviewHost {
    fn edit(
        &mut self,
        expected_revision: DecimalU64,
        request: EditProposal,
    ) -> Result<ProposalDocument, ApiError>;
    fn accept(
        &mut self,
        expected_revision: DecimalU64,
        request: AcceptProposal,
    ) -> Result<AcceptanceReceipt, ApiError>;
    fn inspect(&mut self, request: InspectProposal) -> Result<StoredProposal, ApiError>;
}
