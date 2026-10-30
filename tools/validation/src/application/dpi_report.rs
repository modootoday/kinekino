use crate::{domain::dpi::Scenario, infrastructure::files};
use anyhow::Result;
use serde_json::{Value, json};
use std::path::Path;

pub fn summarize(path: &Path) -> Result<Value> {
    let mut scenarios: Vec<_> = [96, 144, 192]
        .into_iter()
        .map(|dpi| Scenario {
            dpi,
            ..Default::default()
        })
        .collect();
    let mut current = None;
    for event in files::events(path, false)? {
        let data = &event["data"];
        match event["kind"].as_str() {
            Some("window-metrics") => {
                let window = &data["window"];
                let dpi = window["dpi"].as_u64().unwrap_or_default();
                current = scenarios.iter().position(|s| {
                    s.geometry_matches(
                        dpi,
                        data["pixels_per_point"].as_f64().unwrap_or_default(),
                        window["primary_monitor"].as_bool().unwrap_or(true),
                    )
                });
                if let Some(i) = current {
                    let s = &mut scenarios[i];
                    s.window_samples += 1;
                    let full = data["fullscreen"] == true;
                    s.normal_window |= !full;
                    s.fullscreen_bounds_match |= full && window["window"] == window["monitor"];
                    s.controls_hidden |= full && data["controls_visible"] == false;
                    s.controls_visible |= full && data["controls_visible"] == true;
                    s.panel_open |= data["episode_panel"] == true;
                    s.panel_closed |= data["episode_panel"] == false;
                    s.episode_count_max = s
                        .episode_count_max
                        .max(data["episode_count"].as_u64().unwrap_or_default());
                }
            }
            Some("native-playback") => {
                if let Some(i) = current {
                    let s = &mut scenarios[i];
                    s.native_errors += usize::from(!data["error"].is_null());
                    if data["connected"] == true && data["time"].as_f64().unwrap_or_default() > 0. {
                        if data["paused"] == true {
                            s.native_paused_samples += 1;
                        } else {
                            s.native_playing_samples += 1;
                        }
                    }
                    if let Some(volume) = data["volume"].as_f64()
                        && !s.volumes.contains(&volume)
                    {
                        s.volumes.push(volume);
                    }
                }
            }
            Some("episode-selection") => {
                if let Some(i) = current
                    && let Some(index) = data["index"].as_u64()
                {
                    scenarios[i].selected_episodes.push(index);
                }
            }
            _ => (),
        }
    }
    Ok(
        json!({"source_sha256":files::hash(path)?,"scenarios":scenarios,"scope":"actual app HWND telemetry; UI screenshots/actions reviewed separately; nearby playback samples use preceding window sample"}),
    )
}
