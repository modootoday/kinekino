use crate::{
    application::session::BrowserEvent, domain::media::MediaSource, infrastructure::diagnostics,
};
use anyhow::Result;

use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_2;
use webview2_com::{
    CoTaskMemPWSTR, WebResourceResponseReceivedEventHandler,
    WebResourceResponseViewGetContentCompletedHandler,
};
use windows::core::{Interface, PWSTR, w};
use wry::{WebView, WebViewExtWindows};

pub(super) fn observe(view: &WebView, tx: super::events::EventSink) -> Result<()> {
    // These COM callbacks and allocated strings stay on the owning UI thread.
    unsafe {
        let handler = WebResourceResponseReceivedEventHandler::create(Box::new(
            move |sender, args| {
                let Some(args) = args else {
                    return Ok(());
                };
                let request = args.Request()?;
                let mut raw = PWSTR::null();
                request.Uri(&mut raw)?;
                let url = CoTaskMemPWSTR::from(raw).to_string();
                let mut mime = PWSTR::null();
                let _ = args
                    .Response()?
                    .Headers()?
                    .GetHeader(w!("Content-Type"), &mut mime);
                let mime = CoTaskMemPWSTR::from(mime).to_string();
                if let Ok(parsed) = url::Url::parse(&url) {
                    diagnostics::record(
                        "response",
                        serde_json::json!({"host":parsed.host_str(),"mime":mime,"extension":parsed.path().rsplit('.').next().filter(|s|s.len()<12)}),
                    );
                }
                if should_inspect(&url, &mime) {
                    let mut referer = PWSTR::null();
                    let _ = request.Headers()?.GetHeader(w!("Referer"), &mut referer);
                    let mut referer = CoTaskMemPWSTR::from(referer).to_string();
                    if referer.is_empty()
                        && let Some(sender) = sender
                    {
                        let mut raw = PWSTR::null();
                        let _ = sender.Source(&mut raw);
                        referer = CoTaskMemPWSTR::from(raw).to_string();
                    }
                    let callback_tx = tx.freeze();
                    let handler = WebResourceResponseViewGetContentCompletedHandler::create(
                        Box::new(move |error, stream| {
                            diagnostics::record(
                                "manifest-probe",
                                serde_json::json!({"success":error.is_ok(),"stream":stream.is_some()}),
                            );
                            if error.is_err() {
                                return Ok(());
                            }
                            let Some(stream) = stream else {
                                return Ok(());
                            };
                            let mut prefix = [0u8; 4096];
                            let mut read = 0;
                            stream
                                .Read(
                                    prefix.as_mut_ptr().cast(),
                                    prefix.len() as u32,
                                    Some(&mut read),
                                )
                                .ok()?;
                            diagnostics::record(
                                "manifest-prefix",
                                serde_json::json!({"bytes":read,"hls":prefix.starts_with(b"#EXTM3U"),"referer_host":url::Url::parse(&referer).ok().and_then(|u|u.host_str().map(str::to_owned))}),
                            );
                            if let Some(source) = MediaSource::from_manifest(
                                url.clone(),
                                referer.clone(),
                                &prefix[..read as usize],
                            ) {
                                let _ = callback_tx.send(BrowserEvent::Media(source));
                            }
                            Ok(())
                        }),
                    );
                    args.Response()?.GetContent(&handler)?;
                }
                Ok(())
            },
        ));
        let mut token = 0;
        view.webview()
            .cast::<ICoreWebView2_2>()?
            .add_WebResourceResponseReceived(&handler, &mut token)?;
    }
    Ok(())
}

fn should_inspect(url: &str, mime: &str) -> bool {
    let Ok(url) = url::Url::parse(url) else {
        return false;
    };
    url.scheme() == "https"
        && (mime.contains("mpegurl")
            || (url.host_str() == Some("media.example.test") && url.path().ends_with(".txt")))
}
