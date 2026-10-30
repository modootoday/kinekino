---
id: sot:kinekino/ui-gpu
title: UI·제작 캔버스 GPU 렌더링 정본
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# UI·제작 캔버스 GPU 렌더링 정본

상태: 2026-10-18 설계 확장. 현재 구현과 예정 구조를 구분한다. [소스 조사](../../../verification/presentation/gpu-rendering-research-2026-10-18.md), [결정](../../../../adrs/player/adr-018-gpu-canvas-rendering.md), [실행 계획](../../../../plans/player/presentation/gpu-rendering-expansion.md)을 함께 읽는다.

## 현재 기준과 범위

제품/데모 UI는 eframe Glow다. PNG/SVG 이미지와 글꼴 atlas를 사용하는 것은 GL 도입 여부와 별개다. 여백·테두리·곡률·아이콘·타이포 불일치는 [토큰](design-tokens.md)과 [공용 컴포넌트](components.md)의 문제로 계속 수정한다. 렌더러 변경으로 시안 일치가 달성됐다고 판정하지 않는다.

1차 확장은 기존 Glow 안의 OBJ mask/alpha와 A2D 레이어 합성, 이후 PROJ 깊이/카메라다. 표·폼·창 chrome 전체를 별도 셰이더로 재작성하지 않는다. 현재 mpv HWND 재생은 독립 경로이며 그 위에 egui가 자유롭게 합성된다고 가정하지 않는다.

실제 r43 로컬 재생에서 egui 볼륨 popup 상단이 mpv child HWND에 가려지는 것을 관측했다. 단기 UI 대응은 popup 동안 transport 내64단위 공간을 확보해 native 영상 rect를 줄이는 것이다. libmpv 단일 합성의 필요성을 뒷받침하는 구체 사례이며, 단기 대응을 자유로운 overlay 합성 성공으로 표시하지 않는다. 다른 menu·fullscreen idle·첫 프레임의 겹침은 별도 재검토한다.

## 책임과 계약

- domain: 좌표·레이어·색/alpha 의미·시점 정책. GL handle이나 egui 타입 금지.
- application: source asset identity/revision, 원본 pixel 크기, PTS, mask/레이어/카메라의 typed snapshot과 작업 결과. GPU texture ID를 SDK나 CLI에 노출하지 않는다. OBJ mask 계약은 SDK와 조율하며 실제 추론과 DesignSample origin을 구분한다.
- presentation: snapshot→표시 scene 투영, fit/pan/zoom와 hit test, 공용 Canvas 컴포넌트, PaintCallback 연결. 플랫폼/unsafe GL 호출은 infrastructure 어댑터에 맡긴다.
- infrastructure: GL program/buffer/texture/FBO, upload/cache와 명시적 release. 조립 시 생성하고 current context를 가진 렌더 스레드에서 생성/사용/해제한다. callback에는 수명이 보장된 scene snapshot을 전달한다. HMR 교체 단위가 임의로 context나 raw handle을 소유하지 않도록 호스트 경계를 둔다.

예정 모듈은 presentation/canvas/{scene,viewport,component}와 infrastructure/rendering/{glow,resources,diagnostics}로 책임을 나눈다. 공용 계약이 확인되기 전 범용 다중 백엔드 프레임워크를 먼저 만들지 않는다.

## 합성 규칙

### 캔버스 스파이크의 구현 경계

2026-10-19 보강. 비트맵을 제거하는 것을 목표로 삼지 않는다. PNG 디코딩·alpha 추출 같은 CPU 전처리, GPU texture 업로드, GL draw, 화면 present를 각각 기록한다. 기존 A2D의 alpha 표시용 CPU 캐시는 기준 경로로 유지하고, 셰이더 경로와 동일 자산·revision·배율에서 비교한다. PROJ의 현재 egui mesh 평면 표시는 실제 깊이 재구성이나 GPU 깊이 처리의 검증 근거가 아니다.

