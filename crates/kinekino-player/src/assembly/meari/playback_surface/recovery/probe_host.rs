use super::{
    ConstructionFailure, CreationFailure, KeyError, RecoveryKey, RecoveryRegistry,
    ReleaseObservation, ReleaseState, ReservationError, ReservationFailure,
};
use crate::infrastructure::mpv::{
    HiddenSurfaceProbe, NativeSurfaceInput, ProbeError, ProbeRelease,
};

/// Strong UI host retention. Forced Drop is not a successful shutdown or an OS safety guarantee.
pub struct RetainedProbeHost<I, O, C, R> {
    registry: Option<RecoveryRegistry<I, O, C, R>>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct ParentCloseBlocked;
pub type NativeProbeHost =
    RetainedProbeHost<NativeSurfaceInput, Box<HiddenSurfaceProbe>, ProbeError, ProbeError>;
pub struct ProbeStart {
    pub key: RecoveryKey,
    pub state: ReleaseState,
    pub construction_error: Option<ProbeError>,
}
impl<I, O, C, R> RetainedProbeHost<I, O, C, R> {
    pub fn new(capacity: usize) -> Result<Self, ReservationError> {
        Ok(Self {
            registry: Some(RecoveryRegistry::new(capacity)?),
        })
    }
    pub fn reserve(&mut self, input: I) -> Result<RecoveryKey, ReservationFailure<I>> {
        match self.registry.as_mut() {
            Some(registry) => registry.reserve(input),
            None => Err(ReservationFailure {
                input,
                error: ReservationError::Capacity,
            }),
        }
    }
    pub fn construct(
        &mut self,
        key: &RecoveryKey,
        create: impl FnOnce(&I) -> Result<O, ConstructionFailure<O, C>>,
    ) -> Result<(), KeyError> {
        self.registry
            .as_mut()
            .ok_or(KeyError::WrongPhase)?
            .construct(key, create)
    }
    pub fn with_owned<T>(
        &mut self,
        key: &RecoveryKey,
        visit: impl FnOnce(&I, &mut O) -> T,
    ) -> Result<T, KeyError> {
        self.registry
            .as_mut()
            .ok_or(KeyError::WrongPhase)?
            .with_owned(key, visit)
    }
    pub fn state(&self, key: &RecoveryKey) -> Result<ReleaseState, KeyError> {
        self.registry
            .as_ref()
            .ok_or(KeyError::WrongPhase)?
            .state(key)
    }
    pub fn release(
        &mut self,
        key: &RecoveryKey,
        release: impl FnOnce(&mut O) -> Result<ReleaseObservation, R>,
    ) -> Result<ReleaseState, KeyError> {
        self.registry
            .as_mut()
            .ok_or(KeyError::WrongPhase)?
            .release(key, release)
    }
    pub fn retire(&mut self, key: &RecoveryKey) -> Result<(), KeyError> {
        self.registry
            .as_mut()
            .ok_or(KeyError::WrongPhase)?
            .retire(key)
    }
    pub fn recover_creation_failure(
        &mut self,
        key: &RecoveryKey,
    ) -> Result<CreationFailure<I, O, C>, KeyError> {
        self.registry
            .as_mut()
            .ok_or(KeyError::WrongPhase)?
            .recover_creation_failure(key)
    }
    pub fn try_shutdown(&mut self) -> Result<(), ParentCloseBlocked> {
        let Some(registry) = self.registry.take() else {
            return Ok(());
        };
        match registry.shutdown() {
            Ok(()) => Ok(()),
            Err(registry) => {
                self.registry = Some(registry);
                Err(ParentCloseBlocked)
            }
        }
    }
}
impl NativeProbeHost {
    pub fn start_probe(
        &mut self,
        input: NativeSurfaceInput,
    ) -> Result<ProbeStart, Box<ReservationFailure<NativeSurfaceInput>>> {
        let key = self.reserve(input).map_err(Box::new)?;
        self.construct(&key, |input| {
            HiddenSurfaceProbe::prepare_retained(input)
                .map(Box::new)
                .map_err(|error| ConstructionFailure {
                    partial: None,
                    error,
                })
        })
        .expect("fresh reserved key owned by this host");
        // The registry owns the inert probe before any RegisterClass/CreateWindow call below.
        let construction_error =
            match self.with_owned(&key, |input, owner| owner.construct_retained(input)) {
                Ok(result) => result.err(),
                Err(KeyError::WrongPhase) => None, // Typed preparation failure remains in the registry.
                Err(_) => unreachable!("fresh key remains in the host"),
            };
        Ok(ProbeStart {
            state: self.state(&key).expect("fresh key remains in the host"),
            key,
            construction_error,
        })
    }
    pub fn release_probe(&mut self, key: &RecoveryKey) -> Result<ReleaseState, KeyError> {
        self.release(key, |owner| match owner.release()? {
            ProbeRelease::Complete { .. } | ProbeRelease::NotCreated => {
                Ok(ReleaseObservation::Complete)
            }
            ProbeRelease::Unobserved => Ok(ReleaseObservation::Unobserved),
        })
    }
}
