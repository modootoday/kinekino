//! Opens a visible Chrome app window on the desktop driven by the CDP adapter.
//! Usage: DISPLAY=:7 cargo xtask demo-browser [URL]
use kinekino_browser::engine::chromium::cdp::{CdpEngine, CdpLaunch};
use kinekino_browser::engine::port::BrowserEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "about:blank".into());
    let profile = std::env::temp_dir().join("kinekino-demo-browser");
    let spec = CdpLaunch {
        headless: false,
        profile_dir: profile,
        window: (1280, 800),
        app_url: None,
    };
    let mut engine = CdpEngine::launch(&spec)?;
    engine.navigate(&url)?;
    println!("kinekino demo-browser: {url} open; press Enter to close");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    engine.close()?;
    Ok(())
}
