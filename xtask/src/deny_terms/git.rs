//! Reads what a commit range adds: commit messages, added diff lines and changed paths.
use std::process::Command;

pub struct AddedLine {
    pub path: String,
    pub line: usize,
    pub text: String,
}

pub struct Commit {
    pub sha: String,
    pub time: i64,
    pub author: String,
    pub committer: String,
    pub message: String,
}

pub struct Changes {
    /// Short sha, committer time (unix seconds) and message of each commit.
    pub messages: Vec<Commit>,
    pub added: Vec<AddedLine>,
    pub paths: Vec<String>,
}

/// Accepts only `BASE..HEAD` made of revision characters, so no option reaches git.
pub fn validate_range(range: &str) -> Result<(), String> {
    let ok_rev = |r: &str| {
        !r.is_empty()
            && !r.starts_with('-')
            && r.chars()
                .all(|c| c.is_ascii_alphanumeric() || "/_.~^-".contains(c))
    };
    match range.split_once("..") {
        Some((base, head)) if ok_rev(base) && ok_rev(head) && !head.starts_with('.') => Ok(()),
        _ => Err(format!("bad range {range}; expected BASE..HEAD")),
    }
}

pub fn collect(range: &str) -> Result<Changes, String> {
    let (base, head) = range.split_once("..").ok_or("bad range")?;
    let three_dot = format!("{base}...{head}");
    let log = git(&[
        "log",
        "--no-color",
        "--format=%H%x00%ct%x00%ae%x00%ce%x00%B%x1e",
        range,
    ])?;
    let diff = git(&[
        "diff",
        "--no-color",
        "--no-ext-diff",
        "--no-renames",
        "--unified=0",
        &three_dot,
    ])?;
    let names = git(&["diff", "--no-renames", "--name-only", &three_dot])?;
    Ok(Changes {
        messages: parse_log(&log),
        added: parse_added(&diff),
        paths: names
            .lines()
            .filter(|l| !l.is_empty())
            .map(str::to_owned)
            .collect(),
    })
}

fn git(args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(["-c", "core.quotepath=off"])
        .args(args)
        .output()
        .map_err(|e| format!("run git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.first().copied().unwrap_or_default(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn parse_log(log: &str) -> Vec<Commit> {
    log.split('\u{1e}')
        .filter_map(|rec| {
            let mut parts = rec.trim_start_matches('\n').splitn(5, '\0');
            let sha = parts.next()?;
            let time = parts.next()?.trim().parse().ok()?;
            let author = parts.next()?.to_owned();
            let committer = parts.next()?.to_owned();
            let message = parts.next()?.to_owned();
            Some(Commit {
                sha: sha.chars().take(12).collect(),
                time,
                author,
                committer,
                message,
            })
        })
        .collect()
}

/// `+` lines of a `--unified=0` diff with their new-file path and line number.
fn parse_added(diff: &str) -> Vec<AddedLine> {
    let mut out = Vec::new();
    let mut path = String::new();
    let mut next = 0usize;
    for l in diff.lines() {
        if let Some(p) = l.strip_prefix("+++ ") {
            path = p.strip_prefix("b/").unwrap_or(p).to_owned();
        } else if l.starts_with("@@") {
            next = hunk_start(l).unwrap_or(0);
        } else if let Some(text) = l.strip_prefix('+') {
            out.push(AddedLine {
                path: path.clone(),
                line: next,
                text: text.to_owned(),
            });
            next += 1;
        }
    }
    out
}

/// New-side start line of a hunk header `@@ -a,b +c,d @@`.
fn hunk_start(header: &str) -> Option<usize> {
    let plus = header.split_whitespace().find(|w| w.starts_with('+'))?;
    plus[1..].split(',').next()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_must_be_two_plain_revisions() {
        assert!(validate_range("abc123..def456").is_ok());
        assert!(validate_range("origin/main..HEAD").is_ok());
        assert!(validate_range("abc").is_err());
        assert!(validate_range("--output=x..HEAD").is_err());
        assert!(validate_range("a...b").is_err());
        assert!(validate_range("a..b c").is_err());
    }

    #[test]
    fn added_lines_carry_path_and_new_line_number() {
        let diff = "diff --git a/x b/x\n--- a/x\n+++ b/x\n@@ -1,0 +2,2 @@\n+one\n+two\n\
                    diff --git a/y b/y\n--- /dev/null\n+++ b/y\n@@ -0,0 +1 @@\n+three\n";
        let got: Vec<_> = parse_added(diff)
            .into_iter()
            .map(|a| (a.path, a.line, a.text))
            .collect();
        assert_eq!(
            got,
            vec![
                ("x".into(), 2, "one".into()),
                ("x".into(), 3, "two".into()),
                ("y".into(), 1, "three".into()),
            ]
        );
    }

    #[test]
    fn log_records_split_into_short_sha_and_body() {
        let log = "0123456789abcdef\u{0}100\u{0}a@x\u{0}c@x\u{0}subject\n\nbody\n\u{1e}\nfedcba9876543210\u{0}200\u{0}b@x\u{0}d@x\u{0}two\n\u{1e}\n";
        let got = parse_log(log);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].sha, "0123456789ab");
        assert_eq!(got[0].time, 100);
        assert_eq!(got[0].message, "subject\n\nbody\n");
        assert_eq!(
            (got[0].author.as_str(), got[0].committer.as_str()),
            ("a@x", "c@x")
        );
        assert_eq!((got[1].sha.as_str(), got[1].time), ("fedcba987654", 200));
    }
}
