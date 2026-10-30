use kinekino_player::application::{
    api::ErrorCode,
    launch::{self, DesktopMode, desktop_launch_plan},
};

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).into()).collect()
}

#[test]
fn ordinary_launch_uses_the_builds_product_mode_without_selecting_storage() {
    kinekino_platform::install();
    let values = args(&[]);
    let plan = desktop_launch_plan(&values).unwrap();
    let expected = if cfg!(all(windows, feature = "native-proposal-window")) {
        DesktopMode::NativeViewer
    } else {
        DesktopMode::Player
    };
    assert_eq!(plan.mode, expected);
    assert!(plan.project.is_none());
    assert!(launch::review_root(&values).unwrap().is_none());
    assert!(launch::proposal_root(&values).unwrap().is_none());
    assert!(launch::project_root(&values).unwrap().is_none());
    assert!(launch::job_root(&values).unwrap().is_none());
}

#[test]
fn explicit_roots_do_not_change_the_effective_default_or_select_a_project() {
    kinekino_platform::install();
    let directory = std::env::temp_dir();
    let values: Vec<String> = ["review", "proposal", "project", "job"]
        .iter()
        .map(|kind| format!("--{kind}-root={}", directory.display()))
        .collect();
    let plan = desktop_launch_plan(&values).unwrap();
    assert_eq!(plan.mode, desktop_launch_plan(&[]).unwrap().mode);
    assert!(plan.project.is_none());
    assert_eq!(
        launch::review_root(&values).unwrap(),
        Some(directory.clone())
    );
    assert_eq!(
        launch::proposal_root(&values).unwrap(),
        Some(directory.clone())
    );
    assert_eq!(
        launch::project_root(&values).unwrap(),
        Some(directory.clone())
    );
    assert_eq!(launch::job_root(&values).unwrap(), Some(directory));
}

#[cfg(all(windows, feature = "native-proposal-window"))]
#[test]
fn latest_and_pinned_project_selection_work_without_an_explicit_native_flag() {
    kinekino_platform::install();
    let latest = desktop_launch_plan(&args(&["--viewer-project=project-a"]))
        .unwrap()
        .project
        .unwrap();
    let pinned = desktop_launch_plan(&args(&[
        "--viewer-project=project-a",
        "--viewer-project-revision=2",
    ]))
    .unwrap()
    .project
    .unwrap();
    assert_eq!(latest.project_id, pinned.project_id);
    assert!(latest.revision.is_none());
    assert_eq!(pinned.revision.unwrap().0, 2);
    let explicit = desktop_launch_plan(&args(&[
        "--native-viewer",
        "--viewer-project=project-a",
        "--viewer-project-revision=2",
    ]))
    .unwrap()
    .project
    .unwrap();
    assert_eq!(explicit.project_id, pinned.project_id);
    assert_eq!(explicit.revision, pinned.revision);
}

