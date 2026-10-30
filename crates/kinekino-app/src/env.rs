//! Snapshot of the environment variables the composition root reads.
use std::collections::BTreeMap;

const PASSTHROUGH: [&str; 5] = [
    "XDG_CONFIG_HOME",
    "HOME",
    "APPDATA",
    "DISPLAY",
    "WAYLAND_DISPLAY",
];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Env(BTreeMap<String, String>);

impl Env {
    /// `KINEKINO_*` plus config-path/session variables; everything else is not captured.
    pub fn from_process() -> Self {
        Self::from_pairs(std::env::vars())
    }

    pub fn from_pairs<I: IntoIterator<Item = (K, V)>, K: Into<String>, V: Into<String>>(
        pairs: I,
    ) -> Self {
        Self(
            pairs
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .filter(|(k, _)| k.starts_with("KINEKINO_") || PASSTHROUGH.contains(&k.as_str()))
                .collect(),
        )
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.0
            .get(key)
            .map(String::as_str)
            .filter(|v| !v.is_empty())
    }
}
