//! Contract tests for the whisper.cpp local pack. `harness = false`: when `KK_FAKE_WHISPER`
//! is set this binary acts as a fake `whisper-cli`, so no real model or download is used.
use kinekino_core::port::registry::{Availability, Policy};
use kinekino_core::port::{Adapter, contract};
use kinekino_core::port::{OsId, TargetPlatform};
use kinekino_inference::executor::{
    PcmTrack, SpeechRegion, TranscribeJob, transcribe, wav_segment,
};
use kinekino_inference::port::Output;
use kinekino_inference::registry::inference_registry;
use kinekino_inference::settings::Budget;
use kinekino_inference::{
    Consent, CostGuard, InferenceBackend, InferenceCaps, InferenceError, InferenceKind,
    InferenceRequest,
};
use kinekino_inference_local::{
    LOCAL_CAPS, WhisperCliBackend, WhisperConfig, parse_whisper_json, register_whisper_cli,
};
use std::path::PathBuf;
use std::time::Duration;

const SAMPLE: &str = r#"{"systeminfo":"x","model":{"type":"large"},"transcription":[
 {"timestamps":{"from":"00:00:00,000","to":"00:00:00,400"},"offsets":{"from":0,"to":400},"text":" 안녕하세요"},
 {"timestamps":{"from":"00:00:00,400","to":"00:00:00,900"},"offsets":{"from":400,"to":900},"text":" 반갑습니다 "},
 {"offsets":{"from":900,"to":900},"text":"  "}]}"#;

fn fake_main() {
    let args: Vec<String> = std::env::args().collect();
    let value = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .map(|i| args[i + 1].clone())
            .unwrap_or_default()
    };
    let wav = std::fs::read(value("-f")).unwrap_or_default();
    assert_eq!(&wav[..4], b"RIFF", "fake got a non-WAV input");
    assert!(args.iter().any(|a| a == "-oj"));
    let out = PathBuf::from(value("-of")).with_extension("json");
    match std::env::var("KK_FAKE_WHISPER").unwrap().as_str() {
        "ok" => std::fs::write(out, SAMPLE).unwrap(),
        "fail" => {
            eprintln!("error: failed to load model");
            std::process::exit(3);
        }
        "garbage" => std::fs::write(out, "{not json").unwrap(),
        "nojson" => {}
        _ => std::thread::sleep(Duration::from_secs(30)),
    }
}

struct Pack {
    dir: PathBuf,
}

impl Pack {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("kk-whisper-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("ggml-large-v3-turbo.bin"), b"fake").unwrap();
        Self { dir }
    }
    fn config(&self) -> WhisperConfig {
        WhisperConfig {
            program: std::env::current_exe().unwrap(),
            model: self.dir.join("ggml-large-v3-turbo.bin"),
            language: "ko".into(),
            threads: 1,
            timeout: Duration::from_secs(10),
            work_dir: self.dir.join("work"),
        }
    }
    fn backend(&self, mode: &str) -> WhisperCliBackend {
        // SAFETY: harness = false runs tests sequentially on one thread; no concurrent env readers.
        unsafe { std::env::set_var("KK_FAKE_WHISPER", mode) };
        WhisperCliBackend::new(self.config())
    }
}

