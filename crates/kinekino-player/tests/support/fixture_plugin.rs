//! Site-neutral fixture plugin shared by CLI tests (no user plugins, no network).
use std::{fs, path::Path, process::Command};

/// Isolates CLI children from the user's real plugins: `<root>/plugins` holds only the
/// site-neutral fixture plugin (`source/1` browse + `document/1` revision, fixture layout).
pub fn install_fixture_plugin(root: &Path, cli: &Path) {
    let profile = cli.parent().expect("CLI lives in <target>/<profile>");
    let exe = profile.join(format!(
        "fake-source-plugin{}",
        std::env::consts::EXE_SUFFIX
    ));
    if !exe.is_file() {
        let mut build = Command::new(env!("CARGO"));
        build.args([
            "build",
            "--offline",
            "-p",
            "kinekino-app",
            "--bin",
            "fake-source-plugin",
        ]);
        build
            .arg("--target-dir")
            .arg(profile.parent().expect("target dir"));
        if profile.file_name().is_some_and(|n| n == "release") {
            build.arg("--release");
        }
        assert!(build.status().expect("spawn cargo build").success());
    }
    let dir = root.join("plugins").join("fixture");
    fs::create_dir_all(&dir).unwrap();
    let target = dir.join(format!("plugin{}", std::env::consts::EXE_SUFFIX));
    fs::copy(&exe, &target).unwrap();
    let manifest = serde_json::json!({
        "id": "com.example.fixture", "version": "0.1.0", "api": "source/1",
        "capabilities": ["browse"], "media_hosts": ["media.example.test"],
        "executable": target.file_name().unwrap().to_str().unwrap(),
        "document": {
            "host": "catalog.example.test",
            "categories": {"airing": "방영중", "completed": "종영", "movies": "극장판"},
            "kinds": {"c": "series", "e": "episode"},
            "stable_prefix": "catalog",
        },
    });
    fs::write(dir.join("plugin.json"), manifest.to_string()).unwrap();
}
