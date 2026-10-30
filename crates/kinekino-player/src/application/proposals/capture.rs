//! Capture data from one configured provider; origin is not a UI permission.
use crate::domain::proposals::{ProposalStoreOrigin, StoredProposal};

#[derive(Debug)]
pub struct CapturedProposal {
    pub state: StoredProposal,
    pub origin: Option<ProposalStoreOrigin>,
}
