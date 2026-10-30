//! Explicit host RPC only. No global capabilities or legacy endpoint fallback.
use super::*;
use serde::Deserialize;
use serde_json::{Value, json};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Submit {
    request: Request,
    submission: Option<SubmissionIdentity>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InspectSubmission {
    submission: SubmissionIdentity,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Operation {
    operation: HostOperationReference,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Retry {
    operation: HostOperationReference,
    expected_generation: u8,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Schema {
    command: String,
}
fn decode<T: serde::de::DeserializeOwned>(payload: Value) -> Result<T, ApiError> {
    serde_json::from_value(payload).map_err(|_| {
        ApiError::new(
            ErrorCode::InvalidArgument,
            "Strict execution host payload required",
        )
    })
}
impl HostExecution {
    pub fn execute_rpc(&self, request: Request) -> Response {
        let id = request.context.request_id.clone();
        match self.rpc(request) {
            Ok(data) => Response {
                api_version: crate::application::api::API_VERSION,
                request_id: id,
                ok: true,
                data: Some(data),
                error: None,
            },
            Err(error) => Response::failure(id, error),
        }
    }
    fn rpc(&self, request: Request) -> Result<Value, ApiError> {
        request.validate()?;
        if request.context.expected_revision.is_some()
            || request.context.idempotency_key.is_some()
            || request.context.timeout_ms.is_some()
        {
            return Err(ApiError::new(
                ErrorCode::InvalidArgument,
                "Execution RPC context cannot change operation context or promise retry safety",
            ));
        }
        match request.command.as_str() {
            "version" => {
                let _: Empty = decode(request.payload)?;
                Ok(
                    json!({"api_version":"1","host_protocol":"execution-host/1","product":crate::domain::product::PRODUCT.stable_product_id}),
                )
            }
            "capabilities" => {
                let _: Empty = decode(request.payload)?;
                Ok(schema::capabilities())
            }
            "schema" => {
                let payload: Schema = decode(request.payload)?;
                schema::command(&payload.command)
            }
            "execution.submit" => {
                let payload: Submit = decode(request.payload)?;
                if payload.request.command.starts_with("execution.") {
                    return Err(ApiError::new(
                        ErrorCode::InvalidArgument,
                        "Nested execution RPC rejected",
                    ));
                }
                if let Some(identity) = payload.submission {
                    return serde_json::to_value(
                        self.submit_correlated(&identity, payload.request)?,
                    )
                    .map_err(|_| internal("Submission receipt serialization unavailable"));
                }
                Ok(
                    json!({"operation":self.submit(payload.request)?,"completion":"admitted","automatic_retry":false}),
                )
            }
            "execution.submission.inspect" => {
                let payload: InspectSubmission = decode(request.payload)?;
                serde_json::to_value(self.inspect_submission(&payload.submission)?)
                    .map_err(|_| internal("Submission receipt serialization unavailable"))
            }
            "execution.list" => {
                let _: Empty = decode(request.payload)?;
                Ok(json!({"operations":self.retained_references()?}))
            }
            "execution.peek" => {
                let payload: Operation = decode(request.payload)?;
                let state = match self.peek(&payload.operation)? {
                    ExecutionSnapshot::Queued => json!({"phase":"queued"}),
                    ExecutionSnapshot::Running {
                        cancellation_requested,
                    } => {
                        json!({"phase":"running","cancellation_requested":cancellation_requested,"receipt_recovery":self.receipt_recovery(&payload.operation)?})
                    }
                    ExecutionSnapshot::Acknowledged => return Err(stale()),
                    ExecutionSnapshot::Terminal(terminal) => terminal_view(&terminal)?,
                };
                Ok(json!({"operation":payload.operation,"state":state}))
            }
            "execution.retry_receipt" => {
                let payload: Retry = decode(request.payload)?;
                let disposition =
                    match self.retry_receipt(&payload.operation, payload.expected_generation)? {
                        CleanupRetryDisposition::Requested => "requested",
                        CleanupRetryDisposition::AlreadyRequested => "already_requested",
                    };
                Ok(
                    json!({"disposition":disposition,"completion":"unobserved","operation_reexecuted":false}),
                )
            }
            "execution.cancel" => {
                let payload: Operation = decode(request.payload)?;
                let disposition = match self.cancel(&payload.operation)? {
                    CancelDisposition::QueuedCancelled => "queued_cancelled",
                    CancelDisposition::Requested => "requested",
                    CancelDisposition::AlreadyAcknowledged => "already_acknowledged",
                };
                Ok(json!({"disposition":disposition}))
            }
            "execution.acknowledge" => {
                let payload: Operation = decode(request.payload)?;
                Ok(
                    json!({"already_acknowledged":self.acknowledge(&payload.operation)?.already_acknowledged}),
                )
            }
            _ => Err(ApiError::new(
                ErrorCode::Unsupported,
                "Command is not an execution host RPC",
            )),
        }
    }
}
fn terminal_view(terminal: &ExecutionTerminal) -> Result<Value, ApiError> {
    let bounded = matches!(terminal.measurement, EncodingMeasurement::Exact(bytes) if bytes <= crate::application::local_host::MAX_RESPONSE_BYTES - 8192);
    let response = if bounded {
        Some(serde_json::to_value(&*terminal.response).map_err(|_| {
            internal("Original terminal serialization failed; retained without ACK")
        })?)
    } else {
        None
    };
    Ok(
        json!({"phase":"terminal","envelope_valid":terminal.validate_response().is_ok(),"response":response,"response_available":bounded,"original_retained":true,"cancellation_requested":terminal.cancellation_requested,"queued_cancelled":terminal.queued_cancelled}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Wake;
    impl CompletionNotifier for Wake {
        fn notify(&self) -> Result<(), ApiError> {
            Ok(())
        }
    }
    #[test]
    fn discovery_is_host_local_and_rejects_unknown_schema_fields() {
        let host = HostExecution::start(
            "e".repeat(32),
            Arc::new(crate::application::api::PlayerSdk::default()),
            ExecutionBudget {
                max_operations: 1,
                max_request_bytes: 4096,
                max_total_request_bytes: 4096,
                max_terminal_bytes: 4096,
                max_total_terminal_bytes: 4096,
            },
            Arc::new(Wake),
        )
        .unwrap();
        assert_eq!(
            schema::capabilities()["commands"].as_array().unwrap().len(),
            10
        );
        for command in schema::COMMANDS {
            assert_eq!(schema::command(command).unwrap()["command"], *command);
        }
        assert!(!crate::application::api::capabilities::COMMANDS.contains(&"execution.submit"));
        let mut bad = Request::new("schema");
        bad.payload = json!({"command":"execution.peek","extra":true});
        assert!(!host.execute_rpc(bad).ok);
        let mut unknown = Request::new("schema");
        unknown.payload = json!({"command":"production.capture_frame"});
        assert!(!host.execute_rpc(unknown).ok);
        let cutoff = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !matches!(
            host.poll_close().unwrap(),
            ExecutionCloseProgress::Closed(_)
        ) {
            assert!(std::time::Instant::now() < cutoff);
            std::thread::yield_now();
        }
    }
    #[test]
    fn terminal_fault_or_oversize_keeps_original_without_copy_or_ack() {
        let request = Request::new("version");
        let mut terminal = ExecutionTerminal {
            request: Arc::new(request.clone()),
            response: Arc::new(Response::failure(
                request.context.request_id,
                ApiError::new(ErrorCode::EngineFailed, "original retained failure"),
            )),
            cancellation_requested: false,
            queued_cancelled: false,
            measurement: EncodingMeasurement::SerializationFailed,
        };
        for measurement in [
            EncodingMeasurement::SerializationFailed,
            EncodingMeasurement::LowerBound(1),
            EncodingMeasurement::Exact(crate::application::local_host::MAX_RESPONSE_BYTES),
        ] {
            terminal.measurement = measurement;
            let view = terminal_view(&terminal).unwrap();
            assert_eq!(view["response_available"], false);
            assert_eq!(view["response"], Value::Null);
            assert_eq!(view["original_retained"], true);
            assert_eq!(
                terminal.response.error.as_ref().unwrap().message,
                "original retained failure"
            );
        }
        terminal.measurement = EncodingMeasurement::Exact(1024);
        let view = terminal_view(&terminal).unwrap();
        assert_eq!(view["response"]["ok"], false);
        assert_eq!(view["envelope_valid"], true);
        Arc::get_mut(&mut terminal.response).unwrap().request_id = "wrong-original-envelope".into();
        let view = terminal_view(&terminal).unwrap();
        assert_eq!(view["envelope_valid"], false);
        assert_eq!(view["response"]["request_id"], "wrong-original-envelope");
    }
}
