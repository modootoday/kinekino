//! Explicit native pointer interaction and OS visibility observations, never product policy.
use super::Fixture;
use serde_json::{Value, json};
use std::{path::Path, time::Duration};
use windows::{
    Win32::{
        Foundation::{CloseHandle, HWND, LPARAM, POINT, RECT},
        Graphics::Gdi::ClientToScreen,
        System::Threading::{
            OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
            QueryFullProcessImageNameW,
        },
        UI::WindowsAndMessaging::{
            CURSOR_SHOWING, CURSORINFO, EnumWindows, GA_ROOT, GetAncestor, GetClientRect,
            GetCursorInfo, GetCursorPos, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId,
            IsWindowVisible, SetCursorPos, SetForegroundWindow, WindowFromPoint,
        },
    },
    core::{BOOL, PWSTR},
};

struct Search<'a> {
    executable: &'a Path,
    matches: Vec<(HWND, u32)>,
}
unsafe extern "system" fn collect(hwnd: HWND, context: LPARAM) -> BOOL {
    let search = unsafe { &mut *(context.0 as *mut Search<'_>) };
    if !unsafe { IsWindowVisible(hwnd) }.as_bool() {
        return BOOL(1);
    }
    let mut title = [0u16; 256];
    let length = unsafe { GetWindowTextW(hwnd, &mut title) };
    if String::from_utf16_lossy(&title[..length as usize])
        != kinekino_player::domain::product::PRODUCT.display_name
    {
        return BOOL(1);
    }
    let mut pid = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
    }
    if let Ok(process) = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) } {
        let mut path = [0u16; 32768];
        let mut length = path.len() as u32;
        let read = unsafe {
            QueryFullProcessImageNameW(
                process,
                PROCESS_NAME_WIN32,
                PWSTR(path.as_mut_ptr()),
                &mut length,
            )
        };
        unsafe {
            let _ = CloseHandle(process);
        }
        if read.is_ok()
            && std::fs::canonicalize(String::from_utf16_lossy(&path[..length as usize]))
                .ok()
                .as_deref()
                == Some(search.executable)
        {
            search.matches.push((hwnd, pid));
        }
    }
    BOOL(1)
}

struct RestorePointer(POINT);
impl Drop for RestorePointer {
    fn drop(&mut self) {
        unsafe {
            let _ = SetCursorPos(self.0.x, self.0.y);
        }
    }
}
fn sample(stage: &str, hwnd: HWND) -> Value {
    let mut info = CURSORINFO {
        cbSize: std::mem::size_of::<CURSORINFO>() as u32,
        ..Default::default()
    };
    unsafe {
        GetCursorInfo(&mut info).unwrap();
    }
    let target = unsafe { GetAncestor(WindowFromPoint(info.ptScreenPos), GA_ROOT) };
    json!({"stage":stage,"showing":info.flags == CURSOR_SHOWING,
        "cursor_handle_present":!info.hCursor.0.is_null(),
        "position":{"x":info.ptScreenPos.x,"y":info.ptScreenPos.y},
        "target_window_under_pointer":target == hwnd})
}
fn record(fixture: &Fixture, values: &[Value]) {
    std::fs::write(
        fixture.root.join("native-pointer-observations.json"),
        serde_json::to_vec_pretty(values).unwrap(),
    )
    .unwrap();
}
fn visible(fixture: &Fixture, values: &mut Vec<Value>, stage: &str, hwnd: HWND) {
    let observation = sample(stage, hwnd);
    let showing = observation["showing"] == true && observation["cursor_handle_present"] == true;
    let over_target = observation["target_window_under_pointer"] == true;
    values.push(observation);
    record(fixture, values);
    assert!(
        over_target,
        "Pointer probe requires the owned native window under the pointer"
    );
    assert!(
        showing,
        "Cursor disappeared during {stage}; inspect native-pointer-observations.json"
    );
}

pub fn exercise(fixture: &Fixture, executable: &Path) {
    let executable = executable.canonicalize().unwrap();
    let mut search = Search {
        executable: &executable,
        matches: Vec::new(),
    };
    unsafe {
        EnumWindows(
            Some(collect),
            LPARAM(&mut search as *mut Search<'_> as isize),
        )
        .unwrap();
    }
    assert_eq!(
        search.matches.len(),
        1,
        "Pointer probe requires one visible fixed-executable product video window"
    );
    let (hwnd, pid) = search.matches[0];
    let mut original = POINT::default();
    unsafe {
        GetCursorPos(&mut original).unwrap();
    }
    let _restore = RestorePointer(original);
    let mut rect = RECT::default();
    unsafe {
        GetClientRect(hwnd, &mut rect).unwrap();
    }
    assert!(rect.right - rect.left > 120 && rect.bottom - rect.top > 120);
    let mut origin = POINT {
        x: rect.left,
        y: rect.top,
    };
    assert!(unsafe { ClientToScreen(hwnd, &mut origin) }.as_bool());
    let activated = unsafe { SetForegroundWindow(hwnd) }.as_bool();
    let mut values = vec![
        json!({"stage":"target","pid":pid,"foreground_request_accepted":activated,
        "client_origin":{"x":origin.x,"y":origin.y},"client_size":{"width":rect.right-rect.left,"height":rect.bottom-rect.top},
        "automatic_engine_hiding_expected":false,"scope":"standalone_native_video; product_controls_not_verified"}),
    ];
    record(fixture, &values);
    let mut outer = RECT::default();
    unsafe {
        GetWindowRect(hwnd, &mut outer).unwrap();
    }
    assert!(
        origin.y > outer.top,
        "Standalone pointer probe requires visible native window chrome"
    );
    unsafe {
        SetCursorPos(
            origin.x + (rect.right - rect.left) / 2,
            (outer.top + origin.y) / 2,
        )
        .unwrap();
    }
    std::thread::sleep(Duration::from_millis(100));
    visible(fixture, &mut values, "before_video_enter_on_chrome", hwnd);
    let points = [
        ("enter", 1, 2),
        ("move", 3, 4),
        ("near_controls_edge", 1, 8),
    ];
    for (stage, numerator, denominator) in points {
        let x = origin.x + (rect.right - rect.left) * numerator / denominator;
        let y = if stage == "near_controls_edge" {
            origin.y + rect.bottom - rect.top - 20
        } else {
            origin.y + (rect.bottom - rect.top) / 2
        };
        unsafe {
            SetCursorPos(x, y).unwrap();
        }
        std::thread::sleep(Duration::from_millis(100));
        visible(fixture, &mut values, stage, hwnd);
        if stage == "enter" {
            std::thread::sleep(Duration::from_millis(1500));
            visible(fixture, &mut values, "idle_after_enter_1500ms", hwnd);
        }
    }
    unsafe {
        SetCursorPos(original.x, original.y).unwrap();
    }
    std::thread::sleep(Duration::from_millis(100));
    values.push(sample("restored_original_position", hwnd));
    record(fixture, &values);
}
