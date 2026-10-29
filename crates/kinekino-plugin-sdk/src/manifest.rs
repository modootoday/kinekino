//! `plugin.json` schema and validation (ADR 0011 decision 3). Discovery and OS paths are
//! host concerns and live in `kinekino-source-host`.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Component, Path};

pub const MANIFEST_FILE: &str = "plugin.json";

/// Capability names a manifest may declare, in protocol order.
pub const CAPABILITIES: [&str; 5] = ["browse", "search", "series", "episode", "resolve"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub id: String,
    pub version: String,
    /// Protocol family and major, e.g. `source/1`.
    pub api: String,
    pub capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub media_hosts: Vec<String>,
    /// Relative to the plugin directory.
    pub executable: String,
    /// Present when the plugin also serves `document/1`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<DocumentSpec>,
    /// minisign key the executable is signed with (signature verification design G1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature>,
    /// Lowercase hex SHA-256 of the executable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_sha256: Option<String>,
}

/// Site layout a document plugin declares; the host installs it as location rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentSpec {
    pub host: String,
    pub categories: Categories,
    /// First path segment → `series` | `episode`.
    pub kinds: BTreeMap<String, String>,
    pub stable_prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Categories {
    pub airing: String,
    pub completed: String,
    pub movies: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Signature {
    /// minisign key id, 16 hex digits.
    pub key_id: String,
}

fn hex(s: &str, len: usize, lower: bool) -> bool {
    s.len() == len
        && s.bytes().all(|b| {
            b.is_ascii_digit()
                || (b'a'..=b'f').contains(&b)
                || (!lower && (b'A'..=b'F').contains(&b))
        })
}

impl Manifest {
    pub fn parse(json: &str) -> Result<Self, String> {
        let m: Self = serde_json::from_str(json).map_err(|e| format!("manifest: {e}"))?;
        m.validate()?;
        Ok(m)
    }

    /// Structural checks only; protocol compatibility is the host's decision.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() || self.id.contains(['/', '\\']) {
            return Err("manifest: invalid id".into());
        }
        let exe = Path::new(&self.executable);
        if self.executable.is_empty()
            || exe.is_absolute()
            || exe
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::Prefix(_)))
        {
            return Err("manifest: executable must be relative to the plugin dir".into());
        }
        if let Some(c) = self
            .capabilities
            .iter()
            .find(|c| !CAPABILITIES.contains(&c.as_str()))
        {
            return Err(format!("manifest: unknown capability {c}"));
        }
        if let Some(doc) = &self.document
            && doc.kinds.values().any(|k| k != "series" && k != "episode")
        {
            return Err("manifest: document kinds must be series or episode".into());
        }
        if self
            .signature
            .as_ref()
            .is_some_and(|s| !hex(&s.key_id, 16, false))
        {
            return Err("manifest: signature.key_id must be 16 hex digits".into());
        }
        if self
            .entry_sha256
            .as_deref()
            .is_some_and(|h| !hex(h, 64, true))
        {
            return Err("manifest: entry_sha256 must be 64 lowercase hex digits".into());
        }
        Ok(())
    }
}
