//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait InitialInspectionPort {
    fn peek_terminal(&mut self) -> Result<Option<Arc<Result<StoredProposal, ApiError>>>, ApiError>;
    fn retained_terminal(&self) -> Result<Option<Arc<ExecutionTerminal>>, ApiError>;
    fn acknowledge(&mut self) -> Result<(), ApiError>;
    fn cancel(&self) -> Result<(), ApiError>;
}
