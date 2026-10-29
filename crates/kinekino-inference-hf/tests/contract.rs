//! HF adapter vs a loopback mock server: schema parity with `MockBackend`, status → typed
//! error mapping, retry/backoff, cancel, token never leaked. No real network.
use kinekino_core::port::contract;
use kinekino_core::port::credential::{CredentialStore, Secret};
use kinekino_core::port::{ErrorCode, PortResult, ToPortError};
use kinekino_inference::mock::{MockBackend, reference_output};
use kinekino_inference::settings::{TOKEN_ACCOUNT, TOKEN_SERVICE};
use kinekino_inference::{InferenceBackend, InferenceCaps, InferenceError, InferenceRequest};
use kinekino_inference_hf::loopback::LoopbackHttp;
use kinekino_inference_hf::{HfCloudBackend, HfConfig};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const TOKEN: &str = "hf_TESTTOKEN_never_logged";

#[derive(Default)]
struct MemStore(HashMap<(String, String), Secret>);
impl CredentialStore for MemStore {
    fn get(&self, s: &str, a: &str) -> PortResult<Option<Secret>> {
        Ok(self.0.get(&(s.into(), a.into())).cloned())
    }
    fn set(&mut self, s: &str, a: &str, v: Secret) -> PortResult<()> {
        self.0.insert((s.into(), a.into()), v);
        Ok(())
    }
    fn delete(&mut self, s: &str, a: &str) -> PortResult<bool> {
        Ok(self.0.remove(&(s.into(), a.into())).is_some())
    }
}

#[derive(Debug, Clone)]
struct Seen {
    path: String,
    auth: Option<String>,
    body: serde_json::Value,
}

