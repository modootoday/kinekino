use crate::{domain::playback, infrastructure::files};
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use std::path::Path;

fn has_error(value: &Value) -> bool {
    match value {
        Value::Null | Value::Bool(false) => false,
        Value::Number(number) => number.as_f64().is_none_or(|n| n != 0.0),
        Value::String(text) => !text.is_empty(),
        _ => true,
    }
}

pub fn summarize(args: &[String]) -> Result<Value> {
    ensure!(args.len() >= 4, "soak PATH web|native SECONDS [live]");
    let path = Path::new(&args[1]);
    let native = args[2] == "native";
    ensure!(native || args[2] == "web", "invalid playback mode");
    let target: f64 = args[3].parse()?;
    ensure!(
        target.is_finite() && target > 0.,
        "target must be positive and finite"
    );
    let live = args.get(4).is_some_and(|s| s == "live");
    let events = files::events(path, live)?;
    let kind = if native {
        "native-playback"
    } else {
        "web-playback"
    };
    let points = events.iter().filter(|e| e["kind"] == kind).map(|event| {
        let data = if native {
            &event["data"]
        } else {
            &event["data"]["observation"]
        };
        let time = data["time"].as_f64().unwrap_or_default();
        let stamp = event["elapsed_ms"].as_f64().unwrap_or(-1.0) / 1000.0;
        let valid = stamp >= 0.0
            && data["paused"] == false
            && !has_error(&data["error"])
            && playback::eligible_media(
                time,
                data["duration"].as_f64().unwrap_or_default(),
                target,
            )
            && data["width"].as_f64().unwrap_or_default() > 0.0
            && (!native || (data["connected"] == true && data["idle"] == false));
        valid.then_some((stamp, time))
    });
    let run = playback::longest(points);
    let native_errors = events
        .iter()
        .filter(|e| e["kind"] == "native-playback" && has_error(&e["data"]["error"]))
        .count();
    let web: Vec<_> = events
        .iter()
        .filter(|e| e["kind"] == "web-playback")
        .collect();
    let web_errors = web
        .iter()
        .filter(|e| has_error(&e["data"]["observation"]["error"]))
        .count();
    Ok(
        json!({"mode":args[2],"target_seconds":target,"passed":run.wall_seconds.min(run.media_seconds)>=target && native_errors==0 && web_errors==0,
        "longest_continuous":run,"native_errors":native_errors,"web_error_samples":web_errors,
        "known_visible_ads_max":web.iter().filter_map(|e|e["data"]["observation"]["visible_ads"].as_u64()).max(),
        "blocked_requests_max":web.iter().filter_map(|e|e["data"]["blocked"].as_u64()).max(),
        "log_sha256":if live { None } else { Some(files::hash(path)?) }}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn browser_numeric_zero_and_false_are_not_media_errors() {
        for value in [Value::Null, json!(false), json!(0), json!("")] {
            assert!(!has_error(&value));
        }
        for value in [json!(true), json!(3), json!("decode failed")] {
            assert!(has_error(&value));
        }
    }
}
