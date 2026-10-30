//! `kinekino-vad-worker` entry: installs the OS policy, then runs the inert worker body.
fn main() -> std::process::ExitCode {
    kinekino_app::vad_worker()
}
