---
id: adr:kinekino/20261029113523
title: 기존 Glow를 기반으로 제작 GPU 캔버스를 단계 확장
status: draft
created: 2026-10-29
updated: 2026-10-29
tags: [player]
---
# 기존 Glow를 기반으로 제작 GPU 캔버스를 단계 확장

## 재평가 조건과 수용 비용

2026-10-19 소스 재확인. 기존 Glow를 우선하는 비용은 GLSL 자원 수명과 상태 복원을 직접 관리해야 한다는 점이다. Rust↔React 변환 가능 범위와 GPU 렌더러 구현 범위는 분리한다. 웹 대응을 이유로 네이티브 GL callback을 React 코드로 자동 변환 가능한 것으로 취급하지 않는다.

wgpu 전환은 동일 자산·revision·viewport의 비교에서 기존 Glow가 필수 기능, 장치 호환성 또는 관측 후 정한 성능 예산을 충족하지 못할 때 재평가한다. WebGPU 전용 기능이 제품 요구로 확정되는 경우도 별도 재평가 사유다. 이때 shader 이관뿐 아니라 egui texture 공유, 영상 합성, 장치 상실, HMR 자원 해제, 배포 크기와 전 화면 시각 회귀 비용을 비교한다. 참조 프로젝트가 wgpu를 쓴다는 사실만으로 전환을 결정하지 않는다.

첫 비교 결과는 현행 경로·Glow callback·필요 시 wgpu 후보를 동일 표본으로 기록한다. CPU 준비/업로드/그리기 제출/GPU 실행/표시 대기를 분리하고, 미지원 계측은 미측정으로 남긴다. 현재 해당 실행 비교는 수행하지 않았으며 백엔드 변경은 없다.

## 근거

사용자는 GL 검토와 hell-is-us-mod 사례 조사를 요청했다. [조사](../../sots/verification/presentation/gpu-rendering-research-2026-10-18.md) 결과 제품 UI는 이미 Glow이고 참조 프로젝트의 기본 경로는 Wgpu/DX12, GL은 명시적 진단 경로다. 기존 [ADR005](adr-005-native-first-production-workspace.md)의 영상 texture 통합 별도 결정 요구를 구체화한다.

## 대안과 방향

| 대안 | 이점 | 비용·판정 |
|---|---|---|
| 기존 egui 도형/이미지만 유지 | 변경 작고 공용 UI 유지 | 고급 mask/투영 합성의 비용·확장성 관측 필요 |
| 기존 Glow의 PaintCallback 확장 | 현 컨텍스트 활용, OBJ/A2D부터 점진 구현 | GL 상태/자원 수명 계약 필요. 우선 스파이크 |
| wgpu 전면 전환 | 참조의 DX12 구조와 향후 WebGPU 연구 가능 | shader/texture·mpv 연동·전체 UI 회귀 비용. 현재 전환 근거 부족 |
| 별도 WGL HUD 창 복제 | 독립 합성기 구현 사례 존재 | 제품 창/포커스/캡처 수명 중복. 현재 채택하지 않음 |

제품 UI는 Glow를 유지하고 backend 독립 scene 계약 위에 GPU 캔버스를 검증한다. libmpv 영상 합성은 별도 단계이며 zero-copy/성능 향상은 미검증이다. GL→wgpu 변경은 측정과 웹 요구가 기존 경로의 한계를 입증할 때 재검토한다. 상세 책임·검증은 [정본](../../sots/player/presentation/shared/rendering.md), 순서는 [계획](../../plans/player/presentation/gpu-rendering-expansion.md)을 따른다. 이 ADR은 설계 제안이며 새 GPU 캔버스 구현 완료나 사용자 최종 채택 승인 기록이 아니다.
