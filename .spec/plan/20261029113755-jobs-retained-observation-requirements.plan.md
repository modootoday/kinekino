---
id: plan:kinekino/20261029113755
title: JOB 원 commit 관측의 최소 라이브러리 요구
status: active
created: 2026-10-29
updated: 2026-10-29
---
# JOB 원 commit 관측의 최소 라이브러리 요구

상태: 2026-10-23 PM/독립 연구가 확인한 현재 Meari7fb API 공백에 대한 제품 소비자 요구다. 라이브러리 API명·타입명·구현·채택을 발명하거나 확정하지 않는다. lexical gate 입력과 합성하지 않으며 Meari 담당의 별도 계약안을 기다린다. 제품 r3는 runtime/development/ui-jobs-route-r1/HANDOFF-R3.md/source34-r3에 동결했고 작성19/컴파일0/실행0이다. 독립 읽기 검토의 새 확정 P1/P2 없음은 실행 인수가 아니다.

## 원인과 분리할 두 증명

NativeDocumentHandle::dispatch_action은 document_revision을, NativeHostRuntime의 next_turn은 turn_sequence를 증가시킨다. 따라서 성공한 원 commit의 HostCommitReceipt.after와 다음 정상 입력·turn의 current full stamp는 다르다. 기존 arm_workspace_route의 after==current를 그대로 호출하는 일반 복구는 실패한다. try_reproject로 drain하던 fixture 결함도 별개로 발견해 실제 NativeHostRuntime 경로로 바꿨다.

HostEffects의 동기 commit 증명은 유지해야 한다. 같은 실제 turn 시작/끝에 해당하는 원 receipt이며 r3는 projection을 한 turn당 한 번만 한다. 필요한 것은 이 receipt를 합성하거나 새 stamp로 고치는 기능이 아니라, 원 commit 뒤 실제 frame을 기다리는 관측 수명을 별도로 보존하는 library 소유 증명이다.

## 최소 요구

1. 증명은 원 NativeDocumentHandle의 실제 성공 commit에서 library가 발급/보유한다. 제품 JSON·renderer 메시지·DTO·임의 HostStamp로 만들 수 없어야 한다. 원 receipt/full stamp는 수정하지 않고 진단·소유 carrier에 남긴다. source/definition/generation·실제 document/owner identity를 유지한다.
2. library는 후속 input revision/turn sequence만의 전진과 새 성공 source/props projection commit을 구별해야 한다. 제품이 “같아 보이는 props”를 비교하거나 임의로 document_revision을 덮어서 대체하지 않는다. 허용되는 정상 후속 입력·turn의 정확한 범위는 library 계약에 명시한다.
3. 임의 새 projection/source commit, document/owner/source generation/definition 교체, 종료·forced incomplete·revocation은 거절한다. 같은 값의 새 props commit도 새 commit이므로 조용히 이전 증명을 rebasing하지 않는다. 기존 stale/foreign/close guard를 단순 삭제하지 않는다.
4. 실제 후속 frame은 원 document/commit과 같은 유효한 window/surface/context generation 및 실제 제출 provenance에 연결해야 한다. 다른 frame/ticket, 오래된 generation, 제출 전 관측, 닫힌 표면의 보고를 성공으로 취급하지 않는다. 제품의 route desired/observed 결정은 ShellState/StartRouteObservation에 남는다.
5. 실패/Busy 이후 제품은 같은 원 증명과 accepted route intent를 보존해 현재 Viewing export의 명시 callback으로 다시 관측 연결을 요청할 수 있어야 한다. SDK 재제출·attachment 재획득·props 재commit·과거 Jobs export 복원/merge를 요구하지 않는다. 중복 요청의 idempotent/AlreadyArmed/Busy 구분과 폐기·정리 책임을 계약에 명시한다.
6. 새 관측 계약은 기존 HostCommitReceipt의 동기 runtime gate와 구별한다. 기존 strict API의 호환성을 보존하고 원 proof가 없는 경로를 자동 승인하지 않는다. 제품은 지원 전까지 일반 재-arm을 미지원으로 유지한다.

## 제품의 최소 소비 순서

- 원 ViewerCallbacks/ExecutionOwner를 다시 조립하지 않고 이미 성공한 Viewing props+export commit의 receipt와 library 관측 증명을 먼저 보존한다. 그 뒤 fallible observer arm을 시도한다.
- 실패 시 현재 Viewing export의 정확한 opaque onOpenJobs local key만 명시 복구로 수락하고 원 callback batch가 끝난 뒤 진행한다. 오래된 Jobs callback은 거절한다.
- 정상 후속 input/turn 때문에 current full stamp가 바뀐 사실을 그대로 library에 제시하고, 원 증명의 유효성을 library가 판정한다. 제품이 오래된 stamp를 current라고 넘기거나 새로운 receipt를 만든다고 표시하지 않는다.
- 실제 checked frame 보고 전에는 route 관측 완료를 표시하지 않는다. 원 JobsDocument/query owner는 정리 완료까지 유지하며 원 receipt/첫 오류·후속 오류·실제 notification 오류를 보존한다.
- r3의 원 startup notifier 정상 one-shot을 재사용한다. 새 worker/loop나 사용자 오류의 무제한 자동 재시도는 없다. counted notifier·수동 Completion·synthetic frame notice·실제 event-loop/render 증거를 분리한다.

