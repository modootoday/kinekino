use super::*;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| (*s).into()).collect()
}
fn paths() -> (String, String) {
    let root = std::env::temp_dir();
    (
        root.join("native-request.json")
            .to_string_lossy()
            .into_owned(),
        root.to_string_lossy().into_owned(),
    )
}

#[test]
fn both_option_forms_select_same_explicit_paths_and_no_option_preserves_other_modes() {
    let (file, root) = paths();
    let spaced = args(&["--native-proposal-request", &file, "--proposal-root", &root]);
    let equals = args(&[
        &format!("--native-proposal-request={file}"),
        &format!("--proposal-root={root}"),
    ]);
    assert_eq!(
        parse_options(&spaced).unwrap(),
        parse_options(&equals).unwrap()
    );
    assert_eq!(
        parse_options(&spaced).unwrap().unwrap().request_path,
        PathBuf::from(file)
    );
    assert!(
        native_proposal_options(&args(&["--ui-demo"]))
            .unwrap()
            .is_none()
    );
    assert!(
        native_proposal_options(&args(&["--proposal-root", &root]))
            .unwrap()
            .is_none()
    );
}

#[test]
fn absent_values_relative_paths_duplicate_options_and_missing_storage_fail_before_feature_gate() {
    let (file, root) = paths();
    for input in [
        args(&["--native-proposal-request"]),
        args(&["--native-proposal-request="]),
        args(&[
            "--native-proposal-request",
            "relative.json",
            "--proposal-root",
            &root,
        ]),
        args(&["--native-proposal-request", &file]),
        args(&[
            "--native-proposal-request",
            &file,
            "--proposal-root",
            "relative",
        ]),
        args(&["--native-proposal-request", "--proposal-root", &root]),
        args(&[
            "--native-proposal-request",
            &file,
            &format!("--native-proposal-request={file}"),
            "--proposal-root",
            &root,
        ]),
        args(&[
            "--native-proposal-request",
            &file,
            "--proposal-root",
            &root,
            &format!("--proposal-root={root}"),
        ]),
    ] {
        assert_eq!(
            native_proposal_options(&input).unwrap_err().code,
            ErrorCode::InvalidArgument,
            "{input:?}"
        );
    }
}

#[test]
fn preview_smoke_and_both_start_url_forms_cannot_mix_with_native() {
    let (file, root) = paths();
    for conflict in [
        "--ui-demo",
        "--smoke-test",
        "--start-url",
        "--start-url=about:blank",
    ] {
        let input = args(&[
            "--native-proposal-request",
            &file,
            "--proposal-root",
            &root,
            conflict,
        ]);
        assert_eq!(
            native_proposal_options(&input).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert!(crate::application::launch::desktop_mode(&input).is_err());
    }
}

#[test]
fn feature_gate_and_unconnected_desktop_never_silently_fall_back() {
    let (file, root) = paths();
    let input = args(&["--native-proposal-request", &file, "--proposal-root", &root]);
    if crate::infrastructure::os_policy::native_proposal_window_available() {
        assert!(native_proposal_options(&input).unwrap().is_some());
        assert_eq!(
            crate::application::launch::desktop_mode(&input).unwrap(),
            crate::application::launch::DesktopMode::NativeProposal
        );
    } else {
        assert_eq!(
            native_proposal_options(&input).unwrap_err().code,
            ErrorCode::Unsupported
        );
        assert!(
            crate::application::launch::desktop_mode(&input)
                .unwrap_err()
                .to_string()
                .contains("unavailable in this build")
        );
    }
}
