use std::{
    marker::PhantomData,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_REGISTRY: AtomicU64 = AtomicU64::new(1);
#[derive(Debug, PartialEq, Eq)]
pub struct RecoveryKey {
    registry: u64,
    generation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReservationError {
    Capacity,
    CounterExhausted,
}
#[derive(Debug)]
pub struct ReservationFailure<I> {
    pub error: ReservationError,
    pub input: I,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyError {
    Foreign,
    Stale,
    WrongPhase,
}
#[derive(Debug)]
pub struct CreationFailure<I, O, E> {
    pub partial: Option<O>,
    pub input: I,
    pub error: E,
}
#[derive(Debug)]
pub struct ConstructionFailure<O, E> {
    pub partial: Option<O>,
    pub error: E,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseObservation {
    Complete,
    Unobserved,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseState {
    Reserved,
    Constructing,
    Owned,
    CreationFailed,
    Failed,
    Unobserved,
    Released,
}
enum Entry<I, O, C> {
    Reserved(I),
    Owned { owner: O, input: I },
    CreationFailed(CreationFailure<I, O, C>),
}
struct Slot<I, O, C, R> {
    generation: u64,
    entry: Entry<I, O, C>,
    state: ReleaseState,
    release_error: Option<R>,
}

/// Owns values from reservation onward. Dropping this policy is NOT a safe shutdown guarantee.
/// A production host must retain the returned incomplete registry and its parent owner.
pub struct RecoveryRegistry<I, O, C, R> {
    id: u64,
    next_generation: u64,
    capacity: usize,
    slots: Vec<Slot<I, O, C, R>>,
    _ui: PhantomData<Rc<()>>,
}
impl<I, O, C, R> RecoveryRegistry<I, O, C, R> {
    pub fn new(capacity: usize) -> Result<Self, ReservationError> {
        let id = NEXT_REGISTRY
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| ReservationError::CounterExhausted)?;
        Ok(Self {
            id,
            next_generation: 1,
            capacity,
            slots: Vec::new(),
            _ui: PhantomData,
        })
    }
    pub fn reserve(&mut self, input: I) -> Result<RecoveryKey, ReservationFailure<I>> {
        if self.slots.len() >= self.capacity {
            return Err(ReservationFailure {
                error: ReservationError::Capacity,
                input,
            });
        }
        let Some(next) = self.next_generation.checked_add(1) else {
            return Err(ReservationFailure {
                error: ReservationError::CounterExhausted,
                input,
            });
        };
        // Reserve storage before any caller can start an external allocation.
        if self.slots.try_reserve(1).is_err() {
            return Err(ReservationFailure {
                error: ReservationError::Capacity,
                input,
            });
        }
        let generation = self.next_generation;
        self.next_generation = next;
        self.slots.push(Slot {
            generation,
            entry: Entry::Reserved(input),
            state: ReleaseState::Reserved,
            release_error: None,
        });
        Ok(RecoveryKey {
            registry: self.id,
            generation,
        })
    }
    fn index(&self, key: &RecoveryKey) -> Result<usize, KeyError> {
        if key.registry != self.id {
            return Err(KeyError::Foreign);
        }
        self.slots
            .iter()
            .position(|slot| slot.generation == key.generation)
            .ok_or(KeyError::Stale)
    }
    pub fn cancel_reservation(&mut self, key: &RecoveryKey) -> Result<I, KeyError> {
        let index = self.index(key)?;
        if self.slots[index].state != ReleaseState::Reserved {
            return Err(KeyError::WrongPhase);
        }
        let slot = self.slots.swap_remove(index);
        match slot.entry {
            Entry::Reserved(input) => Ok(input),
            _ => unreachable!("phase checked"),
        }
    }
    /// Original input stays here during the borrowed factory. Panic leaves a Constructing tombstone.
    /// The factory must separately preserve partial external resources during its own unwinding.
    pub fn construct(
        &mut self,
        key: &RecoveryKey,
        create: impl FnOnce(&I) -> Result<O, ConstructionFailure<O, C>>,
    ) -> Result<(), KeyError> {
        let index = self.index(key)?;
        if self.slots[index].state != ReleaseState::Reserved {
            return Err(KeyError::WrongPhase);
        }
        self.slots[index].state = ReleaseState::Constructing;
        let result = match &self.slots[index].entry {
            Entry::Reserved(input) => create(input),
            _ => unreachable!("phase checked"),
        };
        // No input/entry is removed until the borrowed callback returns normally.
        let mut slot = self.slots.swap_remove(index);
        let Entry::Reserved(input) = slot.entry else {
            unreachable!("phase checked");
        };
        slot.entry = match result {
            Ok(owner) => {
                slot.state = ReleaseState::Owned;
                Entry::Owned { owner, input }
            }
            Err(failure) => {
                slot.state = ReleaseState::CreationFailed;
                Entry::CreationFailed(CreationFailure {
                    input,
                    partial: failure.partial,
                    error: failure.error,
                })
            }
        };
        self.slots.push(slot);
        Ok(())
    }
    pub fn input(&self, key: &RecoveryKey) -> Result<&I, KeyError> {
        Ok(match &self.slots[self.index(key)?].entry {
            Entry::Reserved(input) | Entry::Owned { input, .. } => input,
            Entry::CreationFailed(failure) => &failure.input,
        })
    }
    /// The entry remains stored during the callback, including unwinding. No registry reentry allowed.
    pub fn with_owned<T>(
        &mut self,
        key: &RecoveryKey,
        visit: impl FnOnce(&I, &mut O) -> T,
    ) -> Result<T, KeyError> {
        let index = self.index(key)?;
        if self.slots[index].state != ReleaseState::Owned {
            return Err(KeyError::WrongPhase);
        }
        match &mut self.slots[index].entry {
            Entry::Owned { owner, input } => Ok(visit(input, owner)),
            _ => Err(KeyError::WrongPhase),
        }
    }
    pub fn creation_failure(
        &self,
        key: &RecoveryKey,
    ) -> Result<Option<&CreationFailure<I, O, C>>, KeyError> {
        Ok(match &self.slots[self.index(key)?].entry {
            Entry::CreationFailed(failure) => Some(failure),
            _ => None,
        })
    }
    pub fn release_error(&self, key: &RecoveryKey) -> Result<Option<&R>, KeyError> {
        Ok(self.slots[self.index(key)?].release_error.as_ref())
    }
    pub fn state(&self, key: &RecoveryKey) -> Result<ReleaseState, KeyError> {
        Ok(self.slots[self.index(key)?].state)
    }
    /// Call only from the existing unique cleanup driver. Does not create a cleanup attempt.
    pub fn release(
        &mut self,
        key: &RecoveryKey,
        release: impl FnOnce(&mut O) -> Result<ReleaseObservation, R>,
    ) -> Result<ReleaseState, KeyError> {
        let index = self.index(key)?;
        let slot = &mut self.slots[index];
        if slot.state == ReleaseState::Released {
            return Ok(ReleaseState::Released);
        }
        let owner = match &mut slot.entry {
            Entry::Owned { owner, .. } => owner,
            Entry::CreationFailed(failure) => {
                failure.partial.as_mut().ok_or(KeyError::WrongPhase)?
            }
            Entry::Reserved(_) => return Err(KeyError::WrongPhase),
        };
        slot.state = match release(owner) {
            Ok(ReleaseObservation::Complete) => ReleaseState::Released,
            Ok(ReleaseObservation::Unobserved) => ReleaseState::Unobserved,
            Err(error) => {
                slot.release_error = Some(error);
                ReleaseState::Failed
            }
        };
        Ok(slot.state)
    }
    /// Only fully released original owners may leave the central retention boundary.
    pub fn retire(&mut self, key: &RecoveryKey) -> Result<(), KeyError> {
        let index = self.index(key)?;
        if self.slots[index].state != ReleaseState::Released {
            return Err(KeyError::WrongPhase);
        }
        self.slots.swap_remove(index);
        Ok(())
    }
    /// Includes reservations and input-only creation failures; callers must explicitly recover them.
    pub fn shutdown(self) -> Result<(), Self> {
        if self.slots.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
    pub fn recover_creation_failure(
        &mut self,
        key: &RecoveryKey,
    ) -> Result<CreationFailure<I, O, C>, KeyError> {
        let index = self.index(key)?;
        if !matches!(&self.slots[index].entry, Entry::CreationFailed(failure) if failure.partial.is_none())
        {
            return Err(KeyError::WrongPhase);
        }
        match self.slots.swap_remove(index).entry {
            Entry::CreationFailed(failure) => Ok(failure),
            _ => unreachable!("phase checked"),
        }
    }
    #[cfg(test)]
    pub(super) fn exhaust_generation(&mut self) {
        self.next_generation = u64::MAX;
    }
}
