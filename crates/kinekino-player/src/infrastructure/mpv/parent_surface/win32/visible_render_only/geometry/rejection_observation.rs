//! Bounded, best-effort stderr metadata. Never an admission or cleanup receipt.
use super::*;
use serde_json::{Value, json};
use std::{
    io::Write,
    sync::atomic::{AtomicBool, Ordering},
};

static ATTEMPTED: AtomicBool = AtomicBool::new(false);
const MAX_BYTES: usize = 8192;

pub(super) fn reject(
    snapshot: &NativeSurfaceSnapshot,
    branch: &'static str,
    bounds: Option<[i32; 4]>,
) -> VisibleError {
    if crate::infrastructure::diagnostics::enabled() && !ATTEMPTED.swap(true, Ordering::Relaxed) {
        let record = payload(snapshot, branch, bounds);
        if let Ok(mut bytes) = serde_json::to_vec(&record)
            && bytes.len() < MAX_BYTES
        {
            bytes.push(b'\n');
            // Diagnostic I/O cannot replace the original Geometry failure.
            let _ = std::io::stderr().lock().write_all(&bytes);
        }
    }
    VisibleError::Geometry
}

fn scalar(value: f64) -> Value {
    json!({"value": value.is_finite().then_some(value), "bits": value.to_bits()})
}
fn rect(rect: meari_native::domain::surface_slot::PhysicalClientRect) -> Value {
    json!([
        scalar(rect.x),
        scalar(rect.y),
        scalar(rect.width),
        scalar(rect.height)
    ])
}
fn text(value: &str) -> Value {
    json!({"value": value.chars().take(128).collect::<String>(), "truncated": value.chars().count() > 128})
}
fn payload(
    snapshot: &NativeSurfaceSnapshot,
    branch: &'static str,
    bounds: Option<[i32; 4]>,
) -> Value {
    let predicate = match branch {
        "visibility_or_plane" if snapshot.visibility != SurfaceVisibility::Visible => "visibility",
        "visibility_or_plane" => "plane",
        "positive_bounds_or_clip_containment" => match (bounds, snapshot.clip) {
            (Some(b), _) if b[2] <= 0 => "nonpositive_width",
            (Some(b), _) if b[3] <= 0 => "nonpositive_height",
            (Some(b), SurfaceClip::Rectangular(c)) if c.x > f64::from(b[0]) => "clip_left",
            (Some(b), SurfaceClip::Rectangular(c)) if c.y > f64::from(b[1]) => "clip_top",
            (Some(b), SurfaceClip::Rectangular(c))
                if c.x + c.width < f64::from(b[0]) + f64::from(b[2]) =>
            {
                "clip_right"
            }
            (Some(_), SurfaceClip::Rectangular(_)) => "clip_bottom",
            _ => "unobserved_predicate",
        },
        other => other,
    };
    let clip = match snapshot.clip {
        SurfaceClip::Rectangular(value) => json!({"kind":"rectangular", "rect":rect(value)}),
        SurfaceClip::Unsupported => json!({"kind":"unsupported"}),
        SurfaceClip::Unknown => json!({"kind":"unknown"}),
    };
    json!({"kind":"native-routed-geometry-rejection", "schema_version":1,
        "authority":"diagnostic_only", "branch":branch, "predicate":predicate, "bounds":bounds,
        "visibility":format!("{:?}", snapshot.visibility), "plane":format!("{:?}", snapshot.plane),
        "rect":snapshot.physical_client_rect.map(rect), "clip":clip,
        "device_scale":scalar(snapshot.device_scale), "layout_zoom":scalar(snapshot.layout_zoom),
        "host_geometry_revision":snapshot.host_geometry_revision, "layout_revision":snapshot.layout_revision,
        "slot_generation":snapshot.slot_generation,
        "frame":{"context_id":text(&snapshot.frame.context_id), "context_generation":snapshot.frame.context_generation,
            "scene_generation":snapshot.frame.scene_generation},
        "document":text(&format!("{:?}", snapshot.document)), "window":text(&format!("{:?}", snapshot.window))})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostic_scalars_keep_nonfinite_bits_and_text_is_bounded() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.0, 1.25] {
            let observed = scalar(value);
            assert_eq!(observed["bits"].as_u64(), Some(value.to_bits()));
            assert_eq!(observed["value"].is_null(), !value.is_finite());
            assert!(serde_json::to_vec(&observed).is_ok());
        }
        let observed = text(&"한".repeat(129));
        assert_eq!(observed["value"].as_str().unwrap().chars().count(), 128);
        assert_eq!(observed["truncated"], true);
        assert_eq!(text("original")["truncated"], false);
    }
}
