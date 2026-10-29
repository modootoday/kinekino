use kinekino_core::source::{SOURCE_API, SourceCapability, SourceManifest};
pub use kinekino_plugin_sdk::MANIFEST_FILE;
use kinekino_plugin_sdk::Manifest;
pub use kinekino_plugin_sdk::manifest::{Categories, DocumentSpec};
use std::path::{Path, PathBuf};

/// A discovered plugin: its source manifest plus the executable to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginManifest {
    pub source: SourceManifest,
    pub executable: PathBuf,
    pub dir: PathBuf,
    /// Present when the plugin also serves the `document/1` page protocol.
    pub document: Option<DocumentSpec>,
}

fn capability(name: &str) -> Option<SourceCapability> {
    Some(match name {
        "browse" => SourceCapability::Browse,
        "search" => SourceCapability::Search,
        "series" => SourceCapability::Series,
        "episode" => SourceCapability::Episode,
        "resolve" => SourceCapability::Resolve,
        _ => return None,
    })
}

impl PluginManifest {
    /// Parses and validates with the SDK schema, then maps to core values.
    pub fn parse(dir: &Path, json: &str) -> Result<Self, String> {
        let raw = Manifest::parse(json)?;
        let capabilities = raw
            .capabilities
            .iter()
            .map(|c| capability(c).ok_or_else(|| format!("manifest: unknown capability {c}")))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            source: SourceManifest {
                id: raw.id,
                version: raw.version,
                api: raw.api,
                capabilities,
                media_hosts: raw.media_hosts,
            },
            executable: dir.join(&raw.executable),
            dir: dir.to_path_buf(),
            document: raw.document,
        })
    }
}

/// Scans `root/*/plugin.json`. Invalid or incompatible manifests are reported, not loaded.
pub fn discover(root: &Path) -> (Vec<PluginManifest>, Vec<(PathBuf, String)>) {
    let (mut ok, mut bad) = (Vec::new(), Vec::new());
    let Ok(entries) = std::fs::read_dir(root) else {
        return (ok, bad);
    };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    for dir in dirs {
        let file = dir.join(MANIFEST_FILE);
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        match PluginManifest::parse(&dir, &text) {
            Ok(m) if m.source.api == SOURCE_API => ok.push(m),
            Ok(m) => bad.push((file, format!("incompatible api {}", m.source.api))),
            Err(e) => bad.push((file, e)),
        }
    }
    (ok, bad)
}

/// Plugin root: `KINEKINO_PLUGIN_DIR`, else `%APPDATA%\kinekino\plugins`, else
/// `$HOME/.local/share/kinekino/plugins`. `get` reads one environment variable.
pub fn default_dir(get: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    let get = |k: &str| get(k).filter(|v| !v.is_empty());
    if let Some(dir) = get("KINEKINO_PLUGIN_DIR") {
        return Some(PathBuf::from(dir));
    }
    if let Some(appdata) = get("APPDATA") {
        return Some(PathBuf::from(appdata).join("kinekino").join("plugins"));
    }
    get("HOME").map(|h| PathBuf::from(h).join(".local/share/kinekino/plugins"))
}
