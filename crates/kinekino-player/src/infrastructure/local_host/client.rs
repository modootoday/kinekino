use super::{TransportOptions, diagnostics, framing, security};
use crate::application::review_source::MutationControl;
use crate::infrastructure::os_policy::ipc;
use crate::{
    application::{
        api::{ApiError, ApplicationClient, ErrorCode, Request, Response},
        local_host::{
            MAX_REQUEST_BYTES, decode_command_response, encode_bounded, response_limit,
            transport_error,
        },
    },
    domain::local_endpoint::LocalEndpoint,
};
#[cfg(test)]
mod controlled_tests;

pub struct EndpointClient {
    endpoint: LocalEndpoint,
    options: TransportOptions,
}
impl EndpointClient {
    pub fn new(endpoint: &str, options: TransportOptions) -> Result<Self, ApiError> {
        Ok(Self {
            endpoint: LocalEndpoint::parse(endpoint)
                .map_err(|m| ApiError::new(ErrorCode::InvalidArgument, m))?,
            options: options.validate()?,
        })
    }
    fn request(&self, request: &Request) -> Result<Response, ApiError> {
        self.request_with_hook(request, || {})
    }
    fn request_with_hook(
        &self,
        request: &Request,
        before_verify: impl FnOnce() + Send + 'static,
    ) -> Result<Response, ApiError> {
        self.request_controlled_with_hook(request, None, before_verify)
    }
    fn request_controlled_with_hook(
        &self,
        request: &Request,
        control: Option<&(dyn MutationControl + Sync)>,
        before_verify: impl FnOnce() + Send + 'static,
    ) -> Result<Response, ApiError> {
        security::require_unimpersonated()
            .map_err(|e| security_failure(e, "require_unimpersonated"))?;
        request.validate()?;
        before_transmission(control)?;
        let bytes = encode_bounded(request, MAX_REQUEST_BYTES)?;
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err(transport_error(ErrorCode::ResourceExhausted, false));
        }
        // A private thread also supports callers already running a Tokio runtime.
        let endpoint = self.endpoint.clone();
        let timeout = self.options.timeout;
        let request_id = request.context.request_id.clone();
        let command = request.command.clone();
        let response_maximum = response_limit(&command);
        std::thread::scope(|scope| {
            std::thread::Builder::new()
                .name("player-endpoint-client".into())
                .spawn_scoped(scope, move || {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .map_err(|_| transport_error(ErrorCode::Internal, false))?;
                    runtime.block_on(async {
                        let mut transmitted = false;
                        let mut phase = "open";
                        let result = tokio::time::timeout(timeout, async {
                            let mut pipe = loop {
                                before_transmission(control)?;
                                match ipc().connect(&endpoint.id()) {
                                    Ok(pipe) => break pipe,
                                    Err(f) if f.source.kind() == std::io::ErrorKind::WouldBlock => {
                                        tokio::time::sleep(std::time::Duration::from_millis(5))
                                            .await
                                    }
                                    Err(f) => {
                                        let e = f.source;
                                        return Err(diagnostics::annotate(
                                            transport_error(
                                                if e.kind() == std::io::ErrorKind::PermissionDenied
                                                {
                                                    ErrorCode::PermissionDenied
                                                } else {
                                                    ErrorCode::NotFound
                                                },
                                                false,
                                            ),
                                            "client",
                                            "open_pipe",
                                            e.raw_os_error(),
                                        ));
                                    }
                                }
                            };
                            phase = "verify_server";
                            before_verify();
                            security::verify(&*pipe, &endpoint)
                                .map_err(|e| security_failure(e, "verify_server"))?;
                            before_transmission(control)?;
                            // Even a partial write can race server dispatch; never retry this exchange.
                            transmitted = true;
                            phase = "write_request";
                            framing::write(&mut pipe, &bytes, MAX_REQUEST_BYTES)
                                .await
                                .map_err(|e| {
                                    diagnostics::annotate(
                                        transport_error(ErrorCode::EngineFailed, true),
                                        "client",
                                        "write_request",
                                        e.raw_os_error(),
                                    )
                                })?;
                            phase = "read_response";
                            let response = framing::read(&mut pipe, response_maximum)
                                .await
                                .map_err(|e| {
                                    diagnostics::annotate(
                                        transport_error(ErrorCode::EngineFailed, true),
                                        "client",
                                        "read_response",
                                        e.raw_os_error(),
                                    )
                                })?;
                            let response =
                                decode_command_response(&response, &request_id, &command)?;
                            // Retain the server handle until all response bytes have been received.
                            let _ = pipe.try_write_now(&[6]);
                            Ok(response)
                        })
                        .await;
                        match result {
                            Ok(result) => result,
                            Err(_) => Err(diagnostics::annotate(
                                transport_error(ErrorCode::Timeout, transmitted),
                                "client_deadline",
                                phase,
                                None,
                            )),
                        }
                    })
                })
                .map_err(|_| transport_error(ErrorCode::ResourceExhausted, false))?
                .join()
                .map_err(|_| transport_error(ErrorCode::Internal, true))?
        })
    }
}
fn before_transmission(control: Option<&(dyn MutationControl + Sync)>) -> Result<(), ApiError> {
    if let Some(control) = control {
        control.checkpoint().map_err(|error| {
            diagnostics::annotate(
                transport_error(error.code, false),
                "client_control",
                "before_request_transmission",
                None,
            )
        })?;
    }
    Ok(())
}
fn security_failure(error: ApiError, operation: &'static str) -> ApiError {
    // Reproduced after server-session expiry; no request was sent or peer accepted.
    // Windows adapter: `GetNamedPipeServerProcessId` + ERROR_PIPE_NOT_CONNECTED (233). The
    // operation name only exists on that adapter, so the code is unambiguous.
    let disconnected = error.details["diagnostic"]["operation"] == "GetNamedPipeServerProcessId"
        && error.details["diagnostic"]["win32_error"] == 233;
    let mut result = transport_error(
        if disconnected {
            ErrorCode::EngineFailed
        } else {
            ErrorCode::PermissionDenied
        },
        false,
    );
    result.details["diagnostic"] = error.details.get("diagnostic").cloned().unwrap_or_else(
        || serde_json::json!({"phase":"peer_identity","operation":operation,"win32_error":null}),
    );
    result
}
impl ApplicationClient for EndpointClient {
    fn execute(&self, request: Request) -> Response {
        self.request(&request)
            .unwrap_or_else(|error| Response::failure(request.context.request_id, error))
    }
    fn execute_controlled(
        &self,
        request: Request,
        control: &(dyn MutationControl + Sync),
    ) -> Response {
        self.request_controlled_with_hook(&request, Some(control), || {})
            .unwrap_or_else(|error| Response::failure(request.context.request_id, error))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::local_host::{LocalHost, TransportDiagnostics};
    use std::{sync::Arc, time::Duration};
    #[test]
    fn identity_classification_keeps_policy_and_access_denials_fail_closed() {
        for (operation, errno) in [
            ("GetNamedPipeServerProcessId", Some(5)),
            ("server_pid_mismatch", None),
            ("server_start_mismatch", None),
            ("server_peer_policy_mismatch", None),
            ("OpenProcess_server", Some(233)),
        ] {
            let error = diagnostics::annotate(
                ApiError::new(ErrorCode::PermissionDenied, "Denied"),
                "peer_identity",
                operation,
                errno,
            );
            let error = security_failure(error, "verify_server");
            assert_eq!(error.code, ErrorCode::PermissionDenied);
            assert_eq!(error.details["delivery"], "not_sent");
            assert!(!error.retryable);
        }
    }
    // Per-message transports (Windows) query the server identity after connect, which fails
    // once the pipe is disconnected (ERROR_PIPE_NOT_CONNECTED) before anything is sent.
    // Connection-bound transports (Unix SO_PEERCRED) keep the identity, so the expiry surfaces
    // at the request write and delivery is conservatively unknown. Both never dispatch.
    #[test]
    fn expired_connected_pipe_keeps_identity_query_cause_and_never_dispatches() {
        use kinekino_core::platform::ipc::MessagePeer;
        let per_message = crate::infrastructure::os_policy::ipc()
            .capabilities()
            .message_peer
            == MessagePeer::PerMessage;
        let trace = TransportDiagnostics::default();
        let host = LocalHost::start_observed(
            Arc::new(crate::application::api::PlayerSdk::default()),
            TransportOptions {
                timeout: Duration::from_millis(50),
            },
            trace.clone(),
        )
        .unwrap();
        let client = EndpointClient::new(host.endpoint(), TransportOptions::default()).unwrap();
        let barrier = trace.clone();
        let error = client
            .request_with_hook(&Request::new("version"), move || {
                assert!(
                    barrier.wait_for("next_instance_ready", 0, Duration::from_secs(1)),
                    "{:?}",
                    barrier.snapshot()
                );
            })
            .unwrap_err();
        eprintln!(
            "expired identity error={error:?}; trace={:?}",
            trace.snapshot()
        );
        assert_eq!(error.code, ErrorCode::EngineFailed);
        let (delivery, operation) = if per_message {
            ("not_sent", "GetNamedPipeServerProcessId")
        } else {
            ("execution_unknown", "write_request")
        };
        assert_eq!(error.details["delivery"], delivery);
        assert_eq!(error.details["diagnostic"]["operation"], operation);
        assert!(error.details["diagnostic"]["win32_error"].is_number());
        assert!(
            !trace
                .snapshot()
                .iter()
                .any(|event| event.operation == "dispatch")
        );
    }
}

#[cfg(test)]
mod export_budget_tests {
    use super::*;
    use crate::application::skip_segments::host;
    use tokio::io::AsyncWriteExt;
    #[test]
    fn command_cap_rejects_oversized_header_without_waiting_for_or_allocating_body() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        runtime.block_on(async {
            for command in host::EXPORT_COMMANDS {
                let maximum = response_limit(command);
                let (mut input, mut output) = tokio::io::duplex(4);
                output.write_u32_le((maximum + 1) as u32).await.unwrap();
                let result = tokio::time::timeout(
                    std::time::Duration::from_secs(1),
                    framing::read(&mut input, maximum),
                )
                .await;
                assert_eq!(
                    result
                        .expect("Header must fail before reading absent body")
                        .unwrap_err()
                        .kind(),
                    std::io::ErrorKind::InvalidData
                );
            }
        });
    }
    #[test]
    fn legacy_frame_limit_stays_four_mib_and_small_valid_frames_remain_readable() {
        assert_eq!(response_limit("version"), 4_194_304);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        runtime.block_on(async {
            let (mut input, mut output) = tokio::io::duplex(8);
            output.write_u32_le(3).await.unwrap();
            output.write_all(b"yes").await.unwrap();
            assert_eq!(
                framing::read(&mut input, response_limit("version"))
                    .await
                    .unwrap(),
                b"yes"
            );
        });
    }
}
