---
id: evidence:kinekino/20261029114153
title: Native 오디오 미등록 준비 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native 오디오 미등록 준비 관측

커서 checkpoint 뒤 중앙이 Meari→UI 소비 검증에 단일 heavy 슬롯을 배정했다. SDK는 root 등록·cargo·GUI 실행 없이 `runtime/coordination/native-audio`의 Rust 후보와 .spec만 작성했다. [등록·인수 계획](../../../plans/player/native-audio-checkpoint.md)에 책임별 map과 아직 필요한 production coordinator/adapter 연결을 분리했다.

후보는20파일이며 기존11파일 baseline은 baseline/와 baseline-sha256.txt에 보존했다. 신규 후보는 AudioIntent·handoff core/tests·web silence owner/tests·native shutdown proof/adapter·WebView native audio backend·generation-scoped pause 요청이다. root domain/application/infrastructure/assembly는 pointer source-final과 git diff --no-index --stat 대조에서 diff0/exit0이었다(CRLF 정규화 경고 별도). SDK API pin732cec0b522431837f728fea480d37a446b7378a770fe89cab0b595b4d444567, Cargo03ee9a12b41a32758e63998653d7888dfb0443dafe9fcf66766209b746539a34, lock50323b23fb1bbf496e3ad42be71b4ccbdc3e1f38de1de5a5154a1ba5d78da5f7을 유지했다. UI/Meari의 미등록 파일까지 전체 tree가 같다는 뜻은 아니다.

후보에 새 Rust 검사29개를 준비했다: handoff13, Session generation/sequence3, strict mute parser1, fixed mute wire1, actual property event→status 전달1, web silence owner10이다. 모두 미실행이다. mock property와 synthetic shutdown proof의 검사는 실제 WebView/mpv 관측을 대체하지 않는다.

개별 rustfmt edition2024/skip_children으로 syntax/format을 확인한 receipt와 source-sha256.txt를 runtime에 보존했다. 첫 format 기록 뒤 native WebView 전체 mute·lease/종료 proof·사용자 의도 보존이 보강됐으므로 최종 후보 hash와 후속 format receipt를 기준으로 인계한다. 아직 type/cargo test/clippy·실제 CLI mute/COM readback·GUI handoff 연결은 검증하지 않았다. 현재 제품의 무음 결함이 수리 완료됐다고 표시하지 않는다.

소스 감사에서 DOM 수신 순번만의 한계를 확인해, periodic WebPlayback 표본으로는 소리를 해제하지 않도록 후보를 바꿨다. Rust `ICoreWebView2_8::IsMuted` 직접 readback이 core의 view_muted 근거다. 외부 API 의미는 [Microsoft 원문](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2_8)과 local 바인딩에서 확인했으며 실제 COM 호출은 아직 없다.
