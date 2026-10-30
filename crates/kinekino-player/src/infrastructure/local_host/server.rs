use super::{
    TransportDiagnostics, TransportOptions,
    diagnostics::{emit, emit_api},
    executor::Executor,
    framing, security,
};
use crate::{
    application::{
        api::{ApiError, ApplicationClientHandle, ErrorCode, Request, Response},
        local_host::{
            MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES, ResponseFormat, encode_response_at_limit,
            response_limit, transport_error,
        },
    },
    domain::local_endpoint::LocalEndpoint,
};
use kinekino_core::platform::ipc::{IpcStream, PeerIdentity};
use std::sync::mpsc;
use tokio::sync::oneshot;
#[cfg(feature = "validation-export-delivery-loss")]
type DeliveryGate = Option<std::sync::Arc<super::validation::ValidationDeliveryLoss>>;
#[cfg(not(feature = "validation-export-delivery-loss"))]
#[derive(Default)]
struct DeliveryGate;
#[cfg(test)]
#[path = "shutdown_tests.rs"]
mod tests;

/// Owns pipe and SDK workers. Explicit shutdown retains unfinished join ownership.
/// Drop closes admission and signals I/O without waiting for synchronous SDK work.
pub struct LocalHost {
    endpoint: LocalEndpoint,
    stop: Option<oneshot::Sender<()>>,
    executor: Executor,
    sdk_thread: Option<std::thread::JoinHandle<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl LocalHost {
    pub fn start(
        client: ApplicationClientHandle,
        options: TransportOptions,
    ) -> Result<Self, ApiError> {
        Self::start_inner(client, options, None)
    }
    pub fn start_observed(
        client: ApplicationClientHandle,
        options: TransportOptions,
        trace: TransportDiagnostics,
    ) -> Result<Self, ApiError> {
        Self::start_inner(client, options, Some(trace))
    }
    fn start_inner(
        client: ApplicationClientHandle,
        options: TransportOptions,
        trace: Option<TransportDiagnostics>,
    ) -> Result<Self, ApiError> {
        #[cfg(feature = "validation-export-delivery-loss")]
        let gate = None;
        #[cfg(not(feature = "validation-export-delivery-loss"))]
        let gate = DeliveryGate;
        Self::start_configured(client, options, trace, gate)
    }
    #[cfg(feature = "validation-export-delivery-loss")]
    pub fn start_validation_delivery(
        client: ApplicationClientHandle,
        options: TransportOptions,
        gate: std::sync::Arc<super::validation::ValidationDeliveryLoss>,
    ) -> Result<Self, ApiError> {
        Self::start_configured(client, options, None, Some(gate))
    }
    fn start_configured(
        client: ApplicationClientHandle,
        options: TransportOptions,
        trace: Option<TransportDiagnostics>,
        _delivery: DeliveryGate,
    ) -> Result<Self, ApiError> {
        security::require_unimpersonated()?;
        let options = options.validate()?;
        let endpoint = security::endpoint()?;
        let target = endpoint.clone();
        let (stop, mut cancelled) = oneshot::channel();
        let (ready, receiver) = mpsc::sync_channel(1);
        let thread = std::thread::Builder::new().name("player-local-host".into()).spawn(move || {
            let startup = (|| {
                let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|_| transport_error(ErrorCode::Internal,false))?;
                let pipe = { let _entered = runtime.enter(); security::listen(&target)? };
                emit(&trace,"host","initial_instance_created",None);
                let (executor,sdk_thread) = Executor::start_observed(client,trace.clone())?;
                Ok::<_,ApiError>((runtime,pipe,executor,sdk_thread))
            })();
            let (runtime,mut pipe,executor,sdk_thread) = match startup { Ok(parts) => parts, Err(error) => { let _ = ready.send(Err(error)); return; } };
            if ready.send(Ok((executor.clone(),sdk_thread))).is_err() {executor.close();return;}
            runtime.block_on(async {
                loop {
                    tokio::select! {
                        biased;
                        _ = &mut cancelled => break,
                        accepted = pipe.accept() => {
                            let mut stream = match accepted { Ok(stream) => stream, Err(error) => { emit(&trace,"host","connect_failed",Some(&error)); break; } };
                            emit(&trace,"host","connected",None);
                            match security::verify_client(&*stream) { Ok(peer) => {
                                emit(&trace,"host","client_verified",None);
                                tokio::select! {
                                    biased;
                                    _ = &mut cancelled => break,
                                    result = tokio::time::timeout(options.timeout,serve(&mut stream,&executor,&peer,&trace,&_delivery)) => {
                                        match &result {
                                            Err(_) => emit(&trace,"host","session_timeout",None),
                                            Ok(Err(error)) => emit(&trace,"host","session_io_failed",Some(error)),
                                            Ok(Ok(_)) => emit(&trace,"host","session_complete",None),
                                        }
                                        if matches!(result,Ok(Ok(SessionOutcome::StopHost))){break;}
                                    }
                                }
                            }, Err(error) => emit_api(&trace,"host","client_verification_failed",&error) }
                            // The listener already owns a fresh instance; dropping the session
                            // stream cancels its buffered I/O (ADR 0007).
                            drop(stream);
                            emit(&trace,"host","next_instance_ready",None);
                        }
                    }
                }
            });
            pipe.cancel();
            drop(pipe);
            executor.close();
            emit(&trace,"host","stopped",None);
            // At least one server handle owns the name throughout the accept loop.
        }).map_err(|_| transport_error(ErrorCode::Internal,false))?;
        match receiver.recv() {
            Ok(Ok((executor, sdk_thread))) => Ok(Self {
                endpoint,
                stop: Some(stop),
                executor,
                sdk_thread: Some(sdk_thread),
                thread: Some(thread),
            }),
            Ok(Err(error)) => {
                let _ = thread.join();
                Err(error)
            }
            Err(_) => {
                let _ = thread.join();
                Err(transport_error(ErrorCode::Internal, false))
            }
        }
    }
    pub fn endpoint(&self) -> &str {
        self.endpoint.as_str()
    }
    pub fn is_running(&self) -> bool {
        self.thread
            .as_ref()
            .is_some_and(|thread| !thread.is_finished())
    }
    /// Closes dispatch admission before cancelling pipe I/O. Does not cancel accepted SDK work.
    pub fn begin_shutdown(mut self) -> super::PendingLocalHost {
        self.signal_shutdown();
        super::PendingLocalHost::new(
            self.executor.clone(),
            self.thread.take(),
            self.sdk_thread.take(),
        )
    }
    fn signal_shutdown(&mut self) {
        self.executor.close();
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }
}
impl Drop for LocalHost {
    fn drop(&mut self) {
        self.signal_shutdown();
    }
}
enum SessionOutcome {
    Complete,
    StopHost,
}
async fn serve(
    pipe: &mut Box<dyn IpcStream>,
    executor: &Executor,
    peer: &PeerIdentity,
    trace: &Option<TransportDiagnostics>,
    _delivery: &DeliveryGate,
) -> std::io::Result<SessionOutcome> {
    let bytes = framing::read_observed(pipe, MAX_REQUEST_BYTES, |operation| {
        emit(trace, "host", operation, None)
    })
    .await?;
    emit(trace, "host", "verify_message", None);
    match security::verify_message(&mut **pipe, peer) {
        Ok(()) => {}
        Err(security::MessageFailure::Denied) => {
            emit(trace, "host", "message_verification_denied", None);
            return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
        }
        Err(security::MessageFailure::RestorationFailed) => {
            emit(trace, "host", "impersonation_restoration_failed", None);
            return Ok(SessionOutcome::StopHost);
        }
    }
    let mut response_maximum = MAX_RESPONSE_BYTES;
    #[cfg(feature = "validation-export-delivery-loss")]
    let mut original_request = None;
    let response = match serde_json::from_slice::<Request>(&bytes) {
        Ok(request) => {
            response_maximum = response_limit(&request.command);
            match request.validate() {
                Ok(()) => {
                    #[cfg(feature = "validation-export-delivery-loss")]
                    {
                        original_request =
                            Some((request.context.request_id.clone(), request.command.clone()));
                    }
                    emit(trace, "host", "dispatch", None);
                    executor.execute(request).await
                }
                Err(error) => Response::failure(request.context.request_id, error),
            }
        }
        Err(_) => Response::failure(
            "cli".into(),
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Invalid local host request JSON",
            ),
        ),
    };
    #[cfg(feature = "validation-export-delivery-loss")]
    if let (Some(gate), Some((request_id, command))) = (_delivery, original_request)
        && gate.after_response(&request_id, &command, &response)
    {
        return Err(std::io::Error::from(std::io::ErrorKind::ConnectionAborted));
    }
    emit(trace, "host", "write_response", None);
    let encoded = encode_response_at_limit(&response, response_maximum, ResponseFormat::Compact)
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidData))?;
    framing::write(pipe, &encoded.json, response_maximum).await?;
    use tokio::io::AsyncReadExt;
    emit(trace, "host", "read_ack", None);
    let acknowledgement = pipe.read_u8().await?;
    if acknowledgement != 6 {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
    }
    Ok(SessionOutcome::Complete)
}
