//! deny-terms: the `Public surface` check (adr:kinekino/20261116065239, shared rules C4).
//! Fails when a commit range adds a private term: commit messages, added diff lines and
//! changed file names in `BASE..HEAD`, plus any text passed through `--text-env` (the pull
//! request title and body). It also fails on a `Co-authored-by` line whose address is not
//! `@users.noreply.github.com` in commits made at or after `--coauthor-since`, and on a commit
//! author or committer outside that domain from `--identity-since` on (GitHub's own web
//! committer is allowed).
//! The term list comes from `KINEKINO_DENY_TERMS` or `--terms-file`; an empty or missing
//! list fails (fail-closed). With the environment list, `KINEKINO_DENY_TERMS_EXTRA` is
//! optional and adds its terms (the union); unset or empty, it adds nothing. Reports name only the place and the term's list number,
//! never the term or the address, because CI logs of a public repository are public.
mod git;
mod terms;
mod trailers;

use std::process::ExitCode;

use terms::Terms;

const ENV_VAR: &str = "KINEKINO_DENY_TERMS";
const EXTRA_VAR: &str = "KINEKINO_DENY_TERMS_EXTRA";
const USAGE: &str = "usage: deny-terms --range BASE..HEAD [--terms-file PATH] \
                     [--text-env NAME]... [--coauthor-since TIME] [--identity-since TIME]";

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
    text_envs: Vec<String>,
    coauthor_since: i64,
    identity_since: i64,
}

fn parse(args: &[String]) -> Result<Opts, String> {
    let mut o = Opts {
        range: String::new(),
        terms_file: None,
        text_envs: Vec::new(),
        coauthor_since: i64::MIN,
        identity_since: i64::MIN,
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = || it.next().cloned().ok_or(format!("{a} needs a value"));
        match a.as_str() {
            "--range" => o.range = value()?,
            "--terms-file" => o.terms_file = Some(value()?),
            "--text-env" => o.text_envs.push(value()?),
            "--coauthor-since" => o.coauthor_since = trailers::parse_utc(&value()?)?,
            "--identity-since" => o.identity_since = trailers::parse_utc(&value()?)?,
            _ => return Err(format!("unknown argument {a}; {USAGE}")),
        }
    }
    if o.range.is_empty() {
        return Err(format!("missing --range; {USAGE}"));
    }
    git::validate_range(&o.range)?;
    Ok(o)
}

fn load_terms(o: &Opts) -> Result<Terms, String> {
    match &o.terms_file {
        Some(path) => {
            let raw = std::fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
            env_terms(Some(raw), None)
        }
        None => env_terms(std::env::var(ENV_VAR).ok(), std::env::var(EXTRA_VAR).ok()),
    }
}

/// The main list is required and must not be empty (fail-closed); the extra list only adds.
fn env_terms(main: Option<String>, extra: Option<String>) -> Result<Terms, String> {
    let raw = main.ok_or(format!("{ENV_VAR} is not set (fail-closed)"))?;
    let mut terms = Terms::parse(&raw);
    if terms.is_empty() {
        return Err(format!(
            "the term list is empty (fail-closed); is the {ENV_VAR} secret registered?"
        ));
    }
    if let Some(extra) = extra {
        terms.extend(Terms::parse_tagged(&extra, "x"));
    }
    Ok(terms)
}

fn scan(o: &Opts) -> Result<usize, String> {
    let terms = load_terms(o)?;
    let changes = git::collect(&o.range)?;
    let mut hits = 0;
    let mut report = |place: String, found: Vec<String>| {
        for n in found {
            eprintln!("hit: {place} (term #{n})");
            hits += 1;
        }
    };
    for c in &changes.messages {
        report(format!("commit {} message", c.sha), terms.hits(&c.message));
    }
    for line in &changes.added {
        report(
            format!("{}:{} added line", line.path, line.line),
            terms.hits(&line.text),
        );
    }
    for path in &changes.paths {
        report(format!("file name {path}"), terms.hits(path));
    }
    for name in &o.text_envs {
        let text = std::env::var(name).unwrap_or_default();
        report(format!("text from {name}"), terms.hits(&text));
    }
    for c in changes
        .messages
        .iter()
        .filter(|c| c.time >= o.coauthor_since)
    {
        let n = trailers::bad_coauthors(&c.message);
        if n > 0 {
            eprintln!(
                "hit: commit {} has {n} Co-authored-by line(s) without a users.noreply.github.com address",
                c.sha
            );
            hits += n;
        }
    }
    for c in changes
        .messages
        .iter()
        .filter(|c| c.time >= o.identity_since)
    {
        if !trailers::author_ok(&c.author) {
            eprintln!(
                "hit: commit {} author is not a users.noreply.github.com address",
                c.sha
            );
            hits += 1;
        }
        if !trailers::committer_ok(&c.committer) {
            eprintln!(
                "hit: commit {} committer is not a users.noreply.github.com address",
                c.sha
            );
            hits += 1;
        }
    }
    println!(
        "deny-terms: {} term(s); {} commit(s), {} added line(s), {} path(s), {} text(s) checked",
        terms.len(),
        changes.messages.len(),
        changes.added.len(),
        changes.paths.len(),
        o.text_envs.len()
    );
    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::env_terms;

    #[test]
    fn main_list_is_required_and_must_not_be_empty() {
        assert!(env_terms(None, Some("gamma".into())).is_err());
        assert!(env_terms(Some("# none\n".into()), Some("gamma".into())).is_err());
    }

    #[test]
    fn extra_list_is_optional_and_adds_its_terms() {
        let main_only = env_terms(Some("alpha".into()), None).unwrap();
        assert_eq!(main_only.len(), 1);
        let empty_extra = env_terms(Some("alpha".into()), Some(String::new())).unwrap();
        assert_eq!(empty_extra.len(), 1);
        let both = env_terms(Some("alpha".into()), Some("gamma".into())).unwrap();
        assert_eq!(both.hits("Alpha, Gamma"), ["1", "x1"]);
    }
}
