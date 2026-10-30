---
id: plan:kinekino/20261029113845
title: Retained source10 제품 소비 최소 delta 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Retained source10 제품 소비 최소 delta 제안

2026-10-23 source-only 읽기 제안이다. Meari private source10.json SHA4C28DCCD68C96B1A188261E3FA205168389CA93289D1263B1450FC7F71E454A4의 API 본문·checked_window/notice 발행·unit/public test를 읽었다. 기존7/new3 및 작성10+doctest1/actual0인 라이브러리 후보이며 독립 리뷰/타입 인수 전이다. canonical library나 S/T에 반영된 것으로 취급하지 않는다. 제품 source34/r3는 불변이다.

## 읽은 API와 제품 연결

native_host::{RetainedCommitProof,RetainedFrameProof}는 private 필드와 Weak 원 handle/epoch를 보유한다. retain_commit(&receipt)는 같은 handle의 최신 성공 commit과 현재 full stamp가 일치할 때만 발급한다. 따라서 Viewing commit 성공 직후, fallible observer 등록 전 보존해야 한다. 원 receipt를 Returning에 먼저 보존하고 proof 발급이 실패해도 그 receipt/owner/오류를 잃지 않는다.

validate_retained_commit(&proof)는 원 handle/epoch/owner/definition 및 open/nonpoison 상태를 검증하고 실제 current stamp를 반환한다. 제품은 proof.receipt()==보존 원 receipt를 확인하며 정상 input/turn의 current stamp 전진을 허용받는다. current를 원 receipt.after로 위장하지 않는다. 새로운 props/source commit 뒤 원 proof를 자동 재발급하거나 recommit하지 않는다.

validate_retained_frame(&commit,&frame)는 checked submission이 발급한 typed proof와 같은 lineage/활성 context를 검사한다. 제품 notice 소비에서는 frame proof의 stamp()/ticket()을 실제 notice의 document/submission.ticket 및 현재 full stamp와 함께 대조하고 기존 host/window/revision/크기/scene 단조성 검사를 유지해야 한다. 공개 FrameTicket만으로 대체하지 않는다. NativeFrameObservation.retained의 정확 타입은 Option<RetainedFrameProof>다. 실제 checked submit→window notice만 Some을 발급하며 synthetic None은 retained 완료를 인증하지 못한다.

## PM 배정 요청 exact 범위 — 아직 작성하지 않음

| 경로 | 최소 책임 |
|---|---|
| src/assembly/meari/player_shell_host/observation.rs | 새 retained 책임 모듈/대기 관측 보유, 실제 notice의 typed proof 전달. 현재 strict 시작·Card 관측 경로와 구별 |
| src/assembly/meari/player_shell_host/observation/card.rs | 기존 strict full-stamp guard 유지. strict/retained 대기의 상호 배타만 추가하여 겹친 arm 거절. source34의 arm_jobs_route도 보존 |
| src/assembly/meari/player_shell_host/observation/retained.rs (신규) | 원 handle+commit proof+accepted route ticket 보유/검증, 같은 요청의 idempotency·다른 active 요청 Busy, 취소/종료/유효 frame 소비. SDK 정책 없음 |
| src/assembly/meari/player_shell_host/observation/retained/tests.rs (신규) | 원 Native runtime/current callback·proof 보존/실제 stamp 전진·None/foreign/stale 거절·동기 strict 유지에 한정한 시험 |
| src/assembly/meari/viewer_host/jobs/route.rs (source34 후속) | Returning이 원 receipt와 commit proof를 함께 보존. 현재 Viewing의 local recovery callback으로 retained arm만 재요청. SDK 재제출/recommit/reattach0 |
| src/assembly/meari/viewer_host/jobs/tests/product_route_fixture.rs (source34 후속) | 기존 synthetic literal 1곳에 retained:None. 원 runtime/proof 검증 관측 helper를 쓰되 실제 제출 proof를 만들어내지 않음 |
| src/assembly/meari/viewer_host/jobs/tests/product_route.rs (source34 후속) | 과거 stamp 불변 단언을 실제 input/turn 전진+원 receipt 불변으로 수정. None의 retained 완료 거절을 명시하고 기존 정상/negative/wake/close 단언 보존 |
| src/assembly/meari/player_shell_host/observation/card/tests.rs | 기존 literal 1곳에 retained:None만 추가, strict 정책 단언 유지 |
| src/assembly/meari/viewer_host/mount_card/tests/capture_readonly/guarded_ipc/product_surface/return_viewing.rs | 기존 literal 1곳에 retained:None만 추가, 기존 strict Card 복귀 근거를 retained 인수로 확대하지 않음 |
| src/assembly/meari/quarantine_surface/frame_delivery/tests.rs | 기존 literal 1곳에 retained:None만 추가, 기존 전송 정책 단언 유지 |

