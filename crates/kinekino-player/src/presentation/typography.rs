//! UI typography from the pinned Noto cache, independent of the operating system's fonts.
use super::design_tokens::text;
use crate::infrastructure::noto_fonts;
use eframe::egui::{
    self, FontData, FontDefinitions, FontFamily, FontId, FontTweak, TextStyle,
    epaint::text::VariationCoords,
};

const REGULAR: &str = text::REGULAR_FACE;
const SEMIBOLD: &str = text::SEMIBOLD_FACE;
const JAPANESE: &str = text::JAPANESE_FACE;
const LATIN: &str = text::LATIN_FACE;

/// Installs the pinned faces; a missing or mismatching cache keeps egui's defaults and is
/// returned so the caller can report `kinekino fonts fetch`.
pub fn install(ctx: &egui::Context) -> Result<(), String> {
    let root = noto_fonts::cache_root();
    let korean = noto_fonts::load(&root, &noto_fonts::KOREAN)?;
    let japanese = noto_fonts::load(&root, &noto_fonts::JAPANESE)?;
    let latin = noto_fonts::load(&root, &noto_fonts::LATIN)?;
    let mut fonts = FontDefinitions::default();
    // The KR/JP variable faces default to wght 100; pin the token weights explicitly.
    let face = |bytes: &Vec<u8>, weight: f32| {
        FontData::from_owned(bytes.clone())
            .tweak(FontTweak {
                coords: VariationCoords::new([("wght", weight)]),
                ..FontTweak::default()
            })
            .into()
    };
    fonts.font_data.insert(LATIN.into(), face(&latin, 400.));
    fonts.font_data.insert(REGULAR.into(), face(&korean, 400.));
    fonts.font_data.insert(SEMIBOLD.into(), face(&korean, 600.));
    fonts
        .font_data
        .insert(JAPANESE.into(), face(&japanese, 400.));
    let proportional = fonts.families.entry(FontFamily::Proportional).or_default();
    proportional.insert(0, LATIN.into());
    proportional.insert(1, REGULAR.into());
    proportional.insert(2, JAPANESE.into());
    let mut heading = vec![SEMIBOLD.into()];
    heading.extend(proportional.iter().cloned());
    fonts
        .families
        .insert(FontFamily::Name(text::HEADING_FAMILY.into()), heading);
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push(REGULAR.into());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push(JAPANESE.into());
    ctx.set_fonts(fonts);
    ctx.all_styles_mut(|style| {
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(text::HEADING, FontFamily::Name(text::HEADING_FAMILY.into())),
        );
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_fonts_cover_korean_and_latin_without_system_fonts() {
        let ctx = egui::Context::default();
        install(&ctx).unwrap();
        let _ = crate::presentation::test_support::run(&ctx, Default::default(), |ui| {
            ui.fonts_mut(|fonts| {
                for family in [
                    FontFamily::Proportional,
                    FontFamily::Name("ui-heading".into()),
                ] {
                    for character in "한글 자막 파일 열기 English 0123456789 éü 日本語の長い題名 から、ここで話します"
                        .chars()
                        .filter(|character| !character.is_whitespace())
                    {
                        assert!(
                            fonts.has_glyph(&FontId::new(13., family.clone()), character),
                            "missing {character:?} in {family:?}"
                        );
                    }
                }
                let mono = FontId::new(13., FontFamily::Monospace);
                assert!(fonts.has_glyphs(&mono, "한글자막"));
                assert!(fonts.glyph_width(&mono, 'E') > 0.);
            });
        });
    }
}
