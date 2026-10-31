//! `kinekino watch [<file>] [--smoke] [--capture <file.png>]` (plan P6): the Meari player
//! window with libmpv frames. Same command on Linux and Windows; `--smoke` closes after
//! the first frames (with `--capture`, after writing a PNG of the window).
use crate::args::Args;
use kinekino_libmpv::LibMpvSurface;
use kinekino_player_ui::{Controller, Options};
use serde_json::json;
use std::path::PathBuf;

fn usage(message: &str) -> (u8, String) {
    (
        2,
        json!({"error": message, "usage": "kinekino watch [<file>] [--smoke] [--capture <file.png>]"})
            .to_string(),
    )
}

pub fn run(args: &Args, env: &crate::Env) -> (u8, String) {
    let mut options = Options::default();
    let mut media = None;
    let mut i = 1;
    while let Some(arg) = args.get(i) {
        match arg {
            "--smoke" => options.smoke = true,
            "--capture" => {
                i += 1;
                match args.get(i) {
                    Some(p) => options.capture = Some(PathBuf::from(p)),
                    None => return usage("--capture needs a .png path"),
                }
            }
            m if media.is_none() && !m.starts_with('-') => media = Some(PathBuf::from(m)),
            other => return usage(&format!("watch: unexpected {other:?}")),
        }
        i += 1;
    }
    if options.capture.is_some() && !options.smoke {
        return usage("--capture needs --smoke");
    }
    if let Some(m) = &media
        && !m.is_file()
    {
        return usage(&format!("watch: no such file {}", m.display()));
    }
    // Audio stays on for real viewing; smoke runs are silent.
    let engine = match LibMpvSurface::new(None, options.smoke) {
        Ok(e) => e,
        Err(e) => {
            return (
                1,
                json!({"error": e.message, "needs": "libmpv"}).to_string(),
            );
        }
    };
    let mut controller = Controller::new(engine)
        .with_catalog(Box::new(crate::plugins::PluginCatalog::discover(env)));
    if let Some(m) = &media {
        controller.open(m);
    }
    match kinekino_player_ui::run(controller, options) {
        Ok(r) => (
            0,
            json!({"ok": true, "frames": r.frames, "uploads": r.uploads, "captured": r.captured, "remounts": r.remounts})
                .to_string(),
        ),
        Err(e) => (1, json!({"error": e}).to_string()),
    }
}
