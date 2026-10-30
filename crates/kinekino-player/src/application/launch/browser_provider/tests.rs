use super::*;

#[test]
fn profile_option_requires_one_value_and_both_original_storage_roots() {
    let root = std::env::temp_dir();
    for args in [
        vec!["--browser-profile-root".into()],
        vec!["--browser-profile-root=".into()],
        vec!["--browser-profile-root".into(), "--job-root".into()],
        vec![
            format!("--browser-profile-root={}", root.display()),
            format!("--browser-profile-root={}", root.display()),
        ],
    ] {
        assert_eq!(
            desktop_configuration(&args, Some(&root), Some(&root))
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
    }
    assert!(
        BrowserProviderConfiguration::from_explicit(root.clone(), root.clone(), "relative".into())
            .is_err()
    );
}

#[test]
fn unrelated_desktop_arguments_do_not_enable_or_select_a_provider() {
    let args = vec![
        "--review-root=unchanged".into(),
        "--record-validation".into(),
    ];
    assert!(desktop_configuration(&args, None, None).unwrap().is_none());
}

#[test]
fn native_profile_uses_original_roots_and_rejects_partial_or_other_modes() {
    let root = std::env::temp_dir();
    let jobs = root.join("original-jobs");
    let catalog = root.join("original-catalog");
    let profile = root.join("original-profile");
    let args = vec![
        format!("--browser-profile-root={}", profile.display()),
        "--record-validation".into(),
    ];
    if !crate::infrastructure::os_policy::native_proposal_window_available() {
        let error = desktop_configuration(&args, Some(&jobs), Some(&catalog)).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidArgument);
        return;
    }
    let selected = desktop_configuration(&args, Some(&jobs), Some(&catalog))
        .unwrap()
        .unwrap();
    assert_eq!(selected.job_root, jobs);
    assert_eq!(selected.catalog_root, catalog);
    assert_eq!(selected.profile_root, profile);
    assert!(desktop_configuration(&args, None, Some(&catalog)).is_err());
    assert!(desktop_configuration(&args, Some(&jobs), None).is_err());
    let mut preview = args;
    preview.push("--ui-demo".into());
    assert!(desktop_configuration(&preview, Some(&jobs), Some(&catalog)).is_err());
}
