//! lint-structure: scans the workspace and compares violations with the allowlist.
mod boundary;
mod rules;
mod scan;
mod source_words;

use std::{collections::BTreeSet, fs, process::ExitCode};

const ALLOWLIST: &str = "xtask/structure-allowlist.txt";

pub fn run(args: &[String]) -> ExitCode {
    let root = match crate::workspace::root() {
        Ok(root) => root,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let mut found: BTreeSet<String> = rules::check(&scan::collect(&root)).into_iter().collect();
    found.extend(boundary::check(&root));
    found.extend(source_words::check(&root));
    let path = root.join(ALLOWLIST);
    if args.iter().any(|a| a == "--write-allowlist") {
        let body: String = found.iter().map(|v| format!("{v}\n")).collect();
        return match fs::write(&path, body) {
            Ok(()) => {
                println!("allowlist written: {} entries", found.len());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("write failed: {e}");
                ExitCode::FAILURE
            }
        };
    }
    let allowed: BTreeSet<String> = fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect();
    let new: Vec<&String> = found.difference(&allowed).collect();
    for v in &new {
        println!("NEW {v}");
    }
    println!(
        "violations: {} total, {} allowlisted, {} new, {} fixed",
        found.len(),
        found.intersection(&allowed).count(),
        new.len(),
        allowed.difference(&found).count()
    );
    let (cfg_files, cfg_hits) = boundary::cfg_total(&root);
    println!("platform cfg outside adapter crates: {cfg_hits} in {cfg_files} files");
    for rule in ["r-cfg", "r-dep", "r-ver", "r-entry", "r-port", "r-src"] {
        let n = found
            .iter()
            .filter(|v| v.starts_with(&format!("{rule} ")))
            .count();
        let a = found
            .intersection(&allowed)
            .filter(|v| v.starts_with(&format!("{rule} ")))
            .count();
        println!("  {rule}: {n} found, {a} allowlisted");
    }
    if new.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