제안은 기존8/new2 총10경로다. source34와 겹치는4(card.rs·jobs/route.rs·fixture·route tests)는 원 r3를 덮어쓰지 않는 후속 revision에 둔다. 추가 기존4(observation.rs·literal tests3)는 최종 API 및 PM 배정 뒤 root preimage를 새로 고정한다. 신규2를 포함하면 제품 전체 입력 상한40경로이며, Meari source10과는 별도 manifest다. 이 계획은 그 조립/실행 승인을 뜻하지 않는다.

strict 대기를 일반 retained 대기로 일괄 바꾸지 않는다. 처음 진입·기존 Card 경로의 source와 정책 인수는 유지하고 JOB→Viewing의 일반 재연결에만 typed retained 관측을 소비한다. 제품 router/SDK application/domain 정책을 라이브러리에 옮기지 않는다.

## 시험 제안과 양성 근거의 한계

1. 원 실제 Viewing commit→proof 보존→fallible arm 실패→현재 Viewing callback의 실제 Action→NativeHostRuntime next turn→같은 proof 재연결. current stamp는 전진하고 원 receipt/proof.receipt는 동일해야 한다. SDK 요청·props commit·attachment 수 추가0.
2. 같은 값 새 projection commit/다른 handle·owner·definition·generation/종료·poison을 거절하고 원 receipt·query owner·첫/후속 오류를 보존한다. partial mutable error의 무효화는 library가 담당하며 제품에서 rollback 성공을 추정하지 않는다.
3. 같은 proof+같은 accepted route ticket의 재연결은 중복 작업이 없어야 하고 다른 active 기대/취소·종료와 구별한다. 기존 strict receipt gate 및 callback 권한은 유지한다.
4. None인 synthetic notice는 새 retained 경로에서 observed 완료를 만들지 않아야 한다. foreign stamp/ticket·서로 다른 proof/notice 조합도 거절한다. 기존 엄격한 synthetic 정책 시험은 과거 범위로 남긴다.
5. 실제 checked submit의 Some proof→제품 observer→route 관측 완료라는 양성 통합은 별도 배정된 실제 checked 경로가 필요하다. 현재 제품 headless fixture에는 public proof 발급기가 없고 추가해서도 안 된다. library 내부 mint 시험을 제품의 실제 window/render 성공으로 승계하지 않는다. OS present/readback/시각·접근성 인수도 별도다.
6. 정상 continuation의 one-shot 및 오류 후 자동 retry0, 원 양 query owner drain/workerStopped/unack0·진단 보존을 회귀한다. 원 r3의 return 관련 synthetic 완료 단언은 None을 인증으로 승격하지 않도록 별도 인수 단계로 나누며, 실패가 사라졌다고 카운트하지 않는다.

현재까지 제품/공유 코드 편집0, 후보 자동 합성0, Cargo/S/T/GUI 실행0이다. source10 API의 독립 리뷰 결과와 최종 pin 뒤 PM이 정확 delta/양성 검증 경계를 배정해야 한다. modoo.today frontend-first-journey의 typed 원 port/기존 공통 책임 유지와 test-fixture-and-mock-build의 소유·합성/실제 증거 구분을 적용했다.

## PM 배정 후 r4 소스 인계

위 단락은 구현 승인 전 기록이다. PM의 명시 existing8/new2 배정 뒤 runtime/development/ui-jobs-retained-r4/에 별도 source40을 작성·동결했다. source SHA2B84E0170A7DA40824F0585A2949B8053004A2DF4D5643593C82911ACEC35ACD, before7D69F48391D104AA2400658872E793D3E5216F198856BC30F3E2483C840789F8, r3대비delta10 5AF49C48DCAAB009D7223F18668EF45B1415582F9752C1375C84E483B25E4C7F, 추가preimage4 F85CAAC69EDDC9A21713E21106B5EFB9DD8102098AB023557674AA0298176CA6다. root740b/library65cc preimage차이0이며 원 r3/librarysource10 및 canonical/S/T는 불변이다. 구현·작성26/실행0과 actualSome 양성 잔여를 stage HANDOFF.md에 구분했다. API 후보와 제품 소스의 타입 인수는 아직 없고 자동 합성·Cargo/GUI 실행은0이다.
