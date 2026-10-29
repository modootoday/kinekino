//! Source-level checks: platform cfg, bin entry shape, port-like traits.
use std::{fs, path::Path};

pub struct Source {
    /// Relative to the crate dir.
    pub rel: String,
    pub text: String,
}

/// Bin entry: `src/main.rs` or `src/bin/*.rs`.
impl Source {
    pub fn is_bin_entry(&self) -> bool {
        self.rel == "src/main.rs"
            || (self.rel.starts_with("src/bin/") && self.rel.matches('/').count() == 2)
    }
}

const ENTRY_MAX_LINES: usize = 15;
const PLATFORM_KEYS: [&str; 6] = [
    "windows",
    "unix",
    "target_os",
    "target_family",
    "target_vendor",
    "target_env",
];
const PORT_SUFFIXES: [&str; 11] = [
    "Port",
    "Engine",
    "Store",
    "Sink",
    "Host",
    "Provider",
    "Backend",
    "Transport",
    "Client",
    "Api",
    "Gateway",
];

pub fn rust_files(dir: &Path) -> Vec<Source> {
    let mut out = Vec::new();
    walk(dir, dir, &mut out);
    out.sort_by(|a, b| a.rel.cmp(&b.rel));
    out
}

fn walk(base: &Path, dir: &Path, out: &mut Vec<Source>) {
    for e in fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if p.is_dir() {
            if !matches!(name.as_str(), "target" | "runtime" | "node_modules") {
                walk(base, &p, out);
            }
        } else if name.ends_with(".rs") {
            let rel = p
                .strip_prefix(base)
                .unwrap_or(&p)
                .to_string_lossy()
                .replace('\\', "/");
            out.push(Source {
                rel,
                text: fs::read_to_string(&p).unwrap_or_default(),
            });
        }
    }
}

/// Number of `cfg`/`cfg!`/`cfg_attr` predicates naming the OS (`#[cfg(test)]` etc. excluded).
pub fn platform_cfg_count(text: &str) -> usize {
    let code = text.lines().filter(|l| !l.trim_start().starts_with("//"));
    let mut n = 0;
    for line in code {
        let mut rest = line;
        while let Some(i) = rest.find("cfg") {
            rest = &rest[i + 3..];
            let pred = rest.trim_start_matches("_attr").trim_start_matches('!');
            if pred.starts_with('(') {
                let end = balanced(pred).unwrap_or(pred.len());
                let bare = strip_strings(&pred[..end]);
                if PLATFORM_KEYS.iter().any(|k| contains_word(&bare, k)) {
                    n += 1;
                }
            }
        }
    }
    n
}

/// Drops `"…"` literals so feature names like `"windows-x"` do not count.
fn strip_strings(s: &str) -> String {
    s.split('"').step_by(2).collect::<Vec<_>>().join(" ")
}

fn balanced(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
    }
    None
}

fn contains_word(hay: &str, word: &str) -> bool {
    hay.match_indices(word).any(|(i, _)| {
        let ok = |c: Option<char>| !c.is_some_and(|c| c.is_alphanumeric() || c == '_');
        ok(hay[..i].chars().next_back()) && ok(hay[i + word.len()..].chars().next())
    })
}

/// Thin entry: short file, no platform cfg, `fn main` body is one call into `kinekino_app`.
pub fn entry_is_thin(text: &str) -> bool {
    let lines: Vec<&str> = text
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with("//"))
        .collect();
    if lines.len() > ENTRY_MAX_LINES || platform_cfg_count(text) > 0 {
        return false;
    }
    let Some(start) = text.find("fn main") else {
        return false;
    };
    let body = &text[start..];
    let body = body.find('{').map_or("", |i| &body[i..]);
    let end = balanced_braces(body).unwrap_or(body.len());
    let body = &body[..end];
    body.contains("kinekino_app::") && body.matches(';').count() <= 1
}

fn balanced_braces(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
    }
    None
}

/// `port/` dirs and `ports.rs`/`port.rs` modules are contract locations inside any crate.
pub fn is_port_location(rel: &str) -> bool {
    rel.split('/')
        .any(|seg| matches!(seg, "port" | "ports" | "ports.rs" | "port.rs"))
}

/// `pub trait` names that look like ports (by suffix).
pub fn port_like_traits(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim_start();
            let rest = l
                .strip_prefix("pub trait ")
                .or_else(|| l.strip_prefix("pub(crate) trait "))?;
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            PORT_SUFFIXES
                .iter()
                .any(|s| name.ends_with(s))
                .then_some(name)
        })
        .collect()
}
