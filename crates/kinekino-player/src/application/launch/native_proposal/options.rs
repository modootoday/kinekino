use crate::application::api::{ApiError, ErrorCode};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeProposalLaunchOptions {
    pub request_path: PathBuf,
    pub proposal_root: PathBuf,
}

pub fn native_proposal_options(
    args: &[String],
) -> Result<Option<NativeProposalLaunchOptions>, ApiError> {
    let Some(options) = parse_options(args)? else {
        return Ok(None);
    };
    if !crate::infrastructure::os_policy::native_proposal_window_available() {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Native proposal windows are unavailable in this build",
        ));
    }
    Ok(Some(options))
}

fn invalid(message: impl Into<String>) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}

fn parse_options(args: &[String]) -> Result<Option<NativeProposalLaunchOptions>, ApiError> {
    let option = "--native-proposal-request";
    let mut selected = None;
    let mut index = 0;
    while index < args.len() {
        let value = if args[index] == option {
            index += 1;
            Some(
                args.get(index)
                    .ok_or_else(|| {
                        invalid("Native proposal request requires an absolute file path")
                    })?
                    .as_str(),
            )
        } else {
            args[index].strip_prefix("--native-proposal-request=")
        };
        if let Some(value) = value {
            if selected.is_some() || value.is_empty() || value.starts_with("--") {
                return Err(invalid(
                    "Native proposal request must have one explicit file path",
                ));
            }
            let path = PathBuf::from(value);
            if !path.is_absolute() || path.file_name().is_none() {
                return Err(invalid(
                    "Native proposal request requires an absolute file path",
                ));
            }
            selected = Some(path);
        }
        index += 1;
    }
    let Some(request_path) = selected else {
        return Ok(None);
    };
    if args.iter().any(|arg| {
        matches!(arg.as_str(), "--ui-demo" | "--smoke-test" | "--start-url")
            || arg.starts_with("--start-url=")
    }) {
        return Err(invalid(
            "Native proposal launch cannot mix with preview or browser startup",
        ));
    }
    let proposal_root = super::super::proposal_root(args)
        .map_err(|error| invalid(error.to_string()))?
        .ok_or_else(|| invalid("Native proposal launch requires an explicit --proposal-root"))?;
    Ok(Some(NativeProposalLaunchOptions {
        request_path,
        proposal_root,
    }))
}

#[cfg(test)]
mod tests;
