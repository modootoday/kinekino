//! Declared model provenance, never runtime admission or quality approval.
use crate::domain::{
    api_values::DecimalU64,
    identifiers::{valid_id, valid_sha256},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelTask {
    Asr,
    Text,
    Ocr,
    Subject,
    Depth,
    SpeakerSegmentation,
    SpeakerEmbedding,
    Vad,
    Alignment,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFormat {
    Safetensors,
    Gguf,
    Onnx,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelFile {
    pub path: String,
    pub bytes: DecimalU64,
    pub sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelOrigin {
    pub repo_id: String,
    pub revision: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelConversion {
    pub recipe: String,
    pub version: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelLicense {
    pub identifier: String,
    pub reference: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredMemory {
    pub ram_bytes: DecimalU64,
    pub vram_bytes: DecimalU64,
    pub evidence_id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelManifest {
    pub schema_version: String,
    pub model_id: String,
    pub task: ModelTask,
    pub repo_id: String,
    pub revision: String,
    pub files: Vec<ModelFile>,
    pub format: ModelFormat,
    pub quantization: String,
    pub source_model: Option<ModelOrigin>,
    pub conversion: Option<ModelConversion>,
    pub tokenizer_files: Vec<String>,
    pub config_files: Vec<String>,
    pub license: ModelLicense,
    pub supported_languages: Vec<String>,
    pub runtime: String,
    pub min_runtime_version: String,
    pub declared_measured_memory: Option<DeclaredMemory>,
    pub evaluation_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelCatalog {
    pub schema_version: String,
    pub models: Vec<ModelManifest>,
}
pub fn fixed_revision(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}
fn repo(value: &str) -> bool {
    value.split('/').count() == 2
        && value.split('/').all(|part| {
            !part.is_empty()
                && part.len() <= 96
                && part != "."
                && part != ".."
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        })
}
pub fn relative_model_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.split('/').all(|part| {
            let base = part.split('.').next().unwrap_or("").to_ascii_uppercase();
            !part.is_empty()
                && part != "."
                && part != ".."
                && !part.ends_with('.')
                && !matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                && !(base.len() == 4
                    && (base.starts_with("COM") || base.starts_with("LPT"))
                    && matches!(base.as_bytes()[3], b'1'..=b'9'))
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        })
}
impl ModelManifest {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != "model-manifest/1"
            || !valid_id(&self.model_id)
            || !relative_model_path(&self.model_id)
            || self.model_id.contains('/')
            || !repo(&self.repo_id)
            || !fixed_revision(&self.revision)
            || self.files.is_empty()
            || self.files.len() > 128
            || !text(&self.quantization, 80)
            || !text(&self.runtime, 80)
            || !text(&self.min_runtime_version, 80)
            || !text(&self.license.identifier, 128)
            || self.supported_languages.len() > 128
            || self.supported_languages.iter().any(|s| !text(s, 32))
        {
            return Err("Invalid fixed model manifest");
        }
        let url =
            url::Url::parse(&self.license.reference).map_err(|_| "Invalid license reference")?;
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || self.license.reference.len() > 8192
        {
            return Err("Invalid license reference");
        }
        let mut paths: BTreeSet<String> = BTreeSet::new();
        let mut total = 0u64;
        for file in &self.files {
            let folded = file.path.to_ascii_lowercase();
            if !relative_model_path(&file.path)
                || !valid_sha256(&file.sha256)
                || file.bytes.0 == 0
                || paths.iter().any(|other| {
                    folded.starts_with(&format!("{other}/"))
                        || other.starts_with(&format!("{folded}/"))
                })
                || !paths.insert(folded)
            {
                return Err("Invalid or duplicate model file");
            }
            total = total
                .checked_add(file.bytes.0)
                .ok_or("Model size overflow")?;
        }
        for names in [&self.tokenizer_files, &self.config_files] {
            if names.len() > 128
                || names
                    .iter()
                    .any(|name| !self.files.iter().any(|file| &file.path == name))
            {
                return Err("Tokenizer/config must reference declared files");
            }
        }
        if self
            .source_model
            .as_ref()
            .is_some_and(|m| !repo(&m.repo_id) || !fixed_revision(&m.revision))
            || self
                .conversion
                .as_ref()
                .is_some_and(|c| !text(&c.recipe, 256) || !text(&c.version, 80))
            || self
                .declared_measured_memory
                .as_ref()
                .is_some_and(|m| !text(&m.evidence_id, 128))
            || self.evaluation_id.as_ref().is_some_and(|s| !text(s, 128))
        {
            return Err("Invalid declared provenance");
        }
        Ok(())
    }
}
impl ModelCatalog {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != "model-catalog/1" || self.models.len() > 64 {
            return Err("Invalid catalog version/size");
        }
        let mut ids = BTreeSet::new();
        for model in &self.models {
            model.validate()?;
            if !ids.insert((model.model_id.to_ascii_lowercase(), model.revision.clone())) {
                return Err("Duplicate model revision");
            }
        }
        Ok(())
    }
}
