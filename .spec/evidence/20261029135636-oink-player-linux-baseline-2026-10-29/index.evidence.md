---
id: evidence:kinekino/20261029135636
title: "oink-player Linux 기준선 보고 (2026-10-29, KST)"
status: active
created: 2026-10-29
updated: 2026-10-29
---
# oink-player Linux 기준선 보고 (2026-10-29, KST)
대상: /workspace/oink-player HEAD 719a839. 저장소 파일 수정·커밋·push 없음. 산출물: runtime/build-cache/native (.gitignore `**/runtime/` 대상 확인).
참고: TRANSFER-CHECKS.md는 루트가 아니라 docs/continuation/TRANSFER-CHECKS.md에 있음(본문 미정독).

## (a) 툴체인
- rustup을 사용자 홈(~/.rustup, ~/.cargo)에 설치, PATH/셸 프로필 수정 안 함(`--no-modify-path`). 사용 시 `export PATH=$HOME/.cargo/bin:$PATH`.
- rust-toolchain.toml에 따라 1.99.1 자동 설치: rustc 1.99.1 (58cf8f9c3 2026-10-13), rustfmt 1.10.0, clippy 0.1.99. 시스템 /usr/bin/cargo 1.85.1은 그대로.
- MSVC/WebView2 SDK는 Linux에 해당 없음.

## (b) 실행 결과
| 명령 | 결과 |
|---|---|
| cargo metadata --locked --offline --no-deps | 통과 |
| cargo fetch --locked | 통과 |
| cargo fmt --all --check | 통과(diff 없음) |
| CARGO_BUILD_JOBS=4 cargo check --workspace --locked | **실패**(exit 101), 루트 `kinekino-player` lib 오류 9건 |
| cargo check -p <패키지> (13개 각각) | 13개 모두 통과 |
| clippy / test | 미실행 |

첫 실패 원문:
```
error[E0432]: unresolved imports `crate::infrastructure::immutable_storage_paths`, `crate::infrastructure::model_review`, `crate::infrastructure::mpv`, `crate::infrastructure::review_runtime`, `crate::infrastructure::review_source`, `crate::infrastructure::timeline_evidence`
  --> src/infrastructure/review_playback_sessions.rs:16:9
note: found an item that was configured out --> src/infrastructure/mod.rs:40:16  (#[cfg(windows)])
```
나머지 오류: `owned_process::infrastructure::windows` 미해결, `image` 크레이트 미해결(cfg(windows) 전용 의존성) 6건, `super::review_source` 미해결. 전체 로그: /workspace/oink-player-linux-check.log
R253 Native import WIP의 compile 결과는 루트 lib 실패에 막혀 Linux에서 판정 불가(미확인).

## (c) 패키지별
| 패키지 | .rs 수 | Windows 종속 파일 | Linux check |
|---|---|---|---|
| kinekino-player (루트, src/) | 1321 | 110 (infrastructure/mod.rs에 cfg(windows) 59개) | **불가**: 비게이트 모듈이 windows 전용 모듈·image를 참조 |
| player-ui-plugins | 12 | 0 | 가능 |
| media-contracts | 8 | 0 | 가능 |
| media-file-publication | 1 | 0 | 가능 |
| media-subtitles | 17 | 0 | 가능 |
| media-jobs | 41 | 2 | 가능(게이트 처리됨) |
| media-catalog | 16 | 0 | 가능 |
| owned-process | 29 | 16 | check는 가능, 단 Linux 구현 범위 미확인(루트가 `infrastructure::windows` 요구) |
| media-vad / -protocol | 11 / 14 | 0 | 가능 |
| media-vad-executor | 35 | 4 | 가능 |
| media-vad-worker | 37 | 10 | 가능 |
| media-source-host (default-members 아님) | 34 | 3 | 가능 |
| media-card-render (default-members 아님) | 20 | 0 | 가능 |
| meari (vendor, 루트 의존성) | - | - | 루트 빌드 중 컴파일됨(오류 없음); meari-native(window)는 미확인 |
'가능'은 check 통과일 뿐 test/clippy·실제 동작은 미확인. 테스트 결과 0건.

루트 Cargo.toml `[target.'cfg(windows)']` 의존성: meari-native(window), eframe(glow), egui_extras, image, egui_taffy, wry 0.57, tokio, webview2-com 0.39.1, windows-core 0.62.2, windows 0.62(Win32 다수). → image/tokio처럼 플랫폼 중립인데 Windows 섹션에 들어간 것이 Linux 실패 직접 원인 중 하나.
하드코딩 경로/.exe: `C:/`, `C:\\`, `.exe"` 문자열이 meari 제외 53개 파일에 있음(테스트/역사 경로 포함 여부는 미분류).