#[test]
fn explicit_browser_preview_and_placement_intents_keep_their_routes() {
    kinekino_platform::install();
    for values in [
        vec!["--smoke-test"],
        vec!["--start-url=https://example.test"],
        vec!["--start-url", "https://example.test"],
        vec!["--record-validation", "--validation-position=0,0"],
    ] {
        let plan = desktop_launch_plan(&args(&values)).unwrap();
        assert_eq!(plan.mode, DesktopMode::Player);
        assert!(plan.project.is_none());
    }
    assert_eq!(
        desktop_launch_plan(&args(&["--ui-demo"])).unwrap().mode,
        DesktopMode::DesignPreview
    );
    for browser in ["--smoke-test", "--start-url", "--start-url=x"] {
        assert_eq!(
            desktop_launch_plan(&args(&["--ui-demo", browser]))
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
    }
}

#[test]
fn explicit_native_selection_preserves_unsupported_and_conflict_errors() {
    kinekino_platform::install();
    let result = desktop_launch_plan(&args(&["--native-viewer"]));
    if cfg!(all(windows, feature = "native-proposal-window")) {
        assert_eq!(result.unwrap().mode, DesktopMode::NativeViewer);
    } else {
        assert_eq!(result.unwrap_err().code, ErrorCode::Unsupported);
    }
    for other in [
        "--native-viewer",
        "--ui-demo",
        "--smoke-test",
        "--start-url=x",
        "--native-proposal-request=x",
        "--validation-position=0,0",
    ] {
        assert_eq!(
            desktop_launch_plan(&args(&["--native-viewer", other]))
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
    }
}

#[test]
fn project_options_are_checked_against_effective_mode() {
    kinekino_platform::install();
    let result = desktop_launch_plan(&args(&["--viewer-project=project-a"]));
    if cfg!(all(windows, feature = "native-proposal-window")) {
        let plan = result.unwrap();
        assert_eq!(plan.mode, DesktopMode::NativeViewer);
        let project = plan.project.unwrap();
        assert_eq!(project.project_id, "project-a");
        assert!(project.revision.is_none());
    } else {
        assert_eq!(result.unwrap_err().code, ErrorCode::InvalidArgument);
    }
    for other in ["--ui-demo", "--smoke-test", "--start-url=x"] {
        assert_eq!(
            desktop_launch_plan(&args(&[other, "--viewer-project=project-a"]))
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
    }
    for values in [
        vec!["--viewer-project-revision=1"],
        vec!["--viewer-project=../x"],
        vec!["--viewer-project=x", "--viewer-project=x"],
        vec!["--viewer-project=x", "--viewer-project-revision=01"],
        vec!["--viewer-project=x", "--viewer-project-revision=0"],
        vec![
            "--viewer-project=x",
            "--viewer-project-revision=1",
            "--viewer-project-revision=2",
        ],
    ] {
        assert_eq!(
            desktop_launch_plan(&args(&values)).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
}

#[cfg(windows)]
#[test]
fn explicit_proposal_intent_keeps_its_mode_and_build_requirement() {
    kinekino_platform::install();
    let directory = std::env::temp_dir();
    let request = format!(
        "--native-proposal-request={}",
        directory.join("request.json").display()
    );
    let root = format!("--proposal-root={}", directory.display());
    let result = desktop_launch_plan(&args(&[&request, &root]));
    if cfg!(feature = "native-proposal-window") {
        let plan = result.unwrap();
        assert_eq!(plan.mode, DesktopMode::NativeProposal);
        assert!(plan.project.is_none());
    } else {
        assert_eq!(result.unwrap_err().code, ErrorCode::Unsupported);
    }
    for other in ["--ui-demo", "--smoke-test", "--start-url=x"] {
        assert_eq!(
            desktop_launch_plan(&args(&[&request, &root, other]))
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
    }
}

#[cfg(all(windows, feature = "native-proposal-window"))]
mod configured_project {
    use super::*;
    use kinekino_player::{
        application::{
            api::{ApplicationClient, PlayerSdk, ProjectClient, Request, RequestContext, Response},
            projects::InspectProject,
        },
        assembly::meari::product_viewer_launch::prepare_project,
        infrastructure::{projects::ProjectFiles, subtitle_files::SubtitleFiles},
    };
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "player-launch-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&root).unwrap();
            Self(root)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    struct NoCalls;
    impl ApplicationClient for NoCalls {
        fn execute(&self, _: Request) -> Response {
            panic!("Absent selection or missing root must not inspect the SDK");
        }
    }

    #[test]
    fn absent_selection_does_not_inspect_or_touch_the_supplied_root() {
        kinekino_platform::install();
        let fixture = Fixture::new();
        let absent = fixture.0.join("never-created");
        assert!(
            prepare_project(None, &NoCalls, Some(&absent))
                .unwrap()
                .is_none()
        );
        assert!(!absent.exists());
    }

    #[test]
    fn selected_project_requires_root_before_any_sdk_call() {
        kinekino_platform::install();
        let plan = desktop_launch_plan(&args(&["--viewer-project=project-a"])).unwrap();
        let result = prepare_project(plan.project, &NoCalls, None);
        assert!(matches!(result, Err(error) if error.code == ErrorCode::NeedsInput));
    }

    #[test]
    fn typed_selection_reaches_the_configured_sdk_without_creating_a_project() {
        kinekino_platform::install();
        let fixture = Fixture::new();
        let root_arg = format!("--project-root={}", fixture.0.display());
        let values = args(&[
            &root_arg,
            "--viewer-project=missing",
            "--viewer-project-revision=2",
        ]);
        let plan = desktop_launch_plan(&values).unwrap();
        let root = launch::project_root(&values).unwrap().unwrap();
        let selection = plan.project.unwrap();
        assert_eq!(selection.revision.unwrap().0, 2);
        let sdk =
            PlayerSdk::new(SubtitleFiles).with_project_store(ProjectFiles::open(&root).unwrap());
        let expected = sdk
            .inspect_project(
                RequestContext::default(),
                InspectProject {
                    project_id: selection.project_id.clone(),
                    revision: selection.revision,
                },
            )
            .unwrap_err();
        let result = prepare_project(Some(selection), &sdk, Some(&root));
        assert!(matches!(result, Err(error) if error.code == expected.code));
        assert_eq!(expected.code, ErrorCode::NotFound);
        assert!(!root.join("p-missing").exists());
        drop(sdk);
        assert!(ProjectFiles::open(&root).is_ok());
    }
}
