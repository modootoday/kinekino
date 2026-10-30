---
id: plan:kinekino/20261029113847
title: JOB retained 실제 checked 제출 — bounded NoMedia child1 후속 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# JOB retained 실제 checked 제출 — bounded NoMedia child1 후속 제안

## 2026-10-23 최종 배정·동결 상태

아래 본문은 최초 읽기 제안의 이력이다. 이후 PM이 기존8/new1의 exact9 소스 작성을 배정했고 private r5에서 완료했다. 추가 승인 경로는 viewer_host/mod.rs의 cfg(test) reflection 반환값을 effects.redraw에 OR하는 한 연결이다. product_route.rs도 승인 범위다. canonical 반영·Cargo·실행은0이다.

정본 인계는 runtime/development/ui-jobs-checked-r5/HANDOFF.md. source46 SHA256=0A7DA60DB5EF1D55745FBA51A2DFDABF6D7F15228D3CD34DB9B4BECA26AF1131, delta9=5CE81E7529650B534B03DF48A7DF360C1CA1D3B34EB3A2F811F0D6175944A284. 기존25/new21, raw 해시 불일치0/preimage 불일치0. r4 후보 원본은 보존했고 r5 union에서는 원40 중37 동일/3 후속 변경+추가6이다. 작성35/compile0/actual0. 독립 소스 리뷰는 새 확정 P1/P2 없음이며 실행 인수가 아니다. PM이 SDK heavy 반환 뒤 제품 gate를 별도로 배정한다.

2026-10-23 읽기 제안이며 새 소스·OS/GUI 실행은0이다. 원 product40/r4 및 library10은 동결한다. PM이 library 독립 gate→제품40 기능 gate를 인수한 뒤 양성 actual1을 별도 단일 heavy에 배정할 후보이며, 지금 실행/구현 승인이 아니다.

## 읽은 기존 경로와 그대로 재사용할 소유

viewer_native_launch/no_media/tests/end_to_end.rs의 prepare_native_execution→prepare_no_media_viewer_launch→prepared.run→finish_typed_viewer를 재사용한다. supervision.rs의 원 Supervisor<WindowsFamily>, Deadline20초+정리2초, creation identity/EOF/family receipt와 inner-success.json exact schema/test/mode/scenario 검사를 유지한다. parent/test_support metadata, synthetic notice, exit0만으로 양성 제출을 대체하지 않는다.

NoMedia의 disabled 영상 경로를 유지하여 모델·영상 worker·다운로드·네트워크를 추가하지 않는다. 새 child가 명시 보유한 evidence/job-store의 기존 read_only factory와 같은 PlayerSdk를 사용해 실제 jobs.list(empty) 한 번을 받는다. durable job worker나 새 application 정책은 만들지 않는다. 원 LocalHost/ExecutionOwner/Native loop/WindowLifecycle만 생성·정리한다. 사용자 소유 player/demo 프로세스는 조작하지 않는다.

## 현재 코드의 차이와 최소 연결

1. prepare_no_media_viewer_launch는 prepare_viewer_document의 standalone 구성을 쓴다. 여기에는 기본 JOB ProductFrame entry가 없다. 공통 prepare_viewer_document_profile의 ProductFrame 모드를 같은 install_frames/원 실패 carrier와 연결하는 cfg(test) 전용 얇은 진입을 제안한다. 기존 public NoMedia 진입·원 normal/changed 두 시험의 의미는 바꾸지 않는다. 다른 startup/loop/cleanup을 복제하지 않는다.
2. viewer_native_probe::reflection은 첫 ObservedDisabled에서 즉시 close를 요청한다. JOB 시나리오는 그 전에 끝나므로 명시적 test-only opt-in probe mode가 필요하다. Probe::new의 기본 동작/기존 Kind 문자열·trace 순서·80개/4096byte 한도·sticky fault·original close closure는 유지한다. 새 mode에서만 최초 disabled 증거를 한 번 보존하고 JOB 왕복 완료 후 닫는다.
3. player_shell_host/delivery.rs는 route.observe 뒤 frame consumer가 요청한 경우만 NativeWindowWake::wake를 부른다. terminal NoMedia consumer가 새 JOB/retained frame마다 Completion을 보장한다고 가정할 수 없다. authoritative route의 phase/observed 전환이 생긴 경우 기존 wake를 한 번 요청하도록 하는 최소 정상 연결과 영향 시험을 별도 제안한다. route를 직접 성공 처리하는 새 guard나 loop는 만들지 않는다. 실제 attached 경로는 frame_wake를 설치한다. Ok(false)의 coalescing과 오류 보존은 원 경로를 유지한다.