### WebView2 종속 지점
- 의존성: wry, webview2-com, windows/windows-core(COM).
- src/infrastructure/browser/ (11파일: mod, requests, responses, location, audio_backend[/actual]) — 요청/응답 가로채기(광고차단 adblock 0.13 연동 지점), 위치, 오디오.
- src/infrastructure/browser_provider/ (16파일: worker, window, navigation_events, frame_*, protocol_observer/targets, media_observer, capture, ui_policy) — 외부 영상 제공자 접근·프레임·캡처.
- 기타: webview_audio.rs, video_surface.rs, window_activity.rs, window_metrics.rs, local_file_picker.rs, mpv/parent_surface/win32/.
- 주입 JS: assets/clean-page.js, episode-observer.js, media-control.js, playback-observer.js (browser/mod.rs에서 include_str!). adblock.rs 자체는 엔진 중립.
- mpv/ 93파일도 Win32 부모 surface 등 종속(상세 미분류).

## (d) Cross-OS 재설계 제안
경계: domain/application/SDK/CLI = 플랫폼 중립(cfg 없음), infrastructure = 트레잇 뒤 OS/엔진 어댑터(windows/linux/macos 모듈), presentation = 엔진 어댑터를 typed host 계약으로만 소비. 핵심 수정: infrastructure/mod.rs 59개 cfg(windows) 중 중립 모듈(review_*, timeline_evidence, immutable_storage_paths 등)을 게이트 해제하거나 포트 트레잇으로 분리, image·tokio를 공통 의존성으로 이동.
1단계: SDK+player-cli가 Linux에서 check/test/clippy 통과(GUI·브라우저 어댑터는 `Unsupported` 스텁). R253 import WIP의 compile/시험도 이 단계에서 Linux로 처음 판정 가능 → 1단계 완료가 R253 검증의 전제.
2단계: BrowserEngine 포트(navigate, 요청 필터 훅, 스크립트 주입/메시지, 프레임·미디어 이벤트, 캡처) 정의, 기존 WebView2를 첫 어댑터로 이관.
3단계: Chromium 어댑터 + GUI(eframe/meari-native) Linux 창 검증, mpv surface 추상화.

### Chromium 기반 대체 후보 (crates.io 확인분)
| 후보 | 버전/최근 갱신 | 라이선스(크레이트) | 성격 |
|---|---|---|---|
| cef (tauri-apps/cef-rs) | 156.2.0+156.0.14 / 2026-10-25 | Apache-2.0 OR MIT | CEF 임베드, 창 내장·요청 가로채기·JS 바인딩 등 WebView2 대체에 가장 근접. CEF 바이너리 배포 크기·Chromium 자체 라이선스 검토는 미확인 |
| chromiumoxide | 0.9.1 / 2026-02-25 | MIT OR Apache-2.0 | CDP(async) 제어, 별도 Chrome 프로세스. 네트워크 가로채기(Fetch) 가능, 앱 창 내장 불가 → 제공자 접근/검증 CLI용 |
| headless_chrome | 1.0.22 / 2026-06-11 | MIT | CDP(sync), 위와 유사 |
| wry (Linux=WebKitGTK) | 기존 0.57 | - | Chromium 아님, 엔진 차이로 요구사항과 불일치 |
| Servo | 제외 | - | 비Chromium, 외부 영상 제공자(DRM/코덱/사이트 호환) 위험 큼(이 판단은 일반론, 실측 미확인) |
권장: GUI 내장 = cef-rs, CLI/헤드리스 제공자 접근 = CDP(chromiumoxide)를 같은 BrowserEngine 포트 뒤에 두기. 주입 JS 4개는 포트의 스크립트 자산으로 유지하되 축소 대상으로 추적.

## 추천 첫 작업 3가지
1. 루트 lib의 Linux 컴파일 복구: 첫 실패 9건 해소(중립 모듈 게이트 해제/포트 분리, image·tokio 이동, owned-process Linux 구현 확인).
2. 1단계 게이트 통과 후 R253 Native import WIP의 compile·test·clippy를 Linux에서 검증(첫 실패 보존).
3. BrowserEngine 포트 설계 문서(docs/spec/adrs) 초안 + cef-rs/chromiumoxide 최소 스파이크 계획(다운로드 보류 정책 때문에 CEF 바이너리 다운로드는 사용자 승인 필요).
