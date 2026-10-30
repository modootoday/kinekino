//! Port traits of the parent module (hexagonal `ports` location).
use super::*;

pub trait MediaPreparationPort: Send + Sync {
    fn writable(&self) -> bool;
    fn prepare(
        &self,
        input: &MediaSelectionRequest,
        control: &dyn MutationControl,
    ) -> Result<MediaPreparationReceipt, ApiError>;
    fn inspect(&self, input: &MediaSelectionRequest) -> Result<MediaPreparationReceipt, ApiError>;
}
