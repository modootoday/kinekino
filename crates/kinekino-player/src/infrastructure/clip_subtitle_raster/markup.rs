use super::*;
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}
fn rgba(color: [u8; 4]) -> String {
    format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        color[0], color[1], color[2], color[3]
    )
}
pub(super) fn document(
    style: &ClipSubtitleStyle,
    text: &str,
    b: SubtitlePixelBox,
    width: u32,
    height: u32,
    fonts: &[String],
) -> String {
    let mut css = String::new();
    let mut families = Vec::new();
    for (i, font) in fonts.iter().enumerate() {
        css.push_str(&format!(
            "@font-face{{font-family:SubtitleFont{i};font-weight:400;src:url('{font}')}}"
        ));
        families.push(format!("SubtitleFont{i}"));
    }
    let align = match style.alignment {
        ClipSubtitleAlignment::Left => "left",
        ClipSubtitleAlignment::Center => "center",
        ClipSubtitleAlignment::Right => "right",
    };
    css.push_str(&format!("html,body{{margin:0;padding:0;width:{width}px;height:{height}px;background:transparent}}#subtitle-slot{{position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;background:{}}}#subtitle-text{{margin:0;padding:0;width:{}px;white-space:pre-wrap;overflow-wrap:anywhere;font-family:{};font-weight:400;font-synthesis:none;font-size:{}px;line-height:{}px;text-align:{align};color:{}}}", b.x,b.y,b.width,b.height,rgba(style.background_rgba),b.width,families.join(","),style.font_px,style.line_height_px,rgba(style.foreground_rgba)));
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><style>{css}</style></head><body><div id=\"subtitle-slot\"><div id=\"subtitle-text\">{}</div></div></body></html>",
        escape(text)
    )
}
