//! cargo xtask entry: dispatches developer automation subcommands.
mod clean_cache;
mod demo;
mod demo_engine;
mod install;
mod spec_check;
mod structure;
mod workspace;

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("lint-structure") => structure::run(&args[1..]),
        Some("install") => install::run(&args[1..]),
        Some("spec-check") => spec_check::run(&args[1..]),
        Some("demo-browser") => demo::browser(&args[1..]),
        Some("demo-engine") => demo_engine::run(&args[1..]),
        Some("clean-cache") => clean_cache::run(&args[1..]),
        Some("print-root") => match workspace::root() {
            Ok(root) => {
                println!("{}", root.display());
                std::process::ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::ExitCode::FAILURE
            }
        },
        _ => {
            eprintln!(
                "usage: cargo xtask <lint-structure [--write-allowlist] | install [--from BINARY] [--bin-dir DIR] | spec-check [SPEC CHECK ARGS] | demo-browser [URL] | demo-engine <cef|chrome> [--hold] | clean-cache [--max-size 8G] [--target-dir DIR] [--dry-run]>"
            );
            std::process::ExitCode::from(2)
        }
    }
}
