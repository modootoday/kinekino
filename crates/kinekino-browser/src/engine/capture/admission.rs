//! Host admission for capture navigate targets and observed media hosts.

/// Deny-by-default admission used by [`super::BrowserCaptureSession`].
pub trait CaptureAdmission: Send + Sync {
    fn admits_navigate_host(&self, host: &str) -> bool;
    fn admits_media_host(&self, host: &str) -> bool;
}

/// Exact, case-insensitive host allowlists for navigate and media.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowlistAdmission {
    navigate_hosts: Vec<String>,
    media_hosts: Vec<String>,
}

impl AllowlistAdmission {
    pub fn new(
        navigate_hosts: impl IntoIterator<Item = impl AsRef<str>>,
        media_hosts: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        Self {
            navigate_hosts: normalize(navigate_hosts),
            media_hosts: normalize(media_hosts),
        }
    }
}

impl CaptureAdmission for AllowlistAdmission {
    fn admits_navigate_host(&self, host: &str) -> bool {
        contains(&self.navigate_hosts, host)
    }

    fn admits_media_host(&self, host: &str) -> bool {
        contains(&self.media_hosts, host)
    }
}

fn normalize(hosts: impl IntoIterator<Item = impl AsRef<str>>) -> Vec<String> {
    let mut hosts: Vec<String> = hosts
        .into_iter()
        .map(|h| h.as_ref().to_ascii_lowercase())
        .filter(|h| !h.is_empty())
        .collect();
    hosts.sort_unstable();
    hosts.dedup();
    hosts
}

fn contains(hosts: &[String], host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    hosts.iter().any(|h| h == &host)
}

/// Extract a host from an http(s) URL. Returns `None` for unparseable or non-http URLs.
pub fn host_of(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    match parsed.scheme() {
        "http" | "https" => parsed.host_str().map(str::to_ascii_lowercase),
        _ => None,
    }
}
