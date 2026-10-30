//! Catalog pages come from a `document/1` source plugin (ADR 0011 M4); no site
//! knowledge lives in this crate.
use crate::application::catalog::CatalogParser;
use kinekino_core::source::document::{
    CatalogError, CatalogPage, ErrorKind, PageLocation, Provenance, RawDocument,
};
use kinekino_source_host::plugin_host::{DocumentPlugin, default_dir, discover};
use std::sync::{Arc, OnceLock};

static PLUGIN: OnceLock<Result<Arc<DocumentPlugin>, CatalogError>> = OnceLock::new();

/// First discovered plugin with a `document` section (sorted by directory name).
pub fn document_plugin() -> Result<Arc<DocumentPlugin>, CatalogError> {
    PLUGIN
        .get_or_init(|| {
            let dir = default_dir(|k| std::env::var(k).ok()).ok_or_else(missing)?;
            let plugin = discover(&dir)
                .0
                .into_iter()
                .find(|p| p.document.is_some())
                .ok_or_else(missing)?;
            DocumentPlugin::open(plugin).map(Arc::new)
        })
        .clone()
}

fn missing() -> CatalogError {
    CatalogError::new(
        ErrorKind::Transport,
        "no document source plugin installed (see `kinekino plugins list`)",
    )
}

/// Parser backed by the installed plugin; without one every parse is a typed error.
pub struct PluginParser;

impl CatalogParser for PluginParser {
    fn revision(&self) -> &str {
        static NONE: &str = "no-document-plugin";
        match PLUGIN.get() {
            Some(Ok(p)) => p.revision(),
            _ => NONE,
        }
    }
    fn parse(&self, raw: &RawDocument) -> Result<CatalogPage, CatalogError> {
        document_plugin()?.parse(raw)
    }
    fn parse_html(
        &self,
        location: &PageLocation,
        html: &str,
        provenance: Provenance,
    ) -> Result<CatalogPage, CatalogError> {
        document_plugin()?.parse_html(location, html, provenance)
    }
}
