//! `kinekino plugins list`: discovered source plugins as JSON (ADR 0011 M3), and the
//! player's 작품 탐색 catalog over the same plugins.
use crate::Env;
use kinekino_core::source::{
    SourceCapability, SourceError, SourceItem, SourcePage, SourceProvider, SourceRef,
};
use kinekino_player_ui::Catalog;
use kinekino_source_host::plugin_host::default_dir;
use kinekino_source_host::plugin_host::{PluginManifest, ProcessSource, discover};
use serde_json::json;
use std::path::PathBuf;

/// `KINEKINO_PLUGIN_DIR`, else the per-user data dir `kinekino/plugins`.
pub fn plugin_dir(env: &Env) -> Option<PathBuf> {
    default_dir(|k| env.get(k).map(str::to_owned))
}

pub fn discovered(env: &Env) -> Vec<PluginManifest> {
    plugin_dir(env).map(|d| discover(&d).0).unwrap_or_default()
}

pub fn list(env: &Env) -> String {
    let dir = plugin_dir(env);
    let (ok, bad) = dir.as_deref().map(discover).unwrap_or_default();
    let plugins: Vec<_> = ok
        .iter()
        .map(|p| {
            json!({
                "id": p.source.id,
                "version": p.source.version,
                "api": p.source.api,
                "capabilities": p.source.capabilities.iter().map(|c| format!("{c:?}").to_lowercase()).collect::<Vec<_>>(),
                "media_hosts": p.source.media_hosts,
                "executable": p.executable,
                "document": p.document.is_some(),
            })
        })
        .collect();
    let rejected: Vec<_> = bad
        .iter()
        .map(|(f, e)| json!({"manifest": f, "error": e}))
        .collect();
    json!({"dir": dir, "plugins": plugins, "rejected": rejected}).to_string()
}

/// 작품 탐색 over the discovered plugins that declare `browse`; one process per call.
pub struct PluginCatalog {
    sources: Vec<ProcessSource>,
}

impl PluginCatalog {
    pub fn discover(env: &Env) -> Self {
        let sources = discovered(env)
            .into_iter()
            .filter(|p| p.source.supports(SourceCapability::Browse))
            .map(ProcessSource::new)
            .collect();
        Self { sources }
    }
}

impl Catalog for PluginCatalog {
    fn sources(&self) -> Vec<SourceItem> {
        self.sources
            .iter()
            .map(|s| {
                let id = s.manifest().id.clone();
                SourceItem {
                    reference: SourceRef {
                        provider: id.clone(),
                        id: String::new(),
                    },
                    title: id,
                }
            })
            .collect()
    }

    fn browse(&self, at: &SourceRef, page: u32) -> Result<SourcePage, SourceError> {
        self.sources
            .iter()
            .find(|s| s.manifest().id == at.provider)
            .ok_or(SourceError::NotFound)?
            .browse(at, page)
    }
}
