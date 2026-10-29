use kinekino_core::port::Adapter;
use kinekino_core::port::registry::{Availability, Registration, Role};
use kinekino_inference::port::Output;
use kinekino_inference::port::require_task;
use kinekino_inference::registry::{DynInference, InferenceRegistry, everywhere};
use kinekino_inference::{
    Chunk, InferenceBackend, InferenceCaps, InferenceError, InferenceKind, InferenceRequest,
    InferenceResponse, InferenceSettings, Provenance, Task,
};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const LOCAL_CAPS: InferenceCaps = InferenceCaps::of(&[Task::Transcribe]);

#[derive(Debug, Clone, PartialEq)]
pub struct WhisperConfig {
    /// `whisper-cli` executable.
    pub program: PathBuf,
    /// ggml model file (`ggml-large-v3-turbo.bin` by default pack).
    pub model: PathBuf,
    /// Spoken language or `auto`.
    pub language: String,
    pub threads: u32,
    pub timeout: Duration,
    /// Scratch directory for the WAV cut and JSON output (one subdir per call).
    pub work_dir: PathBuf,
}

#[derive(Debug)]
pub struct WhisperCliBackend {
    config: WhisperConfig,
    calls: u64,
}

impl WhisperCliBackend {
    pub fn new(config: WhisperConfig) -> Self {
        Self { config, calls: 0 }
    }

    pub fn availability(config: &WhisperConfig) -> Availability {
        if !config.program.is_file() {
            Availability::Missing(format!(
                "whisper-cli not found: {}",
                config.program.display()
            ))
        } else if !config.model.is_file() {
            Availability::Missing(format!("model not found: {}", config.model.display()))
        } else {
            Availability::Available
        }
    }

    fn model_name(&self) -> String {
        self.config
            .model
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    fn invoke(
        &mut self,
        audio: &[u8],
        cancel: &dyn Fn() -> bool,
    ) -> Result<Output, InferenceError> {
        self.calls += 1;
        let dir =
            self.config
                .work_dir
                .join(format!("whisper-{}-{}", std::process::id(), self.calls));
        std::fs::create_dir_all(&dir).map_err(|e| unavailable("work dir", &e))?;
        let result = self.invoke_in(&dir, audio, cancel);
        let _ = std::fs::remove_dir_all(&dir);
        result
    }

    fn invoke_in(
        &self,
        dir: &Path,
        audio: &[u8],
        cancel: &dyn Fn() -> bool,
    ) -> Result<Output, InferenceError> {
        let wav = dir.join("in.wav");
        std::fs::write(&wav, audio).map_err(|e| unavailable("write wav", &e))?;
        let stem = dir.join("out");
        let c = &self.config;
        let mut child = Command::new(&c.program)
            .arg("-m")
            .arg(&c.model)
            .arg("-f")
            .arg(&wav)
            .args([
                "-l",
                &c.language,
                "-t",
                &c.threads.max(1).to_string(),
                "-oj",
                "-np",
                "-of",
            ])
            .arg(&stem)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| unavailable("spawn whisper-cli", &e))?;
        let started = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().map_err(|e| unavailable("wait", &e))? {
                break status;
            }
            let stop = if cancel() {
                Some(InferenceError::Cancelled)
            } else if started.elapsed() >= c.timeout {
                Some(InferenceError::Timeout(format!(
                    "whisper-cli after {:?}",
                    c.timeout
                )))
            } else {
                None
            };
            if let Some(err) = stop {
                let _ = child.kill();
                let _ = child.wait();
                return Err(err);
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        if !status.success() {
            let mut stderr = String::new();
            if let Some(mut pipe) = child.stderr.take() {
                let _ = std::io::Read::read_to_string(&mut pipe, &mut stderr);
            }
            let tail: String = stderr.lines().rev().take(3).collect::<Vec<_>>().join(" | ");
            return Err(InferenceError::Unavailable(format!(
                "whisper-cli exited {status}: {tail}"
            )));
        }
        let json = std::fs::read_to_string(stem.with_extension("json"))
            .map_err(|e| InferenceError::Protocol(format!("whisper-cli json missing: {e}")))?;
        parse_whisper_json(&json)
    }
}

fn unavailable(what: &str, e: &std::io::Error) -> InferenceError {
    InferenceError::Unavailable(format!("{what}: {e}"))
}

/// Parses whisper.cpp `-oj` output: `transcription[].offsets.{from,to}` (ms) and `text`.
pub fn parse_whisper_json(json: &str) -> Result<Output, InferenceError> {
    let bad = |m: &str| InferenceError::Protocol(format!("whisper json: {m}"));
    let doc: Value = serde_json::from_str(json).map_err(|e| bad(&e.to_string()))?;
    let items = doc["transcription"]
        .as_array()
        .ok_or_else(|| bad("no transcription array"))?;
    let mut chunks = Vec::with_capacity(items.len());
    for item in items {
        let ms = |k: &str| {
            item["offsets"][k]
                .as_f64()
                .ok_or_else(|| bad("offsets missing"))
        };
        let (start, end) = (ms("from")? / 1000.0, ms("to")? / 1000.0);
        if end < start {
            return Err(bad("offset end before start"));
        }
        let text = item["text"]
            .as_str()
            .ok_or_else(|| bad("text missing"))?
            .trim();
        if !text.is_empty() {
            chunks.push(Chunk {
                start,
                end,
                text: text.into(),
            });
        }
    }
    let text = chunks
        .iter()
        .map(|c| c.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    Ok(Output::Transcribe { text, chunks })
}

impl Adapter for WhisperCliBackend {
    type Kind = InferenceKind;
    type Caps = InferenceCaps;
    fn kind(&self) -> InferenceKind {
        InferenceKind::LocalPack
    }
    fn capabilities(&self) -> InferenceCaps {
        LOCAL_CAPS
    }
}

impl InferenceBackend for WhisperCliBackend {
    /// Local compute is free; the guard still records a $0 ledger line.
    fn estimate_usd(&self, _request: &InferenceRequest) -> Option<f64> {
        Some(0.0)
    }

    fn run(
        &mut self,
        request: &InferenceRequest,
        cancel: &dyn Fn() -> bool,
    ) -> Result<InferenceResponse, InferenceError> {
        require_task(LOCAL_CAPS, request)?;
        let InferenceRequest::Transcribe { audio, .. } = request else {
            return Err(InferenceError::Unsupported(request.task().name().into()));
        };
        if audio.len() < 44 || &audio[..4] != b"RIFF" {
            return Err(InferenceError::InvalidInput(
                "expected a WAV segment".into(),
            ));
        }
        if cancel() {
            return Err(InferenceError::Cancelled);
        }
        let output = self.invoke(audio, cancel)?;
        Ok(InferenceResponse {
            output,
            provenance: Provenance {
                backend: "local-pack/whisper.cpp".into(),
                model: self.model_name(),
                provider: None,
                cost_estimate_usd: 0.0,
                fallback_from: None,
            },
        })
    }
}

/// Registers the whisper.cpp pack as the local fallback on every OS.
pub fn register_whisper_cli(reg: &mut InferenceRegistry, config: WhisperConfig) {
    let probe = config.clone();
    reg.register(Registration::new(
        InferenceKind::LocalPack,
        LOCAL_CAPS,
        everywhere(Role::Fallback),
        move || WhisperCliBackend::availability(&probe),
        move |_cfg: &InferenceSettings| {
            Ok(Box::new(WhisperCliBackend::new(config.clone())) as Box<DynInference>)
        },
    ));
}
