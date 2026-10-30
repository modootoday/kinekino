---
id: adr:kinekino/20261030190000
title: "ADR 0009: 네이티브 창 중앙 포트 — mpv 렌더 부모·포인터 활동·창 기하, WebView2 계열은 브라우저 엔진 어댑터로"
status: active
created: 2026-10-30
updated: 2026-10-30
rel:
  governs: [crates]
---

# ADR 0009: 네이티브 창 중앙 포트

## Context
- player의 `video_surface`, `window_activity`, `window_metrics`가 Win32(`CreateWindowExW`, `GetCursorPos`, `GetMonitorInfoW`, `GetDpiForWindow`)를 직접 호출해 `cfg(windows)` 뒤에 있었다.
- `browser`, `browser_provider`, `browser_provider_jobs`, `webview_audio`, `playback_workspace`는 WebView2 COM(`webview2_com`)에 묶여 있다. ADR 0002(3엔진 브라우저 포트)에 따르면 WebView2는 OS 포트가 아니라 **브라우저 엔진 어댑터**다.
- 사용자 원칙: Cross-OS, 중앙 인터페이스 필수, OS별 분리.

## Decision
1. **포트**: `kinekino_core::platform::window` — `WindowPort{video_surface, pointer_activity, geometry}`, `VideoSurface{id, layout}`, `PointerActivity{pointer_moved}`, `WindowGeometry`, `OsPolicy::window()`.
2. **핸들은 OS 중립 값**: `NativeWindow::{Win32(HWND), X11(XID)}`. 툴킷 핸들 변환은 player `os_policy::native_window`가 `raw-window-handle` 0.6(업계 표준 창 핸들 교환 크레이트, winit/eframe/wry 공통)으로 한다. core는 이 크레이트에 의존하지 않는다.
3. **UI 스레드 소유**: 포트 객체는 `Send`가 아니다(Win32 창은 생성 스레드 소유).
4. **Windows 어댑터**: `kinekino-platform-windows::window::WindowsWindows`, 기존 코드 이동.
5. **Unix**: `UnsupportedWindows`(모든 연산 `Unsupported`). `has_native_desktop_window()`가 이미 false라 상위 경로가 열리지 않는다. X11 어댑터(x11rb, ADR 0002 S1/S2 스파이크와 같은 근거)는 Linux 네이티브 재생 단계에서 추가한다.
6. **WebView2 계열은 게이트 유지**: `browser*`, `webview_audio`, `playback_workspace`(→`browser_provider_jobs` 의존)는 ADR 0002의 WebView2 엔진 어댑터로 `kinekino-browser`(Adapter 클래스)에 옮겨 `BrowserEngine` 포트 뒤에 둘 때까지 player의 `cfg(windows)`로 남긴다. OS 포트로 감싸면 엔진 개념을 OS 정책에 섞게 되므로 하지 않는다.

## Consequences
- player의 창 모듈 3개는 모든 OS에서 컴파일된다.
- 미검증: Windows 실제 실행(이동 코드, Windows 타깃 clippy만). Linux 네이티브 창 기능 없음(능력 false).
