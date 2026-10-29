//! CDP adapter against installed Google Chrome on a local fixture (no external sites).

use kinekino_browser::engine::chromium::cdp::{CdpEngine, CdpLaunch};
use kinekino_browser::engine::contract::{self, fixture as support};
use kinekino_browser::engine::port::{
    BrowserEngine, BrowserEvent, FrameCapture, InterceptedRequest, NetworkInterceptor,
    RequestDecision,
};
use std::time::Duration;

struct BlockMarker;
impl NetworkInterceptor for BlockMarker {
    fn decide(&self, r: &InterceptedRequest) -> RequestDecision {
        if r.url.ends_with("/blocked.js") {
            RequestDecision::Block
        } else {
            RequestDecision::Allow
        }
    }
}

const PROBE: &str = r#"(async () => {
  const mse = MediaSource.isTypeSupported('video/mp4; codecs="avc1.42E01E, mp4a.40.2"');
  const v = document.getElementById('v');
  for (let i = 0; i < 40 && !(v.currentTime > 0.3); i++) await new Promise(r => setTimeout(r, 100));
  let widevine = 'unsupported';
  try {
    const cfg = [{ initDataTypes: ['cenc'], videoCapabilities: [{ contentType: 'video/mp4; codecs="avc1.42E01E"' }],
                   audioCapabilities: [{ contentType: 'audio/mp4; codecs="mp4a.40.2"' }] }];
    const a = await navigator.requestMediaKeySystemAccess('com.widevine.alpha', cfg);
    await a.createMediaKeys();
    widevine = 'ok';
  } catch (e) { widevine = 'error:' + e.name; }
  return { mse, played: v.currentTime > 0.3, width: v.videoWidth, decoded: v.webkitDecodedFrameCount || 0,
           widevine, blockedRan: !!window.__blockedRan, injected: window.__kk === 1 };
})()"#;

fn run(headless: bool) -> Option<serde_json::Value> {
    if !CdpLaunch::executable().exists() {
        eprintln!("skip: chrome not installed");
        return None;
    }
    let dir = std::env::temp_dir().join(format!("kk-cdp-{}-{headless}", std::process::id()));
    std::fs::create_dir_all(&dir).ok()?;
    let base = support::serve(support::make_clip(&dir));
    let spec = CdpLaunch {
        headless,
        profile_dir: dir.join("profile"),
        window: (800, 600),
        app_url: None,
    };
    let mut engine = CdpEngine::launch(&spec).expect("launch chrome");
    engine
        .set_interceptor(Box::new(BlockMarker))
        .expect("interceptor");
    engine.inject_script("window.__kk = 1;").expect("inject");
    engine.navigate(&format!("{base}/")).expect("navigate");
    std::thread::sleep(Duration::from_millis(300));
    let probe = engine.eval(PROBE).expect("probe");
    let png = engine.capture_png().expect("screenshot");
    assert_eq!(&png[1..4], b"PNG");
    std::fs::write(dir.join("shot.png"), &png).ok();
    let events = engine.drain_events();
    assert!(events.iter().any(
        |e| matches!(e, BrowserEvent::RequestBlocked { url } if url.ends_with("/blocked.js"))
    ));
    engine.close().expect("close");
    let _ = std::fs::remove_dir_all(dir.join("profile"));
    println!(
        "{} probe={probe} shot={}",
        if headless { "headless" } else { "headed" },
        dir.join("shot.png").display()
    );
    Some(probe)
}

#[test]
fn headless_intercept_inject_screenshot_and_h264_aac() {
    let Some(p) = run(true) else { return };
    assert_eq!(p["blockedRan"], false, "interceptor must block /blocked.js");
    assert_eq!(p["injected"], true);
    assert_eq!(p["mse"], true, "H.264+AAC MSE support");
    assert_eq!(p["played"], true, "H.264/AAC clip must play: {p}");
}

/// Headed run on the box desktop (needs DISPLAY); records Widevine availability.
#[test]
#[ignore = "needs DISPLAY; run with --ignored"]
fn headed_widevine_probe() {
    let Some(p) = run(false) else { return };
    assert_eq!(p["played"], true, "{p}");
    assert_eq!(p["widevine"], "ok", "{p}");
}

/// The shared port contract against headless Chrome over CDP.
#[test]
fn headless_cdp_passes_port_contract() {
    if !CdpLaunch::executable().exists() {
        eprintln!("skip: chrome not installed");
        return;
    }
    let dir = std::env::temp_dir().join(format!("kk-contract-{}", std::process::id()));
    let base = support::serve(None);
    let mut engine = CdpEngine::launch(&CdpLaunch {
        headless: true,
        profile_dir: dir.join("profile"),
        window: (800, 600),
        app_url: None,
    })
    .expect("launch chrome");
    let report = contract::run(&mut engine, &base);
    engine.close().expect("close");
    let _ = std::fs::remove_dir_all(&dir);
    println!("{report}");
    assert!(report.passed(), "{:?}", report.failures());
}
