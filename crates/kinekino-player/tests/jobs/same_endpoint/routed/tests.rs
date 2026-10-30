//! Synthetic owner/endpoint correlation only; not actual peer authentication or binary pin proof.
use super::*;
use kinekino_player::domain::product::PRODUCT;

#[test]
fn original_owner_generation_and_selected_pin_must_match_the_explicit_endpoint_manifest() {
    kinekino_platform::install();
    let executable = std::env::current_exe().unwrap();
    let identity = ProcessIdentity {
        pid: std::process::id(),
        creation_filetime: Some(1234),
        executable: executable.clone(),
    };
    let mut manifest = OwnerManifest {
        schema_version: 1,
        run_id: "synthetic-owner-wire-only".into(),
        exclusive_control: true,
        record_validation: true,
        gui_pid: identity.pid,
        gui_creation_filetime: 1234,
        gui_executable: executable,
        gui_sha256: "a".repeat(64),
        endpoint: format!(
            "pipe:{}.v1.{}.{:016x}.{}",
            PRODUCT.stable_product_id,
            identity.pid,
            1234,
            "0".repeat(32)
        ),
        catalog_root: std::env::temp_dir(),
        job_root: std::env::temp_dir().join("synthetic-job-path-only"),
        browser_profile_root: std::env::temp_dir().join("synthetic-profile-path-only"),
        mpv_executable: std::env::current_exe().unwrap(),
        mpv_sha256: "b".repeat(64),
    };
    assert!(validate_owner(&manifest, &identity, &"a".repeat(64)).is_ok());
    manifest.gui_creation_filetime = 1235;
    assert!(validate_owner(&manifest, &identity, &"a".repeat(64)).is_err());
    manifest.gui_creation_filetime = 1234;
    manifest.gui_pid += 1;
    assert!(validate_owner(&manifest, &identity, &"a".repeat(64)).is_err());
    manifest.gui_pid = identity.pid;
    assert!(validate_owner(&manifest, &identity, &"b".repeat(64)).is_err());
    manifest.exclusive_control = false;
    assert!(validate_owner(&manifest, &identity, &"a".repeat(64)).is_err());
}
