---
id: sot:kinekino/acceptance
title: 평가 게이트
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 평가 게이트

2026-10-21 적용: 실제 실행 권한·자원은 [중앙 배정](../../../../plans/player/execution-leases.md)과 [현재 AI 자원 계약](../../../../plans/player/ai/resource-allocation-2026-10-21.md)을 따른다. 현재 기본은 중앙 단일 heavy/worker 배정·CPU2/BelowNormal이며 작업별 정확한 thread/affinity·offline·cache·종료 조건을 기록한다. RAM16GiB/VRAM3GiB/C12GiB는 입장 기준이고 모델 RAM4GiB 등 목표를 측정된 hardcap으로 표시하지 않는다. 새 추론·다운로드·worker 빌드는 작업별 배정 전 시작하지 않는다.

아래 4thread·interop1은 최초 계획과 일부 과거 workload의 설정이다. 현재 권한으로 사용하거나 기존 실행 기록을2thread로 소급 변경하지 않는다. 아래 품질 기준·원문 보존·양자화별 비교 요구는 유지하며, 실행 성공/구조 통과와 사람 품질·제품 지원 승인은 구별한다.

## 보존하는 최초 평가 계획

사용자 추가 조건: 경량 양자화를 모델 선택에 포함하고 AI 실측 전에 자원 절약 절차를 수행한다. 기존 ChatGPT·Terminal 앱은 종료 금지다. 소유 재생 검증 창/수집기만 완료 후 정리하고, 모델 다운로드를 끝낸 뒤 read-only preflight를 기록한다. 다른 앱·WSL·서비스·VM·디스플레이·전원/페이지파일 설정을 건드리지 않는다. worker 하나씩 BelowNormal·CPU4/interop1·오프라인·종료 후 모델 해제를 적용한다. 자원 부족/높은 CPU에서는 기다리고 사용자 앱을 강제로 종료하지 않는다.

양자화 평가는 모델 task별로 한다. 텍스트 GGUF Q4/Q5·int8, ASR Q5/Q8 또는 엔진별 int8, OCR/비전은 지원되는 정적/동적 int8·FP16을 후보로 비교한다. 모든 모델에 일괄 4bit를 강제하지 않는다. 원본 대비 품질·RTF/지연·steady/전체 peak RAM·VRAM·cold load·파일 크기를 측정한다. 동적 int8는 원본을 먼저 로드해 peak가 커질 수 있으므로 배포용 사전 양자화 파일과 같은 메모리라고 주장하지 않는다. 양자화 손실이 큰 언어/태스크는 더 높은 정밀도를 선택한다.

권한 있는 한국어/일본어 대사·BGM·노래·무음·겹침·burn-in 자막·애니메이션 피사체로 평가 세트를 분리한다. ASR CER/WER와 시간 경계 오차/무음 환각, OCR CER와 cue 시작/끝, 요약 근거 일치·원문에 없는 내용, mask IoU/경계 품질·추적 ID/시간 일관성, depth 방향·시간 흔들림·투영 재합성을 측정한다. 정답이 없는 모델 자체 점수를 정확도로 제시하지 않는다.

CPU 기준선, 지원 GPU, 양자화별 품질·메모리·실시간계수(RTF)·첫 결과 지연, 60분 재생 중 작업 경쟁, 취소/중간파일 복구를 시험한다. 현재 호스트 재생 시험은 AI 동시 실행 성능 시험이 아니다. 모델 확정·실제 HF revision/해시 고정·용량/라이선스 목록·최소 Windows 버전·일본어→한국어 번역 품질·3D 대상 도구 규약은 남은 작업이다.
