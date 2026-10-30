//! Parent Rust observations only. Neither a receipt parser nor publication authority.
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeProcessObservation {
    NotStarted,
    Pending,
    Observed {
        child_exit: Option<u32>,
        family_empty: bool,
        receipt_recorded: bool,
        regular_files_eof: bool,
        forced: bool,
    },
}
impl NativeProcessObservation {
    pub fn closed(self) -> bool {
        matches!(
            self,
            Self::NotStarted
                | Self::Observed {
                    child_exit: Some(_),
                    family_empty: true,
                    receipt_recorded: true,
                    regular_files_eof: true,
                    ..
                }
        )
    }
}
pub(crate) struct NativeProcessState(Mutex<NativeProcessObservation>);
impl NativeProcessState {
    pub(crate) fn record(&self, observation: NativeProcessObservation) {
        *self.0.lock().unwrap_or_else(|poison| poison.into_inner()) = observation;
    }
    fn observed(&self) -> NativeProcessObservation {
        *self.0.lock().unwrap_or_else(|poison| poison.into_inner())
    }
}

/// Retains the same state cells registered before original launch, across unwind.
/// Public callers may observe them but cannot register or update arbitrary states.
#[derive(Default)]
pub struct NativeProcessLedger(Mutex<Vec<Arc<NativeProcessState>>>);
impl NativeProcessLedger {
    pub(crate) fn register(&self) -> Arc<NativeProcessState> {
        let state = Arc::new(NativeProcessState(Mutex::new(
            NativeProcessObservation::NotStarted,
        )));
        self.0
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .push(state.clone());
        state
    }
    pub fn observations(&self) -> Vec<NativeProcessObservation> {
        self.0
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .iter()
            .map(|state| state.observed())
            .collect()
    }
}