## 후속 인수에서 필요한 반례

실제 NativeHostRuntime에서 commit 성공→fallible arm 실패→현재 export의 실제 Action 입력→다음 turn→원 증명 재연결을 확인한다. 원 receipt는 동일하고 current stamp는 입력/turn에 따라 실제로 전진해야 한다. SDK 요청·재commit·재attach가 늘지 않아야 한다. 이후 실제 frame 전 관측 미완료와 유효 frame 뒤 완료를 구별한다.

별도 새 projection commit/다른 document·definition·generation/닫기·forced 종료/foreign frame·ticket은 원 증명을 거절하고 원 carrier를 보존해야 한다. 오래된 callback 거절·중복 명시 요청·양 query owner drain/workerStopped/unack0도 확인한다. synthetic frame 정책 fixture만으로 실제 renderer/OS event-loop를 인수하지 않는다.

현재 r3의 성공 재-arm 희망 시험은 원 full stamp 불변을 기대하던 과거 단언을 포함하므로 계약 도입 뒤 별도 revision에서 수정해야 한다. 수정 방향은 실제 input/turn stamp 전진을 인정하되 원 receipt 불변과 library가 발급한 유효 증명을 확인하는 것이다. 실패를 숨기기 위해 단언을 삭제하거나 기존 r3를 덮어쓰지 않는다.

참조 source: packages/meari/packages/meari-native/src/infrastructure/native_host/{handle.rs,runtime.rs,types.rs}, src/assembly/meari/player_shell_host/observation/card.rs, 동결 제품 jobs/route.rs. 전체 UI/라우터/HMR 설계를 확장하는 요구가 아니라 원 commit의 관측 수명을 위한 최소 계약이다.

## 2026-10-23 library65cc 초안 대조

root10d9a532132717904ed00af162aefcb94b080c14/library65ccf91840ef682857c53514b5abb1e5927e5884에서 packages/meari/.spec/plans/retained-commit-observation.md 본문을 읽었다. source-only 제안이며 API/정확 delta 미확정이다. 제품 source34와 원 r3 입력은 변경하지 않았다.

초안의 독립 projection epoch, 원 handle이 발급하는 opaque commit proof, checked submission proof, input/turn과 새 projection 구분, close/poison 무효화, 같은 proof의 명시 재연결·중복 처리 및 기존 strict receipt 유지가 위 제품 최소 요구와 일치한다. FrameTicket DTO를 제출 소유 증명으로 쓰지 않는 점도 일치한다. 다음 세 지점은 API 확정 전 Meari 담당의 추가 읽기/정의를 요청했다.

| 확인 지점 | 필요한 경계 |
|---|---|
| 임의 mutable reproject closure의 실패 | 현재 try_reproject는 update(document)가 Err이면 성공 revision 갱신 전 반환한다. 임의 closure가 이미 부분 변경한 경우에도 원 proof가 유효한 것으로 남지 않도록 무효화/원자 rollback/poison의 보수적 경계를 명시해야 한다. 현재 초안의 ‘성공하면 epoch 전진’만으로 이 경우가 닫히는지는 미확정이다. 실행 반례나 확정 제품 결함으로 계상하지 않는다. input hook 실패에도 같은 분류가 필요하다. |
| observer identity와 proof 수명 | 같은 proof+observer key의 idempotency가 renderer 문자열/DTO의 자기 주장에 의존하지 않도록 등록·identity·해제 권한의 소유자를 정한다. Busy는 원 증명을 소비하지 않고, 취소/재등록/다른 active proof/종료 시 결과를 구별한다. API명은 제안하지 않는다. |
| 실제 제출 증명의 전달 | checked 경로에서 발급된 proof를 실제 Native notice/제품 observer까지 전달하는 typed 소비 경로를 정의한다. 공개 FrameTicket/합성 notice만 있는 경우에는 일반 복구를 승인하지 않는다. 기존 synthetic 정책 시험과 실제 checked submit/event-loop/render의 근거를 분리하고, 과거 notice 소비자·테스트의 호환 영향을 exact delta에 기록한다. |

후속 계약 소비는 PM 배정 이후 별도 제품 revision으로 준비한다. 기존 strict guard 삭제·임의 stamp/receipt 생성·동기 receipt 합성·새 worker/loop·공유 파일 선행 편집·Cargo/GUI 실행은 하지 않았다.
