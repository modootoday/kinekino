use crate::application::api::{ApiError, ErrorCode};
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicU64, Ordering},
};
const CLOSED: u64 = 1 << 63;
const COUNT: u64 = (1 << 16) - 1;
const STEP: u64 = 1 << 16;
const GENERATION: u64 = !(CLOSED | COUNT);

pub(in crate::infrastructure::mpv) struct Fence {
    word: AtomicU64,
    drain: Mutex<()>,
    changed: Condvar,
}
impl Fence {
    #[cfg(test)]
    pub(super) fn exhaust_generation_for_test(&self) {
        self.word.store(GENERATION, Ordering::Release);
    }
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            word: AtomicU64::new(STEP),
            drain: Mutex::new(()),
            changed: Condvar::new(),
        })
    }
    pub fn generation(&self) -> u64 {
        self.word.load(Ordering::Acquire) & GENERATION
    }
    pub fn is_closed(&self) -> bool {
        self.word.load(Ordering::Acquire) & CLOSED != 0
    }
    pub fn validate(&self, expected: u64) -> Result<(), ApiError> {
        let word = self.word.load(Ordering::Acquire);
        if word & CLOSED != 0 {
            return Err(cancelled());
        }
        if word & GENERATION != expected {
            return Err(ApiError::new(
                ErrorCode::RevisionConflict,
                "Native fence generation changed",
            ));
        }
        Ok(())
    }
    pub fn advance(&self) -> Result<u64, ApiError> {
        let mut old = self.word.load(Ordering::Acquire);
        loop {
            if old & CLOSED != 0 {
                return Err(cancelled());
            }
            let generation = old & GENERATION;
            if generation == GENERATION {
                self.close();
                return Err(ApiError::new(
                    ErrorCode::ResourceExhausted,
                    "Native fence generation exhausted",
                ));
            }
            let next = (old & COUNT) | (generation + STEP);
            match self
                .word
                .compare_exchange_weak(old, next, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => return Ok(next & GENERATION),
                Err(actual) => old = actual,
            }
        }
    }
    /// Retire only the original generation; a late old connection cannot advance a newer one.
    pub fn advance_if(&self, expected: u64) -> Result<Option<u64>, ApiError> {
        let mut old = self.word.load(Ordering::Acquire);
        loop {
            if old & CLOSED != 0 {
                return Err(cancelled());
            }
            if old & GENERATION != expected {
                return Ok(None);
            }
            let exhausted = old & GENERATION == GENERATION;
            let next = if exhausted {
                old | CLOSED
            } else {
                (old & COUNT) | ((old & GENERATION) + STEP)
            };
            match self
                .word
                .compare_exchange_weak(old, next, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => {
                    return if exhausted {
                        Err(ApiError::new(
                            ErrorCode::ResourceExhausted,
                            "Native fence generation exhausted",
                        ))
                    } else {
                        Ok(Some(next & GENERATION))
                    };
                }
                Err(actual) => old = actual,
            }
        }
    }
    /// Linearization point; preserves every already committed original permit.
    pub fn close(&self) {
        self.word.fetch_or(CLOSED, Ordering::AcqRel);
    }
    pub fn commit(self: &Arc<Self>, expected: u64) -> Result<WritePermit, ApiError> {
        let mut old = self.word.load(Ordering::Acquire);
        loop {
            if old & CLOSED != 0 {
                return Err(cancelled());
            }
            if old & GENERATION != expected {
                return Err(ApiError::new(
                    ErrorCode::RevisionConflict,
                    "Native fence generation changed",
                ));
            }
            if old & COUNT == COUNT {
                return Err(ApiError::new(
                    ErrorCode::ResourceExhausted,
                    "Native in-flight writes exhausted",
                ));
            }
            match self
                .word
                .compare_exchange_weak(old, old + 1, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => return Ok(WritePermit(self.clone())),
                Err(actual) => old = actual,
            }
        }
    }
    pub fn in_flight(&self) -> u64 {
        self.word.load(Ordering::Acquire) & COUNT
    }
    /// Cleanup-only synchronous wait, never called from a callback or while holding Active.
    pub fn close_and_drain(&self) -> Result<(), ApiError> {
        self.close();
        let mut lock = self.drain.lock().map_err(|_| poisoned())?;
        while self.in_flight() != 0 {
            lock = self.changed.wait(lock).map_err(|_| poisoned())?;
        }
        Ok(())
    }
}
pub(in crate::infrastructure::mpv) struct WritePermit(Arc<Fence>);
impl Drop for WritePermit {
    fn drop(&mut self) {
        // Recover the lock for cleanup: poisoning must not leak a committed count.
        let _lock = self
            .0
            .drain
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        self.0.word.fetch_sub(1, Ordering::AcqRel);
        self.0.changed.notify_all();
    }
}
fn cancelled() -> ApiError {
    ApiError::new(ErrorCode::Cancelled, "Native write admission revoked")
}
fn poisoned() -> ApiError {
    ApiError::new(ErrorCode::Internal, "Native write drain poisoned")
}
