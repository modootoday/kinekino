pub mod browser_provider;
mod historical;
pub mod native_playback;
pub mod native_proposal;
pub mod native_viewer;
pub use historical::{HistoricalVerifierConfig, historical_verifier};

/// Explicit local validation configuration; never populated from browser messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopMode {
    Player,
    DesignPreview,
    NativeProposal,
    NativeViewer,
}

#[derive(Debug)]
pub struct DesktopLaunchPlan {
    pub mode: DesktopMode,
    pub project: Option<crate::application::projects::InspectProject>,
}

pub fn desktop_launch_plan(
    args: &[String],
) -> Result<DesktopLaunchPlan, crate::application::api::ApiError> {
    use crate::application::api::{ApiError, ErrorCode};
    let native = native_viewer::native_viewer_requested(args)?;
    let proposal = native_proposal::native_proposal_options(args)?.is_some();
    let preview = args.iter().any(|arg| arg == "--ui-demo");
    let browser = args.iter().any(|arg| {
        matches!(arg.as_str(), "--smoke-test" | "--start-url") || arg.starts_with("--start-url=")
    });
    if preview && browser {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "design preview cannot run browser startup or navigation",
        ));
    }
    let placement = args
        .iter()
        .any(|arg| arg == "--validation-position" || arg.starts_with("--validation-position="));
    let mode = if native {
        DesktopMode::NativeViewer
    } else if proposal {
        DesktopMode::NativeProposal
    } else if preview {
        DesktopMode::DesignPreview
    } else if browser || placement || !native_viewer::available() {
        DesktopMode::Player
    } else {
        DesktopMode::NativeViewer
    };
    let project = native_viewer::parse_project_selection(args)?;
    if project.is_some() && mode != DesktopMode::NativeViewer {
        return Err(ApiError::new(
            ErrorCode::InvalidArgument,
            "Project selection requires Native Viewer",
        ));
    }
    Ok(DesktopLaunchPlan { mode, project })
}

pub fn desktop_mode(args: &[String]) -> anyhow::Result<DesktopMode> {
    desktop_launch_plan(args)
        .map(|plan| plan.mode)
        .map_err(|error| anyhow::anyhow!("{:?}: {}", error.code, error.message))
}

pub fn start_url(args: &[String]) -> String {
    args.iter()
        .find_map(|a| a.strip_prefix("--start-url="))
        .unwrap_or(if args.iter().any(|a| a == "--smoke-test") {
            "https://catalog.example.test/방영중"
        } else {
            "about:blank"
        })
        .to_owned()
}

/// A local startup option only; browser messages cannot choose review storage.
pub fn review_root(args: &[String]) -> anyhow::Result<Option<std::path::PathBuf>> {
    storage_root(args, "--review-root")
}
pub fn proposal_root(args: &[String]) -> anyhow::Result<Option<std::path::PathBuf>> {
    storage_root(args, "--proposal-root")
}
pub fn project_root(args: &[String]) -> anyhow::Result<Option<std::path::PathBuf>> {
    storage_root(args, "--project-root")
}
/// Explicit local job history selection; absence never chooses a default store.
pub fn job_root(args: &[String]) -> anyhow::Result<Option<std::path::PathBuf>> {
    storage_root(args, "--job-root")
}
fn storage_root(args: &[String], option: &str) -> anyhow::Result<Option<std::path::PathBuf>> {
    let mut selected = None;
    let mut index = 0;
    while index < args.len() {
        let value = if args[index] == option {
            index += 1;
            Some(
                args.get(index)
                    .ok_or_else(|| anyhow::anyhow!("{option} requires an absolute directory"))?
                    .as_str(),
            )
        } else {
            args[index].strip_prefix(&format!("{option}="))
        };
        if let Some(value) = value {
            anyhow::ensure!(selected.is_none(), "{option} may be provided once");
            let root = std::path::PathBuf::from(value);
            anyhow::ensure!(
                root.is_absolute(),
                "{option} requires an absolute directory"
            );
            anyhow::ensure!(
                !args.iter().any(|arg| arg == "--ui-demo"),
                "{option} storage is unavailable in design preview mode"
            );
            selected = Some(root);
        }
        index += 1;
    }
    Ok(selected)
}

