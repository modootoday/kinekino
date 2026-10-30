//! Fixed product-owned style sources and memory assets; no launch-selected paths or fallback IO.
use meari::{
    application::assets::VerifiedAssets,
    domain::{
        assets::{AssetDescriptor, AssetManifest},
        build::{BuildPath, ContentHash, InputRole},
        design::stylesheet::{StylesheetMode, StylesheetProject},
    },
};
use meari_native::infrastructure::stylesheet_assets::{CssAssetReference, rewrite};
use std::collections::BTreeMap;

const STYLES: [(&str, &str); 4] = [
    (
        "crates/kinekino-player/src/presentation/web_ui/styles/tokens.css",
        include_str!("../../presentation/web_ui/styles/tokens.css"),
    ),
    (
        "crates/kinekino-player/src/presentation/web_ui/styles/components.css",
        include_str!("../../presentation/web_ui/styles/components.css"),
    ),
    (
        "crates/kinekino-player/src/presentation/web_ui/styles/studio.css",
        include_str!("../../presentation/web_ui/styles/studio.css"),
    ),
    (
        "crates/kinekino-player/src/presentation/web_ui/styles/subtitles.css",
        include_str!("../../presentation/web_ui/styles/subtitles.css"),
    ),
];

pub struct PreparedProductStyles {
    pub css: String,
    pub assets: VerifiedAssets,
    pub references: Vec<CssAssetReference>,
    pub source_hashes: BTreeMap<String, ContentHash>,
    /// `(notice path, OFL text)` of every delivered Noto face.
    pub font_licenses: Vec<(String, String)>,
}

pub fn prepare_product_styles() -> Result<PreparedProductStyles, String> {
    prepare_styles(&STYLES, false)
}

pub fn prepare_viewing_styles() -> Result<PreparedProductStyles, String> {
    prepare_styles(
        &[
            STYLES[0],
            STYLES[1],
            (
                "crates/kinekino-player/src/presentation/web_ui/styles/viewing.css",
                include_str!("../../presentation/web_ui/styles/viewing.css"),
            ),
        ],
        true,
    )
}

/// One fixed asset/font manifest for both registered workspaces on the same document.
pub fn prepare_product_frame_styles() -> Result<PreparedProductStyles, String> {
    prepare_styles(
        &[
            STYLES[0],
            STYLES[1],
            (
                "crates/kinekino-player/src/presentation/web_ui/styles/viewing.css",
                include_str!("../../presentation/web_ui/styles/viewing.css"),
            ),
            STYLES[2],
            STYLES[3],
        ],
        true,
    )
}