## 후속 exact 소스 제안 — 기존7/new1

| 경로 | 제안 책임 |
|---|---|
| src/assembly/meari/viewer_native_launch/no_media.rs | cfg(test) ProductFrame 준비 경로를 원 factory/install_frames/실패 retention에 연결 |
| src/assembly/meari/viewer_native_launch/no_media/tests/end_to_end.rs | 새 exact child 시험 모듈 등록. 기존 normal/changed 경로·프로토콜 유지 |
| src/assembly/meari/viewer_native_launch/no_media/tests/end_to_end/jobs_retained.rs (신규) | 기존 supervisor를 쓰는 bounded child1/명시 empty job-root·동일 SDK/실행·원 소유 정리·결과 기록 |
| src/assembly/meari/viewer_native_probe.rs | 명시 opt-in mode/한정 phase·원 proof 증거·실제 notice 관측/최종 원 close 요청. 기본 probe와 기존 source 시험 유지 |
| src/assembly/meari/viewer_host/jobs/route.rs | cfg(test) 원 receipt/proof·query-owner 정리 완료 기록, 성공한 retained arm 단계 전환의 원 notifier 단발 요청과 receipt 없는 redraw. 같은 현재 local route intent 중복의 인증·coalescing으로 zero-batch 진행 유지. SDK 정책/호출 변경 없음 |
| src/assembly/meari/viewer_host/jobs/tests/product_route.rs | arm 뒤 단발 wake/redraw·완료/idle 뒤 추가 요청0·원 current key의 중복 admission 진행·old/foreign 거절 회귀 |
| src/assembly/meari/player_shell_host/delivery.rs | 원 route 관측 전환의 one-shot Completion wake 및 test-only notice probe. 모든 document/route/consumer borrow가 끝난 뒤 기술 action/wake |
| src/assembly/meari/player_shell_host/delivery/tests.rs | 전환 때 wake/미전환 때 추가 wake0·실패 보존·원 observer/frame consumer 호환 회귀. metadata 시험을 실제 제출로 세지 않음 |

supervision.rs는 기존 parent_run(exact,mode)와 schema=viewer-no-media-inner-v1을 그대로 재사용하므로 현재 변경 제안에 포함하지 않는다. root Cargo/main/launch/default mode·library10·product40 원본도 바꾸지 않는다. 최초 PM 배정은 기존6/new1의7경로이며, 위 product_route.rs 영향 시험1은 추가 exact 제안이다. 아직 범위 밖 시험 파일을 편집하지 않는다. 예상 밖 추가 파일이 필요하면 다시 exact 제안한다.

## arm·중복·실패·close의 최소 wake 의미

재arm은 새 props commit이 없으므로 receipt에 따른 redraw가 자동으로 생기지 않는다. 성공한 CommittedAwaitingArm→Armed 전환에서만 원 notifier 한 번과 redraw를 요청해 실제 checked 재제출 기회를 만든다. receipt를 제조하거나 projection을 다시 commit하지 않는다. 이미 Armed인 같은 요청은 다시 notify/submit하지 않는다. notify 실패는 원 proof/owner/진단을 보존하고 자동 retry하지 않는다. close가 우선하면 원 close driver가 정리하며 늦은 wake를 복귀 성공으로 세지 않는다.

현재 Armed의 모든 callback을 Capacity로 두면 실제 runtime remaining>0와 제품 zero-batch barrier가 서로 기다릴 수 있다. 같은 current onOpenJobs local key는 기존 verify_jobs_entry로 인증한 뒤 무작업 Accepted로 coalesce하고, 새 SDK/관측 동작을 만들지 않는 후속 수리를 제안한다. 이미 한 개 보존된 동일 current local intent의 중복도 인증 없이 버리거나 queue를 늘리지 않는다. old/foreign key와 다른 역할 callback은 거절한다. 이는 읽기에서 발견한 회귀 대상이며 아직 실행한 반례가 아니다.

