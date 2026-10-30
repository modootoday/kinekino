//! Register as infrastructure/browser/audio_backend/actual.rs after the UI gate returns.
//! Real Browser effects, legacy mpv port, common coordinator and same SDK/CLI.
//! The controlled document is not an observed supported-provider episode.
use super::*;
use crate::{
    application::{
        api::{PlayerSdk, Request},
        audio_handoff::{
            coordinator::{AudioHandoffCoordinator, PreparedAudioHandoff},
            intents::AudioHandoffIntents,
        },
        playback_status::native_status_channel,
    },
    domain::{api_values::DecimalU64, local_video::LocalVideo, native_audio::AudioIntent},
    infrastructure::{
        browser::Browser,
        browser::LegacyAudioHandoffPorts,
        browser_provider::{Apartment, HiddenWindow},
        local_host::{LocalHost, TransportOptions},
        mpv::Mpv,
        native_playback_control::native_control_channel,
        webview_audio::NativeWebViewAudio,
    },
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    fs,
    io::Write,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};
use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_8;
use windows::core::{BOOL, Interface};
use wry::{
    WebContext, WebViewBuilder, WebViewExtWindows,
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
};

mod media;
mod process;

fn record(root: &Path, name: &str, value: &impl serde::Serialize) {
    fs::write(root.join(name), serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
fn cli(root: &Path, endpoint: &str, name: &str, command: &str, payload: Value, exit: i32) -> Value {
    let executable = PathBuf::from(std::env::var_os("JOB_CLI_EXE").expect("Explicit CLI"));
    let mut request = Request::new(command);
    request.payload = payload;
    record(root, &format!("{name}.request.json"), &request);
    let mut child = Command::new(executable)
        .creation_flags(0x08000000)
        .args(["--endpoint", endpoint, "--input", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&request).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    fs::write(root.join(format!("{name}.stdout")), &output.stdout).unwrap();
    fs::write(root.join(format!("{name}.stderr")), &output.stderr).unwrap();
    record(root, &format!("{name}.exit.json"), &output.status.code());
    assert_eq!(
        output.status.code(),
        Some(exit),
        "Inspect preserved CLI result: {name}"
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
fn playing(core: &ICoreWebView2_8) -> bool {
    let mut value = BOOL(0);
    // SAFETY: actual native property on the owned test STA, with a live WebView.
    unsafe { core.IsDocumentPlayingAudio(&mut value) }.unwrap();
    value.as_bool()
}
fn media_diagnostic(browser: &Browser, window: &HiddenWindow, root: &Path) {
    let (sender, receiver) = std::sync::mpsc::channel();
    // Fixed read-only fixture diagnostic. It never supplies silence or readiness
    // evidence; Rust still requires the independent native audio properties.
    browser.view.evaluate_script_with_callback(
        "({ready:document.readyState,title:document.title,media:[...document.querySelectorAll('audio')].map(a=>({paused:a.paused,ended:a.ended,time:a.currentTime,ready:a.readyState,network:a.networkState,error:a.error?.code??null,muted:a.muted,volume:a.volume}))})",
        move |value| { let _ = sender.send(value); },
    ).unwrap();
    pump_until(window, || match receiver.try_recv() {
        Ok(value) => {
            fs::write(root.join("media-diagnostic.json"), value).unwrap();
            true
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => false,
        Err(error) => panic!("Fixture diagnostic callback failed: {error}"),
    });
}
fn pump_until(window: &HiddenWindow, mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        window.pump();
        if condition() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "Actual observation deadline; inspect evidence"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}
fn drive(
    root: &Path,
    name: &str,
    handoff: &mut AudioHandoffCoordinator,
    native: &mut Option<Mpv>,
    browser: &Browser,
) {
    handoff.drive(
        &mut LegacyAudioHandoffPorts::new(native, browser),
        Instant::now(),
    );
    let mut log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join(name).with_extension("jsonl"))
        .unwrap();
    serde_json::to_writer(&mut log, &handoff.observation()).unwrap();
    writeln!(log).unwrap();
    assert!(
        handoff.failure().is_none(),
        "Inspect preserved handoff error"
    );
}

#[test]
#[ignore = "Explicit real WebView2 and colocated trusted mpv; requires central heavy slot"]
fn actual_webview_native_audio_handoff_and_guard_loss_cleanup() {
    let root =
        PathBuf::from(std::env::var_os("WEB_AUDIO_EVIDENCE_DIR").expect("Explicit fresh root"));
    assert!(root.is_absolute());
    fs::create_dir(&root).unwrap();
    let cli_path = PathBuf::from(std::env::var_os("JOB_CLI_EXE").unwrap());
    let expected = std::env::var("JOB_CLI_SHA256").unwrap();
    let sha = crate::infrastructure::content_digest::sha256(
        &mut fs::File::open(cli_path).unwrap(),
        1024 * 1024 * 1024,
    )
    .unwrap()
    .1;
    assert_eq!(sha, expected);
    let mpv_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("mpv.exe");
    let sha = crate::infrastructure::content_digest::sha256(
        &mut fs::File::open(&mpv_path).unwrap(),
        1024 * 1024 * 1024,
    )
    .unwrap()
    .1;
    assert_eq!(sha, std::env::var("NATIVE_MPV_SHA256").unwrap());
    record(
        &root,
        "pins.json",
        &json!({"cli_sha256":expected,"mpv_sha256":sha}),
    );
    let _apartment = Apartment::new().unwrap();
    let window = HiddenWindow::new().unwrap();
    let RawWindowHandle::Win32(handle) = window.window_handle().unwrap().as_raw() else {
        panic!("Win32 parent required")
    };
    let parent = u32::try_from(handle.hwnd.get()).unwrap();
    let mut context = WebContext::new(Some(root.join("profile")));
    let view = WebViewBuilder::new_with_web_context(&mut context)
        .with_autoplay(true)
        .with_visible(false)
        .with_focused(false)
        .with_html("<!doctype html><title>Owned guard</title>")
        .build_as_child(&window)
        .unwrap();
    let generation = Rc::new(Cell::new(1));
    let backend = WebViewAudioBackend::new(&view.webview(), generation.clone()).unwrap();
    let browser = Browser {
        audio_silence: Some(Rc::new(RefCell::new(
            crate::application::audio_handoff::web_silence::WebAudioSilenceOwner::new(backend),
        ))),
        view,
        enabled: Rc::new(RefCell::new(true)),
        generation,
        document: Rc::new(RefCell::new("about:blank".into())),
        visible: Cell::new(false),
    };
    let audio = NativeWebViewAudio::new(&browser.view.webview()).unwrap();
    let native_core = browser.view.webview().cast::<ICoreWebView2_8>().unwrap();
    let mut browser_pid = 0;
    // SAFETY: fixed process identity read from the live owned WebView on its STA.
    unsafe { browser.view.webview().BrowserProcessId(&mut browser_pid) }.unwrap();
    let browser_process = process::OwnedProcess::capture(browser_pid);
    record(
        &root,
        "browser-process.json",
        &json!({"process_id":browser_pid}),
    );
    assert!(!audio.is_muted().unwrap());
    record(
        &root,
        "initial.json",
        &json!({"view_muted":audio.is_muted().unwrap(),"document_playing_audio":playing(&native_core)}),
    );
    let video = root.join("finite.avi");
    media::native_video(&video);
    let source = LocalVideo::selected(video).unwrap();
    let mut native = Some(Mpv::launch_local(&source, parent, 0.).unwrap());
    let mut first_position = None;
    let mut decoded = Vec::new();
    pump_until(&window, || {
        let observation = native.as_mut().unwrap().snapshot().clone();
        let position = observation.time;
        if let Some(position) = position {
            first_position.get_or_insert(position);
        }
        let progressed = position
            .zip(first_position)
            .is_some_and(|(latest, first)| latest - first >= 0.2)
            && observation.width == Some(32)
            && observation.height == Some(32)
            && observation.muted == Some(true);
        decoded.push(observation);
        record(&root, "decoded-native-progress.json", &decoded);
        progressed
    });
    let raw = native.as_ref().unwrap().control_handle();
    let engine = DecimalU64(raw.instance_id());
    let (binder, control) = native_control_channel();
    let intents: AudioHandoffIntents = binder.audio_intents();
    let prepared = PreparedAudioHandoff::local_with_web_view(
        DecimalU64(browser.generation()),
        browser.audio_handoff_available(),
        AudioIntent::new(0., false).unwrap(),
        true,
        Instant::now(),
    )
    .unwrap();
    binder.bind_for_handoff(raw, prepared.requested()).unwrap();
    let mut handoff = prepared.bind(engine, intents.clone()).unwrap();
    let (publisher, status) = native_status_channel();
    let sdk = Arc::new(
        PlayerSdk::default()
            .with_playback_control(control)
            .with_playback_status(status),
    );
    let host = LocalHost::start(sdk, TransportOptions::default()).unwrap();
    let receipt = cli(
        &root,
        host.endpoint(),
        "pending-volume",
        "playback.set_volume",
        json!({"volume_percent":0.}),
        0,
    );
    assert_eq!(receipt["data"]["acceptance"], "deferred_until_handoff");
    pump_until(&window, || {
        drive(
            &root,
            "acquire-latest.json",
            &mut handoff,
            &mut native,
            &browser,
        );
        browser.active_audio_silence_lease().unwrap().is_some()
    });
    assert!(audio.is_muted().unwrap());
    // Newly created media is deliberately not silenced with a DOM flag. The real
    // whole-WebView native property must protect it throughout native release.
    // Match the visible-controller state during production handoff. The owned
    // parent HWND remains hidden; this does not show a desktop window.
    browser.set_visible(true).unwrap();
    browser
        .view
        .load_html(&media::browser_audio_html())
        .unwrap();
    let mut last_diagnostic = Instant::now() - Duration::from_secs(1);
    pump_until(&window, || {
        if last_diagnostic.elapsed() >= Duration::from_millis(500) {
            media_diagnostic(&browser, &window, &root);
            last_diagnostic = Instant::now();
        }
        playing(&native_core)
    });
    assert!(audio.is_muted().unwrap());
    record(
        &root,
        "playing-under-guard.json",
        &json!({"view_muted":true,"document_playing_audio":true}),
    );
    pump_until(&window, || {
        drive(
            &root,
            "release-latest.json",
            &mut handoff,
            &mut native,
            &browser,
        );
        if let Some(player) = native.as_mut() {
            publisher.publish(player.snapshot()).unwrap();
        }
        handoff.is_ready()
    });
    let ready = cli(
        &root,
        host.endpoint(),
        "ready",
        "playback.audio_handoff",
        json!({}),
        0,
    );
    assert_eq!(ready["data"]["ready"], true);
    let observed = cli(
        &root,
        host.endpoint(),
        "native-ready",
        "playback.status",
        json!({}),
        0,
    );
    record(&root, "ready-properties.json", &observed);
    assert_eq!(observed["data"]["muted"], false);
    assert_eq!(observed["data"]["volume_percent"], 0.);
    assert_eq!(observed["data"]["paused"], true);
    assert!(audio.is_muted().unwrap());
    pump_until(&window, || !playing(&native_core));
    handoff.close(&mut LegacyAudioHandoffPorts::new(&mut native, &browser));
    record(&root, "closed.json", &handoff.observation());
    assert!(handoff.cleanup_complete());
    assert!(native.is_none());
    assert!(!audio.is_muted().unwrap());
    assert!(browser.active_audio_silence_lease().unwrap().is_none());
    binder.clear().unwrap();

    // A real native-property guard loss is the controlled failure. No fake stop
    // proof or substitute AudioHandoffPorts implementation is used.
    native = Some(Mpv::launch_local(&source, parent, 0.).unwrap());
    let raw = native.as_ref().unwrap().control_handle();
    let engine = DecimalU64(raw.instance_id());
    let prepared = PreparedAudioHandoff::local_with_web_view(
        DecimalU64(browser.generation()),
        true,
        AudioIntent::new(0., false).unwrap(),
        true,
        Instant::now(),
    )
    .unwrap();
    binder.bind_for_handoff(raw, prepared.requested()).unwrap();
    let mut failed = prepared.bind(engine, intents.clone()).unwrap();
    pump_until(&window, || {
        drive(
            &root,
            "failure-acquire-latest.json",
            &mut failed,
            &mut native,
            &browser,
        );
        browser.active_audio_silence_lease().unwrap().is_some()
    });
    audio.set_muted(false).unwrap();
    failed.drive(
        &mut LegacyAudioHandoffPorts::new(&mut native, &browser),
        Instant::now(),
    );
    record(&root, "guard-loss-cleanup.json", &failed.observation());
    assert!(failed.failure().is_some());
    assert!(failed.cleanup_complete());
    assert!(native.is_none());
    assert!(!audio.is_muted().unwrap());
    assert!(browser.active_audio_silence_lease().unwrap().is_none());
    binder.clear().unwrap();

    // An initially muted WebView must restore the latest explicit SDK user
    // intention, rather than unconditionally restoring its acquisition value.
    browser
        .set_web_user_muted(DecimalU64(browser.generation()), true)
        .unwrap();
    native = Some(Mpv::launch_local(&source, parent, 0.).unwrap());
    let raw = native.as_ref().unwrap().control_handle();
    let engine = DecimalU64(raw.instance_id());
    let prepared = PreparedAudioHandoff::local_with_web_view(
        DecimalU64(browser.generation()),
        true,
        AudioIntent::new(0., false).unwrap(),
        true,
        Instant::now(),
    )
    .unwrap();
    binder.bind_for_handoff(raw, prepared.requested()).unwrap();
    let mut user = prepared.bind(engine, intents).unwrap();
    pump_until(&window, || {
        drive(
            &root,
            "user-ready-latest.json",
            &mut user,
            &mut native,
            &browser,
        );
        user.is_ready()
    });
    for (name, muted) in [("user-mute-true", true), ("user-mute-false", false)] {
        let receipt = cli(
            &root,
            host.endpoint(),
            name,
            "playback.set_muted",
            json!({"muted":muted}),
            0,
        );
        assert_eq!(receipt["data"]["acceptance"], "accepted");
        pump_until(&window, || {
            drive(
                &root,
                "user-intent-latest.json",
                &mut user,
                &mut native,
                &browser,
            );
            native.as_mut().unwrap().snapshot().muted == Some(muted)
        });
        assert!(
            audio.is_muted().unwrap(),
            "User intent must not unmute the leased WebView"
        );
    }
    user.close(&mut LegacyAudioHandoffPorts::new(&mut native, &browser));
    record(&root, "user-intent-restored.json", &user.observation());
    assert!(user.cleanup_complete());
    assert!(native.is_none());
    assert!(!audio.is_muted().unwrap());
    assert!(browser.active_audio_silence_lease().unwrap().is_none());
    binder.clear().unwrap();
    drop(host);
    drop(native_core);
    drop(audio);
    drop(browser);
    drop(context);
    let browser_exited = browser_process.wait(&window);
    record(
        &root,
        "completed.json",
        &json!({"native_owned":false,"web_lease_owned":false,"local_host_dropped":true,"view_dropped":true,"browser_process_id":browser_pid,"browser_exited":browser_exited,"controlled_fixture_only":true}),
    );
    assert!(browser_exited, "Owned WebView browser process did not exit");
}
