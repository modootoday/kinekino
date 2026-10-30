use crate::{application::session::BrowserEvent, domain::navigation};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use webview2_com::{CoTaskMemPWSTR, SourceChangedEventHandler};
use windows::core::PWSTR;
use wry::{WebView, WebViewExtWindows};

fn update_location(url: &str, generation: &Cell<u64>, document: &RefCell<String>) -> bool {
    if !navigation::navigation_allowed(url) || *document.borrow() == url {
        return false;
    }
    if super::generation::advance(generation).is_none() {
        return false;
    }
    *document.borrow_mut() = url.to_owned();
    true
}

pub(super) fn observe(
    view: &WebView,
    tx: super::events::EventSink,
    generation: Rc<Cell<u64>>,
    document: Rc<RefCell<String>>,
) -> anyhow::Result<()> {
    // SourceChanged includes URL changes that do not start a new document navigation.
    // SAFETY: WebView2 and these captured cells remain on the owning UI thread.
    unsafe {
        let handler = SourceChangedEventHandler::create(Box::new(move |sender, _| {
            let Some(sender) = sender else {
                return Ok(());
            };
            let mut raw = PWSTR::null();
            sender.Source(&mut raw)?;
            let url = CoTaskMemPWSTR::from(raw).to_string();
            if update_location(&url, &generation, &document) {
                crate::infrastructure::diagnostics::record(
                    "source-change",
                    serde_json::json!({
                        "host":url::Url::parse(&url).ok().and_then(|u|u.host_str().map(str::to_owned))
                    }),
                );
                let _ = tx.send(BrowserEvent::Navigated(url));
            }
            Ok(())
        }));
        let mut token = 0;
        view.controller()
            .CoreWebView2()?
            .add_SourceChanged(&handler, &mut token)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn engine_history_change_advances_generation_once_and_rejects_external_source() {
        let generation = Cell::new(4);
        let document = RefCell::new("https://catalog.example.test/".to_owned());
        let detail = "https://catalog.example.test/c/example";
        assert!(update_location(detail, &generation, &document));
        assert_eq!(generation.get(), 5);
        assert!(!update_location(detail, &generation, &document));
        assert!(!update_location(
            "https://untrusted.example/",
            &generation,
            &document
        ));
        assert_eq!(*document.borrow(), detail);
        assert_eq!(generation.get(), 5);
    }
}
