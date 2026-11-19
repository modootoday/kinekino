//! WebView2 in a Rust-owned Win32 window, then the port contract (Windows only).
//! Usage: cargo run --locked -p kinekino-browser --features webview2 --example demo_engine_webview2
#[cfg(windows)]
mod win {
    use kinekino_browser::engine::contract::{self, fixture};
    use kinekino_browser::engine::port::{BrowserEngine, ProtocolSession};
    use kinekino_browser::engine::webview2::WebView2Engine;
    use raw_window_handle::{
        HandleError, HasWindowHandle, RawWindowHandle, Win32WindowHandle, WindowHandle,
    };
    use std::num::NonZeroIsize;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, SW_SHOW,
        ShowWindow, WINDOW_EX_STYLE, WNDCLASSW, WS_OVERLAPPEDWINDOW,
    };
    use windows::core::w;

    struct Host(HWND);

    impl HasWindowHandle for Host {
        fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
            let hwnd = NonZeroIsize::new(self.0.0 as isize).ok_or(HandleError::Unavailable)?;
            let raw = RawWindowHandle::Win32(Win32WindowHandle::new(hwnd));
            // SAFETY: the window outlives every borrow; it is destroyed after the WebView.
            Ok(unsafe { WindowHandle::borrow_raw(raw) })
        }
    }

    extern "system" fn proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        // SAFETY: default handling for a plain top-level window.
        unsafe { DefWindowProcW(hwnd, msg, wp, lp) }
    }

    fn open_window() -> windows::core::Result<HWND> {
        // SAFETY: plain Win32 window creation on this (STA) thread.
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let instance = GetModuleHandleW(None)?;
            let class = w!("kinekino-webview2-demo");
            RegisterClassW(&WNDCLASSW {
                lpfnWndProc: Some(proc),
                hInstance: instance.into(),
                lpszClassName: class,
                ..Default::default()
            });
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class,
                w!("kinekino engine demo: webview2"),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                960,
                600,
                None,
                None,
                Some(instance.into()),
                None,
            )?;
            let _ = ShowWindow(hwnd, SW_SHOW);
            Ok(hwnd)
        }
    }

    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        let host = Host(open_window()?);
        let view = wry::WebViewBuilder::new()
            .with_url("about:blank")
            .build(&host)?;
        let mut engine = WebView2Engine::from_view(view)?;
        let base = fixture::serve(fixture::make_clip(&std::env::temp_dir()));
        let report = contract::run(&mut engine, &base);
        print!("{report}");
        // The child-session path the provider uses: an auto-attached dedicated worker takes a
        // call. (A same-origin iframe would share the page's target and attach nothing.)
        engine.set_auto_attach(true)?;
        engine.call(
            None,
            "Runtime.evaluate",
            serde_json::json!({"expression":
                "window.__kkWorker=new Worker(URL.createObjectURL(new Blob(['onmessage=()=>{}'])));1"}),
        )?;
        let mut child = None;
        for _ in 0..50 {
            child = engine
                .drain_protocol_events()
                .into_iter()
                .find(|e| e.method == "Target.attachedToTarget")
                .and_then(|e| e.params["sessionId"].as_str().map(str::to_owned));
            if child.is_some() {
                break;
            }
            engine.call(
                None,
                "Runtime.evaluate",
                serde_json::json!({"expression": "1"}),
            )?;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        let child_ok = match &child {
            Some(id) => engine
                .call(
                    Some(id),
                    "Runtime.runIfWaitingForDebugger",
                    serde_json::json!({}),
                )
                .is_ok(),
            None => false,
        };
        println!("child-session {}", if child_ok { "Pass" } else { "Fail" });
        engine.set_auto_attach(false)?;
        println!("READY {}", engine.capabilities());
        drop(engine);
        // SAFETY: the WebView is gone; the window is no longer borrowed.
        unsafe {
            let _ = DestroyWindow(host.0);
        }
        if !report.passed() || !child_ok {
            return Err(format!("contract failures: {:?}", report.failures()).into());
        }
        Ok(())
    }
}

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    win::main()
}

#[cfg(not(windows))]
fn main() {
    eprintln!("demo_engine_webview2 needs Windows");
}
