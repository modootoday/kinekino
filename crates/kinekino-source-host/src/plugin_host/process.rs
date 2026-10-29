use super::{PluginManifest, wire};
use kinekino_core::source::{
    EpisodeAdjacency, SourceCapability, SourceError, SourceItem, SourceManifest, SourcePage,
    SourceProvider, SourceRef, StreamCandidate,
};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

/// Cap on one response line; a misbehaving plugin cannot grow host memory unbounded.
const MAX_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;

pub struct ProcessSource {
    plugin: PluginManifest,
}

impl ProcessSource {
    pub fn new(plugin: PluginManifest) -> Self {
        Self { plugin }
    }

    fn call(
        &self,
        capability: SourceCapability,
        request: &wire::Request,
    ) -> Result<wire::Response, SourceError> {
        if !self.plugin.source.supports(capability) {
            return Err(SourceError::Unsupported(capability));
        }
        let unavailable = |e: std::io::Error| SourceError::Unavailable(e.to_string());
        let mut child = Command::new(&self.plugin.executable)
            .current_dir(&self.plugin.dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(unavailable)?;
        let line =
            serde_json::to_string(request).map_err(|e| SourceError::Invalid(e.to_string()))?;
        {
            let mut stdin = child
                .stdin
                .take()
                .ok_or_else(|| SourceError::Unavailable("stdin".into()))?;
            stdin.write_all(line.as_bytes()).map_err(unavailable)?;
            stdin.write_all(b"\n").map_err(unavailable)?;
        }
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| SourceError::Unavailable("stdout".into()))?;
        let mut reply = String::new();
        let read =
            BufReader::new(std::io::Read::take(stdout, MAX_RESPONSE_BYTES)).read_line(&mut reply);
        let _ = child.kill();
        let _ = child.wait();
        read.map_err(unavailable)?;
        let response: wire::Response = serde_json::from_str(reply.trim())
            .map_err(|e| SourceError::Invalid(format!("plugin reply: {e}")))?;
        match response {
            wire::Response::Error { kind, message } => Err(match kind.as_str() {
                "not_found" => SourceError::NotFound,
                "unsupported" => SourceError::Unsupported(capability),
                "invalid" => SourceError::Invalid(message),
                _ => SourceError::Unavailable(message),
            }),
            other => Ok(other),
        }
    }

    fn page(
        &self,
        capability: SourceCapability,
        request: wire::Request,
    ) -> Result<SourcePage, SourceError> {
        match self.call(capability, &request)? {
            wire::Response::Page { items, next_page } => Ok(SourcePage {
                items: items
                    .into_iter()
                    .map(|i| SourceItem {
                        reference: from_wire(i.reference),
                        title: i.title,
                    })
                    .collect(),
                next_page,
            }),
            _ => Err(SourceError::Invalid("plugin reply: expected page".into())),
        }
    }
}

fn to_wire(r: &SourceRef) -> wire::Ref {
    wire::Ref {
        provider: r.provider.clone(),
        id: r.id.clone(),
    }
}
fn from_wire(r: wire::Ref) -> SourceRef {
    SourceRef {
        provider: r.provider,
        id: r.id,
    }
}

impl SourceProvider for ProcessSource {
    fn manifest(&self) -> &SourceManifest {
        &self.plugin.source
    }
    fn browse(&self, category: &SourceRef, page: u32) -> Result<SourcePage, SourceError> {
        self.page(
            SourceCapability::Browse,
            wire::Request::Browse {
                category: to_wire(category),
                page,
            },
        )
    }
    fn search(&self, query: &str, page: u32) -> Result<SourcePage, SourceError> {
        self.page(
            SourceCapability::Search,
            wire::Request::Search {
                query: query.into(),
                page,
            },
        )
    }
    fn series(&self, series: &SourceRef) -> Result<SourcePage, SourceError> {
        self.page(
            SourceCapability::Series,
            wire::Request::Series {
                series: to_wire(series),
            },
        )
    }
    fn episode(&self, episode: &SourceRef) -> Result<EpisodeAdjacency, SourceError> {
        match self.call(
            SourceCapability::Episode,
            &wire::Request::Episode {
                episode: to_wire(episode),
            },
        )? {
            wire::Response::Adjacency { previous, next } => Ok(EpisodeAdjacency {
                previous: previous.map(from_wire),
                next: next.map(from_wire),
            }),
            _ => Err(SourceError::Invalid(
                "plugin reply: expected adjacency".into(),
            )),
        }
    }
    fn resolve(&self, episode: &SourceRef) -> Result<Vec<StreamCandidate>, SourceError> {
        match self.call(
            SourceCapability::Resolve,
            &wire::Request::Resolve {
                episode: to_wire(episode),
            },
        )? {
            wire::Response::Streams { streams } => {
                let hosts_ok = streams.iter().all(|s| {
                    url_host(&s.url).is_some_and(|h| self.plugin.source.allows_media_host(h))
                });
                if !hosts_ok {
                    return Err(SourceError::Invalid(
                        "stream host not declared in manifest".into(),
                    ));
                }
                Ok(streams
                    .into_iter()
                    .map(|s| StreamCandidate {
                        url: s.url,
                        headers: s.headers,
                    })
                    .collect())
            }
            _ => Err(SourceError::Invalid(
                "plugin reply: expected streams".into(),
            )),
        }
    }
}

fn url_host(url: &str) -> Option<&str> {
    let rest = url.strip_prefix("https://")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let host = authority.rsplit('@').next()?;
    Some(host.split(':').next().unwrap_or(host))
}
