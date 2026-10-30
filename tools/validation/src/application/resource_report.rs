use crate::infrastructure::files;
use anyhow::Result;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

pub fn summarize(path: &Path) -> Result<Value> {
    let events = files::events(path, false)?;
    let mut groups: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    for event in &events {
        groups
            .entry(event["label"].as_str().unwrap_or("unknown").to_owned())
            .or_default()
            .push(event);
    }
    let groups: Vec<_> = groups.into_iter().map(|(label, values)| {
        let private = |v: &&Value| v["private_bytes"].as_f64().unwrap_or_default() / 1048576.0;
        json!({"label":label,"samples":values.len(),"root_id":values[0]["root_id"],
            "start_private_mib":private(&values[0]),"end_private_mib":private(values.last().unwrap()),
            "peak_private_mib":values.iter().map(private).fold(0.0_f64,f64::max),
            "elapsed_span_seconds":(values.last().unwrap()["elapsed_ms"].as_f64().unwrap_or_default()-values[0]["elapsed_ms"].as_f64().unwrap_or_default())/1000.0})
    }).collect();
    Ok(json!({"groups":groups,"log_sha256":files::hash(path)?,
        "scope":"process ancestry buckets; shared WebView can belong to exited stress root; not comparable web/native totals; no leak conclusion"}))
}
