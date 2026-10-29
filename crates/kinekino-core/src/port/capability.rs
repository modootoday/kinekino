//! Capability vocabulary contract: each port defines its own bit set and implements this.
use std::fmt::{Debug, Display};

/// A port's capability set (copyable, comparable, printable as `[a, b]`).
pub trait CapabilityTable: Copy + Eq + Debug + Display + 'static {
    fn empty() -> Self;
    fn contains_all(self, required: Self) -> bool;
    /// Capabilities in `required` that `self` lacks.
    fn missing(self, required: Self) -> Self;
    fn is_empty(self) -> bool;
    /// Stable names of the members (used by the capabilities CLI).
    fn names(self) -> Vec<&'static str>;
}

/// Adapter identity of one port (stable lower-kebab name for CLI/JSON).
pub trait AdapterKind: Copy + Eq + Debug + 'static {
    fn name(self) -> &'static str;
}
