use super::{SkipHostApi, SkipHostRegistry, contracts::*};
use crate::{
    application::api::{ApiError, ErrorCode, Request},
    domain::api_values::DecimalU64,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
fn decode<T: DeserializeOwned>(request: &Request) -> Result<T, ApiError> {
    serde_json::from_value(request.payload.clone())
        .map_err(|_| invalid("Invalid strict skip-session payload"))
}
fn encode(value: impl Serialize) -> Result<Value, ApiError> {
    serde_json::to_value(value)
        .map_err(|_| ApiError::new(ErrorCode::Internal, "Cannot encode skip session response"))
}
fn invalid(message: &'static str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}
pub(super) fn execute(registry: &SkipHostRegistry, request: &Request) -> Result<Value, ApiError> {
    if super::is_export_command(&request.command) {
        return execute_export(registry, request);
    }
    if request.context.idempotency_key.is_some() || request.context.timeout_ms.is_some() {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Skip session idempotency and operation timeout are unavailable",
        ));
    }
    let mutation = matches!(
        request.command.as_str(),
        "segments.edit"
            | "segments.accept_skip"
            | "segments.set_skip_policy"
            | "segments.skip"
            | "segments.override"
            | "segments.close"
    );
    let revision = if mutation {
        let value = request
            .context
            .expected_revision
            .as_ref()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .ok_or_else(|| invalid("Mutation requires current skip session expected_revision"))?;
        DecimalU64(value)
    } else {
        if request.context.expected_revision.is_some() {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Skip reads/open/evaluate do not accept expected_revision",
            ));
        }
        DecimalU64(0)
    };
    if !registry.capability().sessions {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Skip host has closed",
        ));
    }
    match request.command.as_str() {
        "segments.open" => encode(registry.open(decode::<OpenSkipSession>(request)?)?),
        "segments.inspect" => encode(registry.inspect(decode::<SessionRef>(request)?)?),
        "segments.edit" => {
            encode(registry.edit(decode::<SessionRequest<EditInput>>(request)?, revision)?)
        }
        "segments.accept_skip" => {
            encode(registry.verify(decode::<SessionRequest<VerifyInput>>(request)?, revision)?)
        }
        "segments.set_skip_policy" => {
            encode(registry.set_policy(decode::<SessionRequest<PolicyInput>>(request)?, revision)?)
        }
        "segments.evaluate" => {
            encode(registry.evaluate(decode::<SessionRequest<EvaluateInput>>(request)?)?)
        }
        "segments.skip" => {
            encode(registry.skip(decode::<SessionRequest<SkipInput>>(request)?, revision)?)
        }
        "segments.override" => encode(
            registry
                .override_session(decode::<SessionRequest<OverrideInput>>(request)?, revision)?,
        ),
        "segments.close" => encode(registry.close(decode::<SessionRef>(request)?, revision)?),
        _ => Err(invalid("Unknown skip session command")),
    }
}

enum ExportRequest {
    Submit(SkipExportRequest),
    List(ListSkipExports),
    Peek(ReadSkipExport),
    Consume(ReadSkipExport),
}
fn export_session(reference: &SessionRef) -> Result<(), ApiError> {
    use crate::domain::identifiers::{valid_id, valid_sha256};
    if !valid_id(&reference.skip_session_id)
        || reference.binding.engine_instance_id.0 == 0
        || !valid_sha256(&reference.binding.resolution_artifact_id)
    {
        return Err(invalid("Invalid bounded export session binding"));
    }
    Ok(())
}
fn export_origin(origin: &SkipExportOrigin) -> Result<(), ApiError> {
    export_session(&origin.session)?;
    if origin.pass_id.0 == 0
        || origin.request_revision.0 == 0
        || !crate::domain::identifiers::valid_id(&origin.segment_id)
    {
        return Err(invalid("Invalid bounded export origin"));
    }
    Ok(())
}
fn export_encode(value: &impl Serialize) -> Result<Value, ApiError> {
    let bytes = crate::application::playback_session::v1::encode_native_export_v1(value)?;
    serde_json::from_slice(&bytes).map_err(|_| {
        ApiError::new(
            ErrorCode::Internal,
            "Cannot decode bounded export projection",
        )
    })
}
pub(crate) struct ExportPreflight {
    decoded: ExportRequest,
    revision: DecimalU64,
}
pub(crate) fn export_preflight(request: &Request) -> Result<ExportPreflight, ApiError> {
    // Shape and the SDK's strict codecs precede context and feature availability.
    let decoded = match request.command.as_str() {
        super::SKIP_EXPORT => {
            let input = decode::<SkipExportRequest>(request)?;
            export_session(&input.session)?;
            if !crate::domain::identifiers::valid_id(&input.input.segment_id) {
                return Err(invalid("Invalid export segment identity"));
            }
            ExportRequest::Submit(input)
        }
        super::EXPORT_LIST => {
            let input = decode::<ListSkipExports>(request)?;
            export_origin(&input.origin)?;
            ExportRequest::List(input)
        }
        super::EXPORT_PEEK | super::EXPORT_CONSUME => {
            let input = decode::<ReadSkipExport>(request)?;
            export_origin(&input.origin)?;
            if request.command == super::EXPORT_PEEK {
                ExportRequest::Peek(input)
            } else {
                ExportRequest::Consume(input)
            }
        }
        _ => return Err(invalid("Unknown native export command")),
    };
    let revision = if matches!(&decoded, ExportRequest::Submit(_)) {
        DecimalU64(
            request
                .context
                .expected_revision
                .as_ref()
                .filter(|value| {
                    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
                })
                .and_then(|value| value.parse::<u64>().ok())
                .filter(|value| *value > 0)
                .ok_or_else(|| {
                    invalid("Export submit requires current positive expected_revision")
                })?,
        )
    } else {
        if request.context.expected_revision.is_some() {
            return Err(ApiError::new(
                ErrorCode::Unsupported,
                "Export list/read/consume do not accept expected_revision",
            ));
        }
        DecimalU64(0)
    };
    if request.context.idempotency_key.is_some() || request.context.timeout_ms.is_some() {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Native export idempotency and operation timeout unavailable",
        ));
    }
    Ok(ExportPreflight { decoded, revision })
}
fn execute_export(registry: &SkipHostRegistry, request: &Request) -> Result<Value, ApiError> {
    let ExportPreflight { decoded, revision } = export_preflight(request)?;
    if !registry.export_capability().available {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Native export wire integration is unavailable",
        ));
    }
    match decoded {
        ExportRequest::Submit(input) => {
            export_encode(&registry.skip_native_export_call(input, revision)?)
        }
        ExportRequest::List(input) => export_encode(&registry.list_native_exports(input)?),
        ExportRequest::Peek(input) => export_encode(&registry.peek_native_export(input)?),
        ExportRequest::Consume(input) => export_encode(&registry.consume_native_export(input)?),
    }
}
