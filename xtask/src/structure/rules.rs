//! Structure rules R2/R5/R15 and the single module style (no `x.rs` beside `x/`).
use super::scan::Tree;
use std::collections::BTreeMap;

const MAX_DIRECT_RS: usize = 8;
const MAX_LINES: usize = 600;
const ENTRY: &[&str] = &["mod.rs", "lib.rs", "main.rs"];
const BANNED: &[&str] = &[
    "utils", "util", "common", "misc", "helpers", "helper", "shared", "stuff", "base", "manager",
];

pub fn check(tree: &Tree) -> Vec<String> {
    let mut out = Vec::new();
    let mut per_dir: BTreeMap<&str, usize> = BTreeMap::new();
    for f in &tree.files {
        let (dir, name) = f.rel.rsplit_once('/').unwrap_or(("", &f.rel));
        if !ENTRY.contains(&name) {
            *per_dir.entry(dir).or_default() += 1;
        }
        if f.lines > MAX_LINES {
            out.push(format!("file-lines {} ({})", f.rel, bucket(f.lines)));
        }
        let stem = name.trim_end_matches(".rs");
        if BANNED.contains(&stem) {
            out.push(format!("banned-name {}", f.rel));
        }
        if tree.dirs.iter().any(|d| *d == format!("{dir}/{stem}")) && !ENTRY.contains(&name) {
            out.push(format!("file-and-dir {}", f.rel));
        }
    }
    for d in &tree.dirs {
        let last = d.rsplit('/').next().unwrap_or(d);
        if BANNED.contains(&last) {
            out.push(format!("banned-name {d}/"));
        }
    }
    for (dir, n) in per_dir {
        if n > MAX_DIRECT_RS {
            out.push(format!("flat-dir {dir}"));
        }
    }
    out
}

/// Coarse line bucket so small edits do not churn the allowlist.
fn bucket(lines: usize) -> String {
    format!(">{}", lines / 100 * 100)
}
