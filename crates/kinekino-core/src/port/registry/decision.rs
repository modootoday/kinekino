//! Selection outcome with per-candidate rejection reasons.
use super::{Role, Selection};
use crate::port::capability::{AdapterKind, CapabilityTable};
use crate::port::error::{ErrorCode, PortError};
use crate::port::target::TargetPlatform;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection<C> {
    /// No platform rule matches this OS/display.
    NotOnPlatform,
    /// Supported here but only when a policy names it.
    ExplicitOnly,
    /// `Policy::Only` named another adapter.
    ExcludedByPolicy,
    /// Probe failed (not compiled in, runtime missing).
    NotAvailable(String),
    /// Declared table lacks these required capabilities.
    Missing(C),
}

impl<C: CapabilityTable> fmt::Display for Rejection<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotOnPlatform => f.write_str("not supported on this platform"),
            Self::ExplicitOnly => f.write_str("explicit only (name it in policy)"),
            Self::ExcludedByPolicy => f.write_str("excluded by policy"),
            Self::NotAvailable(w) => write!(f, "not available: {w}"),
            Self::Missing(m) => write!(f, "lacks {m}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Considered<K, C> {
    pub kind: K,
    pub role: Option<Role>,
    pub declared: C,
    /// `None` when the candidate qualifies.
    pub rejection: Option<Rejection<C>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision<K, C> {
    pub port: &'static str,
    pub platform: TargetPlatform,
    pub required: C,
    pub selected: Option<Selection<K>>,
    pub considered: Vec<Considered<K, C>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectError<K, C> {
    /// No adapter has a rule for this platform.
    UnsupportedPlatform,
    /// Candidates exist but none qualifies.
    NoCapableAdapter { rejected: Vec<(K, Rejection<C>)> },
}

impl<K: AdapterKind, C: CapabilityTable> SelectError<K, C> {
    pub fn to_port_error(&self, port: &str) -> PortError {
        match self {
            Self::UnsupportedPlatform => PortError::new(
                ErrorCode::Unsupported,
                format!("{port}: no adapter for this platform"),
            ),
            Self::NoCapableAdapter { rejected } => {
                let why: Vec<String> = rejected
                    .iter()
                    .map(|(k, r)| format!("{} {r}", k.name()))
                    .collect();
                PortError::new(
                    ErrorCode::Unavailable,
                    format!("{port}: no capable adapter: {}", why.join("; ")),
                )
            }
        }
    }
}

impl<K: AdapterKind, C: CapabilityTable> Decision<K, C> {
    pub fn into_result(self) -> Result<Selection<K>, SelectError<K, C>> {
        if let Some(s) = self.selected {
            return Ok(s);
        }
        let on_platform = self.considered.iter().any(|c| c.role.is_some());
        if !on_platform {
            return Err(SelectError::UnsupportedPlatform);
        }
        Err(SelectError::NoCapableAdapter {
            rejected: self
                .considered
                .into_iter()
                .filter(|c| c.role.is_some())
                .filter_map(|c| c.rejection.map(|r| (c.kind, r)))
                .collect(),
        })
    }

    /// Stable JSON for `kinekino capabilities --json`.
    pub fn to_json(&self) -> String {
        let quote = |s: &str| format!("\"{s}\"");
        let names = |c: C| {
            let v: Vec<String> = c.names().into_iter().map(quote).collect();
            format!("[{}]", v.join(","))
        };
        let cands: Vec<String> = self
            .considered
            .iter()
            .map(|c| {
                format!(
                    "{{\"kind\":\"{}\",\"role\":{},\"declared\":{},\"status\":{}}}",
                    c.kind.name(),
                    c.role.map_or("null".into(), |r| quote(r.name())),
                    names(c.declared),
                    c.rejection
                        .as_ref()
                        .map_or("\"ok\"".into(), |r| quote(&r.to_string().replace('"', "'")))
                )
            })
            .collect();
        format!(
            "{{\"port\":\"{}\",\"platform\":{{\"os\":\"{}\",\"display\":{}}},\"required\":{},\"selected\":{},\"candidates\":[{}]}}",
            self.port,
            self.platform.os.name(),
            self.platform
                .display
                .map_or("null".into(), |d| quote(d.name())),
            names(self.required),
            self.selected.map_or("null".into(), |s| format!(
                "{{\"kind\":\"{}\",\"role\":\"{}\"}}",
                s.kind.name(),
                s.role.name()
            )),
            cands.join(",")
        )
    }
}
