//! Explicit local project selection; no directory scan, live-origin creation or GUI automation.
use crate::{
    application::{
        api::{ApiError, ErrorCode},
        projects::InspectProject,
    },
    domain::{api_values::DecimalU64, identifiers::valid_id},
};

pub fn project_selection(args: &[String]) -> Result<Option<InspectProject>, ApiError> {
    super::super::desktop_launch_plan(args).map(|plan| plan.project)
}

pub(in crate::application::launch) fn parse_project_selection(
    args: &[String],
) -> Result<Option<InspectProject>, ApiError> {
    let mut id = None;
    let mut revision = None;
    let mut index = 0;
    while index < args.len() {
        for (option, target) in [
            ("--viewer-project", &mut id),
            ("--viewer-project-revision", &mut revision),
        ] {
            let value = if args[index] == option {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| invalid("Project selection value is missing"))?;
                if value.starts_with("--") {
                    return Err(invalid("Project selection value is missing"));
                }
                Some(value.clone())
            } else {
                args[index]
                    .strip_prefix(&format!("{option}="))
                    .map(str::to_owned)
            };
            if let Some(value) = value {
                if target.replace(value).is_some() {
                    return Err(invalid("Project selection option may be supplied once"));
                }
                break;
            }
        }
        index += 1;
    }
    if id.is_none() && revision.is_none() {
        return Ok(None);
    }
    let project_id = id
        .filter(|id| valid_id(id))
        .ok_or_else(|| invalid("An explicit valid project ID is required"))?;
    let revision = revision
        .map(|text| {
            let value = text
                .parse::<u64>()
                .map_err(|_| invalid("Project revision must be a positive decimal integer"))?;
            if value == 0 || value.to_string() != text {
                return Err(invalid(
                    "Project revision must be a positive canonical decimal integer",
                ));
            }
            Ok(DecimalU64(value))
        })
        .transpose()?;
    Ok(Some(InspectProject {
        project_id,
        revision,
    }))
}
fn invalid(message: &str) -> ApiError {
    ApiError::new(ErrorCode::InvalidArgument, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| (*v).into()).collect()
    }
    #[test]
    fn absent_selection_keeps_default_without_scanning() {
        assert!(project_selection(&args(&["player"])).unwrap().is_none());
    }
    #[test]
    fn explicit_revision_and_latest_remain_distinct() {
        if !crate::infrastructure::os_policy::native_proposal_window_available() {
            return;
        }
        let selected = project_selection(&args(&[
            "player",
            "--native-viewer",
            "--viewer-project=project-a",
            "--viewer-project-revision",
            "2",
        ]))
        .unwrap()
        .unwrap();
        assert_eq!(selected.project_id, "project-a");
        assert_eq!(selected.revision.unwrap().0, 2);
        assert!(
            project_selection(&args(&[
                "player",
                "--native-viewer",
                "--viewer-project",
                "project-a"
            ]))
            .unwrap()
            .unwrap()
            .revision
            .is_none()
        );
    }
    #[test]
    fn rejects_ambiguous_or_unscoped_selection() {
        for values in [
            vec!["--smoke-test", "--viewer-project=x"],
            vec![
                "--native-viewer",
                "--viewer-project=x",
                "--viewer-project=x",
            ],
            vec!["--native-viewer", "--viewer-project-revision=1"],
            vec![
                "--native-viewer",
                "--viewer-project=x",
                "--viewer-project-revision=01",
            ],
            vec![
                "--native-viewer",
                "--viewer-project=x",
                "--viewer-project-revision=0",
            ],
            vec!["--native-viewer", "--viewer-project=../x"],
        ] {
            assert!(project_selection(&args(&values)).is_err());
        }
    }
}