/// Serves `replies` in order (status, body), one per connection; records requests.
fn mock_server(replies: Vec<(u16, String)>) -> (String, Arc<Mutex<Vec<Seen>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = seen.clone();
    std::thread::spawn(move || {
        for (status, body) in replies {
            let Ok((mut s, _)) = listener.accept() else {
                return;
            };
            let mut buf = Vec::new();
            let mut tmp = [0u8; 4096];
            let (head_end, len) = loop {
                let n = s.read(&mut tmp).unwrap();
                buf.extend_from_slice(&tmp[..n]);
                if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&buf[..i]).to_string();
                    let len = head
                        .lines()
                        .find_map(|l| {
                            l.to_ascii_lowercase()
                                .strip_prefix("content-length: ")
                                .map(|v| v.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    break (i + 4, len);
                }
            };
            while buf.len() < head_end + len {
                let n = s.read(&mut tmp).unwrap();
                buf.extend_from_slice(&tmp[..n]);
            }
            let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
            log.lock().unwrap().push(Seen {
                path: head.split_whitespace().nth(1).unwrap().to_string(),
                auth: head
                    .lines()
                    .find_map(|l| l.strip_prefix("Authorization: ").map(str::to_string)),
                body: serde_json::from_slice(&buf[head_end..head_end + len]).unwrap_or_default(),
            });
            let resp = format!(
                "HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            s.write_all(resp.as_bytes()).unwrap();
        }
    });
    (url, seen)
}

fn backend(url: &str, retries: u32, with_token: bool) -> HfCloudBackend {
    let mut store = MemStore::default();
    if with_token {
        store
            .set(TOKEN_SERVICE, TOKEN_ACCOUNT, Secret::new(TOKEN))
            .unwrap();
    }
    let cfg = HfConfig {
        base_url: url.into(),
        connect_timeout: Duration::from_secs(2),
        total_timeout: Duration::from_secs(5),
        max_retries: retries,
        backoff: Duration::from_millis(1),
    };
    HfCloudBackend::new(cfg, Box::new(LoopbackHttp), &store).unwrap()
}

fn requests() -> Vec<(InferenceRequest, String, &'static str)> {
    let chat = |c: &str| {
        format!(r#"{{"choices":[{{"message":{{"role":"assistant","content":"{c}"}}}}]}}"#)
    };
    vec![
        (
            InferenceRequest::Transcribe { model: "openai/whisper-large-v3".into(), audio: vec![1, 2, 3] },
            r#"{"text":" hello world","chunks":[{"timestamp":[0.0,0.5],"text":" hello"},{"timestamp":[0.5,1.0],"text":" world"}]}"#.into(),
            "/hf-inference/models/openai/whisper-large-v3",
        ),
        (InferenceRequest::Generate { model: "Qwen/Qwen3-8B:cheapest".into(), prompt: "p".into() }, chat("ok"), "/v1/chat/completions"),
        (InferenceRequest::Ocr { model: "Qwen/Qwen2.5-VL-7B-Instruct:fastest".into(), image: vec![9] }, chat("TITLE"), "/v1/chat/completions"),
        (
            InferenceRequest::Embed { model: "e/m".into(), text: "t".into() },
            "[[0.25,0.5,0.25]]".into(),
            "/hf-inference/models/e/m/pipeline/feature-extraction",
        ),
    ]
}

#[test]
fn schema_parity_with_mock_backend() {
    let reqs = requests();
    let (url, seen) = mock_server(reqs.iter().map(|(_, b, _)| (200, b.clone())).collect());
    let mut hf = backend(&url, 0, true);
    let mut mock = MockBackend::default();
    contract::check_identity(&hf, InferenceCaps::all());
    for (req, _, path) in &reqs {
        let a = hf.run(req, &|| false).unwrap();
        let b = mock.run(req, &|| false).unwrap();
        assert_eq!(a.output, b.output, "{path}");
        assert_eq!(a.output, reference_output(req));
        assert_eq!(a.provenance.backend, "hf-cloud");
        assert_eq!(a.provenance.model, req.model());
    }
    let seen = seen.lock().unwrap();
    for ((_, _, path), s) in reqs.iter().zip(seen.iter()) {
        assert_eq!(&s.path, path);
        assert_eq!(s.auth.as_deref(), Some(&*format!("Bearer {TOKEN}")));
    }
    assert_eq!(seen[0].body["parameters"]["return_timestamps"], true);
    assert_eq!(seen[1].body["model"], "Qwen/Qwen3-8B:cheapest");
}

#[test]
fn status_codes_map_to_typed_errors_without_leaking_token() {
    let gen_req = InferenceRequest::Generate {
        model: "m".into(),
        prompt: "p".into(),
    };
    let leak = format!("echo {TOKEN}");
    for (status, code, retryable) in [
        (401, ErrorCode::Auth, false),
        (403, ErrorCode::Auth, false),
        (402, ErrorCode::Quota, false),
        (429, ErrorCode::Quota, true),
        (503, ErrorCode::Unavailable, true),
        (400, ErrorCode::InvalidInput, false),
    ] {
        let (url, _) = mock_server(vec![(status, leak.clone())]);
        let e = backend(&url, 0, true).run(&gen_req, &|| false).unwrap_err();
        let p = e.to_port_error();
        assert_eq!((p.code, p.retryable), (code, retryable), "{status}");
        assert!(
            !format!("{e:?}{p:?}{}", p.to_json()).contains(TOKEN),
            "{status} leaked token"
        );
    }
    let (url, _) = mock_server(vec![(200, "not json".into())]);
    assert!(matches!(
        backend(&url, 0, true).run(&gen_req, &|| false),
        Err(InferenceError::Protocol(_))
    ));
}

#[test]
fn retries_only_retryable_then_succeeds() {
    let gen_req = InferenceRequest::Generate {
        model: "m".into(),
        prompt: "p".into(),
    };
    let ok = r#"{"choices":[{"message":{"content":"ok"}}]}"#.to_string();
    let (url, seen) = mock_server(vec![(429, "slow".into()), (502, "x".into()), (200, ok)]);
    assert!(backend(&url, 3, true).run(&gen_req, &|| false).is_ok());
    assert_eq!(seen.lock().unwrap().len(), 3);
    let (url, seen) = mock_server(vec![(401, "no".into()), (200, "{}".into())]);
    assert!(backend(&url, 3, true).run(&gen_req, &|| false).is_err());
    assert_eq!(seen.lock().unwrap().len(), 1);
    let (url, seen) = mock_server(vec![(503, "a".into()), (503, "b".into())]);
    let e = backend(&url, 1, true).run(&gen_req, &|| false).unwrap_err();
    assert!(matches!(e, InferenceError::Unavailable(_)));
    assert_eq!(seen.lock().unwrap().len(), 2);
}

#[test]
fn cancel_missing_token_unsupported_and_offline() {
    let gen_req = InferenceRequest::Generate {
        model: "m".into(),
        prompt: "p".into(),
    };
    let mut hf = backend("http://127.0.0.1:9", 0, false);
    assert_eq!(hf.run(&gen_req, &|| true), Err(InferenceError::Cancelled));
    assert!(matches!(
        hf.run(&gen_req, &|| false),
        Err(InferenceError::Auth(_))
    ));
    contract::check_unsupported(
        "diarize",
        hf.run(
            &InferenceRequest::Diarize {
                model: "d".into(),
                audio: vec![],
            },
            &|| false,
        ),
    );
    let mut offline = backend("http://127.0.0.1:9", 0, true);
    let e = offline.run(&gen_req, &|| false).unwrap_err();
    assert_eq!(e.to_port_error().code, ErrorCode::Unavailable);
    assert_eq!(hf.estimate_usd(&gen_req), None);
}

mod executors {
    use super::*;
    use kinekino_inference::executor::{
        Frame, PcmTrack, SpeechRegion, StepError, TranscribeJob, diarize, ocr_frames, translate,
    };
    use kinekino_inference::settings::Budget;
    use kinekino_inference::{Chunk, Consent, ConsentScope, CostGuard};

    fn job(model: &str) -> TranscribeJob<'_> {
        TranscribeJob {
            project: "p",
            job: "j",
            month: "2026-10",
            model,
            confirmed: true,
        }
    }

    fn consent() -> Consent {
        let mut c = Consent::default();
        c.grant(ConsentScope::Project, "p", "j");
        c
    }

    fn chat(c: &str) -> String {
        format!(r#"{{"choices":[{{"message":{{"role":"assistant","content":"{c}"}}}}]}}"#)
    }

    #[test]
    fn translate_over_loopback_hf() {
        let (url, seen) = mock_server(vec![(200, chat("안녕")), (200, chat("잘 가"))]);
        let mut hf = backend(&url, 0, true);
        let cues = vec![
            Chunk {
                start: 0.0,
                end: 1.0,
                text: "hello".into(),
            },
            Chunk {
                start: 1.0,
                end: 2.0,
                text: "bye".into(),
            },
        ];
        let mut guard = CostGuard::new(Budget::default());
        let t = translate(
            &mut hf,
            &mut guard,
            &consent(),
            &job("Qwen/Qwen3-8B:cheapest"),
            "Korean",
            &cues,
            &|| false,
        )
        .unwrap();
        assert_eq!(t.cues[1].text, "잘 가");
        assert_eq!((t.cues[1].start, t.cues[1].end), (1.0, 2.0));
        let seen = seen.lock().unwrap();
        assert_eq!(seen[0].path, "/v1/chat/completions");
        let prompt = seen[0].body["messages"][0]["content"].as_str().unwrap();
        assert!(prompt.contains("Korean") && prompt.contains("<cue>hello</cue>"));
    }

    #[test]
    fn ocr_over_loopback_hf_and_auth_failure_keeps_partial() {
        let (url, _) = mock_server(vec![
            (200, chat("TITLE")),
            (200, chat("TITLE")),
            (401, "no".into()),
        ]);
        let mut hf = backend(&url, 0, true);
        let frames: Vec<Frame> = (0..3)
            .map(|i| Frame {
                time: i as f64,
                png: vec![i],
            })
            .collect();
        let mut guard = CostGuard::new(Budget::default());
        let e = ocr_frames(
            &mut hf,
            &mut guard,
            &consent(),
            &job("Qwen/Qwen2.5-VL-7B-Instruct:fastest"),
            &frames,
            1.0,
            &|| false,
        )
        .unwrap_err();
        let StepError::Failed {
            index,
            error,
            partial,
        } = e
        else {
            panic!()
        };
        assert_eq!(index, 2);
        assert_eq!(error.to_port_error().code, ErrorCode::Auth);
        assert_eq!(partial.spans.len(), 1);
        assert_eq!((partial.spans[0].start, partial.spans[0].end), (0.0, 2.0));
        assert!(!format!("{error:?}").contains(TOKEN));
    }

    #[test]
    fn diarize_is_unsupported_on_hf_without_network() {
        let (url, seen) = mock_server(vec![]);
        let mut hf = backend(&url, 0, true);
        let track = PcmTrack {
            sample_rate: 16_000,
            samples: vec![0; 32_000],
        };
        let mut guard = CostGuard::new(Budget::default());
        let e = diarize(
            &mut hf,
            &mut guard,
            &consent(),
            &job("pyannote/x"),
            &track,
            &[SpeechRegion {
                start: 0.0,
                end: 1.0,
            }],
            &|| false,
        )
        .unwrap_err();
        assert!(
            matches!(
                e,
                StepError::Failed {
                    index: 0,
                    error: InferenceError::Unsupported(_),
                    ..
                }
            ),
            "{e:?}"
        );
        assert!(seen.lock().unwrap().is_empty());
    }
}
