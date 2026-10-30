//! Retained hidden, disabled NoMedia child. Host frame/effect wiring is a separate integration.
mod cleanup;
mod construction;
pub(crate) mod frame_delivery;
#[cfg(test)]
mod tests;

use crate::infrastructure::mpv::{HiddenSurfaceProbe, NativeSurfaceInput, SurfaceInputFailure};
pub use construction::{ConstructionStatus, QuarantineCheckpoint};
use meari_native::{
    application::window_lifecycle::{
        ChildCreationFailure, ChildLease, CleanupRequest, CompletionSignal,
    },
    infrastructure::window::{
        NativeSurfaceRegistrar, NativeSurfaceRegistrationFailure, NativeSurfaceReservation,
    },
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
    sync::Arc,
};

#[cfg(test)]
thread_local! {
    static INERT_FAILURE_ONCE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
fn register_inert_failure_for_test(
    registrar: NativeSurfaceRegistrar<QuarantineChild>,
    reservation: NativeSurfaceReservation,
) -> Result<QuarantineRegistration, Box<QuarantineRegistrationFailure>> {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            INERT_FAILURE_ONCE.with(|flag| flag.set(false));
        }
    }
    INERT_FAILURE_ONCE.with(|flag| assert!(!flag.replace(true)));
    let _reset = Reset;
    register_no_media(registrar, reservation)
}

#[derive(Clone, Debug)]
pub enum QuarantineError {
    Input(Arc<crate::infrastructure::mpv::SurfaceInputError>),
    Child(Arc<crate::infrastructure::mpv::ChildInputError>),
    Probe(Arc<crate::infrastructure::mpv::ProbeError>),
    Unavailable,
    Identity,
}
impl std::fmt::Display for QuarantineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "retained quarantine: {self:?}")
    }
}
impl std::error::Error for QuarantineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Input(e) => Some(e.as_ref()),
            Self::Child(e) => Some(e.as_ref()),
            Self::Probe(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}
enum OriginalInput {
    Ready(Box<NativeSurfaceInput>),
    Rejected(Box<SurfaceInputFailure>),
}
struct OwnerState {
    input: OriginalInput,
    probe: Option<HiddenSurfaceProbe>,
    construction: ConstructionState,
    first_error: Option<QuarantineError>,
    request: Option<CleanupRequest>,
    signal: Option<CompletionSignal>,
    redraw_requested: bool,
    revoked: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConstructionState {
    Prepared,
    Entered,
    Constructed,
    Failed,
}
pub struct QuarantineChild {
    owner: Rc<RefCell<OwnerState>>,
    diagnostic: Option<QuarantineError>,
}

/// Retains the original registrar/coordinator and the sole original child authority.
pub struct QuarantineRegistration {
    pub registrar: NativeSurfaceRegistrar<QuarantineChild>,
    pub lease: ChildLease,
    pub checkpoint: QuarantineCheckpoint,
}
/// The actual registration failure retains its cleanup lease; partial child stays in coordinator.
pub struct QuarantineRegistrationFailure {
    pub registrar: NativeSurfaceRegistrar<QuarantineChild>,
    pub failure: NativeSurfaceRegistrationFailure<QuarantineError>,
    pub checkpoint: Option<QuarantineCheckpoint>,
}
pub fn register_no_media(
    registrar: NativeSurfaceRegistrar<QuarantineChild>,
    reservation: NativeSurfaceReservation,
) -> Result<QuarantineRegistration, Box<QuarantineRegistrationFailure>> {
    let mut checkpoint = None;
    let result = registrar.register_reserved(reservation, |_, parent, slot| {
        let (input, probe, error) = match NativeSurfaceInput::new(parent, slot) {
            Ok(input) => match HiddenSurfaceProbe::prepare_retained(&input) {
                Ok(probe) => (OriginalInput::Ready(Box::new(input)), Some(probe), None),
                Err(error) => (
                    OriginalInput::Ready(Box::new(input)),
                    None,
                    Some(QuarantineError::Probe(Arc::new(error))),
                ),
            },
            Err(failure) => {
                let error = QuarantineError::Input(Arc::new(failure.error.clone()));
                (OriginalInput::Rejected(failure), None, Some(error))
            }
        };
        #[cfg(test)]
        let error = error.or_else(|| {
            INERT_FAILURE_ONCE
                .with(|flag| flag.replace(false).then_some(QuarantineError::Unavailable))
        });
        let owner = Rc::new(RefCell::new(OwnerState {
            input,
            probe,
            construction: if error.is_some() {
                ConstructionState::Failed
            } else {
                ConstructionState::Prepared
            },
            first_error: error.clone(),
            request: None,
            signal: None,
            redraw_requested: false,
            revoked: false,
        }));
        checkpoint = Some(QuarantineCheckpoint {
            owner: Rc::downgrade(&owner),
        });
        let child = QuarantineChild {
            owner,
            diagnostic: error.clone(),
        };
        match error {
            Some(error) => Err(ChildCreationFailure {
                error,
                partial: Some(child),
            }),
            None => Ok(child),
        }
    });
    match result {
        Ok(lease) => Ok(QuarantineRegistration {
            registrar,
            lease,
            checkpoint: checkpoint.expect("successful factory installs checkpoint"),
        }),
        Err(failure) => Err(Box::new(QuarantineRegistrationFailure {
            registrar,
            failure,
            checkpoint,
        })),
    }
}
