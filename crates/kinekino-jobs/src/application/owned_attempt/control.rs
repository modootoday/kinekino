use super::AttemptBinding;
use crate::application::JobAttemptIdentityV1;
use std::{sync::Arc, time::Instant};

/// Trusted parent observation only; deliberately has no publication methods.
pub trait AttemptControl: Send + Sync {
    fn cancellation_requested(&self) -> bool;
}

/// Borrowed attempt context, neither cloneable nor deserializable.
/// No production constructor is exposed before P1b admission is integrated.
///
/// ```compile_fail
/// use kinekino_jobs::application::owned_attempt::AttemptScope;
/// fn engine(scope: AttemptScope<'_>) {
///     scope.reserve_publication().unwrap();
/// }
/// ```
///
/// ```compile_fail
/// use kinekino_jobs::application::owned_attempt::AttemptScope;
/// fn engine(scope: AttemptScope<'_>) {
///     let duplicate = scope.clone();
/// }
/// ```
pub struct AttemptScope<'a> {
    pub(super) binding: Arc<AttemptBinding>,
    pub(super) control: ControlSource<'a>,
}
pub(super) enum ControlSource<'a> {
    Original(&'a dyn crate::application::ExecutionControl),
    #[cfg(test)]
    Restricted(&'a dyn AttemptControl),
}

impl AttemptScope<'_> {
    pub fn identity(&self) -> &JobAttemptIdentityV1 {
        &self.binding.identity
    }

    pub fn deadline(&self) -> Instant {
        self.binding.deadline
    }

    pub fn cancellation_requested(&self) -> bool {
        match self.control {
            ControlSource::Original(control) => control.cancellation_requested(),
            #[cfg(test)]
            ControlSource::Restricted(control) => control.cancellation_requested(),
        }
    }
}
