//! Runtime host admission for player navigation and player documents (ADR 20261105211630).
use std::sync::OnceLock;

/// Exact, case-insensitive host set used by navigation and player-document policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedHosts {
    hosts: Vec<String>,
    player_hosts: Vec<String>,
}

impl AdmittedHosts {
    /// Production constructor: every listed host is both navigable and a player host.
    pub fn new(hosts: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        let hosts = normalize(hosts);
        Self {
            player_hosts: hosts.clone(),
            hosts,
        }
    }

    /// Restrict which admitted hosts may activate the native player / document origin.
    /// Used by the test fixture so the catalog host stays navigable but not a player page.
    pub fn with_player_hosts(
        mut self,
        player_hosts: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        self.player_hosts = normalize(player_hosts);
        self
    }

    /// Fixture hosts only (`catalog` / `frame` / `media` example.test). No real CDN.
    pub fn fixture() -> Self {
        Self::new([
            "catalog.example.test",
            "frame.example.test",
            "media.example.test",
        ])
        .with_player_hosts(["frame.example.test", "media.example.test"])
    }

    pub fn empty() -> Self {
        Self {
            hosts: Vec::new(),
            player_hosts: Vec::new(),
        }
    }

    pub fn admits(&self, host: &str) -> bool {
        contains(&self.hosts, host)
    }

    pub fn admits_player(&self, host: &str) -> bool {
        contains(&self.player_hosts, host)
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

static INSTALLED: OnceLock<AdmittedHosts> = OnceLock::new();

/// Process-wide install. Returns `false` if already installed (same contract as `install_rules`).
pub fn install_admitted_hosts(hosts: AdmittedHosts) -> bool {
    INSTALLED.set(hosts).is_ok()
}

/// Active policy: test override → installed → cfg(test) fixture → empty (deny-by-default).
pub(crate) fn active_hosts() -> AdmittedHosts {
    #[cfg(test)]
    {
        if let Some(hosts) = test_override() {
            return hosts;
        }
    }
    if let Some(hosts) = INSTALLED.get() {
        return hosts.clone();
    }
    #[cfg(test)]
    {
        AdmittedHosts::fixture()
    }
    #[cfg(not(test))]
    {
        AdmittedHosts::empty()
    }
}

#[cfg(test)]
mod test_override_slot {
    use super::AdmittedHosts;
    use std::cell::RefCell;

    thread_local! {
        pub static OVERRIDE: RefCell<Option<AdmittedHosts>> = const { RefCell::new(None) };
    }
}

#[cfg(test)]
fn test_override() -> Option<AdmittedHosts> {
    test_override_slot::OVERRIDE.with(|slot| slot.borrow().clone())
}

/// Test-only scoped override so parallel unit tests do not poison the process `OnceLock`.
#[cfg(test)]
pub(crate) fn with_admitted_hosts<R>(hosts: AdmittedHosts, f: impl FnOnce() -> R) -> R {
    test_override_slot::OVERRIDE.with(|slot| {
        let previous = slot.replace(Some(hosts));
        let result = f();
        *slot.borrow_mut() = previous;
        result
    })
}

/// Fixture plus a document-only secondary CDN origin used by preserved application tests.
#[cfg(test)]
pub(crate) fn secondary_cdn_test_hosts() -> AdmittedHosts {
    AdmittedHosts::fixture().with_player_hosts([
        "frame.example.test",
        "media.example.test",
        "michealcdn.com",
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admits_is_exact_and_case_insensitive() {
        let hosts = AdmittedHosts::new(["Media.Example.Test"]);
        assert!(hosts.admits("media.example.test"));
        assert!(hosts.admits("MEDIA.EXAMPLE.TEST"));
        assert!(!hosts.admits("media.example.test.evil.test"));
        assert!(!hosts.admits("sub.media.example.test"));
    }

    #[test]
    fn fixture_keeps_catalog_navigable_but_not_a_player_host() {
        let hosts = AdmittedHosts::fixture();
        assert!(hosts.admits("catalog.example.test"));
        assert!(!hosts.admits_player("catalog.example.test"));
        assert!(hosts.admits_player("frame.example.test"));
        assert!(hosts.admits_player("media.example.test"));
        assert!(!hosts.admits("michealcdn.com"));
        assert!(!hosts.admits_player("michealcdn.com"));
    }
}
