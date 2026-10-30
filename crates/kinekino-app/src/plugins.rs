//! `kinekino plugins list`: discovered source plugins as JSON (ADR 0011 M3).
use crate::Env;
use kinekino_source_host::plugin_host::default_dir;
use kinekino_source_host::plugin_host::{PluginManifest, discover};
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
