use super::{AttemptBinding, SettledAttempt, SettlementKind, same_binding};
use crate::application::ExecutionControl;
use kinekino_media_contracts::errors::{ApiError, ErrorCode};
use std::sync::Arc;

/// Non-cloneable, no Deserialize; only the core gate can issue this value.
///
/// ```compile_fail
/// use kinekino_jobs::application::owned_attempt::PublicationPermit;
/// fn require_wire<T: serde::de::DeserializeOwned>() {}
/// require_wire::<PublicationPermit>();
/// ```
pub struct PublicationPermit {
    pub(super) binding: Arc<AttemptBinding>,
}

/// No public constructor. P1b must supply the original coordinator/CAS control;
/// engines receive only AttemptScope, never this capability or legacy control.
pub struct SettlementPublicationGate<'a> {
    pub(super) binding: Arc<AttemptBinding>,
    pub(super) control: &'a dyn ExecutionControl,
}

impl SettlementPublicationGate<'_> {
    pub fn reserve<T>(
        &self,
        settled: &mut SettledAttempt<T>,
    ) -> Result<PublicationPermit, ApiError> {
        if !same_binding(&self.binding, &settled.binding)
            || self.control.execution_identity() != Some(&self.binding.identity)
            || self.control.deadline() != self.binding.deadline
            || settled.permit_issued
            || settled.publication_reserved
            || settled.kind != SettlementKind::Graceful
            || settled.diagnostic.first().is_some()
            || settled.candidate.is_none()
        {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Settled publication requires the original successful attempt",
            ));
        }
        if self.binding.clock.now() >= self.binding.deadline {
            let error = ApiError::new(ErrorCode::Timeout, "Original deadline expired");
            settled.record_failure(&error);
            return Err(error);
        }
        // Existing durable CAS is still the cancellation/publication linearization.
        if let Err(error) = self.control.reserve_publication() {
            settled.record_failure(&error);
            return Err(error);
        }
        // This fact survives a post-CAS cutoff; never reserve this attempt again.
        settled.publication_reserved = true;
        if self.binding.clock.now() >= self.binding.deadline {
            let error = ApiError::new(
                ErrorCode::Timeout,
                "Original deadline expired during publication CAS",
            );
            settled.record_failure(&error);
            return Err(error);
        }
        settled.permit_issued = true;
        Ok(PublicationPermit {
            binding: Arc::clone(&self.binding),
        })
    }
}
