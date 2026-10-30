---
id: plan:kinekino/20261029113513
title: Gemma R4 runtime controls와 flush 인계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma R4 runtime controls와 flush 인계

2026-10-19 PM 소유 배정. SDK 합의는 기존 RunReceipt v1을 자동 확장하지 않고 새 source pin/profile로 sidecar를 명시 검증한다. 이후 사용자 우선순위 변경으로 API/SDK/UI가 사용자 실측 가능한 상태로 완성될 때까지 새 추론·다운로드·실측 예약·cold worker compile을 연기한다. 이미 실행 중인 건강한 R4 vendor 검사 묶음만 자연 완료·증거 보존하며 제품을 추가 모델 실험 때문에 대기시키지 않는다.

- [x] SDK 필수 resolved/evidence6역할·미관측 차단·configured4 의미와 sidecar/commit 방향 합의.
- [x] 독립 workcopy1,891파일을 base archive+FEA overlay로 생성·R3 전체 bytes 대조. copy manifest `3be51ec6258cd587aecc20be5a185fff874f21446df21fe735001630214c5082`; 실패한 빈 최초시도는 별도 보존.
- [x] scheduler/prefix 실제 getter, GGUF native Gemma Text branch/pipeline 귀속과 controls producer source 구현. core의 prefix 용량 회귀 포함1388pass/5ignored. 실제 실행 controls 관측은 후속.
- [x] raw sync 성공 뒤 immutable commit sync→별도 ACK producer source와 모델 없는 commit/immutable 회귀. worker의 오류·누락·다른run/hash/tamper 회귀 source는 준비했으나 native 실행은 후속.
- [x] source closure1,893/외부 async-trait53 pin 및 별도 target 고정. 기존 -j1 BelowNormal offline core check/test/strict all-target Clippy/CLI build 전부exit0, 각 단계 뒤 source pin 일치.
- [x] 기존 검사 bundle 자연terminal0과 EXE99,183,104B/SHA ec3d9371632dc7b134c3655432a6fe585463dd2b84b768e7a59b8412af815156 및 원문21receipt를 별도 archive에 보존. 새 worker 없이 native copy/hash, static archive manifest B92B49FEB9E8228F170FE0EF9E4E82802D5513A7E39A8832839C4F53AA7FC630.
- [ ] worker/collector/input mapping/RunReceipt 코드는 정적 준비 단계다. 새 cold compile·검증·worker pin과 archive profile admission은 CLI 코어 준비 및 중앙 조율 이후 재개한다.
- [ ] 제품 준비 뒤 PM에 단일 실제3case 명령·자원예산을 인계한다. 현재 새 추론·실측 예약은 없으며 SDK 새 source profile admission/strictjoin 본체 검증도 후속이다.

R3 FEA/엔진/worker/actual과 제품 SDK/UI src는 변경하지 않는다. raw complete·ACK 존재를 독립 OS 내구성·capture 인증·비간섭·품질 통과로 승격하지 않는다. 현재 R3 full CLI 실패는 입력artifact 미공급에 따른 입구 실패이며 strictjoin 본체의 실제 거부가 아니다.
