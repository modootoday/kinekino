//! Plan P6 settings window on Meari: one native code path for Windows, Linux and macOS.
//! The page model and its handling stay in `kinekino-app`; this crate only renders the
//! page with the shared design tokens/components and turns user actions into the page
//! protocol (`page`, `apply`, `dialog`, `ready`, `close`).
mod session;
mod source;
mod styles;
mod window;

pub use session::Session;
pub use styles::{Styles, fetch_fonts, prepare_with};
pub use window::MeariSettingsWindow;

/// The invalid batch a smoke run applies: shows the atomic-reject path on screen.
pub const SMOKE_EDITS: &str =
    r#"{"inference.budget.monthly_usd":3,"inference.budget.confirm_ratio":2}"#;

/// Writes RGBA pixels as PNG.
pub fn write_png(
    path: &std::path::Path,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(rgba).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())
}
