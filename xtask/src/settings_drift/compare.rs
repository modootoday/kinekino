//! Compares a settings file with live settings, looking only at what the file states.
use serde_json::Value;

/// Keys that identify an array item (ruleset rules, status checks, labels).
const KEYS: [&str; 3] = ["type", "name", "context"];

/// Differences between `expected` (the file) and `live` (GitHub), as readable lines.
/// Keys missing from the file are ignored. Arrays must have the same length, and every
/// expected element must match some live element by the same rule.
pub fn diff(expected: &Value, live: &Value, path: &str) -> Vec<String> {
    let mut out = Vec::new();
    walk(expected, live, path, &mut out);
    out
}

fn walk(expected: &Value, live: &Value, path: &str, out: &mut Vec<String>) {
    match (expected, live) {
        (Value::Object(e), Value::Object(l)) => {
            for (key, ev) in e {
                let p = format!("{path}.{key}");
                match l.get(key) {
                    Some(lv) => walk(ev, lv, &p, out),
                    None => out.push(format!("{p}: missing")),
                }
            }
        }
        (Value::Array(e), Value::Array(l)) => {
            if e.len() != l.len() {
                out.push(format!(
                    "{path}: {} item(s) in the file, {} live",
                    e.len(),
                    l.len()
                ));
            }
            for (i, ev) in e.iter().enumerate() {
                if l.iter().any(|lv| diff(ev, lv, "").is_empty()) {
                    continue;
                }
                // Pair by an identifying key so the report names the field that differs.
                let paired = KEYS.iter().find_map(|k| {
                    let id = ev.get(*k)?;
                    let lv = l.iter().find(|lv| lv.get(*k) == Some(id))?;
                    Some((format!("{path}[{k}={id}]"), lv))
                });
                match paired {
                    Some((p, lv)) => walk(ev, lv, &p, out),
                    None => out.push(format!("{path}[{i}]: no matching live item for {ev}")),
                }
            }
        }
        _ if expected == live => {}
        _ => out.push(format!("{path}: file {expected}, live {live}")),
    }
}

#[cfg(test)]
mod tests {
    use super::diff;
    use serde_json::json;

    #[test]
    fn ignores_keys_the_file_does_not_state() {
        let file = json!({"a": 1, "b": {"c": true}});
        let live = json!({"a": 1, "b": {"c": true, "server_added": 5}, "z": 0});
        assert!(diff(&file, &live, "x").is_empty());
    }

    #[test]
    fn reports_changed_and_missing_values() {
        let file = json!({"a": 1, "b": 2});
        let live = json!({"a": 3});
        assert_eq!(
            diff(&file, &live, "x"),
            vec!["x.a: file 1, live 3", "x.b: missing"]
        );
    }

    #[test]
    fn arrays_match_by_content_not_order() {
        let file = json!([{"context": "lint"}, {"context": "test"}]);
        let live = json!([{"context": "test", "integration_id": 1}, {"context": "lint"}]);
        assert!(diff(&file, &live, "x").is_empty());
        let renamed = json!([{"type": "checks", "parameters": {"strict": true}}]);
        let live2 = json!([{"type": "checks", "parameters": {"strict": false}}]);
        assert_eq!(
            diff(&renamed, &live2, "x"),
            vec!["x[type=\"checks\"].parameters.strict: file true, live false"]
        );
        let fewer = json!([{"context": "lint"}]);
        let out = diff(&file, &fewer, "x");
        assert_eq!(out.len(), 2, "{out:?}");
    }
}
