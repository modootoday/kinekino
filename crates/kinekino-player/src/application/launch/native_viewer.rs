//! Explicit local experiment selection. It carries no media, selector, or window authority.
use crate::application::api::{ApiError, ErrorCode};
mod project;
pub(super) use project::parse_project_selection;
pub use project::project_selection;

pub(super) fn available() -> bool {
    crate::infrastructure::os_policy::native_proposal_window_available()
}

pub fn native_viewer_requested(args: &[String]) -> Result<bool, ApiError> {
    let selected = parse_selection(args)?;
    if selected && !available() {
        return Err(ApiError::new(
            ErrorCode::Unsupported,
            "Native Viewer is unavailable in this build",
        ));
    }
    Ok(selected)
}
fn parse_selection(args: &[String]) -> Result<bool, ApiError> {
    let mut selected = false;
    for arg in args {
        if arg == "--native-viewer" {
            if selected {
                return Err(invalid("Native Viewer may be selected once"));
            }
            selected = true;
        } else if arg.starts_with("--native-viewer=") {
            return Err(invalid("Native Viewer is a flag without a value"));
        }
    }
    if selected
        && args.iter().any(|arg| {
            matches!(
                arg.as_str(),
                "--ui-demo"
                    | "--smoke-test"
                    | "--start-url"
                    | "--native-proposal-request"
                    | "--validation-position"
            ) || arg.starts_with("--start-url=")
                || arg.starts_with("--native-proposal-request=")
                || arg.starts_with("--validation-position=")
        })
    {
        return Err(invalid(
            "Native Viewer cannot mix with preview, proposal, browser startup, or unsupported placement options",
        ));
    }
    Ok(selected)
}
fn invalid(message: &'static str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).into()).collect()
    }
    #[test]
    fn explicit_selection_preserves_default_and_configured_storage() {
        assert!(!parse_selection(&args(&[])).unwrap());
        assert!(
            parse_selection(&args(&[
                "--native-viewer",
                "--record-validation",
                "--review-root=C:/review"
            ]))
            .unwrap()
        );
        assert_eq!(
            super::super::desktop_mode(&args(&[])).unwrap(),
            if available() {
                super::super::DesktopMode::NativeViewer
            } else {
                super::super::DesktopMode::Player
            }
        );
    }
    #[test]
    fn duplicate_or_value_selection_is_rejected() {
        for values in [
            vec!["--native-viewer", "--native-viewer"],
            vec!["--native-viewer=false"],
            vec!["--native-viewer="],
        ] {
            assert!(parse_selection(&args(&values)).is_err());
        }
    }
    #[test]
    fn other_execution_intents_are_not_silently_dropped() {
        for other in [
            "--ui-demo",
            "--smoke-test",
            "--start-url=https://example.test",
            "--native-proposal-request=C:/request.json",
            "--validation-position=0,0",
        ] {
            assert!(parse_selection(&args(&["--native-viewer", other])).is_err());
        }
    }
    #[test]
    fn explicit_build_support_is_preserved() {
        let result = native_viewer_requested(&args(&["--native-viewer"]));
        if crate::infrastructure::os_policy::native_proposal_window_available() {
            assert!(result.unwrap());
            assert_eq!(
                super::super::desktop_mode(&args(&["--native-viewer"])).unwrap(),
                super::super::DesktopMode::NativeViewer
            );
        } else {
            assert_eq!(result.unwrap_err().code, ErrorCode::Unsupported);
        }
    }
}
