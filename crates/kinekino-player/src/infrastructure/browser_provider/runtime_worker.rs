use super::{Shared, capture::Capture, worker::apply};
use crate::application::{
    api::{ApiError, ErrorCode},
    browser_provider::*,
};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

pub(super) fn drive(
    capture: &Capture,
    shared: &Arc<Mutex<Shared>>,
    ticket: &BrowserTicket,
    native_drained: bool,
) -> Result<(), ApiError> {
    if !native_drained {
        return Ok(());
    }
    let request = {
        let mut state = shared
            .lock()
            .map_err(|_| ApiError::new(ErrorCode::Internal, "Browser runtime state unavailable"))?;
        super::work(&mut state, ticket)
            .ok()
            .and_then(|work| work.take_frame_runtime_probe(Instant::now()))
    };
    if let Some(request) = request
        && let Err(error) = capture.observe_frame_runtime(request)
        && error.code != ErrorCode::RevisionConflict
    {
        apply(shared, ticket, BrowserEvent::Failed(error))?;
    }
    Ok(())
}
