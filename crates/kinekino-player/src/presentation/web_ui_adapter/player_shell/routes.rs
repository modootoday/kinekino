use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteId(String);
impl RouteId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteKind {
    Viewing,
    Studio,
}
#[derive(Clone, Debug)]
pub struct RouteDescriptor {
    id: RouteId,
    kind: RouteKind,
    path: String,
    workspace: Option<String>,
    source: Option<String>,
}
impl RouteDescriptor {
    pub fn id(&self) -> &RouteId {
        &self.id
    }
    pub fn kind(&self) -> RouteKind {
        self.kind
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn workspace(&self) -> Option<&str> {
        self.workspace.as_deref()
    }
    /// A declaration, never proof of parsing, runtime registration or mount.
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum CatalogError {
    InvalidJson,
    UnsupportedVersion,
    NotDeclarationCatalog,
    InvalidRoute,
    DuplicateRoute,
    EmptyCatalog,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteWire {
    id: String,
    path: String,
    source: Option<String>,
    workspace: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogWire {
    schema_version: u32,
    status: String,
    router_owner: String,
    view_routes: Vec<RouteWire>,
    studio_routes: Vec<RouteWire>,
    #[serde(rename = "selection_query_keys")]
    _selection_query_keys: Vec<String>,
    #[serde(rename = "forbidden_url_payloads")]
    _forbidden_url_payloads: Vec<String>,
    #[serde(rename = "scope_audit")]
    _scope_audit: serde_json::Value,
}
pub struct RouteCatalog {
    entries: BTreeMap<String, RouteDescriptor>,
}
impl RouteCatalog {
    pub fn bundled() -> Result<Self, CatalogError> {
        Self::parse(include_str!("../../web_ui/routing/routes.json"))
    }
    pub(super) fn parse(json: &str) -> Result<Self, CatalogError> {
        let catalog: CatalogWire =
            serde_json::from_str(json).map_err(|_| CatalogError::InvalidJson)?;
        if catalog.schema_version != 1 {
            return Err(CatalogError::UnsupportedVersion);
        }
        if catalog.status != "product-route-contract-not-runtime-registered"
            || catalog.router_owner != "meari-react-router-compatibility"
        {
            return Err(CatalogError::NotDeclarationCatalog);
        }
        let mut entries = BTreeMap::new();
        for (kind, routes) in [
            (RouteKind::Viewing, catalog.view_routes),
            (RouteKind::Studio, catalog.studio_routes),
        ] {
            for route in routes {
                let prefix = match kind {
                    RouteKind::Viewing => "view.",
                    RouteKind::Studio => "studio.",
                };
                if !route.id.starts_with(prefix)
                    || !route.id.bytes().all(|byte| {
                        byte.is_ascii_lowercase()
                            || byte.is_ascii_digit()
                            || matches!(byte, b'.' | b'-')
                    })
                    || !route.path.starts_with('/')
                    || route.path.starts_with("//")
                    || route.path.contains("://")
                    || route.source.as_ref().is_some_and(|source| {
                        !(source.starts_with("viewing/") || source.starts_with("studio/"))
                            || source.contains('\\')
                            || source
                                .split('/')
                                .any(|part| matches!(part, "" | "." | ".."))
                    })
                {
                    return Err(CatalogError::InvalidRoute);
                }
                let id = route.id.clone();
                if entries
                    .insert(
                        id,
                        RouteDescriptor {
                            id: RouteId(route.id),
                            kind,
                            path: route.path,
                            workspace: route.workspace,
                            source: route.source,
                        },
                    )
                    .is_some()
                {
                    return Err(CatalogError::DuplicateRoute);
                }
            }
        }
        if entries.is_empty() {
            return Err(CatalogError::EmptyCatalog);
        }
        Ok(Self { entries })
    }
    pub fn get(&self, id: &str) -> Option<&RouteDescriptor> {
        self.entries.get(id)
    }
    pub fn iter(&self) -> impl Iterator<Item = &RouteDescriptor> {
        self.entries.values()
    }
}
