//! Tiny port used by registry and harness tests.
use crate::port::capability::{AdapterKind, CapabilityTable};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caps(pub u8);

impl fmt::Display for Caps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.names().join(", "))
    }
}

impl CapabilityTable for Caps {
    fn empty() -> Self {
        Caps(0)
    }
    fn contains_all(self, r: Self) -> bool {
        self.0 & r.0 == r.0
    }
    fn missing(self, r: Self) -> Self {
        Caps(r.0 & !self.0)
    }
    fn is_empty(self) -> bool {
        self.0 == 0
    }
    fn names(self) -> Vec<&'static str> {
        ["a", "b", "c"]
            .into_iter()
            .enumerate()
            .filter(|(i, _)| self.0 & (1 << i) != 0)
            .map(|(_, n)| n)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Alpha,
    Beta,
    Gamma,
}

impl AdapterKind for Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Alpha => "alpha",
            Kind::Beta => "beta",
            Kind::Gamma => "gamma",
        }
    }
}
