//! Allowlisted demo artwork, read at runtime from the private design folder
//! (`.spec.local/kinekino/design/20261029113508-ui/demo`, or `$KINEKINO_DEMO_ASSETS`). It is not part of the
//! public tree; when it is absent every lookup is `None` and callers draw placeholders.
//! Only the fixed file names below are opened; no model-provided path is used.
mod alpha;
use eframe::egui;

pub struct DemoAsset {
    pub uri: &'static str,
    pub bytes: &'static [u8],
    pub size: egui::Vec2,
    pub transparent: bool,
}

pub fn find(id: &str) -> Option<DemoAsset> {
    let (uri, file, size, transparent): (&str, &str, egui::Vec2, bool) = match id {
        "demo-forest-subject-mask-v1" => (
            "bytes://demo/mask/forest-subject-v1.svg",
            "masks/forest-subject-v1.svg",
            egui::vec2(1672., 941.),
            true,
        ),
        "demo-anime-coast-v1" => (
            "bytes://demo/anime/coastal-platform-v1.png",
            "anime/coastal-platform-v1.png",
            egui::vec2(1672., 941.),
            false,
        ),
        "demo-anime-forest-v1" => (
            "bytes://demo/anime/forest-stream-v1.png",
            "anime/forest-stream-v1.png",
            egui::vec2(1672., 941.),
            false,
        ),
        "demo-anime-forest-plate-v1" => (
            "bytes://demo/anime/forest-plate-v1.png",
            "anime/forest-plate-v1.png",
            egui::vec2(1672., 941.),
            false,
        ),
        "demo-anime-traveler-v2" => (
            "bytes://demo/anime/traveler-layer-v2.png",
            "anime/traveler-layer-v2.png",
            egui::vec2(1024., 1536.),
            true,
        ),
        _ => return None,
    };
    let bytes = load(file)?;
    Some(DemoAsset {
        uri,
        bytes,
        size,
        transparent,
    })
}

fn root() -> std::path::PathBuf {
    std::env::var_os("KINEKINO_DEMO_ASSETS").map_or_else(
        || {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../.spec.local/kinekino/design/20261029113508-ui/demo")
        },
        std::path::PathBuf::from,
    )
}

/// Each allowlisted file is read once and kept for the process lifetime.
fn load(file: &'static str) -> Option<&'static [u8]> {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<HashMap<&'static str, Option<&'static [u8]>>>> = OnceLock::new();
    let mut cache = CACHE.get_or_init(Default::default).lock().ok()?;
    *cache.entry(file).or_insert_with(|| {
        std::fs::read(root().join(file))
            .ok()
            .map(|b| &*Box::leak(b.into_boxed_slice()))
    })
}

impl DemoAsset {
    pub fn paint(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        if self.transparent {
            crate::presentation::components::checkerboard::paint(ui, rect);
        }
        let ratios = rect.size() / self.size;
        let scale = ratios.x.min(ratios.y);
        let image_rect = egui::Rect::from_center_size(rect.center(), self.size * scale);
        self.paint_in_rect(ui, image_rect, rect);
    }

    pub fn paint_in_rect(&self, ui: &mut egui::Ui, image_rect: egui::Rect, clip: egui::Rect) {
        let mut image_ui = ui.new_child(egui::UiBuilder::new().max_rect(clip));
        image_ui.set_clip_rect(clip.intersect(ui.clip_rect()));
        egui::Image::from_bytes(self.uri, self.bytes).paint_at(&image_ui, image_rect);
    }
}
