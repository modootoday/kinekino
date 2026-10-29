//! `settings.inference` (settings page + `kinekino settings` share this schema). No secrets:
//! the token lives only in `CredentialStore` under [`TOKEN_SERVICE`].
use crate::consent::ConsentScope;
use serde::{Deserialize, Serialize};

pub const TOKEN_SERVICE: &str = "kinekino.huggingface";
pub const TOKEN_ACCOUNT: &str = "default";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InferenceMode {
    #[default]
    Cloud,
    LocalFirst,
    LocalOnly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Models {
    pub asr: String,
    pub ocr: String,
    pub segment: String,
    pub translate: String,
    pub summarize: String,
}

impl Default for Models {
    fn default() -> Self {
        Self {
            asr: "openai/whisper-large-v3".into(),
            ocr: "Qwen/Qwen2.5-VL-7B-Instruct:fastest".into(),
            segment: "Qwen/Qwen2.5-VL-7B-Instruct:fastest".into(),
            translate: "Qwen/Qwen3-8B:cheapest".into(),
            summarize: "Qwen/Qwen3-8B:cheapest".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Budget {
    pub total_usd: f64,
    pub per_call_usd: f64,
    pub monthly_usd: f64,
    pub confirm_ratio: f64,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            total_usd: 5.0,
            per_call_usd: 0.5,
            monthly_usd: 10.0,
            confirm_ratio: 0.8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Network {
    pub connect_timeout_s: u64,
    pub total_timeout_s: u64,
    pub max_retries: u32,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            connect_timeout_s: 10,
            total_timeout_s: 120,
            max_retries: 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiarizationMode {
    #[default]
    Off,
    Local,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Consent {
    pub scope: ConsentScope,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Diarization {
    pub mode: DiarizationMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct InferenceSettings {
    pub mode: InferenceMode,
    pub models: Models,
    pub budget: Budget,
    pub consent: Consent,
    pub local_packs: Vec<String>,
    pub network: Network,
    pub diarization: Diarization,
}

impl InferenceSettings {
    /// Validation errors as `key: reason` (settings page and CLI show them verbatim).
    pub fn validate(&self) -> Vec<String> {
        let mut e = Vec::new();
        let b = &self.budget;
        for (k, v) in [
            ("budget.total_usd", b.total_usd),
            ("budget.per_call_usd", b.per_call_usd),
            ("budget.monthly_usd", b.monthly_usd),
        ] {
            if !(v.is_finite() && v >= 0.0) {
                e.push(format!("{k}: must be a non-negative number"));
            }
        }
        if !(b.confirm_ratio > 0.0 && b.confirm_ratio <= 1.0) {
            e.push("budget.confirm_ratio: must be in (0, 1]".into());
        }
        if b.per_call_usd > b.total_usd {
            e.push("budget.per_call_usd: exceeds budget.total_usd".into());
        }
        let n = &self.network;
        if n.connect_timeout_s == 0 || n.total_timeout_s < n.connect_timeout_s {
            e.push("network: need 0 < connect_timeout_s <= total_timeout_s".into());
        }
        if n.max_retries > 10 {
            e.push("network.max_retries: at most 10".into());
        }
        let m = &self.models;
        for (k, v) in [
            ("models.asr", &m.asr),
            ("models.ocr", &m.ocr),
            ("models.segment", &m.segment),
            ("models.translate", &m.translate),
            ("models.summarize", &m.summarize),
        ] {
            if v.trim().is_empty() || v.chars().any(char::is_whitespace) {
                e.push(format!("{k}: model id must be non-empty without spaces"));
            }
        }
        if self.mode == InferenceMode::LocalOnly && self.local_packs.is_empty() {
            e.push("mode: local_only needs at least one installed local pack".into());
        }
        e
    }

    /// Dotted-key read (`kinekino settings get inference.<key>`).
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        let mut v = serde_json::to_value(self).ok()?;
        for part in key.split('.') {
            v = v.get(part)?.clone();
        }
        Some(v)
    }

    /// Dotted-key write; `raw` is JSON, or a bare string. Rejects unknown keys and
    /// values that fail [`Self::validate`].
    pub fn set(&self, key: &str, raw: &str) -> Result<Self, String> {
        if key == "token" || key.starts_with("token.") {
            return Err("token: use the secure token input, not settings".into());
        }
        let value = serde_json::from_str(raw)
            .unwrap_or_else(|_| serde_json::Value::String(raw.to_string()));
        let mut root = serde_json::to_value(self).map_err(|e| e.to_string())?;
        let mut slot = &mut root;
        for part in key.split('.') {
            slot = slot
                .get_mut(part)
                .ok_or_else(|| format!("{key}: unknown key"))?;
        }
        *slot = value;
        let next: Self = serde_json::from_value(root).map_err(|e| format!("{key}: {e}"))?;
        let errs = next.validate();
        if errs.is_empty() {
            Ok(next)
        } else {
            Err(errs.join("; "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid_and_match_plan() {
        let s = InferenceSettings::default();
        assert!(s.validate().is_empty());
        assert_eq!(s.get("budget.total_usd").unwrap(), 5.0);
        assert_eq!(s.get("mode").unwrap(), "cloud");
        assert_eq!(s.get("consent.scope").unwrap(), "project");
        assert_eq!(s.get("diarization.mode").unwrap(), "off");
    }

    #[test]
    fn set_validates_and_rejects_unknown_and_token() {
        let s = InferenceSettings::default();
        let s2 = s.set("mode", "local_first").unwrap();
        assert_eq!(s2.mode, InferenceMode::LocalFirst);
        assert!(s.set("budget.confirm_ratio", "1.5").is_err());
        assert!(s.set("nope", "1").is_err());
        assert!(s.set("token", "hf_x").is_err());
        assert!(s.set("mode", "local_only").is_err());
    }

    #[test]
    fn partial_file_fills_defaults_and_unknown_fails() {
        let s: InferenceSettings = serde_json::from_str(r#"{"budget":{"monthly_usd":3}}"#).unwrap();
        assert_eq!(s.budget.monthly_usd, 3.0);
        assert_eq!(s.budget.total_usd, 5.0);
        assert!(serde_json::from_str::<InferenceSettings>(r#"{"token":"x"}"#).is_err());
    }
}
