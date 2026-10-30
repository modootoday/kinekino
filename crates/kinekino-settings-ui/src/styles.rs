//! Shared product tokens/components plus the derived settings layout, with the fonts
//! and design icons they reference delivered from memory (no runtime file IO).
use meari::{
    application::assets::VerifiedAssets,
    domain::{
        assets::{AssetDescriptor, AssetManifest},
        build::{BuildPath, ContentHash, InputRole},
        design::{
            fonts,
            stylesheet::{StylesheetMode, StylesheetProject},
        },
    },
};

/// Pinned Noto families delivered to the settings document (Latin, Korean, Japanese).
pub const FACES: [&str; 3] = ["Noto Sans", "Noto Sans KR", "Noto Sans JP"];
use std::collections::BTreeMap;

const STYLES: [(&str, &str); 3] = [
    (
        "crates/kinekino-player/src/presentation/web_ui/styles/tokens.css",
        include_str!("../../kinekino-player/src/presentation/web_ui/styles/tokens.css"),
    ),
    (
        "crates/kinekino-player/src/presentation/web_ui/styles/components.css",
        include_str!("../../kinekino-player/src/presentation/web_ui/styles/components.css"),
    ),
    (
        "crates/kinekino-settings-ui/src/settings.css",
        include_str!("settings.css"),
    ),
];

macro_rules! asset {
    ($path:literal) => {
        (
            $path,
            include_bytes!(concat!("../../../", $path)).as_slice(),
        )
    };
}
/// Every icon the shared CSS names must be declared, used here or not.
const ICONS: [(&str, &[u8]); 17] = [
    asset!("assets/design/icons/brand.svg"),
    asset!("assets/design/window/minimize.svg"),
    asset!("assets/design/window/maximize.svg"),
    asset!("assets/design/window/restore.svg"),
    asset!("assets/design/window/close.svg"),
    asset!("assets/design/icons/folder.svg"),
    asset!("assets/design/icons/search.svg"),
    asset!("assets/design/icons/previous.svg"),
    asset!("assets/design/icons/play.svg"),
    asset!("assets/design/icons/pause.svg"),
    asset!("assets/design/icons/next.svg"),
    asset!("assets/design/icons/volume.svg"),
    asset!("assets/design/icons/list.svg"),
    asset!("assets/design/icons/fullscreen.svg"),
    asset!("assets/design/icons/undo.svg"),
    asset!("assets/design/icons/warning.svg"),
    asset!("assets/design/icons/info.svg"),
];

fn face(family: &str) -> Result<fonts::Pin, String> {
    fonts::MANIFEST
        .iter()
        .find(|p| p.family == family && p.style == "normal")
        .copied()
        .ok_or_else(|| format!("{family} is not in the pinned Noto manifest"))
}

/// Downloads (once) and verifies the pinned Noto UI set into the font cache.
pub fn fetch_fonts() -> Result<serde_json::Value, String> {
    let cache = meari_native::infrastructure::fonts::FontCache::default_location();
    let fetched = cache.ensure(fonts::UI_SCRIPTS)?;
    Ok(serde_json::json!({
        "cache": cache.root(),
        "revision": fonts::REVISION,
        "files": fetched.iter().map(|(pin, path)| serde_json::json!({
            "family": pin.family, "style": pin.style, "path": path, "sha256": pin.sha256,
            "license": "OFL-1.1", "notice": cache.notice_path(pin),
        })).collect::<Vec<_>>(),
    }))
}

pub struct Styles {
    pub css: String,
    pub assets: VerifiedAssets,
}

pub fn prepare() -> Result<Styles, String> {
    prepare_with(&STYLES, &ICONS)
}

/// Fonts, the given stylesheets (first is the entry) and every icon they name, as one
/// verified asset set. Shared with the player window so both use one delivery path.
pub fn prepare_with(styles: &[(&str, &str)], icons: &[(&str, &[u8])]) -> Result<Styles, String> {
    let mut manifest = AssetManifest {
        schema_version: 1,
        files: BTreeMap::new(),
    };
    let mut blobs = BTreeMap::new();
    // Pinned Noto faces from the verified download cache; never compiled into the binary.
    let cache = meari_native::infrastructure::fonts::FontCache::default_location();
    let mut faces = String::new();
    let mut fonts = Vec::new();
    for face in FACES.map(face).into_iter().collect::<Result<Vec<_>, _>>()? {
        let bytes = cache.load(&face).map_err(|e| {
            format!("{e}; run `kinekino fonts fetch` to download the pinned Noto set")
        })?;
        let path = format!("fonts/noto/{}", face.file);
        let license = format!("OFL-1.1; fonts/noto/{}-OFL.txt", face.dir);
        fonts.push((path, bytes, license));
    }
    let entries = fonts
        .iter()
        .map(|(path, bytes, license)| {
            (
                (path.as_str(), bytes.as_slice()),
                InputRole::Font,
                "font/ttf",
                Some(license.as_str()),
            )
        })
        .chain(
            icons
                .iter()
                .map(|(p, b)| ((*p, *b), InputRole::Icon, "image/svg+xml", None)),
        );
    for ((path, bytes), role, media_type, license) in entries {
        let hash = ContentHash::of(bytes);
        manifest.files.insert(
            BuildPath::try_from((*path).to_owned())?,
            AssetDescriptor {
                role,
                content: hash.clone(),
                length: bytes.len() as u64,
                media_type: media_type.into(),
                license: license.map(str::to_owned),
            },
        );
        blobs.insert(hash, bytes.to_vec());
    }
    let project = StylesheetProject {
        mode: StylesheetMode::Css,
        entry: styles[0].0.into(),
        bridge_theme: false,
        files: styles
            .iter()
            .map(|(path, css)| ((*path).into(), (*css).into()))
            .collect(),
    };
    for (index, (path, _, _)) in fonts.iter().enumerate() {
        let output = manifest.files[&BuildPath::try_from(path.clone())?].output_path()?;
        faces.push_str(&format!(
            "@font-face{{font-family:\"{}\";src:url(\"/{}\") format(\"truetype\");font-style:normal;font-weight:100 900;font-display:swap;}}\n",
            FACES[index],
            output.as_str()
        ));
    }
    let rewritten = meari_native::infrastructure::stylesheet_assets::rewrite(&project, &manifest)
        .map_err(|e| e.to_string())?;
    let css = std::iter::once(faces.as_str())
        .chain(
            styles
                .iter()
                .map(|(path, _)| rewritten.project.files[*path].as_str()),
        )
        .collect::<Vec<_>>()
        .join("\n");
    Ok(Styles {
        css,
        assets: VerifiedAssets::new(manifest, blobs)?,
    })
}
