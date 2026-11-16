//! The three comparisons: rulesets, labels and repository settings.
use std::path::Path;

use serde_json::{Value, json};

use super::compare::diff;
use super::github;

/// One finding for the report.
pub struct Finding {
    pub area: String,
    pub detail: String,
}

/// Notes that are not drift (for example a field this token cannot read).
pub type Notes = Vec<String>;

fn read_json(path: &Path) -> Result<Value, String> {
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

pub fn rulesets(root: &Path, repo: &str, out: &mut Vec<Finding>) -> Result<(), String> {
    let dir = root.join(".github/rulesets");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|e| format!("read {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    let live_list = github::get(&format!("repos/{repo}/rulesets?per_page=100"))?;
    for file in files {
        let expected = read_json(&file)?;
        let name = expected["name"]
            .as_str()
            .ok_or(format!("{}: no name", file.display()))?;
        let area = format!("ruleset `{name}`");
        let id = live_list
            .as_array()
            .and_then(|l| l.iter().find(|r| r["name"] == name))
            .and_then(|r| r["id"].as_u64());
        let Some(id) = id else {
            out.push(Finding {
                area,
                detail: "not applied: no live ruleset with this name".into(),
            });
            continue;
        };
        let live = github::get(&format!("repos/{repo}/rulesets/{id}"))?;
        for d in diff(&expected, &live, name) {
            out.push(Finding {
                area: area.clone(),
                detail: d,
            });
        }
    }
    Ok(())
}

pub fn labels(root: &Path, repo: &str, out: &mut Vec<Finding>) -> Result<(), String> {
    let expected = read_json(&root.join(".github/labels.json"))?;
    let live = github::get(&format!("repos/{repo}/labels?per_page=100"))?;
    for label in expected
        .as_array()
        .ok_or(".github/labels.json: not an array")?
    {
        let name = label["name"].as_str().unwrap_or_default();
        let found = live.as_array().and_then(|l| {
            l.iter().find(|x| {
                x["name"]
                    .as_str()
                    .is_some_and(|n| n.eq_ignore_ascii_case(name))
            })
        });
        let Some(found) = found else {
            out.push(Finding {
                area: "labels".into(),
                detail: format!("`{name}`: missing"),
            });
            continue;
        };
        let lower = |v: &Value| json!(v.as_str().unwrap_or_default().to_ascii_lowercase());
        let mut want = label.clone();
        want["color"] = lower(&label["color"]);
        let mut have = found.clone();
        have["color"] = lower(&found["color"]);
        for d in diff(&want, &have, name) {
            out.push(Finding {
                area: "labels".into(),
                detail: d,
            });
        }
    }
    Ok(())
}

pub fn repository(
    root: &Path,
    repo: &str,
    out: &mut Vec<Finding>,
    notes: &mut Notes,
) -> Result<(), String> {
    let mut expected = read_json(&root.join(".github/settings/repository.json"))?;
    let live = github::get(&format!("repos/{repo}"))?;
    let obj = expected
        .as_object_mut()
        .ok_or("repository.json: not an object")?;
    for key in [
        "security_and_analysis",
        "allow_auto_merge",
        "delete_branch_on_merge",
    ] {
        if obj.contains_key(key) && live.get(key).is_none() {
            obj.remove(key);
            notes.push(format!(
                "`{key}` is not readable with this token; check it when applying"
            ));
        }
    }
    for d in diff(&expected, &live, "repository") {
        out.push(Finding {
            area: "repository settings".into(),
            detail: d,
        });
    }
    Ok(())
}