delivery는 route phase/observed의 실제 전환만 bounded pending wake로 보존한다. 이후 frame consumer 오류나 wake 오류가 생겨도 전환 알림을 잃지 않고 원 delivery failure/notice queue와 함께 유지한다. 모든 borrow가 끝난 뒤 원 wake를 요청하고 성공(Ok(false) coalesced 포함) 후만 pending을 비운다. 동일 frame 재전달/미전환에서는 추가 wake가 없고 terminal/close는 알림보다 우선한다. observer guard를 직접 성공 처리하는 경로는 만들지 않는다.

## 실제 상태 전이와 관측 순서

실제 초기 Viewing frame→현재 source의 onOpenJobs action→NativeHostRuntime callback admission→SDK jobs.list 실제 empty 결과→JOB props+실 frame 관측→현재 onNavigate(VIEW) action→원 Viewing commit/retain_commit→실 checked_window PreparedScene.submit→RetainedFrameProof(Some) 발급→window notice→제품 retained observer의 원 commit/frame validator→View route 관측→정상 Completion→Jobs query owner 정리/normal Viewing 복원→최종 실제 frame 뒤 원 close bit→run 반환→finish_typed_viewer다.

기술 action은 현재 Native source callback path/owner에서만 dispatch한다. probe가 state.observe_route·admit_callback을 직접 부르거나 SDK 정책을 우회하지 않는다. action을 ViewerHost.after_dispatch 안에서 넣으면 runtime receipt.before/after를 바꿀 수 있으므로 금지한다. 실제 notice delivery에서 원 관측/borrow들이 끝난 뒤 action을 넣고 기존 wake로 다음 원 runtime turn이 소비하게 한다. 원 source34/r4의 turn당 projection1회 규칙은 유지한다.

probe는 원 commit 직후 clone한 opaque proof/원 receipt를 UI thread 안에 제한 보유한다. 실제 Some notice의 stamp/ticket과 proof를 대조하고 validate_retained_frame을 통과한 결과만 기록한다. 원 route 상태의 관측 성공도 함께 확인한다. 다음 normal Viewing props commit은 원 proof를 무효화할 수 있으므로 실제 제출 검증은 그 전에 기록하고, 이후 새 proof로 바꿔 과거 성공을 만들지 않는다. 최종 query-owner release/close는 별도 phase 증거다.

기존 NoMedia ProofObserved(영상 비활성화 표면 증명)를 새 retained JOB 복귀 성공으로 세지 않는다. 최초 Viewing만 렌더하고 닫는 실행도 실패다. JobEntered→실제 list 결과→ReturnCommit/원 proof→ActualRetainedSubmitted/validator 통과→ReturnObserved→JobsOwnerReleased→CloseIntent→InnerClose→RunReturned→CleanupReturned의 순서와 같은 원 host/owner/context를 요구한다. SDK list1/get0/mutation0, 재commit·reattach·재제출 부재를 각 phase와 연결한다.

자동 종료는 이 시험의 원 child에만 적용한다. 기본 probe의 첫 disabled frame close와 새 opt-in mode의 최종 phase close를 혼합하지 않는다. 실패/시간초과는 원 supervisor/receipt/raw/원 cleanup carrier에 남긴다. parent forced 종료나 family unknown을 양성 GUI 정리로 세지 않는다. 성공은 child exit0+정확 inner record+원 family Empty/EOF/forced false와 원 endpoint/SDK/Native child-first cleanup 증거를 모두 요구한다.

## 인수 순서·한계

library10 실제 타입/정책 gate, product40 실제 NativeHostRuntime headless gate, 위7 source 독립 review/기본 probe 회귀를 먼저 수행한다. 이후에만 중앙 단일 heavy/원 resource admission 아래 새 child1을 배정한다. 기존 SDK·영상 기능 검사를 GUI로 반복하지 않는다. 이 actual1도 OS compositor present/readback·사람 키보드/IME·전체 화면 시각/접근성·13화면 디자인 승인 근거가 아니다.

modoo.today frontend-first-journey의 원 typed port/공통 조립 재사용과 test-fixture-and-mock-build의 원 자원·실패·진단 carrier 및 실제/합성 증거 분리를 적용했다. 지금은 읽기 제안으로만 인계하며 제품40 외 private 소스 확장·OS/GUI 실행0을 유지한다.
