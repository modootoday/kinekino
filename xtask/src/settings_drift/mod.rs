//! settings-drift: compares the live repository settings with the files that are their canon
//! (`.github/rulesets/*.json`, `.github/labels.json`, `.github/settings/repository.json`;
//! shared automation rules C2.6–C2.11). It only reads and reports; applying the files is the
//! owner's task from their own computer (C2.7). With `--issue` it opens or updates one
//! `needs-owner` issue on drift. Only keys present in the files are compared.
mod checks;
mod compare;
mod github;

use std::process::ExitCode;

const USAGE: &str = "usage: settings-drift --repo OWNER/NAME [--issue]";

pub fn run(args: &[String]) -> ExitCode {
    match parse(args).and_then(|o| check(&o)) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("settings-drift: {e}");
            ExitCode::from(2)
        }
    }
}

struct Opts {
    repo: String,
    issue: bool,
}

fn parse(args: &[String]) -> Result<Opts, String> {
    let mut o = Opts {
        repo: String::new(),
        issue: false,
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--repo" => o.repo = it.next().cloned().ok_or("--repo needs a value")?,
            "--issue" => o.issue = true,
            _ => return Err(format!("unknown argument {a}; {USAGE}")),
        }
    }
    let valid = o.repo.split_once('/').is_some_and(|(a, b)| {
        let ok = |s: &str| {
            !s.is_empty()
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
        };
        ok(a) && ok(b)
    });
    if !valid {
        return Err(format!("bad or missing --repo; {USAGE}"));
    }
    Ok(o)
}

/// Ok(true) when there is no drift, or when `--issue` recorded it.
fn check(o: &Opts) -> Result<bool, String> {
    let root = crate::workspace::root()?;
    let mut findings = Vec::new();
    let mut notes = Vec::new();
    checks::rulesets(&root, &o.repo, &mut findings)?;
    checks::labels(&root, &o.repo, &mut findings)?;
    checks::repository(&root, &o.repo, &mut findings, &mut notes)?;
    for n in &notes {
        println!("note: {n}");
    }
    if findings.is_empty() {
        println!("settings-drift: live settings match the files");
        return Ok(true);
    }
    let body = report(&findings, &notes);
    print!("{body}");
    if o.issue {
        println!("settings-drift: {}", github::upsert_issue(&o.repo, &body)?);
        return Ok(true);
    }
    Ok(false)
}

fn report(findings: &[checks::Finding], notes: &[String]) -> String {
    let mut s = String::from(
        "The live repository settings differ from the files in `.github/` \
         (shared automation rules C2.6). Apply the files from the owner's computer (C2.7), \
         or change the files by pull request if the live value is the one to keep.\n\n",
    );
    for f in findings {
        s.push_str(&format!("- {}: {}\n", f.area, f.detail));
    }
    if !notes.is_empty() {
        s.push_str("\nNot compared:\n");
        for n in notes {
            s.push_str(&format!("- {n}\n"));
        }
    }
    s.push_str("\nThis issue is updated by the `Settings drift` workflow; close it once the settings match.\n");
    s
}
