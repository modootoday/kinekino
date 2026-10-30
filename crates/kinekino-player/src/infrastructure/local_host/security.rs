//! Endpoint identity and peer checks over the OS IPC port (ADR 0007). OS-neutral: the adapter
//! owns addresses, ACLs and how a peer identity is derived.
use super::diagnostics;
use crate::{
    application::api::ApiError,
    domain::{local_endpoint::LocalEndpoint, product::PRODUCT},
    infrastructure::os_policy::ipc,
};
use kinekino_core::platform::ipc::{IpcFailure, IpcListener, IpcStream, PeerIdentity};

pub(super) fn denied(failure: &IpcFailure) -> ApiError {
    diagnostics::denied(failure.operation, failure.source.raw_os_error())
}
pub(super) fn endpoint() -> Result<LocalEndpoint, ApiError> {
    let id = ipc()
        .fresh_endpoint(PRODUCT.stable_product_id)
        .map_err(|e| denied(&IpcFailure::new("fresh_endpoint", e)))?;
    LocalEndpoint::from_id(&id).map_err(|_| diagnostics::denied("endpoint_format", None))
}
/// Must run inside a tokio runtime context.
pub(super) fn listen(endpoint: &LocalEndpoint) -> Result<Box<dyn IpcListener>, ApiError> {
    ipc().listen(&endpoint.id()).map_err(|f| {
        diagnostics::annotate(denied(&f), "host", f.operation, f.source.raw_os_error())
    })
}
pub(super) fn require_unimpersonated() -> Result<(), ApiError> {
    ipc().ensure_thread_identity().map_err(|f| denied(&f))
}
/// Client side: the connected server must be the endpoint's process (pid + start) and permitted.
pub(super) fn verify(stream: &dyn IpcStream, endpoint: &LocalEndpoint) -> Result<(), ApiError> {
    let peer = stream.peer().map_err(|f| denied(&f))?;
    if peer.process_id != endpoint.process_id {
        return Err(diagnostics::denied("server_pid_mismatch", None));
    }
    if peer.process_start != endpoint.process_start {
        return Err(diagnostics::denied("server_start_mismatch", None));
    }
    let local = ipc().local_peer().map_err(|f| denied(&f))?;
    if !ipc().permits(&local, &peer.identity) {
        return Err(diagnostics::denied("server_peer_policy_mismatch", None));
    }
    Ok(())
}
pub(super) fn verify_client(stream: &dyn IpcStream) -> Result<PeerIdentity, ApiError> {
    let peer = stream.peer().map_err(|f| denied(&f))?.identity;
    let local = ipc().local_peer().map_err(|f| denied(&f))?;
    if !ipc().permits(&local, &peer) {
        return Err(diagnostics::denied("client_peer_policy_mismatch", None));
    }
    Ok(peer)
}
pub(super) enum MessageFailure {
    Denied,
    RestorationFailed,
}
pub(super) fn verify_message(
    stream: &mut dyn IpcStream,
    peer: &PeerIdentity,
) -> Result<(), MessageFailure> {
    match stream.message_peer() {
        Ok(actual) if ipc().permits(peer, &actual) => Ok(()),
        Ok(_) => Err(MessageFailure::Denied),
        Err(failure) if failure.fatal => Err(MessageFailure::RestorationFailed),
        Err(_) => Err(MessageFailure::Denied),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::api::{ApplicationClient, ErrorCode, Request};
    use crate::infrastructure::local_host::{EndpointClient, TransportOptions};

    #[test]
    fn second_listener_on_the_same_endpoint_is_rejected() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let _entered = runtime.enter();
        let endpoint = endpoint().unwrap();
        let _listener = listen(&endpoint).unwrap();
        assert!(listen(&endpoint).is_err());
    }
    #[test]
    fn wrong_process_and_reused_process_identity_are_rejected_before_transmission() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        for wrong_pid in [false, true] {
            let original = endpoint().unwrap();
            let suffix = original.as_str().rsplit('.').next().unwrap();
            let pid = if wrong_pid {
                original.process_id + 1
            } else {
                original.process_id
            };
            let start = if wrong_pid {
                original.process_start
            } else {
                original.process_start + 1
            };
            let forged = LocalEndpoint::parse(&format!(
                "pipe:{}.v1.{pid}.{start:016x}.{suffix}",
                PRODUCT.stable_product_id
            ))
            .unwrap();
            let mut listener = {
                let _entered = runtime.enter();
                listen(&forged).unwrap()
            };
            let client = EndpointClient::new(forged.as_str(), TransportOptions::default()).unwrap();
            let response = client.execute(Request::new("version"));
            assert_eq!(response.error.unwrap().code, ErrorCode::PermissionDenied);
            runtime.block_on(async {
                use tokio::io::AsyncReadExt;
                let mut stream = listener.accept().await.unwrap();
                let mut bytes = [0u8; 4];
                // The client hung up without sending anything.
                assert_eq!(stream.read(&mut bytes).await.unwrap(), 0);
            });
        }
    }
}
