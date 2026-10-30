use crate::{application::session::BrowserEvent, infrastructure::adblock};
use anyhow::Result;
use std::{cell::RefCell, rc::Rc};
use webview2_com::Microsoft::Web::WebView2::Win32::*;
use webview2_com::{CoTaskMemPWSTR, WebResourceRequestedEventHandler};
use windows::{
    Win32::System::Com::IStream,
    core::{Interface, PWSTR, w},
};
use wry::{WebView, WebViewExtWindows};

pub(super) fn install_filter(
    view: &WebView,
    tx: super::events::EventSink,
    enabled: Rc<RefCell<bool>>,
) -> Result<()> {
    let extra = std::fs::read_to_string("filters.local.txt").unwrap_or_default();
    let engine = adblock::engine(&extra);
    let environment = view.environment();
    // All COM interfaces and callbacks remain on the WebView's owning UI thread.
    unsafe {
        let core = view.webview();
        core.cast::<ICoreWebView2_22>()?
            .AddWebResourceRequestedFilterWithRequestSourceKinds(
                w!("*"),
                COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                COREWEBVIEW2_WEB_RESOURCE_REQUEST_SOURCE_KINDS_ALL,
            )?;
        let handler = WebResourceRequestedEventHandler::create(Box::new(move |sender, args| {
            let Some(args) = args else {
                return Ok(());
            };
            let request = args.Request()?;
            let _ = tx.send(BrowserEvent::Requested);
            let mut uri = PWSTR::null();
            request.Uri(&mut uri)?;
            let url = CoTaskMemPWSTR::from(uri).to_string();
            let mut method = PWSTR::null();
            request.Method(&mut method)?;
            let method = CoTaskMemPWSTR::from(method).to_string();
            let mut source = PWSTR::null();
            let _ = request.Headers()?.GetHeader(w!("Referer"), &mut source);
            let mut source = CoTaskMemPWSTR::from(source).to_string();
            if source.is_empty()
                && let Some(sender) = sender
            {
                let mut current = PWSTR::null();
                let _ = sender.Source(&mut current);
                source = CoTaskMemPWSTR::from(current).to_string();
            }
            let mut context = COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL;
            args.ResourceContext(&mut context)?;
            if *enabled.borrow()
                && adblock::blocked(&engine, &url, &source, resource_kind(context), &method)
            {
                let response = environment.CreateWebResourceResponse(
                    None::<&IStream>,
                    403,
                    w!("Blocked"),
                    w!("Content-Type: text/plain\r\n"),
                )?;
                args.SetResponse(&response)?;
                let _ = tx.send(BrowserEvent::Blocked);
            } else if let Some(source) =
                crate::domain::media::MediaSource::from_request(url, source)
            {
                let _ = tx.send(BrowserEvent::Media(source));
            }
            Ok(())
        }));
        let mut token = 0;
        core.add_WebResourceRequested(&handler, &mut token)?;
    }
    Ok(())
}

pub(crate) fn resource_kind(context: COREWEBVIEW2_WEB_RESOURCE_CONTEXT) -> &'static str {
    match context {
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_SCRIPT => "script",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_IMAGE => "image",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_STYLESHEET => "stylesheet",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FONT => "font",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_MEDIA => "media",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_XML_HTTP_REQUEST
        | COREWEBVIEW2_WEB_RESOURCE_CONTEXT_FETCH => "xmlhttprequest",
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT => "subdocument",
        _ => "other",
    }
}
