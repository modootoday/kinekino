//! Read-only actual creator/liveness/counter checks over the OS counter port
//! (ADR 0008). DTO fields are comparison inputs.
use super::{changed, io};
use kinekino_core::platform::clock::{CounterSample, LiveCreator};
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};
use kinekino_vad_executor::{
    application::CounterClock,
    domain::{CounterDomain, CounterReading},
};
use std::cell::Cell;

pub struct VerifiedParentClock {
    parent: Box<dyn LiveCreator>,
    expected: CounterDomain,
    last_ticks: Cell<Option<u64>>,
}
impl VerifiedParentClock {
    pub fn connect(expected: &CounterDomain) -> Result<Self, ApiError> {
        expected.validate()?;
        let parent = kinekino_core::platform::os()
            .counter()
            .open_creator()
            .map_err(io)?;
        if parent.pid() == 0 || parent.pid() != expected.parent_pid {
            return Err(changed("Worker creator differs from original parent"));
        }
        let clock = Self {
            parent,
            expected: expected.clone(),
            last_ticks: Cell::new(None),
        };
        clock.sample()?;
        Ok(clock)
    }
}
fn domain(sample: &CounterSample) -> CounterDomain {
    CounterDomain {
        kind: sample.identity.kind.into(),
        frequency_hz: DecimalU64(sample.identity.frequency_hz),
        parent_pid: sample.identity.parent_pid,
        parent_creation_filetime: DecimalU64(sample.identity.parent_creation),
    }
}
impl CounterClock for VerifiedParentClock {
    fn sample(&self) -> Result<CounterReading, ApiError> {
        let Some(sample) = self.parent.sample().map_err(io)? else {
            return Err(changed("Original parent is no longer live"));
        };
        let domain = domain(&sample);
        if domain.frequency_hz.0 == 0 {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Child counter range unsupported",
            ));
        }
        if domain != self.expected {
            return Err(changed(
                "Observed live parent identity or frequency differs",
            ));
        }
        if self
            .last_ticks
            .get()
            .is_some_and(|last| sample.ticks < last)
        {
            return Err(changed("Observed child counter regressed"));
        }
        self.last_ticks.set(Some(sample.ticks));
        Ok(CounterReading {
            domain,
            ticks: sample.ticks,
        })
    }
}
