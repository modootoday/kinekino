---
id: evidence:kinekino/20261029113948
title: GPU 렌더링 조사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# GPU 렌더링 조사

조사일: 2026-10-18. 소스 관측이며 GPU 성능 측정·새 렌더러 실행 검증은 아니다.

2026-10-19 재확인: 제품 `Cargo.toml`의 eframe 0.36.2/glow, `src/presentation/mod.rs:53`과 `design_preview.rs:20`의 명시적 Glow, mpv 어댑터의 `--wid`/`--hwdec=auto-safe`를 확인했다. 참조 `src/ui/compositor/runtime.rs`는 기본 composition과 명시적 gl/opengl 분기를 유지하며 `src/ui/app/graphics.rs`는 공유 DX12 device로 WgpuConfiguration을 조립한다. 기존 표의 행 번호는 최초 조사 당시 위치다. 공식 egui custom_3d_glow 예제와 mpv render_gl.h도 재열람했다. 버전 고정 API와 실제 배포 libmpv의 일치 여부는 구현 진입 시 다시 확인한다. 이번 재확인은 소스·문서에 한정하며 실행 성능 근거를 추가하지 않는다.

## 제품에서 확인한 사실

- `Cargo.toml:15`: eframe 0.36.2, glow 기능 사용. `src/presentation/mod.rs:41`, `design_preview.rs:19` 모두 Renderer::Glow를 명시한다. UI 전체가 CPU 비트맵으로 그려진다는 가정은 현재 소스와 맞지 않는다. 실제 장치의 GPU 가속 여부·vendor·driver는 이번 조사에서 실행 측정하지 않았다.
- `src/presentation/design_assets.rs`: PNG를 egui::Image::from_bytes로 전달한다. 비트맵은 콘텐츠 자산 형식이며 UI 렌더 백엔드와 다른 문제다. 이 파일만으로 디코딩 시간·업로드 횟수·캐시 효율을 단정하지 않는다.
- `src/infrastructure/mpv/mod.rs`: 별도 mpv.exe에 --wid와 --hwdec=auto-safe를 전달한다. 실제 선택된 VO/GPU API/하드웨어 디코더는 해당 옵션만으로 확정할 수 없다. UI GL 컨텍스트와 영상이 단일 텍스처 합성으로 통합된 구조는 아니다.

## 참조 프로젝트의 실제 구조

참조 루트: `%USERPROFILE%/hell-is-us-mod`의 조사 당시 작업 트리. 실행 검증이나 복제는 하지 않았다.

| 소스 | 관측 |
|---|---|
| Cargo.toml:25–27 | eframe 0.36의 wgpu/glow, egui_taffy 0.14 병행 |
| src/ui/app.rs:78 | 패널 기본 렌더러는 Wgpu |
| src/ui/compositor/runtime.rs | 기본 HUD는 composition::start, HIUMOD_HUD_BACKEND=gl/opengl일 때만 명시적 GL 진단 경로 |
| src/ui/compositor/composition.rs | 공유 DX12 장치와 DirectComposition, 별도 창/프레임 루프 |
| src/ui/compositor/native/gl.rs | glutin/WGL OpenGL 3.3, RGBA8 하드웨어 config, context/surface 수명과 swap 소유 |
| src/ui/compositor/opengl.rs | egui_glow::Painter, prepare→frame→release, 비활성 cadence와 종료 처리 |
| src/ui/compositor/world/gl_frame.rs | backend 독립 snapshot을 egui PaintCallback/CallbackFn으로 렌더, current context에서 GPU 자원 해제 |

따라서 “참조 프로젝트의 기본 구조가 GL”이라고 일반화하지 않는다. 가져올 핵심은 scene/renderer 분리, 명시적 자원 수명, 합성 경계, 계측 구조다. 게임 HUD용 별도 투명 창·swap interval 0·독립 상시 루프는 제품 요구와 동일하지 않다.

## 공식 자료와 판단

### 참조 코드의 적용 대응표 — 2026-10-19 재검토

| 참조의 구체 구현 | 제품 설계에 적용할 책임 | 그대로 복제하지 않을 부분 |
|---|---|---|
| `world/gl_frame.rs::prepare_gl/release_gl` | current context 안에서 자원 생성·해제, 종료 전 명시 정리 | 독립 HUD context를 제품에 추가 생성 |
| `World::draw`의 prepared/job 일치 확인과 `CallbackFn` | asset revision과 scene generation 일치 확인 뒤 callback 제출 | 게임 카메라 읽기와 HUD 전용 좌표 계산 |
| `app/graphics.rs::Graphics`의 start/close/Drop | 조립 계층의 렌더러 소유와 명시 종료, 중복 해제 방지 | 공유 DX12 장치 구성을 Glow에 혼합 |
| `compositor/runtime.rs`의 명시적 GL 분기 | 선택된 backend와 진단 결과를 별도로 기록 | 기본 경로를 GL이라고 오인하거나 환경 변수만으로 성공 판정 |

위 대응표는 소스 재열람 결과다. 제품 GPU callback 구현·성능 개선·하드웨어 가속 성공을 입증하지 않는다. 제품 고정 버전의 [eframe Renderer 0.36.2](https://docs.rs/eframe/0.36.2/eframe/enum.Renderer.html)도 재확인했다. 구현 순서와 미실행 게이트는 기존 정본·계획을 유지한다.

재확인: 제품과 데모의 `Renderer::Glow`, mpv의 `--wid`/`--hwdec=auto-safe`, 참조의 `Renderer::Wgpu`와 명시적 GL 분기를 현재 작업 트리에서 다시 확인했다. 제품에 고정된 버전의 [egui_glow 0.36.2](https://docs.rs/egui_glow/0.36.2/egui_glow/)와 [PaintCallback 0.36.2](https://docs.rs/egui/0.36.2/egui/struct.PaintCallback.html)를 API 기준으로 삼는다. latest/main 링크는 참고 자료이며 제품 의존성 업그레이드 근거가 아니다. 이번 재확인은 소스·공식 문서 조사이며 실제 GPU 장치 계측은 수행하지 않았다.

- [egui Glow 공식 패키지](https://docs.rs/egui_glow/latest/egui_glow/), [공식 custom_3d_glow 예제](https://github.com/emilk/egui/blob/main/examples/custom_3d_glow/src/main.rs): egui PaintCallback에 custom GL 작업을 연결할 수 있다. 기존 Glow 캔버스 확장이 가장 작은 변경이라는 판단의 근거다.
- [libmpv 공식 예제 설명](https://github.com/mpv-player/mpv-examples/blob/master/libmpv/README.md), [render API](https://github.com/mpv-player/mpv/blob/master/include/mpv/render.h), [OpenGL 계약](https://github.com/mpv-player/mpv/blob/master/include/mpv/render_gl.h): 호스트 GL context와 render API를 사용하는 영상 통합 경로가 있다. 현재 subprocess IPC와는 다른 수명/스레드/배포 계약을 요구한다. zero-copy와 성능 향상은 보장하지 않는다.

GLSL/WebGL과 WGSL/WebGPU 간 shader·자원 호환은 자동으로 성립하지 않는다. 브라우저 지원 범위, context loss, libmpv의 웹 대체 경로는 별도 스파이크 대상이다. 상세 결정은 [렌더링 정본](../../player/presentation/shared/rendering.md), 단계는 [계획](../../../plans/player/presentation/gpu-rendering-expansion.md)에 둔다.
