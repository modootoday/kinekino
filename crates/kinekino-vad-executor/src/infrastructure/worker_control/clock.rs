//! Parent-only counter adapter over the OS counter port (ADR 0008). No Instant
//! internals are read or serialized.
use crate::{
    application::CounterClock,
    domain::{CounterDomain, CounterReading},
};
use kinekino_core::platform::clock::CounterSample;
use kinekino_media_contracts::{
    api_values::DecimalU64,
    errors::{ApiError, ErrorCode},
};

pub(crate) fn reading(sample: CounterSample) -> Result<CounterReading, ApiError> {
    let domain = CounterDomain {
        kind: sample.identity.kind.into(),
        frequency_hz: DecimalU64(sample.identity.frequency_hz),
        parent_pid: sample.identity.parent_pid,
        parent_creation_filetime: DecimalU64(sample.identity.parent_creation),
    };
    domain.validate()?;
    Ok(CounterReading {
        domain,
        ticks: sample.ticks,
    })
}

pub(super) struct ParentQpcClock;
impl CounterClock for ParentQpcClock {
    fn sample(&self) -> Result<CounterReading, ApiError> {
        let sample = kinekino_core::platform::os()
            .counter()
            .sample_current()
            .map_err(|error| {
                ApiError::new(
                    ErrorCode::EngineFailed,
                    format!("Original parent counter observation failed: {error}"),
                )
            })?;
        reading(sample)
    }
}
