mod adopt_apply;
mod adoption;
mod application;
mod exact;
mod infrastructure;
mod json;
mod paths;
mod snapshot;
mod sprite_audio_replay;
mod sprite_metadata;
mod sprite_pcm;
mod stage;

fn main() -> std::process::ExitCode {
    match application::run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("custody stopped: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
