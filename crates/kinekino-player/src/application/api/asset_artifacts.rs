use super::{ApiError, ErrorCode, Request};
mod response_schema;
use crate::application::{
    asset_artifacts::{AssetArtifactInspector, InspectAssetArtifact},
    review_source::MutationControl,
};
use serde_json::{Value, json};
struct Observe;
impl MutationControl for Observe {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
pub(super) fn execute(
    request: &Request,
    port: Option<&dyn AssetArtifactInspector>,
) -> Result<Value, ApiError> {
    if request.context.timeout_ms.is_some()
        || request.context.idempotency_key.is_some()
        || request.context.expected_revision.is_some()
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Artifact inspection accepts no mutation/timeout context",
        ));
    }
    let input: InspectAssetArtifact =
        serde_json::from_value(request.payload.clone()).map_err(|_| {
            ApiError::new(
                ErrorCode::InvalidArgument,
                "Strict artifact identity/SHA payload required",
            )
        })?;
    if !crate::domain::identifiers::valid_id(&input.artifact_id)
        || !crate::domain::identifiers::valid_sha256(&input.publication_sha256)
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Exact artifact identity/SHA required",
        ));
    }
    let result = port
        .ok_or_else(|| {
            ApiError::new(
                ErrorCode::Unsupported,
                "Asset artifact inspector unavailable",
            )
        })?
        .inspect_artifact(&input, &Observe)?;
    serde_json::to_value(result).map_err(|_| {
        ApiError::new(
            ErrorCode::Internal,
            "Artifact inspection response encoding failed",
        )
    })
}
pub(super) fn describe(available: bool) -> Value {
    let input = json!({"type":"object","additionalProperties":false,"required":["artifact_id","publication_sha256"],"properties":{"artifact_id":{"type":"string","pattern":"^[A-Za-z0-9_.-]{1,80}$"},"publication_sha256":{"type":"string","pattern":"^[a-f0-9]{64}$"}}});
    let mut context = super::schema::context_schema();
    for field in ["timeout_ms", "idempotency_key", "expected_revision"] {
        context["properties"][field] = json!({"type":"null"});
    }
    json!({"schema_version":"1","command":"asset_artifacts.inspect","status":if available{"supported"}else{"unsupported"},"request":{"type":"object","additionalProperties":false,"required":["context","command","payload"],"properties":{"context":context,"command":{"const":"asset_artifacts.inspect"},"payload":input}},"response":response_schema::response(),"error_codes":ErrorCode::ALL,"constraints":{"backend_available":available,"point_in_time_only":true,"source_admitted":false,"jobs_completion_authority":false,"quality_approved":false,"timeout_supported":false,"in_flight_cancel":false}})
}
pub(super) fn capabilities(value: &mut Value, available: bool) {
    if let Some(commands) = value["commands"].as_array_mut() {
        for row in commands {
            if row["command"] == "asset_artifacts.inspect" {
                row["status"] = json!(if available {
                    "supported"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["asset_artifacts"] = describe(available)["constraints"].clone();
}
