---
id: plan:kinekino/20261029113947
title: 네이티브 영상 합성 backend 비교 범위
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 네이티브 영상 합성 backend 비교 범위

2026-10-20 source/공식 header 읽기 단계다. 기본 backend 채택·새 라이브러리 다운로드·실행·성능 실측은 하지 않았다. 호환 HWND probe의 작성 상태는 `runtime/coordination/sdk-win32-probe-r1/scope.md`로 분리한다.

## 기존 결정과 제약

[ADR002](../../adrs/player/adr-002-native-playback.md)는 현재 프로세스/IPC+child HWND를 유지하고 texture 요구 시 libmpv를 재검토한다. [ADR004](../../adrs/player/adr-004-episode-navigation-chrome.md)는 overlay 대신 공간 제거/복원, [ADR005](../../adrs/player/adr-005-native-first-production-workspace.md)는 HWND 위 UI 합성을 전제하지 않는다. [ADR018](../../adrs/player/adr-018-gpu-canvas-rendering.md)은 GPU canvas 제안이며 libmpv 합성 완료나 최종 채택이 아니다.

현재 Meari VelloCpu+softbuffer 경로와 child HWND만으로 반투명 DOM 컨트롤/skip overlay의 최종 UX를 보장할 수 없다. HWND 호환성과 단일 scene 합성 검증은 별개다.

## 공식 upstream 관측

[mpv master render.h](https://raw.githubusercontent.com/mpv-player/mpv/master/include/mpv/render.h)를 읽었다. 현재 upstream은 OpenGL/SW/D3D11 render API를 열거하고 render context를 재생 전 생성·core 파괴 전 해제하도록 규정한다. callback 안 render 호출을 금지하고 render thread와 일반 libmpv 호출의 wait 의존을 제한한다. 이는 최신 upstream 설명이며 프로젝트 고정 배포 binary/header/binding 지원을 입증하지 않는다.

동일 header는 SW에서 변환·확대·OSD가 CPU 단일 thread로 처리되어 실시간에 느릴 수 있다고 명시한다. 그러므로 기존 Rust SW frame 경로가 있다는 이유만으로 기본 backend로 선정하지 않는다. D3D11/GL도 현재 CPU scene과 자동 zero-copy 합성이 되는 것은 아니다.

## 현재 로컬 공급물과 추가 확보 gate

PM의 로컬 읽기 조사 최종 전달에 따르면 `runtime/mpv/mpv.exe`는 PE v0.41.0-1200-gdde067107이며 sot005의 shinchiro release20261017 / mpv-x86_64-20261017-git-dde0671072.7z provenance에 대응한다. 조사한 runtime/mpv top/doc/installer에는 libmpv DLL/header가 없고 root Cargo에도 mpv FFI crate가 없다. 현재 확인 대상은 subprocess 공급물이며 해당 버전 문자열을 libmpv Render API의 실제 지원 증거로 사용하지 않는다.

| PM 로컬 읽기 근거 | bytes | SHA256 |
| --- | ---: | --- |
| runtime/mpv/mpv.exe | 121051136 | F8BF7E8FD8D6696E35441C55091498060E4594F724A132FD6C46579341BC0790 |
| mpv-x86_64-20261017-git-dde0671072.7z | 33999565 | 2821D9EEF879215E99975FAE9616E1777F5E3747168EDD3A571C41E21B8ABBBC |

PM 연구에서 확인한 source pin은 축약 revision이다. 전체 build closure·서명·NOTICE 및 재배포 조건은 미인수다. 위 표는 PM이 전달한 동일 로컬 근거이며 SDK가 새 다운로드·실행이나 독립 재해시를 수행한 결과가 아니다.

후속 확보 gate는 배포 후보별 libmpv DLL·정확한 대응 header/소스 revision·아키텍처·동적 의존 DLL·재배포 라이선스와 구성 옵션·파일 SHA를 함께 고정하는 것이다. mpv.exe의 provenance만으로 DLL ABI를 추정하지 않는다. 후보 확보와 재배포 조건 검토는 별도 배정 전 미실행이며 새 다운로드를 하지 않는다.

정적 export/header 대조 후 실제 DLL 로드·client API version·필수 render symbol과 backend별 context 생성 가능 여부를 별도 실행 gate로 확인한다. 최신 master D3D11 상수가 있다는 사실과 후보 binary의 D3D11 지원은 분리한다. 지원 누락/ABI 불일치는 typed unsupported로 남기고 임의 fallback이나 standalone 창으로 성공 처리하지 않는다. 라이선스/ABI 확보만 통과해도 실제 frame·색·overlay·성능·수명 인수가 끝난 것은 아니다.

## 후속 별도 구현 후보

`src/infrastructure/mpv/render_api/`는 동적 라이브러리 검증/ABI·context/worker·callback·release를 책임별로 분리한다. 현재 process/IPC 정책을 FFI 안에 복제하지 않는다. 실제 binary SHA·라이선스·정확 header revision·export symbol·binding을 먼저 고정하고 callback은 상태/wake만 기록한다. render context/engine/core 해제 순서와 join 관측은 기존 application cleanup 계약에 연결한다.

Meari에는 backend 중립 frame lease와 scene 삽입 경계를 협상한다. frame의 owner/generation/sequence/크기/format/stride 또는 GPU 동기화·color/alpha·실제 관측 시간·release owner가 필요하며 렌더 texture/버퍼를 public boolean ACK로 대체하지 않는다. producer scene commit과 원 frame lifetime을 묶고 취소·stale·device loss·drop에 원 자원을 보존한다. 복사 비용/overlay 입력·clip·fullscreen은 실제 매체와 scene으로 별도 측정한다.

비교 인수는 HWND의 제한된 opaque plane, SW의 CPU/copy 비용, GL 또는 D3D11의 실제 배포 지원 및 Meari renderer 호환성을 각각 검증한다. 현재 backend 중립 lease 타입이나 Render API Rust 코드는 작성하지 않았다. 이 문서는 범위 후보이며 Cargo/API/main/GUI 변경을 승인한 것으로 취급하지 않는다.
