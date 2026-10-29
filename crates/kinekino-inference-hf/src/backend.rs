use crate::loopback::LoopbackHttp;
use crate::port::{HttpRequest, HttpTransport, TransportError};
use crate::wire;
use kinekino_core::port::Adapter;
use kinekino_core::port::credential::{CredentialStore, Secret};
use kinekino_core::port::registry::{Availability, Registration, Role};
use kinekino_inference::port::require_task;
use kinekino_inference::registry::{DynInference, InferenceRegistry, everywhere};
use kinekino_inference::settings::{TOKEN_ACCOUNT, TOKEN_SERVICE};
use kinekino_inference::{
    InferenceBackend, InferenceCaps, InferenceError, InferenceKind, InferenceRequest,
    InferenceResponse, InferenceSettings, Provenance, Task,
};
use std::time::Duration;

pub const HF_ROUTER: &str = "https://router.huggingface.co";

pub const HF_CAPS: InferenceCaps =
    InferenceCaps::of(&[Task::Transcribe, Task::Ocr, Task::Generate, Task::Embed]);

#[derive(Debug, Clone)]
pub struct HfConfig {
    pub base_url: String,
    pub connect_timeout: Duration,
    pub total_timeout: Duration,
    pub max_retries: u32,
    /// First backoff; doubles per retry.
    pub backoff: Duration,
}

impl HfConfig {
    pub fn from_settings(s: &InferenceSettings, base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            connect_timeout: Duration::from_secs(s.network.connect_timeout_s),
            total_timeout: Duration::from_secs(s.network.total_timeout_s),
            max_retries: s.network.max_retries,
            backoff: Duration::from_millis(500),
        }
    }
}

pub struct HfCloudBackend {
    config: HfConfig,
    transport: Box<dyn HttpTransport>,
    token: Option<Secret>,
}

impl HfCloudBackend {
    /// Reads the token from the credential store once; it never enters config or logs.
    pub fn new(
        config: HfConfig,
        transport: Box<dyn HttpTransport>,
        store: &dyn CredentialStore,
    ) -> Result<Self, InferenceError> {
        let token = store
            .get(TOKEN_SERVICE, TOKEN_ACCOUNT)
            .map_err(|e| InferenceError::Auth(e.message))?;
        Ok(Self {
            config,
            transport,
            token,
        })
    }

    fn redact(&self, text: &str) -> String {
        let mut t: String = text.chars().take(200).collect();
        if let Some(tok) = &self.token
            && !tok.expose().is_empty()
        {
            t = t.replace(tok.expose(), "***");
        }
        t
    }

    fn attempt(&self, req: &InferenceRequest) -> Result<InferenceResponse, InferenceError> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| InferenceError::Auth("no Hugging Face token registered".into()))?;
        let (path, body) = wire::encode(req)
            .ok_or_else(|| InferenceError::Unsupported(req.task().name().into()))?;
        let http = HttpRequest {
            url: format!("{}{path}", self.config.base_url),
            bearer: Some(token.expose().to_string()),
            body: serde_json::to_vec(&body)
                .map_err(|e| InferenceError::InvalidInput(e.to_string()))?,
        };
        let resp = self
            .transport
            .post_json(
                &http,
                self.config.connect_timeout,
                self.config.total_timeout,
            )
            .map_err(|e| match e {
                TransportError::Timeout(m) => InferenceError::Timeout(self.redact(&m)),
                TransportError::Connect(m) => InferenceError::Unavailable(self.redact(&m)),
                TransportError::Refused(m) => InferenceError::InvalidInput(self.redact(&m)),
            })?;
        let snippet = || self.redact(&String::from_utf8_lossy(&resp.body));
        match resp.status {
            200..=299 => Ok(InferenceResponse {
                output: wire::decode(req, &resp.body)?,
                provenance: Provenance {
                    backend: "hf-cloud".into(),
                    model: req.model().into(),
                    provider: req.model().rsplit_once(':').map(|(_, p)| p.to_string()),
                    cost_estimate_usd: 0.0,
                    fallback_from: None,
                },
            }),
            401 | 403 => Err(InferenceError::Auth(format!(
                "{}: {}",
                resp.status,
                snippet()
            ))),
            402 => Err(InferenceError::Budget(format!("402: {}", snippet()))),
            429 => Err(InferenceError::RateLimited(snippet())),
            408 => Err(InferenceError::Timeout(snippet())),
            500..=599 => Err(InferenceError::Unavailable(format!(
                "{}: {}",
                resp.status,
                snippet()
            ))),
            s => Err(InferenceError::InvalidInput(format!("{s}: {}", snippet()))),
        }
    }
}

impl Adapter for HfCloudBackend {
    type Kind = InferenceKind;
    type Caps = InferenceCaps;
    fn kind(&self) -> InferenceKind {
        InferenceKind::HfCloud
    }
    fn capabilities(&self) -> InferenceCaps {
        HF_CAPS
    }
}

impl InferenceBackend for HfCloudBackend {
    /// Per-task HF prices are unconfirmed (plan P1b), so the guard always asks.
    fn estimate_usd(&self, _request: &InferenceRequest) -> Option<f64> {
        None
    }

    fn run(
        &mut self,
        request: &InferenceRequest,
        cancel: &dyn Fn() -> bool,
    ) -> Result<InferenceResponse, InferenceError> {
        require_task(HF_CAPS, request)?;
        let mut delay = self.config.backoff;
        let mut attempt = 0;
        loop {
            if cancel() {
                return Err(InferenceError::Cancelled);
            }
            match self.attempt(request) {
                Err(e) if e_retryable(&e) && attempt < self.config.max_retries => {
                    attempt += 1;
                    std::thread::sleep(delay);
                    delay = delay.saturating_mul(2);
                }
                other => return other,
            }
        }
    }
}

fn e_retryable(e: &InferenceError) -> bool {
    use kinekino_core::port::ToPortError;
    e.retryable()
}

/// Registers the HF adapter as the default on every OS. The factory builds with the
/// loopback-only transport against `base_url`; the TLS transport arrives with the P7 gate.
pub fn register_hf_cloud(
    reg: &mut InferenceRegistry,
    base_url: String,
    token: impl Fn() -> Option<Secret> + 'static,
) {
    reg.register(Registration::new(
        InferenceKind::HfCloud,
        HF_CAPS,
        everywhere(Role::Default),
        || Availability::Available,
        move |cfg: &InferenceSettings| {
            let backend = HfCloudBackend {
                config: HfConfig::from_settings(cfg, &base_url),
                transport: Box::new(LoopbackHttp),
                token: token(),
            };
            Ok(Box::new(backend) as Box<DynInference>)
        },
    ));
}