pub fn validation_position(args: &[String]) -> anyhow::Result<Option<[f32; 2]>> {
    let Some(value) = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--validation-position="))
    else {
        return Ok(None);
    };
    anyhow::ensure!(
        args.iter().any(|a| a == "--record-validation"),
        "validation position requires recording"
    );
    let (x, y) = value
        .split_once(',')
        .ok_or_else(|| anyhow::anyhow!("position must be x,y"))?;
    let position = [x.parse::<f32>()?, y.parse::<f32>()?];
    anyhow::ensure!(
        position.iter().all(|n| n.is_finite() && n.abs() <= 32768.),
        "position outside supported desktop range"
    );
    Ok(Some(position))
}

#[cfg(test)]
mod tests {
    #[test]
    fn job_root_reuses_explicit_local_storage_selection() {
        let args = |values: &[&str]| values.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(super::job_root(&args(&[])).unwrap(), None);
        let absolute = std::env::temp_dir().to_string_lossy().into_owned();
        assert_eq!(
            super::job_root(&args(&["--job-root", &absolute])).unwrap(),
            Some(absolute.clone().into())
        );
        assert_eq!(
            super::job_root(&args(&[&format!("--job-root={absolute}")])).unwrap(),
            Some(absolute.clone().into())
        );
        for invalid in [
            vec!["--job-root"],
            vec!["--job-root="],
            vec!["--job-root", "relative"],
            vec!["--job-root", &absolute, "--job-root", &absolute],
            vec!["--ui-demo", "--job-root", &absolute],
        ] {
            assert!(super::job_root(&args(&invalid)).is_err());
        }
    }
    use super::*;
    #[test]
    fn review_storage_is_explicit_and_cannot_be_silently_ignored_or_ambiguous() {
        let args = |values: &[&str]| values.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(review_root(&args(&[])).unwrap(), None);
        let absolute = std::env::temp_dir().to_string_lossy().into_owned();
        assert_eq!(
            review_root(&args(&["--review-root", &absolute])).unwrap(),
            Some(absolute.clone().into())
        );
        assert!(review_root(&args(&["--review-root"])).is_err());
        assert!(review_root(&args(&["--review-root="])).is_err());
        assert!(review_root(&args(&["--review-root", "relative"])).is_err());
        assert!(
            review_root(&args(&[
                "--review-root",
                &absolute,
                "--review-root",
                &absolute
            ]))
            .is_err()
        );
        assert!(review_root(&args(&["--ui-demo", "--review-root", &absolute])).is_err());
    }
    #[test]
    fn preview_is_explicit_and_does_not_mix_with_browser_validation() {
        let args = |values: &[&str]| values.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            desktop_mode(&args(&[])).unwrap(),
            if native_viewer::available() {
                DesktopMode::NativeViewer
            } else {
                DesktopMode::Player
            }
        );
        assert_eq!(
            desktop_mode(&args(&["--ui-demo", "--record-validation"])).unwrap(),
            DesktopMode::DesignPreview
        );
        assert!(desktop_mode(&args(&["--ui-demo", "--smoke-test"])).is_err());
        assert!(
            desktop_mode(&args(&[
                "--ui-demo",
                "--start-url=https://catalog.example.test/방영중"
            ]))
            .is_err()
        );
    }
    #[test]
    fn placement_requires_explicit_validation_and_finite_coordinates() {
        let args = |s: &[&str]| s.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(validation_position(&args(&["--validation-position=3500,100"])).is_err());
        assert!(
            validation_position(&args(&[
                "--record-validation",
                "--validation-position=NaN,100"
            ]))
            .is_err()
        );
        assert_eq!(
            validation_position(&args(&[
                "--record-validation",
                "--validation-position=3500,100"
            ]))
            .unwrap(),
            Some([3500., 100.])
        );
    }
}
