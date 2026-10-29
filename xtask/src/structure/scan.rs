//! Walks source directories (crates, src, packages, tests, xtask) and records .rs files.
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct RustFile {
    pub rel: String,
    pub lines: usize,
}

pub struct Tree {
    pub files: Vec<RustFile>,
    pub dirs: Vec<String>,
}

const ROOTS: &[&str] = &["crates", "src", "packages", "tests", "xtask"];
const SKIP: &[&str] = &[
    "target",
    "runtime",
    "meari",
    "vendor",
    "node_modules",
    ".git",
];

pub fn collect(root: &Path) -> Tree {
    let mut tree = Tree {
        files: Vec::new(),
        dirs: Vec::new(),
    };
    for r in ROOTS {
        walk(root, &root.join(r), &mut tree);
    }
    tree
}

fn walk(root: &Path, dir: &Path, tree: &mut Tree) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    tree.dirs.push(rel(root, dir));
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if p.is_dir() {
            if !SKIP.contains(&name) {
                walk(root, &p, tree);
            }
        } else if name.ends_with(".rs") {
            let lines = fs::read_to_string(&p)
                .map(|s| s.lines().count())
                .unwrap_or(0);
            tree.files.push(RustFile {
                rel: rel(root, &p),
                lines,
            });
        }
    }
}

fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}
