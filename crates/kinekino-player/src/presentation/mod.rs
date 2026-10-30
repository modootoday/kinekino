mod caption_controls;
mod components;
mod control_icon;
mod design_assets;
mod design_preview;
mod design_tokens;
mod icon_asset;
mod layout;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub(crate) mod playback_range;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub(crate) mod playback_time;
mod player_app;
mod player_theme;
mod typography;
mod validation;
#[cfg(all(windows, feature = "native-proposal-window"))]
pub mod web_ui_adapter;
mod window_frame;

pub fn run(smoke: bool, start_url: String, position: Option<[f32; 2]>) -> anyhow::Result<()> {
    run_with_client(
        smoke,
        start_url,
        position,
        std::sync::Arc::<crate::application::api::PlayerSdk>::new(Default::default()),
        std::rc::Rc::new(crate::application::subtitles::path_picker::TextPathOnly),
    )
}

pub fn run_with_client(
    smoke: bool,
    start_url: String,
    position: Option<[f32; 2]>,
    client: crate::application::api::ApplicationClientHandle,
    paths: crate::application::subtitles::path_picker::SubtitlePathPickerHandle,
) -> anyhow::Result<()> {
    run_with_native_status(smoke, start_url, position, client, paths, None)
}

pub fn run_with_native_status(
    smoke: bool,
    start_url: String,
    position: Option<[f32; 2]>,
    client: crate::application::api::ApplicationClientHandle,
    paths: crate::application::subtitles::path_picker::SubtitlePathPickerHandle,
    publisher: Option<crate::application::playback_status::NativeStatusPublisher>,
) -> anyhow::Result<()> {
    run_with_native_playback(smoke, start_url, position, client, paths, publisher, None)
}

pub fn run_with_native_playback(
    smoke: bool,
    start_url: String,
    position: Option<[f32; 2]>,
    client: crate::application::api::ApplicationClientHandle,
    paths: crate::application::subtitles::path_picker::SubtitlePathPickerHandle,
    publisher: Option<crate::application::playback_status::NativeStatusPublisher>,
    binder: Option<crate::infrastructure::native_playback_control::NativeControlBinder>,
) -> anyhow::Result<()> {
    let result = std::rc::Rc::new(std::cell::Cell::new(false));
    let app_result = result.clone();
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size([1180., 760.])
        .with_decorations(false)
        .with_min_inner_size([760., 480.]);
    if let Some(position) = position {
        viewport = viewport.with_position(position);
    }
    let options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        crate::domain::product::PRODUCT.display_name,
        options,
        Box::new(move |cc| {
            Ok(Box::new(
                player_app::PlayerApp::new(
                    cc,
                    smoke.then_some(app_result),
                    start_url,
                    client,
                    paths,
                )?
                .with_native_status(publisher)
                .with_native_control(binder),
            ))
        }),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    anyhow::ensure!(
        !smoke || result.get(),
        "WebView startup test did not observe a finished page and resource requests"
    );
    Ok(())
}

#[cfg(test)]
mod test_support;

/// Launch the offline design review host without constructing media adapters.
pub fn run_preview(position: Option<[f32; 2]>) -> anyhow::Result<()> {
    design_preview::run(position)
}
