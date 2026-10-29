//! Cross-OS boundary rules (ADR cross-os-foundation): R-cfg, R-dep, R-ver, R-entry, R-port.
mod manifest;
mod source;
#[cfg(test)]
mod tests;

use manifest::Manifest;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// OS/engine adapters; the only crates allowed platform `cfg`.
    Adapter,
    /// Port traits and shared vocabulary.
    Contract,
    /// Wires adapters to ports.
    Composition,
    /// Executables; depend on composition only.
    Bin,
    /// Domain and application crates.
    Domain,
}

/// Crate class table. Every workspace crate must be listed (unlisted = `r-dep … unclassified`).
pub const CLASSES: &[(&str, Class)] = &[
    ("kinekino-platform", Class::Adapter),
    ("kinekino-platform-unix", Class::Adapter),
    ("kinekino-platform-windows", Class::Adapter),
    ("kinekino-browser", Class::Adapter),
    ("kinekino-settings-ui", Class::Adapter),
    ("kinekino-host", Class::Adapter),
    ("kinekino-inference-hf", Class::Adapter),
    ("kinekino-inference-local", Class::Adapter),
    ("kinekino-libmpv", Class::Adapter),
    ("kinekino-player-ui", Class::Adapter),
    ("kinekino-core", Class::Contract),
    ("kinekino-media-contracts", Class::Contract),
    ("kinekino-plugin-sdk", Class::Contract),
    ("kinekino-app", Class::Composition),
    ("kinekino-cli", Class::Bin),
    ("kinekino-player", Class::Domain),
    ("kinekino-ui-plugins", Class::Domain),
    ("kinekino-subtitles", Class::Domain),
    ("kinekino-jobs", Class::Domain),
    ("kinekino-process", Class::Domain),
    ("kinekino-vad", Class::Domain),
    ("kinekino-vad-executor", Class::Domain),
    ("kinekino-vad-worker", Class::Domain),
    ("kinekino-source-host", Class::Domain),
    ("kinekino-card-render", Class::Domain),
    ("kinekino-inference", Class::Domain),
    ("kinekino-xtask", Class::Domain),
];

pub fn class_of(name: &str) -> Option<Class> {
    CLASSES.iter().find(|(n, _)| *n == name).map(|(_, c)| *c)
}

/// Runs all five rules over `crates/*` and `xtask`.
pub fn check(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for m in manifest::workspace(root) {
        out.extend(check_crate(root, &m));
    }
    out
}

fn check_crate(root: &Path, m: &Manifest) -> Vec<String> {
    let mut out = Vec::new();
    for d in m.deps.iter().filter(|d| !d.workspace) {
        out.push(format!("r-ver {} {}", m.name, d.name));
    }
    let Some(class) = class_of(&m.name) else {
        out.push(format!("r-dep {} unclassified", m.name));
        return out;
    };
    for d in m
        .deps
        .iter()
        .filter(|d| d.kind == manifest::DepKind::Normal)
    {
        if let Some(v) = dep_violation(class, &d.name) {
            out.push(format!("r-dep {} -> {} ({v})", m.name, d.name));
        }
    }
    for f in source::rust_files(&root.join(&m.dir)) {
        let rel = format!("{}/{}", m.dir, f.rel);
        let text = &f.text;
        if class != Class::Adapter && source::platform_cfg_count(text) > 0 {
            out.push(format!("r-cfg {rel}"));
        }
        // xtask is repo tooling, not a product binary.
        if m.name != "kinekino-xtask" && f.is_bin_entry() && !source::entry_is_thin(text) {
            out.push(format!("r-entry {rel}"));
        }
        if class != Class::Contract && !source::is_port_location(&f.rel) {
            for t in source::port_like_traits(text) {
                out.push(format!("r-port {rel} {t}"));
            }
        }
    }
    out
}

fn dep_violation(class: Class, dep: &str) -> Option<&'static str> {
    let dep_class = class_of(dep);
    match (class, dep_class) {
        (Class::Bin, Some(Class::Composition)) => None,
        (Class::Bin, _) => Some("bin depends on composition only"),
        (Class::Domain | Class::Contract, Some(Class::Adapter | Class::Composition)) => {
            Some("domain/contract must not depend on adapters")
        }
        (Class::Contract, Some(Class::Domain | Class::Bin)) => {
            Some("contract depends on contracts only")
        }
        (_, Some(Class::Bin)) => Some("nothing depends on a bin"),
        _ => None,
    }
}

/// Platform cfg occurrences outside adapter crates (for the report).
pub fn cfg_total(root: &Path) -> (usize, usize) {
    let (mut files, mut hits) = (0, 0);
    for m in manifest::workspace(root) {
        if class_of(&m.name) == Some(Class::Adapter) {
            continue;
        }
        for f in source::rust_files(&root.join(&m.dir)) {
            let n = source::platform_cfg_count(&f.text);
            if n > 0 {
                files += 1;
                hits += n;
            }
        }
    }
    (files, hits)
}
