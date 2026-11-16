//! Reads repository settings and manages the drift issue through the `gh` CLI (adapter).
use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::Value;

pub const ISSUE_TITLE: &str = "Settings drift: live repository settings differ from .github files";

pub fn get(path: &str) -> Result<Value, String> {
    let out = run(
        &["api", "-H", "Accept: application/vnd.github+json", path],
        None,
    )?;
    serde_json::from_str(&out).map_err(|e| format!("gh api {path}: bad JSON: {e}"))
}

/// Opens the drift issue, or replaces the body of the one already open.
pub fn upsert_issue(repo: &str, body: &str) -> Result<String, String> {
    let list = run(
        &[
            "issue",
            "list",
            "--repo",
            repo,
            "--state",
            "open",
            "--search",
            &format!("in:title \"{ISSUE_TITLE}\""),
            "--json",
            "number,title",
        ],
        None,
    )?;
    let open: Value = serde_json::from_str(&list).map_err(|e| format!("gh issue list: {e}"))?;
    let existing = open.as_array().and_then(|items| {
        items
            .iter()
            .find(|i| i["title"] == ISSUE_TITLE)
            .and_then(|i| i["number"].as_u64())
    });
    if let Some(n) = existing {
        let n = n.to_string();
        run(
            &["issue", "edit", &n, "--repo", repo, "--body-file", "-"],
            Some(body),
        )?;
        return Ok(format!("updated issue #{n}"));
    }
    let create = |label: Option<&str>| {
        let mut args = vec!["issue", "create", "--repo", repo, "--title", ISSUE_TITLE];
        if let Some(l) = label {
            args.extend(["--label", l]);
        }
        args.extend(["--body-file", "-"]);
        run(&args, Some(body))
    };
    match create(Some("needs-owner")) {
        Ok(url) => Ok(format!("opened {}", url.trim())),
        // The label may not exist yet (labels are part of what drifts); open it unlabelled.
        Err(_) => {
            create(None).map(|url| format!("opened {} without the needs-owner label", url.trim()))
        }
    }
}

fn run(args: &[&str], stdin: Option<&str>) -> Result<String, String> {
    let mut child = Command::new("gh")
        .args(args)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("run gh: {e}"))?;
    if let (Some(text), Some(mut pipe)) = (stdin, child.stdin.take()) {
        pipe.write_all(text.as_bytes())
            .map_err(|e| format!("gh stdin: {e}"))?;
    }
    let out = child.wait_with_output().map_err(|e| format!("gh: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "gh {} failed: {}",
            args.iter().take(2).copied().collect::<Vec<_>>().join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}
