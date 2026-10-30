use super::{FontSelection, OptionalFrameRenderRequest, RenderError};
use meari::{
    application::assets::VerifiedAssets,
    domain::{
        assets::AssetDescriptor,
        build::{BuildPath, ContentHash, InputRole},
    },
};

pub(super) struct RenderAssets {
    pub frame_url: String,
    pub font_urls: Vec<String>,
    pub font_sha256: Vec<ContentHash>,
}

pub(super) fn descriptor<'a>(
    assets: &'a VerifiedAssets,
    path: &BuildPath,
) -> Result<&'a AssetDescriptor, RenderError> {
    assets
        .manifest()
        .files
        .get(path)
        .ok_or_else(|| RenderError::Asset("asset is undeclared".into()))
}

pub(super) fn resource<'a>(
    assets: &'a VerifiedAssets,
    descriptor: &AssetDescriptor,
) -> Result<&'a [u8], RenderError> {
    let path = descriptor.output_path().map_err(RenderError::Asset)?;
    assets
        .resource(&path)
        .map(|(_, bytes)| bytes)
        .ok_or_else(|| RenderError::Asset("verified asset bytes missing".into()))
}

pub(super) fn prepare(
    request: &OptionalFrameRenderRequest<'_>,
) -> Result<RenderAssets, RenderError> {
    let frame_url = if let Some(pin) = &request.frame {
        let frame = descriptor(&request.assets, &pin.asset)?;
        if frame.role != InputRole::Image || frame.media_type != "image/png" {
            return Err(RenderError::InvalidInput(
                "frame must be a declared PNG image",
            ));
        }
        if frame.content != pin.sha256 {
            return Err(RenderError::FramePinMismatch);
        }
        frame
            .output_path()
            .map_err(RenderError::Asset)?
            .as_str()
            .to_owned()
    } else {
        String::new()
    };
    let mut font_urls = Vec::new();
    let mut font_sha256 = Vec::new();
    for path in font_paths(&request.fonts) {
        let font = descriptor(&request.assets, path)?;
        if font.role != InputRole::Font
            || font.license.is_none()
            || !matches!(font.media_type.as_str(), "font/ttf" | "font/otf")
        {
            return Err(RenderError::InvalidInput(
                "declared licensed common font required",
            ));
        }
        font_urls.push(
            font.output_path()
                .map_err(RenderError::Asset)?
                .as_str()
                .to_owned(),
        );
        font_sha256.push(font.content.clone());
    }
    Ok(RenderAssets {
        frame_url,
        font_urls,
        font_sha256,
    })
}

fn font_paths(fonts: &FontSelection) -> Vec<&BuildPath> {
    let mut paths = vec![&fonts.regular, &fonts.semibold];
    if let Some(fallback) = &fonts.fallback {
        paths.push(fallback);
    }
    paths
}
