use super::{Shared, capture::Capture, protocol_observer::ProtocolMessage, worker::apply};
use crate::application::{
    api::{ApiError, ErrorCode},
    browser_network::decode,
    browser_provider::*,
};
use std::sync::{Arc, Mutex};

/// Protocol parsing and response decisions run outside native callbacks and without COM in shared state.
pub(super) fn drive(
    capture: &mut Capture,
    shared: &Arc<Mutex<Shared>>,
    ticket: &BrowserTicket,
    native_drained: bool,
) -> Result<(), ApiError> {
    if let Err(error) = capture.drive_protocol() {
        apply(shared, ticket, BrowserEvent::Failed(error))?;
        return Ok(());
    }
    for _ in 0..MAX_COMMANDS {
        let Some(message) = capture.next_protocol() else {
            break;
        };
        let event = match message {
            ProtocolMessage::Event {
                received_unix_ms,
                name,
                session,
                json,
            } => match decode(name, &json) {
                Ok(event) => {
                    // Apply the protocol proof before allowing a paused target to continue.
                    apply(
                        shared,
                        ticket,
                        BrowserEvent::NetworkObservedAt {
                            received_unix_ms,
                            session: session.clone(),
                            event: event.clone(),
                        },
                    )?;
                    if let Err(error) = capture.observe_protocol_target(&session, &event) {
                        apply(shared, ticket, BrowserEvent::Failed(error))?;
                    }
                    continue;
                }
                Err(error) => BrowserEvent::Failed(error),
            },
            ProtocolMessage::Body {
                read,
                json,
                observed_unix_ms,
            } => BrowserEvent::NetworkBody {
                read: *read,
                json,
                observed_unix_ms,
            },
            ProtocolMessage::BodyUnavailable { read } => {
                BrowserEvent::NetworkBodyUnavailable(*read)
            }
        };
        apply(shared, ticket, event)?;
    }
    if native_drained && capture.protocol_body_capacity() {
        let read = {
            let mut state = shared.lock().map_err(|_| {
                ApiError::new(ErrorCode::Internal, "Browser network state unavailable")
            })?;
            super::work(&mut state, ticket)
                .ok()
                .and_then(BrowserWork::take_network_body)
        };
        if let Some(read) = read
            && let Err(error) = capture.read_protocol_body(read)
        {
            apply(shared, ticket, BrowserEvent::Failed(error))?;
        }
    }
    Ok(())
}
