use super::carrier::{ClosingHost, Readiness, ReadyForDependencies};
use kinekino_player::{
    application::api::{ApiError, ErrorCode},
    infrastructure::local_host::{LocalHost, LocalHostShutdownStatus, WorkerJoin},
};
use std::{
    io::{Read, Write},
    time::{Duration, Instant},
};

/// PM-selected CLI transport observation policy, not an overall cleanup guarantee.
const HOST_SHUTDOWN_OBSERVATION_BUDGET: Duration = Duration::from_secs(30);
const HOST_SHUTDOWN_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[must_use = "Incomplete execution still owns the original shutdown carrier"]
pub(crate) enum HostRunOutcome<D> {
    Complete {
        result: Result<(), ApiError>,
        local_host_shutdown: Option<LocalHostShutdownStatus>,
    },
    Incomplete(AbnormalHost<D>),
    DependencyIncomplete(super::DependencyIncomplete),
}
#[must_use = "Dependency cleanup is separate from transport readiness"]
pub(crate) enum TransportOutcome<D> {
    Ready(ReadyForDependencies<D>),
    Incomplete(AbnormalHost<D>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IoPhase {
    ReadyWrite,
    ReadyFlush,
    Read,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AbnormalReason {
    ObservationBudgetExpired,
    WorkerPanicked,
}

#[must_use = "Retain until the terminal process boundary; dropping does not prove cleanup"]
pub(crate) struct AbnormalHost<D> {
    retained: RetainedHost<D>,
    reason: AbnormalReason,
}
impl<D> AbnormalHost<D> {
    #[cfg(test)]
    pub(super) fn into_owner(self) -> ClosingHost<D> {
        self.retained.owner
    }
    pub(crate) fn exit_code(&self) -> u8 {
        self.retained.owner.first_error().map_or_else(
            || ApiError::new(ErrorCode::EngineFailed, "Local host shutdown incomplete").exit_code(),
            ApiError::exit_code,
        )
    }
    pub(crate) fn diagnostic(&self, host: &str) -> serde_json::Value {
        serde_json::json!({
            "event":"host_shutdown_incomplete", "host":host,
            "reason":format!("{:?}",self.reason),
            "first_error":self.retained.owner.first_error(),
            "local_host_shutdown":super::carrier::LocalHostShutdownView::from(self.retained.transport),
            "budget_expired":self.retained.budget_expired,
            "cleanup_complete":false, "sdk_cancellation_confirmed":false
        })
    }
}

/// Ready I/O and input run before the transport observation budget starts.
pub(crate) fn serve_transport<D, R: Read, W: Write>(
    local: LocalHost,
    dependency: D,
    input: &mut R,
    output: &mut W,
    ready: &serde_json::Value,
    map_io: impl Fn(IoPhase, std::io::Error) -> ApiError,
    before_close: impl FnOnce(),
) -> TransportOutcome<D> {
    let first_error = transport_input(input, output, ready, map_io);
    before_close();
    let owner = ClosingHost::begin(local, dependency, first_error);
    observe(owner, Instant::now() + HOST_SHUTDOWN_OBSERVATION_BUDGET)
}
fn transport_input<R: Read, W: Write>(
    input: &mut R,
    output: &mut W,
    ready: &serde_json::Value,
    map_io: impl Fn(IoPhase, std::io::Error) -> ApiError,
) -> Option<ApiError> {
    if let Err(error) = writeln!(output, "{ready}") {
        return Some(map_io(IoPhase::ReadyWrite, error));
    }
    if let Err(error) = output.flush() {
        return Some(map_io(IoPhase::ReadyFlush, error));
    }
    let mut bytes = [0u8; 256];
    loop {
        match input.read(&mut bytes) {
            Ok(0) => return None,
            Ok(_) => {}
            Err(error) => return Some(map_io(IoPhase::Read, error)),
        }
    }
}
pub(super) fn observe<D>(mut owner: ClosingHost<D>, deadline: Instant) -> TransportOutcome<D> {
    loop {
        match advance(owner, deadline) {
            DriveOutcome::Ready(ready) => return TransportOutcome::Ready(ready),
            DriveOutcome::Pending(retained) | DriveOutcome::Partial(retained) => {
                if retained.budget_expired {
                    return TransportOutcome::Incomplete(AbnormalHost {
                        retained,
                        reason: AbnormalReason::ObservationBudgetExpired,
                    });
                }
                owner = retained.owner;
                std::thread::sleep(HOST_SHUTDOWN_POLL_INTERVAL);
            }
            DriveOutcome::Panicked(retained) => {
                return TransportOutcome::Incomplete(AbnormalHost {
                    retained,
                    reason: AbnormalReason::WorkerPanicked,
                });
            }
        }
    }
}

/// Cleanup is invoked explicitly by the owning host, never inside this driver.
pub(crate) fn preserve_first_error<T>(
    first: Option<ApiError>,
    cleanup: Result<T, ApiError>,
) -> Result<T, ApiError> {
    preserve_first_error_to(first, cleanup, &mut std::io::stderr().lock())
}
pub(super) fn preserve_first_error_to<T, W: Write>(
    first: Option<ApiError>,
    cleanup: Result<T, ApiError>,
    diagnostic: &mut W,
) -> Result<T, ApiError> {
    match (first, cleanup) {
        (None, result) => result,
        (Some(first), Ok(_)) => Err(first),
        (Some(first), Err(additional)) => {
            let _ = writeln!(
                diagnostic,
                "{}",
                serde_json::json!({"event":"additional_dependency_shutdown_error","error":additional})
            );
            Err(first)
        }
    }
}

#[must_use = "An incomplete outcome still owns the original host and dependency"]
pub(crate) enum DriveOutcome<D> {
    Ready(ReadyForDependencies<D>),
    Pending(RetainedHost<D>),
    Partial(RetainedHost<D>),
    Panicked(RetainedHost<D>),
}
#[must_use = "Preserve this owner after deadline expiry or a worker panic"]
pub(crate) struct RetainedHost<D> {
    pub(crate) owner: ClosingHost<D>,
    pub(crate) transport: LocalHostShutdownStatus,
    pub(crate) budget_expired: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Phase {
    Ready,
    Pending,
    Partial,
    Panicked,
}
pub(super) fn phase(status: LocalHostShutdownStatus) -> Phase {
    if status.pipe == WorkerJoin::Panicked || status.sdk == WorkerJoin::Panicked {
        Phase::Panicked
    } else if status.is_quiescent() {
        Phase::Ready
    } else if status.pipe == WorkerJoin::Joined || status.sdk == WorkerJoin::Joined {
        Phase::Partial
    } else {
        Phase::Pending
    }
}
/// One observation step. No wait loop, SDK cancellation, dependency shutdown or process exit.
/// The deadline reports budget expiry; it never disposes of an incomplete owner.
pub(crate) fn advance<D>(owner: ClosingHost<D>, deadline: Instant) -> DriveOutcome<D> {
    let (owner, transport) = match owner.try_ready() {
        Readiness::Ready(ready) => return DriveOutcome::Ready(ready),
        Readiness::Incomplete(owner, transport) => (owner, transport),
    };
    let phase = phase(transport);
    let retained = RetainedHost {
        owner,
        transport,
        budget_expired: Instant::now() >= deadline,
    };
    match phase {
        Phase::Pending => DriveOutcome::Pending(retained),
        Phase::Partial => DriveOutcome::Partial(retained),
        Phase::Panicked => DriveOutcome::Panicked(retained),
        Phase::Ready => unreachable!("the original Pending owner did not issue Ready"),
    }
}