impl Drop for Pack {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn track() -> PcmTrack {
    PcmTrack {
        sample_rate: 16_000,
        samples: vec![0; 16_000 * 4],
    }
}

fn request() -> InferenceRequest {
    let audio = wav_segment(
        &track(),
        SpeechRegion {
            start: 0.0,
            end: 1.0,
        },
    )
    .unwrap();
    InferenceRequest::Transcribe {
        model: "local".into(),
        audio,
    }
}

fn parses_whisper_json() {
    let Output::Transcribe { text, chunks } = parse_whisper_json(SAMPLE).unwrap() else {
        panic!()
    };
    assert_eq!(text, "안녕하세요 반갑습니다");
    assert_eq!(chunks.len(), 2);
    assert_eq!((chunks[1].start, chunks[1].end), (0.4, 0.9));
    for bad in [
        "{}",
        "[]",
        r#"{"transcription":[{"text":"a"}]}"#,
        r#"{"transcription":[{"offsets":{"from":5,"to":1},"text":"a"}]}"#,
    ] {
        assert!(
            matches!(parse_whisper_json(bad), Err(InferenceError::Protocol(_))),
            "{bad}"
        );
    }
}

fn identity_and_unsupported() {
    let pack = Pack::new("identity");
    let mut b = pack.backend("ok");
    contract::check_identity(
        &b,
        InferenceCaps::of(&[kinekino_inference::Task::Transcribe]),
    );
    assert_eq!(b.kind(), InferenceKind::LocalPack);
    let ocr = InferenceRequest::Ocr {
        model: "m".into(),
        image: vec![1],
    };
    contract::check_unsupported("ocr", b.run(&ocr, &|| false));
    let raw = InferenceRequest::Transcribe {
        model: "m".into(),
        audio: vec![0; 8],
    };
    assert!(matches!(
        b.run(&raw, &|| false),
        Err(InferenceError::InvalidInput(_))
    ));
    assert_eq!(b.run(&request(), &|| true), Err(InferenceError::Cancelled));
}

fn runs_fake_cli_and_cleans_scratch() {
    let pack = Pack::new("ok");
    let mut b = pack.backend("ok");
    let resp = b.run(&request(), &|| false).unwrap();
    assert_eq!(resp.provenance.backend, "local-pack/whisper.cpp");
    assert_eq!(resp.provenance.model, "ggml-large-v3-turbo.bin");
    assert_eq!(resp.provenance.cost_estimate_usd, 0.0);
    assert!(matches!(resp.output, Output::Transcribe { ref chunks, .. } if chunks.len() == 2));
    let left = std::fs::read_dir(pack.dir.join("work")).unwrap().count();
    assert_eq!(left, 0, "per-call scratch must be removed");
}

fn maps_process_failures() {
    let pack = Pack::new("fail");
    let err = pack.backend("fail").run(&request(), &|| false).unwrap_err();
    assert!(
        matches!(&err, InferenceError::Unavailable(m) if m.contains("failed to load model")),
        "{err:?}"
    );
    assert!(matches!(
        pack.backend("garbage").run(&request(), &|| false),
        Err(InferenceError::Protocol(_))
    ));
    assert!(matches!(
        pack.backend("nojson").run(&request(), &|| false),
        Err(InferenceError::Protocol(_))
    ));
    let mut cfg = pack.config();
    cfg.program = pack.dir.join("missing-whisper-cli");
    assert!(matches!(
        WhisperCliBackend::new(cfg).run(&request(), &|| false),
        Err(InferenceError::Unavailable(_))
    ));
}

fn timeout_and_cancel_kill_the_child() {
    let pack = Pack::new("hang");
    let mut cfg = pack.config();
    cfg.timeout = Duration::from_millis(300);
    // SAFETY: single-threaded harness, see `Pack::backend`.
    unsafe { std::env::set_var("KK_FAKE_WHISPER", "hang") };
    let started = std::time::Instant::now();
    let err = WhisperCliBackend::new(cfg)
        .run(&request(), &|| false)
        .unwrap_err();
    assert!(matches!(err, InferenceError::Timeout(_)), "{err:?}");
    assert!(started.elapsed() < Duration::from_secs(5));
    let started = std::time::Instant::now();
    let deadline = started + Duration::from_millis(200);
    let err = pack
        .backend("hang")
        .run(&request(), &|| std::time::Instant::now() > deadline)
        .unwrap_err();
    assert_eq!(err, InferenceError::Cancelled);
    assert!(started.elapsed() < Duration::from_secs(5));
}

fn registry_probe_and_local_only_policy() {
    let pack = Pack::new("registry");
    let linux = TargetPlatform {
        os: OsId::Linux,
        display: None,
    };
    let mut reg = inference_registry();
    register_whisper_cli(&mut reg, pack.config());
    let d = reg.explain(linux, LOCAL_CAPS, Policy::Only(InferenceKind::LocalPack));
    assert_eq!(d.selected.map(|s| s.kind), Some(InferenceKind::LocalPack));
    let mut cfg = pack.config();
    cfg.model = pack.dir.join("absent.bin");
    assert!(
        matches!(WhisperCliBackend::availability(&cfg), Availability::Missing(m) if m.contains("absent.bin"))
    );
    let mut reg = inference_registry();
    register_whisper_cli(&mut reg, cfg);
    let d = reg.explain(linux, LOCAL_CAPS, Policy::Only(InferenceKind::LocalPack));
    assert!(d.selected.is_none());
}

fn executor_end_to_end_without_consent() {
    let pack = Pack::new("e2e");
    let mut b = pack.backend("ok");
    let mut guard = CostGuard::new(Budget::default());
    let job = TranscribeJob {
        project: "p",
        job: "j",
        month: "2026-10",
        model: "local",
        confirmed: false,
    };
    let regions = [
        SpeechRegion {
            start: 0.5,
            end: 1.5,
        },
        SpeechRegion {
            start: 2.0,
            end: 3.0,
        },
    ];
    let t = transcribe(
        &mut b,
        &mut guard,
        &Consent::default(),
        &job,
        &track(),
        &regions,
        &|| false,
    )
    .unwrap();
    let starts: Vec<f64> = t.chunks.iter().map(|c| c.start).collect();
    assert_eq!(starts, vec![0.5, 0.9, 2.0, 2.4]);
    assert_eq!(guard.total(), 0.0);
    assert!(guard.to_jsonl().lines().count() == 2);
}

fn main() {
    if std::env::var_os("KK_FAKE_WHISPER").is_some() && std::env::args().any(|a| a == "-oj") {
        return fake_main();
    }
    let tests: [(&str, fn()); 7] = [
        ("parses_whisper_json", parses_whisper_json),
        ("identity_and_unsupported", identity_and_unsupported),
        (
            "runs_fake_cli_and_cleans_scratch",
            runs_fake_cli_and_cleans_scratch,
        ),
        ("maps_process_failures", maps_process_failures),
        (
            "timeout_and_cancel_kill_the_child",
            timeout_and_cancel_kill_the_child,
        ),
        (
            "registry_probe_and_local_only_policy",
            registry_probe_and_local_only_policy,
        ),
        (
            "executor_end_to_end_without_consent",
            executor_end_to_end_without_consent,
        ),
    ];
    let filter: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .collect();
    let mut ran = 0;
    for (name, test) in tests {
        if filter.is_empty() || filter.iter().any(|f| name.contains(f.as_str())) {
            test();
            println!("test {name} ... ok");
            ran += 1;
        }
    }
    println!("test result: ok. {ran} passed; 0 failed; 0 ignored");
}
