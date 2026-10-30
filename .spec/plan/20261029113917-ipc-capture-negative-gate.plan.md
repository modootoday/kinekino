---
id: plan:kinekino/20261029113917
title: 캡처 차단의 원 호스트 IPC 인수 후보
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 캡처 차단의 원 호스트 IPC 인수 후보

2026-10-21 PM 명시 배정에서 원 E23과 typed adapter B155를 무수리 합성해 실제 IPC3와 adapter7·직접 영향 회귀51이 통과했고 슬롯을 반환했다. [실제 결과](../../../sots/verification/presentation/capture-ipc-adapter-2026-10-21.md)가 source/BIN·종료·보호 근거와 미완료 범위를 기록한다. 기존 [CPU 차단51](../../../sots/verification/presentation/project-capture-readonly-2026-10-21.md) 및 SDK backend 권한 인수와 구별한다. 아래 절차는 이번에 사용한 고정 실행 계약이며 재실행 권한이 아니다.

재배정 시 실제 root/library/before와 원 소스를 다시 대조하고, 해당 변경을 포함한 새 Native 시험 BIN과 실제 등록 명단을 고정한다. 공개 caller 거절·retained inspection 편집기 바인딩 거절·body panic 뒤 원 owner 종료의3개를 각각 별도 `--exact --ignored --test-threads=1 --nocapture` 프로세스로 실행한다. 같은 BIN을 사용해도 원 stdout/stderr/terminal receipt/EOF/Job은 시험별로 회수한다. 다른 owner 슬롯에서 실행하거나 전체 ignored 시험을 함께 실행하지 않는다.

원 host의 Complete(endpoint quiescent, SDK worker stopped, unacknowledged0)와 실제 시험 결과를 함께 확인한다. 세 번째 시험의 주입 panic 로그는 원 payload 보존과 종료 확인을 검사하기 위한 것이며, 최종 시험 상태를 panic 문자열만으로 판정하지 않는다. 미완료/종료 오류/종료 panic은 실패 carrier로 원 owner와 함께 보고하고 natural Drop을 성공으로 승계하지 않는다.

호스트 생성 이전 setup 실패·deadline/secondary-close 주입·실제 창/프레임은 이 후보의 완료 근거가 아니다. IPC 성공을 SDK 원자 capture/source guard나 제품 시각 승인으로 확대하지 않는다. 원자 guard와 지원 복구는 [캡처 권한 복구 계획](capture-live-binding-recovery.md)의 SDK 계약을 따른다.
