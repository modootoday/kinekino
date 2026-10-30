---
id: plan:kinekino/20261029113844
title: 제작 GPU 렌더링 확장 계획
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제작 GPU 렌더링 확장 계획

2026-10-18. [렌더링 정본](../../../sots/player/presentation/shared/rendering.md)과 [ADR018](../../../adrs/player/adr-018-gpu-canvas-rendering.md)을 따른다. 전체 UI 시안 대응 작업과 병행하며 렌더러 연구 때문에 해당 작업을 대체하지 않는다.

1. 완료: 제품 Glow/이미지/mpv HWND 경계와 참조 GL/Wgpu 기본 경로의 소스 조사. [근거](../../../sots/verification/presentation/gpu-rendering-research-2026-10-18.md).
2. 예정: SDK와 OBJ mask 원본 pixel/PTS/alpha/track/origin 계약을 맞추고 동일 DesignSample을 현재 도형 경로와 GPU 경로 양쪽에 공급한다. scene/viewport와 backend 자원을 분리한다.
3. 예정: Glow callback으로 mask/alpha 한 기능의 최소 스파이크. shader 생성/해제·상태 복원·캐시 교체·resize/종료를 포함하며 공용 UI는 유지한다. 기존 가상 모니터 한 player에서 1:1 및 확대 비교, baseline 대비 성능/자원 기록.

   진입 조건: vendor/renderer/version과 실제 context 정보를 기록하고 baseline 표본의 source/revision/PTS를 고정한다. 종료 조건: 정본의 표본별 시각 실패 조건을 통과하고, 업로드량·CPU/GPU 시간·idle·반복 생성/해제 결과를 남긴다. GPU timer 미지원·소프트웨어 가속·미측정 수치를 명시한다. 실패 시 기존 표시 경로를 유지한다.
4. 예정: 통과한 합성기를 A2D 레이어/PROJ에 확장. SDR 색·alpha·clip·selection/input 역변환 회귀를 각 화면에서 확인한다.
5. 예정: 동일 캔버스 영상이 필요한 사례를 선정하고 libmpv 배포/FFI/재생 수명 스파이크. subprocess 대비 실제 미디어 동기·드롭·CPU/GPU 비용을 확인한다. 실패 시 기존 adapter를 유지하며 실패 이유를 기록한다.
6. 예정: 독립 Rust↔React/HMR 세션과 host-owned GPU capability·reload 자원 해제 경계를 협의. WASM/WebGL/WebGPU는 별도 웹 표본으로 검증하며 네이티브 성공을 웹 호환 증거로 사용하지 않는다.

완료 표시는 실행 관측 링크가 있을 때만 한다. 현재 2–6은 미구현/미검증이며 이번 문서 작업에서 제품 renderer/Cargo 의존성·모니터/DPI·VM을 변경하지 않았다.

## 2026-10-19 조사 보강

백엔드 선택 재평가 조건과 수용 비용은 [ADR018](../../../adrs/player/adr-018-gpu-canvas-rendering.md#재평가-조건과-수용-비용)에 추가했다. 첫 스파이크 보고서는 동일 표본의 현행 경로를 기준으로 삼으며, wgpu 후보의 구현 비용을 포함한다. 단순한 렌더러 명칭이나 비트맵 자산 사용 여부를 성능 판정 근거로 쓰지 않는다.

제품의 Glow 설정과 참조 `src/ui/compositor/runtime.rs`의 GL 명시 분기를 다시 확인했다. 첫 스파이크의 비교 기록은 CPU 전처리/texture 업로드/GL draw/present를 구분하고 첫 로드와 캐시 적중 프레임을 분리한다. backend/callback 불일치의 무출력, 동일 revision 반복 업로드, reload/종료 뒤 미해제 자원도 실패 조건에 포함한다. 구체 계약은 [정본의 구현 경계](../../../sots/player/presentation/shared/rendering.md#캔버스-스파이크의-구현-경계)를 따른다. 이번 보강은 문서 변경이며 새 GPU 코드 실행이나 성능 측정은 아니다.
