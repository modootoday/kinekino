---
id: evidence:kinekino/20261029113932
title: 선택 프로젝트 caller와 제품 capture surface 기능 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 선택 프로젝트 caller와 제품 capture surface 기능 검증

2026-10-21 UI 단독 headless gate 관측이다. 제품 전체 완료·GUI 시각 인수·생산 모델 실행의 근거가 아니다.

## 고정 입력과 실행

- root: `048808af6d10c0329637bcd003e8cb3482987c89`
- Meari: `0b9ea031ce5c08a5bf99ed9852b668cf2bfce038`
- Cargo.lock: `E469A7204B9DC2AC82AC729E36FD8AC41449E1C382E9167A28A00A45C45789A2`
- 최종 manifest: `runtime/quality-review/ui-capture-product-r1/final-source33-r9.json`
- manifest SHA256: `1224B5772000BFCB8EE6C6A648171C3E5EA6C983FD9E5437990F3E9FC353CFE2`
- Native libtest: `kinekino_player-de5b10d79d4a860f.exe`, 169730560 bytes
- BIN SHA256: `0979EC35B6417CE0D674EDFEAD325CC6708338253DD095270FA9749C1EE05B40`
- 고정 사본: `runtime/quality-review/ui-capture-product-r1/artifacts/native-libtest-r9.exe`

원 caller15·surface22 합성31과 모든 수리 전 입력을 보존했다. 최종33은 기존18/신규15이며 추가2 preimage는 `revisions/r7/added-preimage2.json`에 있다. root에 후보를 적용하거나 index/commit/gitlink를 변경하지 않았다. 책임 채택은 PM 소유다.

기존 S/T, offline/locked, conservative2 CPU2/mask3/BelowNormal/descendant Job, Cargo j1, test thread1을 사용했다. 다운로드·새 target·whole clean·GUI·Native-loop·미디어/모델 실행은 없다.

## 실제 결과

최종 BIN 등록981 중 지정 고유116 통과: normal97와 별도 exact ignored19다. 이전 BIN의 성공을 승계하지 않았다. 최종 fmt/default strict/native strict/build/list 및 지정 시험의 receipt43은 전부 terminal/exit0이다. 전체 gate receipt169는 terminal이며 초기 실패40도 원문으로 보존했다.

실제 이름은 `actual-tests-r9.json`, 최종 receipt는 `actual-receipts-r9.json`, 원 실패는 `preserved-failed-receipts.json`에 있다. full all-platform Native metadata는 macOS offline cache miss101이며 성공으로 처리하지 않았다. 같은 Windows scope metadata는 default356/native486이고 이전 SDK Windows default356과 tuple 추가/삭제0, lock 불변이다.

신규 제품7은 실제 project inspect·제품 graph·소유 RadioGroup 항목·callback·SDK/IPC·ExecutionOwner를 연결한다. 9회 편집과 기존 immutable artifact 재열기, publication 영수증 손실 후 같은 bytes/metadata 저장 재시도, 닫기 중 원 ACK carrier 보존, legacy disabled/noevent/무변경, stale 오류의 local text 보존, 명시 결과 소비/버리기와 dirty close를 검증했다. concrete SDK ACK 실패 주입 근거로 저장 fault를 대신하지 않는다.

신규7의 원 host는 모두 Complete, pipe/SDK Joined, dispatch Completed, worker stopped, unacknowledged0, primary panic false, close panic false, close errors0이다. 기존 guarded panic 기대 시험1은 primary panic true 원문을 보존하며 해당 시험 자체는 통과했다. PID 부재를 이 cleanup 영수증 대신 사용하지 않는다.

원 Cargo/시험 세션의 EOF를 회수했다. 최종 시험 원 세션61955 exit0/EOF, 보호 감사 원 세션99035 exit0/EOF다. 전체 receipt의 원 launcher/child 고유333 PID는 최종 관측에서 모두 부재했다.

## 수리와 파서 경계

