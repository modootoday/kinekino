use super::*;
use std::time::Duration;
struct Original {
    state: Cell<u8>,
}
impl WorkerControl for Original {
    fn checkpoint(&self) -> Result<WorkerControlState, ApiError> {
        match self.state.get() {
            0 => Ok(WorkerControlState::Continue),
            1 => Ok(WorkerControlState::CancellationRequested),
            _ => Err(ApiError::new(
                ErrorCode::PermissionDenied,
                "original marker IO fault",
            )),
        }
    }
}
#[test]
fn fallible_signal_error_does_not_become_cached_cancellation() {
    let original = Original {
        state: Cell::new(0),
    };
    let cutoff = Instant::now() + Duration::from_secs(1);
    let control = WorkerAnalysisControl::new(&original, cutoff).unwrap();
    original.state.set(2);
    let error = control.checkpoint().unwrap_err();
    assert_eq!(error.code, ErrorCode::PermissionDenied);
    assert_eq!(error.message, "original marker IO fault");
    assert!(!control.cancellation_requested());
    assert_eq!(control.deadline(), cutoff);
}
#[test]
fn genuine_cancel_observation_and_fixed_cutoff_are_preserved() {
    let original = Original {
        state: Cell::new(0),
    };
    let cutoff = Instant::now() + Duration::from_secs(1);
    let control = WorkerAnalysisControl::new(&original, cutoff).unwrap();
    original.state.set(1);
    assert!(control.checkpoint().unwrap());
    assert!(control.cancellation_requested());
    assert_eq!(control.deadline(), cutoff);
}
#[test]
fn expired_original_projection_is_not_restarted() {
    let original = Original {
        state: Cell::new(0),
    };
    assert_eq!(
        WorkerAnalysisControl::new(&original, Instant::now())
            .err()
            .unwrap()
            .code,
        ErrorCode::Timeout
    );
}
