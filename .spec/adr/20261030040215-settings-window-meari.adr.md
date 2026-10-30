---
id: adr:kinekino/20261030040215
title: 설정 창을 Meari native로 렌더
status: active
created: 2026-10-30T04:02:15Z
updated: 2026-10-30T04:02:18Z
---

# 설정 창을 Meari native로 렌더

## Context

사용자 방침: kinekino의 모든 UI는 Meari를 근간으로 한다. P6 설정 창 첫 구현(a820298)은 Win32 창 + WebView2(wry) 위에 HTML/JS 페이지(`gui.html`)를 올렸고 디자인 토큰·아이콘·글꼴을 쓰지 않았다(`var(--aw-…)` 0건, 자산 참조 0건). Windows 밖에서는 `unsupported`였다. 플레이어 화면의 WebView2 선택([ADR-001](20261029113529-adr-001-rust-ui-webview2.adr.md))은 웹 재생 엔진에 관한 것이며 설정 창 같은 제품 UI의 근거가 아니다.

## Decision

- 설정 창은 새 crate `kinekino-settings-ui`가 Meari로 렌더한다: 페이지 모델 JSON → 고정 행 TSX(`useState` 슬롯, 타입 props) → `meari-native` `InteractiveDocument` → `NativeWindowDriver`(winit + softbuffer). Windows·Linux·macOS가 같은 코드 경로이며 `cfg`가 없다.
- 포트는 `SettingsWindowPort::run(title, smoke, on_message)`이다. 어댑터가 `{"op":"page"}`를 먼저 보내고, 클릭·입력은 Meari 소스 상태에서 읽어 `apply`(원자적)·`dialog`·`close` 메시지로 보낸다. 페이지 모델·적용·한도 대화상자 처리는 `kinekino-app` 그대로다.
- 스타일은 플레이어와 같은 `tokens.css`·`components.css`를 쓰고, 설정 전용 배치(`settings.css`)는 토큰 변수만 쓴다. 정본과 시안 대응은 [settings-ui-design](../sot/settings-ui-design.sot.md).
- 스모크 캡처는 Meari 문서 래스터(PNG)다(이전: BMP 창 캡처).
- 제거: `kinekino-platform-windows`의 설정 창과 `wry` 의존, 추가 windows 기능(`Win32_Storage_Xps`, `Win32_System_LibraryLoader`), `kinekino-platform::settings_window()`, `gui.html`.
- 유지: `kinekino-browser`의 WebView2 provider(플러그인 브라우저 엔진이며 UI가 아님).

## Consequences

- Linux 설정 창은 mpv(P2)·#28과 무관하게 열린다. 플레이어 Linux GUI 셸(#29)은 여전히 그 둘에 막혀 있다.
- OS 창 프레임을 쓴다. 제품 공통 caption(`aw-window-chrome`)을 쓰는 사용자 정의 프레임은 플레이어 셸과 함께 맞출 후속 항목이다.
- Meari native는 Linux 인수(acceptance)가 upstream에서 끝나지 않았다. 박스 xvfb 스모크가 이 저장소의 근거다.
