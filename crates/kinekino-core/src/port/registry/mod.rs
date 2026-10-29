//! `Registry<P>`: adapter factories registered with their declared capabilities, platform
//! rules and an availability probe; `select` picks one, `build` instantiates it.
mod decision;
#[cfg(test)]
mod tests;
#[cfg(test)]
pub(crate) mod tests_support;

pub use decision::{Considered, Decision, Rejection, SelectError};

use super::capability::{AdapterKind, CapabilityTable};
use super::error::{ErrorCode, PortError, PortResult};
use super::target::{DisplayServer, OsId, TargetPlatform};

/// Preference of an adapter on a platform. `Explicit` adapters are only chosen by policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    Default,
    Fallback,
    Explicit,
}

impl Role {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Fallback => "fallback",
            Self::Explicit => "explicit",
        }
    }
}

/// Where an adapter is supported. `display: None` matches any session, headless included.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformRule {
    pub os: OsId,
    pub display: Option<DisplayServer>,
    pub role: Role,
}

impl PlatformRule {
    pub const fn new(os: OsId, display: Option<DisplayServer>, role: Role) -> Self {
        Self { os, display, role }
    }

    fn matches(&self, p: TargetPlatform) -> bool {
        self.os == p.os && (self.display.is_none() || self.display == p.display)
    }
}

/// Result of an availability probe (runtime installed, compiled in, ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Availability {
    Available,
    Missing(String),
}

/// Caller policy on top of the platform order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy<K> {
    /// Default, then fallback, in registration order.
    PlatformOrder,
    /// Try this adapter first (even an `Explicit` one), then the platform order.
    Prefer(K),
    /// Only this adapter.
    Only(K),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection<K> {
    pub kind: K,
    pub role: Role,
}

type Probe = Box<dyn Fn() -> Availability>;
type Factory<P, Cfg> = Box<dyn Fn(&Cfg) -> PortResult<Box<P>>>;

pub struct Registration<P: ?Sized, K, C, Cfg> {
    pub kind: K,
    pub declared: C,
    pub rules: Vec<PlatformRule>,
    probe: Probe,
    factory: Factory<P, Cfg>,
}

impl<P: ?Sized, K, C, Cfg> Registration<P, K, C, Cfg> {
    pub fn new(
        kind: K,
        declared: C,
        rules: Vec<PlatformRule>,
        probe: impl Fn() -> Availability + 'static,
        factory: impl Fn(&Cfg) -> PortResult<Box<P>> + 'static,
    ) -> Self {
        Self {
            kind,
            declared,
            rules,
            probe: Box::new(probe),
            factory: Box::new(factory),
        }
    }

    fn role_on(&self, p: TargetPlatform) -> Option<Role> {
        self.rules
            .iter()
            .filter(|r| r.matches(p))
            .map(|r| r.role)
            .min()
    }
}

pub struct Registry<P: ?Sized, K, C, Cfg> {
    port: &'static str,
    entries: Vec<Registration<P, K, C, Cfg>>,
}

impl<P: ?Sized, K: AdapterKind, C: CapabilityTable, Cfg> Registry<P, K, C, Cfg> {
    pub fn new(port: &'static str) -> Self {
        Self {
            port,
            entries: Vec::new(),
        }
    }

    pub fn port(&self) -> &'static str {
        self.port
    }

    /// Registers an adapter; a later registration of the same kind replaces the earlier one.
    pub fn register(&mut self, r: Registration<P, K, C, Cfg>) {
        self.entries.retain(|e| e.kind != r.kind);
        self.entries.push(r);
    }

    pub fn kinds(&self) -> Vec<K> {
        self.entries.iter().map(|e| e.kind).collect()
    }

    pub fn declared(&self, kind: K) -> Option<C> {
        self.entries
            .iter()
            .find(|e| e.kind == kind)
            .map(|e| e.declared)
    }

    /// Full decision with every candidate's verdict (for the capabilities CLI).
    pub fn explain(
        &self,
        platform: TargetPlatform,
        required: C,
        policy: Policy<K>,
    ) -> Decision<K, C> {
        let mut order: Vec<(usize, Option<Role>)> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, e)| (i, e.role_on(platform)))
            .collect();
        order.sort_by_key(|(i, role)| {
            let preferred = match policy {
                Policy::Prefer(k) | Policy::Only(k) => self.entries[*i].kind != k,
                Policy::PlatformOrder => true,
            };
            (preferred, role.unwrap_or(Role::Explicit), *i)
        });
        let mut considered = Vec::new();
        let mut selected = None;
        for (i, role) in order {
            let e = &self.entries[i];
            let verdict = self.verdict(e, role, required, policy);
            if verdict.is_none() && selected.is_none() {
                selected = Some(Selection {
                    kind: e.kind,
                    role: role.unwrap_or(Role::Explicit),
                });
            }
            considered.push(Considered {
                kind: e.kind,
                role,
                declared: e.declared,
                rejection: verdict,
            });
        }
        Decision {
            port: self.port,
            platform,
            required,
            selected,
            considered,
        }
    }

    fn verdict(
        &self,
        e: &Registration<P, K, C, Cfg>,
        role: Option<Role>,
        required: C,
        policy: Policy<K>,
    ) -> Option<Rejection<C>> {
        let chosen = matches!(policy, Policy::Prefer(k) | Policy::Only(k) if k == e.kind);
        if matches!(policy, Policy::Only(k) if k != e.kind) {
            return Some(Rejection::ExcludedByPolicy);
        }
        match role {
            None => return Some(Rejection::NotOnPlatform),
            Some(Role::Explicit) if !chosen => return Some(Rejection::ExplicitOnly),
            _ => {}
        }
        if let Availability::Missing(why) = (e.probe)() {
            return Some(Rejection::NotAvailable(why));
        }
        let missing = e.declared.missing(required);
        (!missing.is_empty()).then_some(Rejection::Missing(missing))
    }

    pub fn select(
        &self,
        platform: TargetPlatform,
        required: C,
        policy: Policy<K>,
    ) -> Result<Selection<K>, SelectError<K, C>> {
        self.explain(platform, required, policy).into_result()
    }

    pub fn build(&self, selection: Selection<K>, config: &Cfg) -> PortResult<Box<P>> {
        let e = self
            .entries
            .iter()
            .find(|e| e.kind == selection.kind)
            .ok_or_else(|| {
                PortError::new(
                    ErrorCode::Unavailable,
                    format!("{}: {} is not registered", self.port, selection.kind.name()),
                )
            })?;
        (e.factory)(config)
    }

    /// `select` then `build`; selection failures become `Unsupported`/`Unavailable`.
    pub fn select_and_build(
        &self,
        platform: TargetPlatform,
        required: C,
        policy: Policy<K>,
        config: &Cfg,
    ) -> PortResult<(Selection<K>, Box<P>)> {
        let sel = self
            .select(platform, required, policy)
            .map_err(|e| e.to_port_error(self.port))?;
        self.build(sel, config).map(|p| (sel, p))
    }
}
