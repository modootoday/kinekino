//! Official CEF created as a child of a Rust-owned X11 window, then the port contract.
//! Usage: DISPLAY=:7 cargo xtask demo-engine cef [--hold]   (needs KINEKINO_CEF_DIR)
// X11-only demo: other targets get a stub so `--workspace --all-targets` still builds.
#[cfg(target_os = "linux")]
mod linux {
    use kinekino_browser::engine::chromium::cef::{CefEngine, CefLaunch, subprocess_entry};
    use kinekino_browser::engine::contract::{self, fixture};
    use kinekino_browser::engine::embed::x11::X11Host;
    use kinekino_browser::engine::port::BrowserEngine;

    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        if let Some(code) = subprocess_entry() {
            std::process::exit(code);
        }
        let hold = std::env::args().any(|a| a == "--hold");
        let host = X11Host::open("kinekino engine demo: cef", 960, 600)?;
        let base = fixture::serve(None);
        let cache = std::env::var_os("KINEKINO_CEF_CACHE").map_or_else(
            || std::env::temp_dir().join("kk-demo-cef-cache"),
            Into::into,
        );
        let mut engine = CefEngine::launch(&CefLaunch {
            parent: u64::from(host.window),
            size: (960, 600),
            cache_dir: cache,
            initial_url: "about:blank".into(),
        })?;
        let report = contract::run(&mut engine, &base);
        print!("{report}");
        engine.navigate(&format!("{base}/"))?;
        println!("READY {}", engine.capabilities());
        let wait = if hold { 3600 } else { 4 };
        engine.pump_for(std::time::Duration::from_secs(wait));
        engine.close();
        if !report.passed() {
            return Err(format!("contract failures: {:?}", report.failures()).into());
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    linux::main()
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("demo_engine_cef needs Linux/X11");
}
