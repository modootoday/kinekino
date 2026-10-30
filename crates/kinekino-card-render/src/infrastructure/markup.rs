use crate::domain::{
    CardTemplate, CardText, FontWeight, FrameFit, Rect, Rgb, TextAlignment, TextOrder, TextSlot,
};

pub(crate) fn document<'a>(
    template: &CardTemplate,
    selection: impl Into<CardText<'a>>,
    frame_url: &str,
    fonts: &[String],
) -> String {
    document_with_page(template, selection, frame_url, fonts, None)
}

pub(crate) fn document_with_page<'a>(
    template: &CardTemplate,
    selection: impl Into<CardText<'a>>,
    frame_url: &str,
    fonts: &[String],
    page: Option<&crate::domain::PageFooter>,
) -> String {
    let selection = selection.into();
    let mut css = String::new();
    for (index, url) in fonts.iter().enumerate() {
        let (family, weight) = match index {
            0 => ("CardCommon", 400),
            1 => ("CardCommon", 600),
            _ => ("CardFallback", 400),
        };
        css.push_str(&format!(
            "@font-face{{font-family:'{family}';font-weight:{weight};src:url('{url}')}}"
        ));
    }
    let fallback = if fonts.len() > 2 { ",CardFallback" } else { "" };
    css.push_str(&format!("html,body{{margin:0;padding:0;width:{}px;height:{}px;background:{}}}#card{{position:relative;width:{}px;height:{}px;overflow:visible}}.slot{{position:absolute;overflow:visible}}.content{{margin:0;padding:0;white-space:pre-wrap;overflow-wrap:anywhere;font-family:CardCommon{fallback};font-synthesis:none}}", template.canvas.width, template.canvas.height, color(template.background), template.canvas.width, template.canvas.height));
    let fit = match template.fit {
        FrameFit::Contain => "contain",
        FrameFit::Cover => "cover",
    };
    css.push_str(&format!(
        "#frame-layer{{position:absolute;{}object-fit:{fit};object-position:{}% {}%}}",
        rect(template.frame),
        template.frame_anchor_x_percent,
        template.frame_anchor_y_percent
    ));
    css.push_str(&slot("front", template.front));
    css.push_str(&slot("back", template.back));
    let page_node = if let Some(page) = page {
        css.push_str(&slot("page", page.slot));
        text_node("page", &page.text())
    } else {
        String::new()
    };
    let front = text_node("front", selection.front);
    let back = text_node("back", selection.back);
    let texts = match template.text_order {
        TextOrder::FrontThenBack => format!("{front}{back}"),
        TextOrder::BackThenFront => format!("{back}{front}"),
    };
    let frame = if frame_url.is_empty() {
        String::new()
    } else {
        format!("<img id=\"frame-layer\" src=\"{frame_url}\">")
    };
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><style>{css}</style></head><body><div id=\"card\">{frame}{texts}{page_node}</div></body></html>"
    )
}

fn text_node(id: &str, text: &str) -> String {
    format!(
        "<div id=\"{id}-slot\" class=\"slot\"><div id=\"{id}-content\" class=\"content\">{}</div></div>",
        escape(text)
    )
}

fn slot(id: &str, slot: TextSlot) -> String {
    let weight = match slot.weight {
        FontWeight::Regular => 400,
        FontWeight::Semibold => 600,
    };
    let align = match slot.alignment {
        TextAlignment::Start => "start",
        TextAlignment::Center => "center",
        TextAlignment::End => "end",
    };
    format!(
        "#{id}-slot{{{}}}#{id}-content{{width:{}px;font-size:{}px;line-height:{}px;font-weight:{weight};text-align:{align};color:{}}}",
        rect(slot.rect),
        slot.rect.width,
        slot.font_px,
        slot.line_height_px,
        color(slot.color)
    )
}

fn rect(rect: Rect) -> String {
    format!(
        "left:{}px;top:{}px;width:{}px;height:{}px;",
        rect.x, rect.y, rect.width, rect.height
    )
}

fn color(Rgb(r, g, b): Rgb) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
