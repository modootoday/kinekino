//! Installed Chrome embedded into a Rust-owned X11 window, then the port contract.
//! Usage: DISPLAY=:7 cargo xtask demo-engine chrome [--hold]
// X11-only demo: other targets get a stub so `--workspace --all-targets` still builds.
#[cfg(target_os = "linux")]
mod linux {
    use kinekino_browser::engine::chromium::chrome_embed::{ChromeEmbedLaunch, SystemChromeEngine};
    use kinekino_browser::engine::contract::{self, fixture};
    use kinekino_browser::engine::embed::x11::X11Host;
    use kinekino_browser::engine::port::BrowserEngine;

    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        let hold = std::env::args().any(|a| a == "--hold");
        let host = X11Host::open("kinekino engine demo: system-chrome", 960, 600)?;
        let profile = std::env::temp_dir().join(format!("kk-demo-chrome-{}", std::process::id()));
        let mut engine = SystemChromeEngine::launch(&ChromeEmbedLaunch {
            parent: host.window,
            size: (960, 600),
            profile_dir: profile.clone(),
        })?;
        let base = fixture::serve(fixture::make_clip(&std::env::temp_dir()));
        let report = contract::run(&mut engine, &base);
        print!("{report}");
        println!("click_offset={:?}", engine.click_offset());
        engine.navigate(&format!("{base}/"))?;
        println!("READY {}", engine.capabilities());
        if hold {
            std::io::stdin().read_line(&mut String::new())?;
        } else {
            std::thread::sleep(std::time::Duration::from_secs(4));
        }
        engine.close()?;
        let _ = std::fs::remove_dir_all(profile);
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
    eprintln!("demo_engine_chrome needs Linux/X11");
}
