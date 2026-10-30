use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::windows::process::CommandExt,
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        UI::WindowsAndMessaging::{
            EnumWindows, GW_OWNER, GWL_EXSTYLE, GWL_STYLE, GetClassNameW, GetParent, GetWindow,
            GetWindowLongPtrW, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
            PostMessageW, WM_CLOSE,
        },
    },
    core::BOOL,
};

pub struct SavedRun {
    pub exit: Option<i32>,
    pub events: Vec<Value>,
    pub closed_windows: usize,
}
struct Windows {
    pid: u32,
    handles: Vec<HWND>,
    observed: Vec<Value>,
}
unsafe extern "system" fn collect(window: HWND, parameter: LPARAM) -> BOOL {
    // SAFETY: EnumWindows synchronously passes the live Windows value owned by close_owned.
    let owned = unsafe { &mut *(parameter.0 as *mut Windows) };
    let mut pid = 0;
    // SAFETY: window is supplied by EnumWindows and pid is a valid writable value.
    unsafe {
        GetWindowThreadProcessId(window, Some(&mut pid));
    }
    if pid == owned.pid {
        let mut title = [0u16; 512];
        let mut class = [0u16; 256];
        // SAFETY: both buffers are live writable UTF-16 arrays for this enumerated window.
        let (title_len, class_len) = unsafe {
            (
                GetWindowTextW(window, &mut title),
                GetClassNameW(window, &mut class),
            )
        };
        let title = String::from_utf16_lossy(&title[..title_len as usize]);
        let class = String::from_utf16_lossy(&class[..class_len as usize]);
        // SAFETY: read-only metadata queries for this enumerated child-owned window.
        let (visible, parent, owner, style, extended_style) = unsafe {
            (
                IsWindowVisible(window).as_bool(),
                GetParent(window).ok().map(|value| value.0 as usize),
                GetWindow(window, GW_OWNER)
                    .ok()
                    .map(|value| value.0 as usize),
                GetWindowLongPtrW(window, GWL_STYLE),
                GetWindowLongPtrW(window, GWL_EXSTYLE),
            )
        };
        let product = visible
            && title == kinekino_player::domain::product::PRODUCT.display_name
            && parent.is_none()
            && owner.is_none()
            && style & 0x40000000 == 0;
        owned.observed.push(json!({"hwnd":window.0 as usize,"pid":pid,"title":title,"class":class,"visible":visible,"parent":parent,"owner":owner,"style":style,"extended_style":extended_style,"selected_product_root":product}));
        if product {
            owned.handles.push(window);
        }
    }
    true.into()
}
fn close_owned(pid: u32, directory: &Path) -> usize {
    let mut owned = Windows {
        pid,
        handles: Vec::new(),
        observed: Vec::new(),
    };
    // SAFETY: callback data stays live throughout this synchronous enumeration.
    unsafe {
        EnumWindows(Some(collect), LPARAM((&mut owned as *mut Windows) as isize)).unwrap();
    }
    fs::write(
        directory.join("owned-windows.json"),
        serde_json::to_vec_pretty(&owned.observed).unwrap(),
    )
    .unwrap();
    for handle in &owned.handles {
        // SAFETY: callback checked the exact child PID; WM_CLOSE is an ordinary close request.
        unsafe {
            PostMessageW(Some(*handle), WM_CLOSE, WPARAM(0), LPARAM(0)).unwrap();
        }
    }
    owned.handles.len()
}
pub fn run(binary: &Path, directory: &Path, input: &Path, store: &Path, close: bool) -> SavedRun {
    let mut child = Command::new(binary)
        .creation_flags(0x08000000)
        .args(["--record-validation", "--native-proposal-request"])
        .arg(input)
        .arg("--proposal-root")
        .arg(store)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(
            fs::File::create(directory.join("stderr.log")).unwrap(),
        ))
        .spawn()
        .unwrap();
    let pid = child.id();
    fs::write(directory.join("argv.json"),json!({"binary":binary,"pid":pid,"args":["--record-validation","--native-proposal-request",input.to_str().unwrap(),"--proposal-root",store.to_str().unwrap()]}).to_string()).unwrap();
    let stdout = child.stdout.take().unwrap();
    let path = directory.join("stdout.log");
    let (sender, receiver) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut log = fs::File::create(path).unwrap();
        for line in BufReader::new(stdout).lines() {
            let line = line.unwrap();
            writeln!(log, "{line}").unwrap();
            log.flush().unwrap();
            if let Ok(value) = serde_json::from_str::<Value>(&line) {
                let _ = sender.send(value);
            }
        }
    });
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut events = Vec::new();
    let mut closed_windows = 0;
    let mut close_sent = false;
    let mut frame_seen = false;
    let mut forced = false;
    let status = loop {
        while let Ok(event) = receiver.try_recv() {
            if close
                && !close_sent
                && event["kind"] == "native-proposal-window"
                && event["data"]["kind"] == "FrameSubmitted"
            {
                frame_seen = true;
            }
            events.push(event);
        }
        if frame_seen && !close_sent {
            closed_windows = close_owned(pid, directory);
            close_sent = closed_windows > 0;
        }
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            forced = true;
            child.kill().unwrap();
            break child.wait().unwrap();
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    reader.join().unwrap();
    events.extend(receiver.try_iter());
    fs::write(directory.join("exit.json"),json!({"pid":pid,"exit":status.code(),"forced":forced,"closed_windows":closed_windows,"child_waited":true}).to_string()).unwrap();
    assert!(
        !forced,
        "owned process exceeded deadline; original logs retained at {}",
        directory.display()
    );
    SavedRun {
        exit: status.code(),
        events,
        closed_windows,
    }
}
