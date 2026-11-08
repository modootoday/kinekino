use super::{ApiError, ErrorCode, Request};
mod schema;
use crate::application::{asset_source_archives::*, review_source::MutationControl};
pub(super) use schema::describe;
use serde_json::{Value, json};
struct Observe;
impl MutationControl for Observe {
    fn checkpoint(&self) -> Result<(), ApiError> {
        Ok(())
    }
}
fn encode(value: impl serde::Serialize) -> Result<Value, ApiError> {
    serde_json::to_value(value).map_err(|_| {
        ApiError::new(
            ErrorCode::Internal,
            "Source archive response encoding failed",
        )
    })
}
pub(super) fn is_command(s: &str) -> bool {
    matches!(
        s,
        "asset_sources.export" | "asset_sources.inspect" | "asset_sources.import"
    )
}
pub(super) fn execute(
    request: &Request,
    port: Option<&dyn AssetSourceArchivePort>,
) -> Result<Value, ApiError> {
    if request.context.timeout_ms.is_some()
        || request.context.idempotency_key.is_some()
        || request.context.expected_revision.is_some()
    {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Source storage accepts no mutation/timeout context",
        ));
    }
    let port = port
        .ok_or_else(|| ApiError::new(ErrorCode::Unsupported, "Source archive host unavailable"))?;
    match request.command.as_str() {
        "asset_sources.export" => {
            let input: ExportAssetSources = serde_json::from_value(request.payload.clone())
                .map_err(|_| {
                    ApiError::new(
                        ErrorCode::InvalidArgument,
                        "Strict source export payload required",
                    )
                })?;
            input.validate()?;
            encode(port.export(&input, &Observe)?)
        }
        "asset_sources.inspect" => {
            let input: InspectAssetSources = serde_json::from_value(request.payload.clone())
                .map_err(|_| {
                    ApiError::new(
                        ErrorCode::InvalidArgument,
                        "Strict source inspect payload required",
                    )
                })?;
            input.validate()?;
            encode(port.inspect(&input, &Observe)?)
        }
        "asset_sources.import" => {
            let input: ImportAssetSources = serde_json::from_value(request.payload.clone())
                .map_err(|_| {
                    ApiError::new(
                        ErrorCode::InvalidArgument,
                        "Strict source import payload required",
                    )
                })?;
            input.validate()?;
            encode(port.import(&input, &Observe)?)
        }
        _ => Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Unknown source archive command",
        )),
    }
}
pub(super) fn capabilities(value: &mut Value, available: bool) {
    if let Some(rows) = value["commands"].as_array_mut() {
        for row in rows {
            if row["command"].as_str().is_some_and(is_command) {
                row["status"] = json!(if available {
                    "experimental"
                } else {
                    "unsupported"
                });
            }
        }
    }
    value["asset_sources"] = json!({"backend_available":available,"maturity":"experimental","explicit_project_pin":true,"arbitrary_paths":false,"jobs_publication":false,"quality_approved":false,"review_admitted":false,"timeout_supported":false,"in_flight_cancel":false});
}
