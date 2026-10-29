use super::document_wire::{DOCUMENT_PROTOCOL, DocRequest, DocResponse, PROTOCOL_FIELD};
use super::{DocumentSpec, PluginManifest};
use kinekino_core::source::document::{
    CatalogError, CatalogFetchPort, CatalogPage, Category, EpisodeDocumentData, ErrorKind,
    LocationRules, PageLocation, Provenance, RawDocument, ResourceKind,
};
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};

/// Raw pages are bounded by the plugin; JSON byte arrays expand them, so allow headroom.
const MAX_REPLY_BYTES: u64 = 64 * 1024 * 1024;

/// Location rules taken from a plugin manifest's `document` section.
pub struct DocumentRules(DocumentSpec);

impl DocumentRules {
    pub fn new(spec: DocumentSpec) -> Self {
        Self(spec)
    }
}

impl LocationRules for DocumentRules {
    fn host(&self) -> &str {
        &self.0.host
    }
    fn category_path(&self, category: Category) -> &str {
        let c = &self.0.categories;
        match category {
            Category::Airing => &c.airing,
            Category::Completed => &c.completed,
            Category::Movies => &c.movies,
        }
    }
    fn classify(&self, segments: &[&str]) -> Option<ResourceKind> {
        match self.0.kinds.get(*segments.first()?)?.as_str() {
            "series" => Some(ResourceKind::Series),
            "episode" => Some(ResourceKind::Episode),
            _ => None,
        }
    }
    fn stable_prefix(&self) -> &str {
        &self.0.stable_prefix
    }
}

/// `document/1` client: one plugin process per call.
pub struct DocumentPlugin {
    plugin: PluginManifest,
    revision: String,
}

fn transport(e: impl std::fmt::Display) -> CatalogError {
    CatalogError::new(ErrorKind::Transport, format!("document plugin: {e}"))
}

impl DocumentPlugin {
    /// Installs the manifest's location rules (first install wins) and reads the parser revision.
    pub fn open(plugin: PluginManifest) -> Result<Self, CatalogError> {
        let spec = plugin
            .document
            .clone()
            .ok_or_else(|| transport("manifest has no document section"))?;
        let _ = kinekino_core::source::document::install_rules(Box::new(DocumentRules::new(spec)));
        let mut this = Self {
            plugin,
            revision: String::new(),
        };
        match this.call(&DocRequest::Revision)? {
            DocResponse::Revision(r) => this.revision = r,
            _ => return Err(transport("expected revision")),
        }
        Ok(this)
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }

    fn call(&self, request: &DocRequest) -> Result<DocResponse, CatalogError> {
        let mut value = serde_json::to_value(request).map_err(transport)?;
        value[PROTOCOL_FIELD] = DOCUMENT_PROTOCOL.into();
        let mut child = Command::new(&self.plugin.executable)
            .current_dir(&self.plugin.dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(transport)?;
        {
            let mut stdin = child.stdin.take().ok_or_else(|| transport("stdin"))?;
            writeln!(stdin, "{value}").map_err(transport)?;
        }
        let stdout = child.stdout.take().ok_or_else(|| transport("stdout"))?;
        let mut reply = String::new();
        let read = BufReader::new(stdout.take(MAX_REPLY_BYTES)).read_line(&mut reply);
        let _ = child.kill();
        let _ = child.wait();
        read.map_err(transport)?;
        match serde_json::from_str(reply.trim()).map_err(transport)? {
            DocResponse::Error(e) => Err(e),
            other => Ok(other),
        }
    }

    pub fn parse(&self, raw: &RawDocument) -> Result<CatalogPage, CatalogError> {
        match self.call(&DocRequest::Parse { raw: raw.clone() })? {
            DocResponse::Page(p) => Ok(p),
            _ => Err(transport("expected page")),
        }
    }

    pub fn parse_html(
        &self,
        location: &PageLocation,
        html: &str,
        provenance: Provenance,
    ) -> Result<CatalogPage, CatalogError> {
        let request = DocRequest::ParseHtml {
            location: location.as_str().into(),
            html: html.into(),
            provenance,
        };
        match self.call(&request)? {
            DocResponse::Page(p) => Ok(p),
            _ => Err(transport("expected page")),
        }
    }

    pub fn episode_document(&self, html: &[u8]) -> Result<EpisodeDocumentData, CatalogError> {
        match self.call(&DocRequest::EpisodeDocument {
            html: html.to_vec(),
        })? {
            DocResponse::Episode(e) => Ok(e),
            _ => Err(transport("expected episode document")),
        }
    }
}

impl CatalogFetchPort for DocumentPlugin {
    fn fetch(&self, location: &PageLocation) -> Result<RawDocument, CatalogError> {
        match self.call(&DocRequest::Fetch {
            location: location.as_str().into(),
        })? {
            DocResponse::Raw(r) => Ok(r),
            _ => Err(transport("expected raw document")),
        }
    }
}

/// Installs the test fixture layout (`catalog.example.test`, `/c/…` series, `/e/…`
/// episodes) when no plugin rules are installed yet. For tests and fixtures only.
pub fn install_fixture_rules() {
    let spec = DocumentSpec {
        host: "catalog.example.test".into(),
        categories: super::Categories {
            airing: "방영중".into(),
            completed: "종영".into(),
            movies: "극장판".into(),
        },
        kinds: [("c", "series"), ("e", "episode")]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect(),
        stable_prefix: "catalog".into(),
    };
    let _ = kinekino_core::source::document::install_rules(Box::new(DocumentRules::new(spec)));
}
