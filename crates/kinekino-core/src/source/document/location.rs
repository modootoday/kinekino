//! Source page locations. Generic URL hygiene lives here; the site-specific host,
//! category paths and resource layout come from the installed `LocationRules`.
use super::{CatalogError, ErrorKind};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Airing,
    Completed,
    Movies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    Category,
    Series,
    Episode,
}

/// Site layout supplied by the active source adapter.
pub trait LocationRules: Send + Sync {
    fn host(&self) -> &str;
    fn category_path(&self, category: Category) -> &str;
    /// Classifies non-category path segments (already non-empty checked by caller).
    fn classify(&self, segments: &[&str]) -> Option<ResourceKind>;
    fn stable_prefix(&self) -> &str;
}

static RULES: OnceLock<Box<dyn LocationRules>> = OnceLock::new();

/// Installs the process-wide location rules. Returns `false` if already installed.
pub fn install_rules(rules: Box<dyn LocationRules>) -> bool {
    RULES.set(rules).is_ok()
}

fn rules() -> Option<&'static dyn LocationRules> {
    RULES.get().map(|r| r.as_ref())
}

const CATEGORIES: [Category; 3] = [Category::Airing, Category::Completed, Category::Movies];

impl Category {
    /// Category page on the installed source; a typed error (never a panic) when no
    /// source plugin has installed location rules in this process.
    pub fn location(self) -> Result<PageLocation, CatalogError> {
        let rules = rules().ok_or_else(|| {
            CatalogError::new(
                ErrorKind::InvalidLocation,
                "No source location rules installed (no document source plugin)",
            )
        })?;
        PageLocation::parse(&format!(
            "https://{}/{}",
            rules.host(),
            rules.category_path(self)
        ))
    }
}

#[derive(Debug, Clone)]
pub struct PageLocation {
    url: Url,
    kind: ResourceKind,
}
impl PageLocation {
    pub fn parse(raw: &str) -> Result<Self, CatalogError> {
        let invalid =
            || CatalogError::new(ErrorKind::InvalidLocation, "Unsupported catalog location");
        let rules = rules().ok_or_else(invalid)?;
        if raw.len() > 8192 {
            return Err(invalid());
        }
        let url = Url::parse(raw).map_err(|_| invalid())?;
        if url.scheme() != "https"
            || url.host_str() != Some(rules.host())
            || url.port_or_known_default() != Some(443)
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
        {
            return Err(invalid());
        }
        let segments = url.path_segments().ok_or_else(invalid)?.collect::<Vec<_>>();
        let category = CATEGORIES.iter().any(|c| {
            let mut candidate = url.clone();
            candidate.set_path(&format!("/{}", rules.category_path(*c)));
            url.path() == candidate.path()
        });
        let kind = if category {
            ResourceKind::Category
        } else if segments.len() >= 2 && segments[1..].iter().all(|segment| !segment.is_empty()) {
            rules.classify(&segments).ok_or_else(invalid)?
        } else {
            return Err(invalid());
        };
        for (key, value) in url.query_pairs() {
            if key != "page"
                || value
                    .parse::<u32>()
                    .ok()
                    .filter(|v| *v > 0 && *v <= 10000)
                    .is_none()
            {
                return Err(invalid());
            }
        }
        if url.query() == Some("")
            || url.query_pairs().count() > 1
            || kind != ResourceKind::Category && url.query().is_some()
        {
            return Err(invalid());
        }
        Ok(Self { url, kind })
    }
    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }
    pub fn kind(&self) -> ResourceKind {
        self.kind
    }
    pub fn resolve(&self, href: &str) -> Result<Self, CatalogError> {
        let url = self
            .url
            .join(href)
            .map_err(|_| CatalogError::new(ErrorKind::InvalidLocation, "Invalid catalog link"))?;
        Self::parse(url.as_str())
    }
    pub fn stable_id(&self) -> String {
        let prefix = rules().map(|r| r.stable_prefix()).unwrap_or("source");
        format!("{prefix}:{}", self.url.path())
    }
}
