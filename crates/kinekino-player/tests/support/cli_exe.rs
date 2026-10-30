//! Locates the `kinekino` CLI binary built by the sibling `kinekino-cli` package.
//! `CARGO_BIN_EXE_*` only covers this package's own bins, so resolve next to the test runner
//! (`<target>/<profile>/deps/<test>` -> `<target>/<profile>/kinekino[.exe]`).
// Some test roots reach the CLI only from Windows-gated cases.
#[allow(dead_code)]
pub fn kinekino_cli_exe() -> &'static str {
    static PATH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    PATH.get_or_init(|| {
        if let Some(explicit) = std::env::var_os("KINEKINO_CLI_EXE") {
            return explicit.into_string().expect("KINEKINO_CLI_EXE must be UTF-8");
        }
        let runner = std::env::current_exe().expect("test executable path");
        let profile = runner
            .parent()
            .and_then(std::path::Path::parent)
            .expect("test executable lives in <profile>/deps");
        let cli = profile.join(format!("kinekino{}", std::env::consts::EXE_SUFFIX));
        if !cli.is_file() {
            // `cargo test` does not build sibling-package bins; build it once, offline.
            let target_dir = profile.parent().expect("target dir");
            let mut build = std::process::Command::new(env!("CARGO"));
            build.args(["build", "--offline", "-p", "kinekino-cli", "--target-dir"]);
            build.arg(target_dir);
            if profile.file_name().is_some_and(|n| n == "release") {
                build.arg("--release");
            }
            let status = build.status().expect("spawn cargo build -p kinekino-cli");
            assert!(status.success(), "cargo build -p kinekino-cli failed");
        }
        assert!(
            cli.is_file(),
            "kinekino CLI not built at {}; run `cargo test --workspace` or build -p kinekino-cli first",
            cli.display()
        );
        cli.into_os_string().into_string().expect("UTF-8 target path")
    })
}