fn prepare_styles(styles: &[(&str, &str)], viewing: bool) -> Result<PreparedProductStyles, String> {
    let mut manifest = AssetManifest {
        schema_version: 1,
        files: BTreeMap::new(),
    };
    let mut blobs = BTreeMap::new();
    let mut add = |path: &str,
                   bytes: &[u8],
                   role: InputRole,
                   media_type: &str,
                   license: Option<&str>|
     -> Result<(), String> {
        let hash = ContentHash::of(bytes);
        manifest.files.insert(
            BuildPath::try_from(path.to_owned())?,
            AssetDescriptor {
                role,
                content: hash.clone(),
                length: bytes.len() as u64,
                media_type: media_type.into(),
                license: license.map(str::to_owned),
            },
        );
        blobs.insert(hash, bytes.to_vec());
        Ok(())
    };
    // Pinned Noto faces from the verified cache (never compiled in, never downloaded here).
    let root = crate::infrastructure::noto_fonts::cache_root();
    let mut delivered = Vec::new();
    let mut font_licenses = Vec::new();
    for face in crate::infrastructure::noto_fonts::PRODUCT_FACES {
        let bytes = crate::infrastructure::noto_fonts::load(&root, &face)?;
        let notice = format!("fonts/noto/{}-OFL.txt", face.dir);
        let source = crate::infrastructure::noto_fonts::source_path(&face);
        add(
            &source,
            &bytes,
            InputRole::Font,
            "font/ttf",
            Some(&format!("OFL-1.1; {notice}")),
        )?;
        delivered.push((face.family, source));
        font_licenses.push((
            notice,
            crate::infrastructure::noto_fonts::notice(&root, &face)?,
        ));
    }
    macro_rules! icon {
        ($directory:literal, $name:literal) => {
            add(
                concat!("assets/design/", $directory, "/", $name, ".svg"),
                include_bytes!(concat!(
                    "../../../../../assets/design/",
                    $directory,
                    "/",
                    $name,
                    ".svg"
                )),
                InputRole::Icon,
                "image/svg+xml",
                None,
            )?;
        };
    }
    icon!("window", "minimize");
    icon!("window", "maximize");
    icon!("window", "restore");
    icon!("window", "close");
    icon!("icons", "brand");
    icon!("icons", "folder");
    icon!("icons", "search");
    icon!("icons", "previous");
    icon!("icons", "play");
    icon!("icons", "pause");
    icon!("icons", "next");
    icon!("icons", "volume");
    icon!("icons", "list");
    icon!("icons", "fullscreen");
    icon!("icons", "info");
    icon!("icons", "subtitles");
    icon!("icons", "back");
    icon!("icons", "segments");
    icon!("icons", "subject");
    icon!("icons", "clip");
    icon!("icons", "image");
    icon!("icons", "cards");
    icon!("icons", "layers");
    icon!("icons", "cube");
    icon!("icons", "model");
    icon!("icons", "export");
    icon!("icons", "warning");
    icon!("icons", "merge");
    icon!("icons", "plus");
    icon!("icons", "delete");
    icon!("icons", "undo");
    icon!("icons", "redo");
    if viewing {
        icon!("icons", "start-mark");
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
    let mut faces = String::new();
    for (family, source) in delivered {
        let output = manifest.files[&BuildPath::try_from(source)?].output_path()?;
        faces.push_str(&format!(
            "@font-face{{font-family:\"{family}\";src:url(\"/{}\") format(\"truetype\");font-style:normal;font-weight:100 900;font-display:swap;}}\n",
            output.as_str()
        ));
    }
    let rewritten = rewrite(&project, &manifest).map_err(|error| error.to_string())?;
    let css = std::iter::once(faces.as_str())
        .chain(
            styles
                .iter()
                .map(|(path, _)| rewritten.project.files[*path].as_str()),
        )
        .collect::<Vec<_>>()
        .join("\n");
    Ok(PreparedProductStyles {
        css,
        assets: VerifiedAssets::new(manifest, blobs)?,
        references: rewritten.references,
        source_hashes: styles
            .iter()
            .map(|(path, css)| ((*path).into(), ContentHash::of(css.as_bytes())))
            .collect(),
        font_licenses,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Style sources are declared repo-root-relative like their `url()` references and assets.
    #[test]
    fn every_registered_style_set_resolves_its_asset_urls() {
        for prepared in [
            prepare_product_styles(),
            prepare_viewing_styles(),
            prepare_product_frame_styles(),
        ] {
            let styles = prepared.unwrap();
            assert!(!styles.css.contains("../assets/"));
        }
    }
    #[test]
    fn product_sources_rewrite_all_fonts_and_masks_to_verified_content() {
        let prepared = prepare_product_styles().unwrap();
        assert_eq!(prepared.references.len(), 33);
        assert_eq!(prepared.assets.manifest().files.len(), 35);
        assert!(
            prepared
                .assets
                .manifest()
                .files
                .keys()
                .any(|path| path.as_str().ends_with("icons/pause.svg"))
        );
        assert_eq!(prepared.source_hashes.len(), 4);
        assert!(!prepared.css.contains("../../../../assets"));
        assert!(prepared.css.contains("font-family:\"Noto Sans KR\""));
        for reference in prepared.references {
            assert!(prepared.assets.resource(&reference.output).is_some());
            assert!(prepared.css.contains(reference.output.as_str()));
        }
        for (_, text) in prepared.font_licenses {
            assert!(text.contains("SIL OPEN FONT LICENSE"));
        }
    }

    #[test]
    fn native_full_product_css_delivers_three_ttf_faces_and_paints_the_chrome_token() {
        use meari_native::{
            application::Renderer, domain::Viewport, infrastructure::document::InteractiveDocument,
        };
        let prepared = prepare_product_styles().unwrap();
        let fonts: Vec<_> = prepared
            .assets
            .manifest()
            .files
            .values()
            .filter(|asset| asset.role == InputRole::Font)
            .map(|asset| asset.output_path().unwrap().as_str().to_owned())
            .collect();
        let component = meari::parse("type Props={};export function Probe(props:Props){return <main className=\"aw-window-chrome\"><span>한글 본문</span><span className=\"probe-semibold\">강조</span><span className=\"probe-jp\">日本語かな</span><span className=\"aw-icon aw-icon-brand\"/></main>;}", meari::application::Language::Tsx).unwrap();
        let css = format!(
            "{}\n.probe-semibold{{font-family:'Noto Sans KR';font-weight:600}}.probe-jp{{font-family:'Noto Sans JP';font-weight:400}}",
            prepared.css
        );
        let viewport = Viewport {
            width: 320,
            height: 160,
            dark: true,
        };
        let mut document = InteractiveDocument::new_with_assets(
            component,
            &css,
            viewport.clone(),
            prepared.assets,
        )
        .unwrap();
        let pixels = document.renderer.rasterize(&viewport).unwrap();
        let report = document.renderer.asset_report().unwrap().unwrap();
        assert_eq!(report.rejected, 0, "{report:?}");
        assert!(report.failures.is_empty(), "{report:?}");
        for font in fonts {
            assert!(
                report.delivered.get(&font).is_some_and(|count| *count > 0),
                "Missing {font}: {report:?}"
            );
        }
        let pixel = (5 * 320 + 5) * 4;
        assert_eq!(&pixels[pixel..pixel + 4], &[25, 27, 31, 255]);
    }
}