원 로그·BIN·manifest는 덮어쓰지 않았다. `REPAIRS.md`와 revision별 파일을 따른다.

- 검증한 Disk 입력을 canonical VerbatimDisk로 바꾼 뒤 새 Disk-only guard에 재입력하던 factory를 수리했다. 원 검증/ancestor guard를 유지하며 original root에 고정 자식을 결합한다. prefix 회귀는 Disk 성공·canonical identity·fresh VerbatimDisk 입력 거절을 실제 확인했다.
- record bool 단항식 대신 Rust에서 같은 disabled 값을 투영한다.
- JSX logical/ConditionalExpression은 미지원이다. r6 삼항 후보는 실행·S materialization0이며 NullLiteral 자체는 Empty를 지원한다.
- 기존 trusted NodeSlot으로 Rust 조립 계층이 raw6을 소비하고 Rail exact5 props를 검증한 뒤 ReadRoot ReactNode1로 전달한다. false는 실제 Empty/child 미생성이며 malformed 값은 오류다.
- parser 단일 Props/단일 export 규격에 맞춰 CapturedCardPanel을 별도 모듈로 분리한다. 전체 inventory·고정 entry·module/export/hash 검증을 유지한다. 미사용 import는 parser가 거절하므로 선언용 import2를 제거했다.
- 제품 시험은 callback 그룹 root가 아니라 정확 owner 아래 `project-card-1-0-0` radio 항목을 기존 renderer adapter로 활성화한다. direct callback·정책 우회·widget flatten은 없다.

## lifecycle 근거의 범위

| 항목 | 실제 근거 | 한계 |
|---|---|---|
| 없음/false/잘못된 값/readonly 투영 | capture_choices::slot_tests 4 | helper 수준이며 host mount lifecycle 증거가 아님 |
| 기본 실제 read graph | viewer_callbacks::tests::actual_read_graph_reprojects_with_original_scope_and_commit_receipt | 기본 Empty 경로 회귀이며 표시 전환 전체 아님 |
| 선택된 실제 Rail→Captured panel, 이미 drain한 old read callback 거절 | product_card_callbacks_edit_accept_retry_and_retain_original_ack_evidence | 동일 host_id·무추가 SDK 요청·실제 입력/저장/dirty close 확인; 반복 Rail 토글 아님 |
| 원 keyed workspace 교체와 callback 퇴역 | direct_keyed_workspace_switch_keeps_original_host_and_revokes_old_callback_export | 작은 child fixture이며 capture Rail instance 검증으로 확대하지 않음 |
| preflight/commit 실패의 last-good 보존 | callback_preflight_rejection_keeps_program_export_and_original_sdk_attachment, rejected_projection_retains_old_local_key_and_success_replaces_only_after_commit | 공통 경로 근거이며 Rail 토글 fault 주입 아님 |

실제 capture host의 false→true→false, 반복 true에서 안정 child identity, 제거된 focus/a11y, Rail callback을 미리 drain한 뒤 제거/재표시하여 거절하는 경우, 해당 전환의 failed commit last-good는 미검증이다. 별도 source-only 후속으로 준비하고 다음 실행 배정을 받는다. 현재116에 포함하지 않는다.

## 보호 감사

root1882, library479, index/docs6, parent raw38, SDK raw39, 보호 BIN518의 변경0이다. S baseline3142 중 기존18 변경은 모두 명시 후보이며 신규15도 최종 manifest와 일치한다. SDK39 snapshot mismatch0, 최종33 raw/S mismatch0, root/S lock 일치다. 원 binary와 초기 실패를 삭제하지 않았다.

근거 파일: `final-state-r9.json`, `final33-r9-check.json`, `protection-r9-summary.json`, `snapshot-expected-r9.json`, `fixed-bin-r9.json`.

다음 작업은 PM 책임 채택과 별도 lifecycle 후속, 이후 승인된 플랫폼/시각 검증이다. 기본 GUI 프로젝트 chooser와 전체 디자인 시안 일치는 이 기능 gate로 승인되지 않는다.
