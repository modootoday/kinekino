---
id: evidence:kinekino/20261029113517
title: 관측 기록 읽는 기준
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 관측 기록 읽는 기준

검증 문서는 당시 표본·빌드의 기록이다. 본문의 진행 중/미검증 표현은 기록 시점에 한정하며 후속 결과와 함께 읽는다. 문서의 존재나 계획 체크리스트를 실행 증거로 사용하지 않는다.

| 쟁점 | 최신 근거·해석 |
|---|---|
| 웹·mpv30초 표본 | [sot005](playback/sot-005-playback-validation.md). 이후 UI·회차 구현은 별도 기록 |
| 단일 창·회차/자동숨김 | [sot006](presentation/sot-006-integrated-ui-validation.md) 이후 [sot007](presentation/sot-007-episode-controls-validation.md), [sot008](playback/sot-008-transition-reliability.md). sot006의 미구현 목록은 당시 시점 |
| 반복·장시간 | [sot010](playback/sot-010-host-endurance-validation.md)에 웹30분59초/mpv60분28초 완료. SourceChanged 수정 전 빌드 범위와 수정 후 짧은 회귀를 구별 |
| VM/ISO | [sot009](environment/sot-009-isolated-validation-environment.md)는 실패 이력. 과거 정리 차단 표현의 후속 결과는 [sot017](environment/sot-017-environment-cleanup.md)의 사용자 apply 후 폴더 부재 확인 |
| DPI | [sot015](display/sot-015-host-virtual-display-review.md)는 드라이버/모니터, [sot016](display/sot-016-app-dpi-matrix.md)는 실제 앱 HWND/UI 표본. 새 UI 검증으로 승계하지 않음 |
| AI | [모델 상태](ai/model-status.md)의20개 역할/모델 묶음에서 제한 입력 실제 추론을 관측했다. ASR·화자·OCR·비전·텍스트·검색·일부 양자화 결과와 품질 미승인을 구분한다. GPU/재생 경쟁·전체 품질 승인은 별개 |
| 새 SDK·CLI·시안 | [SDK/실제CLI 전수 대조](application/sdk-capability-parity-2026-10-18.md)와 [UI 후속 인수](../../plans/player/presentation/visual-fidelity-follow-up.md)를 따른다. 조건부5명령 지원과47미지원, preview·실제 GUI 동작·미검증 상태를 구분한다 |

최신 작업 우선순위는 [계획](../../plans/player/plan-004-priority-queue.md), 문서 이관 검사는 [이관 기록](../../plans/player/plan-005-spec-reorganization.md)을 따른다. 추가 DPI/VM/안정성 시험 보류는 유지한다.
