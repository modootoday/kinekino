---
id: sot:kinekino/jobs
title: 작업 대기열
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 작업 대기열

구성·상태별 형태는 [요소별 시각 계약](jobs-visual.md)을 적용한다.

## 시각화

- [작업 상태와 오류](../../../../artifacts/design/v2/studio/queue-refined.png)

이미지는 배치 참고이며 아래 요소 계약과 공통 필드·상태 규칙이 구현 기준이다.

상태: UI/UX 설계 승인(2026-10-18). 생성 이미지 디테일은 실제 Rust 공통 컴포넌트로 검토한다. 구현 완료와 구분한다. [공통 계약](../sot-020-ui-contract.md)의 프레임·입력 검증·상태·접근성·저장·Rust 계층 규칙을 상속한다. 기본/hover/pressed/selected/disabled/focus는 모든 조작 요소에 적용한다.

## 요소 계약

| 요소 ID | 요소 | 표시와 동작 |
|---|---|---|
| JOB-01 | 필터 | 전체/입력 필요/대기/실행/실패/완료 |
| JOB-02 | 행 | 종류·소스·상태·실제 진행·출력, 선택 상세 |
| JOB-03 | 시작 | 준비 완료만, 대량 시작 요약 후 사용자 실행 |
| JOB-04 | 순서 | 대기 작업 우선순위·의존성, 실행 작업 강제 중단 아님 |
| JOB-05 | 세부 단계 | 소스 확인·모델 준비·실행·검증·완료 |
| JOB-06 | 진행률 | bytes/프레임/구간 근거·불확정 spinner 구분 |
| JOB-07 | 취소 | 요청→worker 종료→partial 정리, completed 출력 보존 |
| JOB-08 | 일시정지 | 재개 지원 작업만, 불지원은 취소/다시 실행 설명 |
| JOB-09 | 재시도 | failed item만, 동일 revision·중복 방지 |
| JOB-10 | 입력 해결 | 모델/경로/트랙 등 해당 탭으로 이동 |
| JOB-11 | 결과 열기 | 검증 완료 산출물만, 고정 viewer/OS 포트 |
| JOB-12 | 기록 제거 | 이력과 출력 삭제를 분리 |
| JOB-13 | 오래된 결과 | 입력 revision 다름 배지, 현재 문서 자동 덮어쓰기 금지 |
| JOB-14 | 앱 복구 | 실행 중 기록을 복구 확인 상태로, 자동 중복 실행 금지 |

## 상태와 오류

NeedsInput/Failed/Cancelled를 같은 빨간 실패로 합치지 않는다. ETA는 추정이며 모델없음/대기 중 처리 완료 퍼센트를 만들지 않는다. UI 스레드에서 추론/다운로드하지 않는다.

## 저장 계약

Job(id,type,input_revision,dependencies,state,progress,attempt,output_refs,error_code,idempotency_key), journal atomic writes.

## 인수 기준

의존성 실패·취소경합·앱종료복구·재시도 중복·오래된 revision·완료파일 보존.

각 요소 ID의 유스케이스는 application에, 검증 정책/값 객체는 domain에, OS·추론·파일·프로세스는 infrastructure에 분리한다. presentation은 입력과 렌더링을 담당한다. 모델 미선정/encoder·플랫폼 지원 여부 미확정 항목은 완료로 표시하지 않고 해당 기능을 비활성 또는 NeedsInput으로 제공한다.

