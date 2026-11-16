//! deny-terms: fails when a commit range adds any private term (adr:kinekino/20261116065239).
//! Checks commit messages, added diff lines and changed file names in `BASE..HEAD`.
//! The term list comes from `KINEKINO_DENY_TERMS` or `--terms-file`; an empty or missing
//! list fails (fail-closed). Reports name only the place and the term's list number,
//! never the term, because CI logs of a public repository are public.
mod git;
mod terms;

use std::process::ExitCode;

use terms::Terms;

const ENV_VAR: &str = "KINEKINO_DENY_TERMS";
const USAGE: &str = "usage: deny-terms --range BASE..HEAD [--terms-file PATH]";

pub fn run(args: &[String]) -> ExitCode {
    match parse(args).and_then(|o| scan(&o)) {
        Ok(0) => {
            println!("deny-terms: 0 hits");
            ExitCode::SUCCESS
        }
        Ok(n) => {
            eprintln!("deny-terms: {n} hit(s); remove them before merging");
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("deny-terms: {e}");
            ExitCode::FAILURE
        }
    }
}

struct Opts {
    range: String,
    terms_file: Option<String>,
}

fn parse(args: &[String]) -> Result<Opts, String> {
    let mut range = None;
    let mut terms_file = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--range" => range = Some(it.next().ok_or("--range needs a value")?.clone()),
            "--terms-file" => {
                terms_file = Some(it.next().ok_or("--terms-file needs a value")?.clone());
            }
            _ => return Err(format!("unknown argument {a}; {USAGE}")),
        }
    }
    let range = range.ok_or(format!("missing --range; {USAGE}"))?;
    git::validate_range(&range)?;
    Ok(Opts { range, terms_file })
}

fn load_terms(o: &Opts) -> Result<Terms, String> {
    let raw = match &o.terms_file {
        Some(path) => std::fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?,
        None => {
            std::env::var(ENV_VAR).map_err(|_| format!("{ENV_VAR} is not set (fail-closed)"))?
        }
    };
    let terms = Terms::parse(&raw);
    if terms.is_empty() {
        return Err("the term list is empty (fail-closed)".into());
    }
    Ok(terms)
}

fn scan(o: &Opts) -> Result<usize, String> {
    let terms = load_terms(o)?;
    let changes = git::collect(&o.range)?;
    let mut hits = 0;
    for (commit, message) in &changes.messages {
        for n in terms.hits(message) {
            eprintln!("hit: commit {commit} message (term #{n})");
            hits += 1;
        }
    }
    for line in &changes.added {
        for n in terms.hits(&line.text) {
            eprintln!("hit: {}:{} added line (term #{n})", line.path, line.line);
            hits += 1;
        }
    }
    for path in &changes.paths {
        for n in terms.hits(path) {
            eprintln!("hit: file name {path} (term #{n})");
            hits += 1;
        }
    }
    println!(
        "deny-terms: {} term(s); {} commit(s), {} added line(s), {} path(s) checked",
        terms.len(),
        changes.messages.len(),
        changes.added.len(),
        changes.paths.len()
    );
    Ok(hits)
}