최소 스파이크는 원본 texture와 mask texture를 입력받아 원본/마스크/합성 모드를 그리는 하나의 callback으로 제한한다. 도구 모음·선택 UI·토큰·입력 좌표 변환은 공용 컴포넌트를 유지한다. backend 종류와 callback 타입을 조립 시 검사한다. egui는 backend에 맞지 않는 callback을 그리지 않을 수 있으므로, 빈 캔버스를 성공으로 처리하지 않고 명시 진단과 기존 경로 복귀를 제공한다. API 기준은 [egui PaintCallback 0.36.2](https://docs.rs/egui/0.36.2/egui/struct.PaintCallback.html)다.

기준 결과에는 첫 자산 로드/첫 프레임과 캐시 적중 후 정상 프레임을 분리한다. 정지·연속 pan/zoom·mask revision 변경·resize 표본별로 texture 업로드 수와 bytes를 기록한다. 동일 revision의 정지 화면에서 반복 업로드가 없어야 하며, scene 교체와 종료 뒤 미해제 자원 수를 확인한다. 계측 가능한 GPU timer가 없으면 채택 근거를 CPU 시간만으로 확대하지 않는다.

원본 pixel 좌표에서 fit/pan/zoom 단일 변환을 계산하고 이미지·mask·윤곽·선택 handle·입력 역변환이 공유한다. 원본 canvas에 정렬된 alpha와 source PTS를 사용한다. asset identity/revision이 같으면 매 프레임 전체 업로드하지 않는다. 캐시는 byte 예산·교체·해제 정책과 context generation을 가진다.

프리뷰의 색 공간과 straight/premultiplied alpha를 입력 메타데이터/변환 지점에서 명시한다. 토큰 색도 같은 출력 규약을 따른다. 초기 비교 표본은 SDR/sRGB로 제한하고 HDR/톤매핑/최종 출력 동일성은 별도 검증한다. checkerboard·반투명 가장자리·검정/흰 배경에서 이중 alpha/halo를 확인한다.

callback은 viewport/scissor/FBO/blend/depth 등 변경 상태와 egui 재진입 계약을 문서화하고 필요한 상태를 복원한다. 매 프레임 shader compile·CPU readback·전체 texture 재생성을 하지 않는다. 숨김/정지 화면은 변경 시 repaint를 사용한다. resize/context loss/종료 시 오래된 handle 재사용을 막고 명시 오류/복구 상태를 제공한다.

## 영상 통합과 웹 확장

### HMR 호스트와 캔버스 경계

아래는 예정 계약이며 현재 구현 완료를 뜻하지 않는다. 호스트가 context와 자원 registry를 소유하고 UI 교체 모듈에는 직렬화 가능한 scene과 capability 식별자만 제공한다. 식별자는 context generation과 asset revision으로 검증하며, reload 뒤 이전 generation의 명령은 거절한다. Rust↔React 변환의 지원 범위에는 scene 의미와 입력 이벤트만 포함하고 GL callback·셰이더·raw texture의 자동 등가 변환을 포함하지 않는다.

reload는 새 scene 검증 → 프레임 경계에서 교체 → 이전 callback 참조 소멸 확인 → current context에서 자원 해제 순서다. 호환되지 않는 상태는 명시적으로 초기화하고 실패 시 마지막 정상 scene을 유지한다. 웹 구현은 같은 capability에 별도 WebGL/WebGPU 어댑터를 제공하며, 미지원 기능은 진단한다. 동일 창 유지·반복 reload의 자원 증가·종료 순서를 별도 인수 항목으로 검증한다.

libmpv render API를 통한 동일 캔버스 영상 합성은 별도 2차 스파이크다. SDK 재생 유스케이스는 유지하고 subprocess와 embedded adapter를 교체 가능하게 한다. FFI 배포/라이선스·update callback 스레드·seek/PTS 동기·종료 순서·디코더 호환·자막·색상·프레임 드롭·장치별 비용을 통과한 뒤 채택한다. 기존 mpv.exe가 libmpv 배포물을 대신한다고 가정하지 않는다.

WASM을 위해 scene/토큰/데이터 계약에서 WGL/HWND/native pointer를 배제한다. Glow→WebGL과 wgpu→WebGPU 대안은 목표 브라우저/기능별 측정 후 선택한다. native libmpv와 웹 영상 공급자는 분리한다. 독립 Rust↔React/HMR 도구의 IR에는 GPU 자원을 직렬화하지 않고 host-owned canvas capability 경계만 협의한다.

## 백엔드 진단과 단계별 채택 기준

GL 활성화 설정과 실제 하드웨어 가속 성공은 별개로 기록한다. 첫 스파이크의 진단 결과에는 GL vendor/renderer/version, context generation, framebuffer 크기, 지원 texture 한도, 업로드 횟수·bytes를 포함한다. 소프트웨어 renderer이거나 GPU timer 미지원이면 해당 사실을 표시하고 GPU 시간은 미측정으로 남긴다. CPU 제출 시간으로 GPU 시간을 대체하지 않는다. 진단용 동기화/readback은 일반 표시 루프에서 제외한다.

| 표본 | 현재 경로 | GL 스파이크 비교 | 채택을 막는 실패 |
|---|---|---|---|
| OBJ 원본·마스크·합성 | 이미지와 egui 도형 | 동일 source/revision/PTS 및 좌표 변환으로 alpha 합성 | 가장자리 halo, 원본과 mask 오정렬, 잘못된 PTS |
| A2D 투명 레이어 | 이미지와 공용 레이어 UI | 같은 순서·가시성·alpha와 checkerboard | 레이어 순서/색상 차이, 숨긴 레이어 잔상 |
| PROJ 깊이·카메라 | 설계 표본 표시 | 깊이 texture와 카메라 변환 분리 | 깊이 단위/축/clip 불일치, 입력 역변환 오류 |
| 실제 재생 영상 | mpv child HWND | 별도 libmpv render API 어댑터 | popup 가림, seek 후 오래된 프레임, 종료 교착 |

각 단계에서 기준 이미지와 토큰·공용 컴포넌트를 유지한다. 교체는 캔버스 내부 구현 단위로 수행하고, 실패 시 기존 표시 경로로 돌아갈 수 있게 한다. 스파이크 성공 여부와 제품 기본 경로 전환 여부를 별도로 기록한다. 실제 디코딩·렌더링·표시 시점을 분리해 계측하며 zero-copy는 전송 경로를 입증했을 때만 사용한다.

## 시각·성능 검증 기록

기존 가상 모니터에서 동일 fixture/viewport의 전후 1:1·확대 비교, mask 윤곽/alpha/clip/좌표 입력, resize·숨김·복귀·종료, 실제 미디어 PTS를 확인한다. p50/p95 CPU/GPU frame time, texture upload bytes, VRAM 추정, idle CPU, dropped frame을 같은 표본으로 기록한다. 사전 수치 목표는 baseline 관측 뒤 정한다. DPI/VM/모니터 구성은 변경하지 않으며 검토 player는 한 개만 실행하고 소유 프로세스를 종료한다. 현재 이 게이트는 미실행이다.
