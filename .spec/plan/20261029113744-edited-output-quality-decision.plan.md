---
id: plan:kinekino/20261029113744
title: 편집·채택 출력의 별도 품질 판정 후속
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 편집·채택 출력의 별도 품질 판정 후속

## 현재 root 재감사: 기존 연결·reachable 경계 — 재구현0

2026-10-20 PM 배정 읽기 감사다. `runtime/coordination/ui-output-review-reachability-audit/read-only-audit.json`에 감사 root commit·C21 원 pin21개의 현재 SHA를 기록했고 **21개 모두 byte 차이0**을 확인했다. wire18/SDK3endpoint를 다시 구현하거나 품질 UI가 없는 것으로 간주하지 않는다. 현재 읽기 대조는 새 자동/GUI 실행 증거가 아니며 원 C21의27pass와 구분한다. capture6 pin497E…B792는 보존하고 새 source·Cargo·GUI 실행0이다.

### 실제 reachable 경로

| 진입/관측 | 현재 코드와 도달 범위 | 완료로 확대하지 않을 범위 |
|---|---|---|
| 인수 없는 기본 `player.exe` | `application/launch.rs`의 Player → `main.rs`의 `presentation::run_with_native_playback` → `presentation/mod.rs`의 eframe/Glow. 공통 SDK/LocalHost는 조립하나 TSX ProductWindow 품질 컴포넌트 mount 분기로 가지 않는다. | 새 품질 UI가 기본 사용자 화면에서 reachable하다고 표시하지 않는다. |
| 명시 NativeProposal | 같은 main의 명시 입력 검증·configured client/LocalHost → `assembly/meari/product_native_launch.rs::prepare_launch_window` → `prepare_product_frame_window`. 실제 proposal/source/revision/hash와 store가 필요하다. | 시험 proposal/빈 가짜 기본 대상이나 다른 SDK를 만들어 기본 Player로 위장하지 않는다. |
| 공통 Native ProductWindow | `product_window/mod.rs:237` Native8 register_output_reviews → `document_factory.rs:550~575` 실제 capability 검사/기존 controller 활성화 → `window_bridge.rs:565` 같은 state/pending을 project_component_props에 투영. 세 원본 TSX가 동일 ProposalOutputReview를 mount한다. | 기존 Native5/Immediate profile은 품질 Unsupported/disabled이며 서로 같은 등록이라고 보지 않는다. |
| 원 headless 인수 | C21 product4는 Cards/Highlights/cue 교정/document repair→typed 교정의 실제 원본 Native DOM/callback/같은 worker/save/accept/inspect/select/Deferred record/history/cleanup. 영향23 포함27pass다. 런처2의 실제 공통 frame 소비도 별도 인수됐다. | 합성 Deferred를 사람 품질 승인·모델 표본 통과로 세지 않는다. 과거 renderer/base 인수를 최신 전체 Native/Web 시각·OS 입력 성공으로 전이하지 않는다. |
| 사용자 인수·제품 기본 전환 | 수동 GUI/실제 keyboard·IME·focus·공통 frame caption/실제 화면·전체 route/native video 연결은 별도 작업이다. 기본 Player 전환의 빈 초기 route/store/영상 소유 계약은 명시 proposal editor와 다르다. | 사용자 GUI 검토 보류를 해제하거나 default mode를 한 줄 바꿔 완성으로 표시하지 않는다. |

`panel_model.rs`/일반 `projection.rs`의 legacy ReviewView만 검색하면 별도 `output_review_state.rs`/`output_review_panel.rs` 공통 투영을 놓친다. `ProposalOutputReview`는 `web_ui/components/proposal-output-review.tsx`, 원 소비자는 `studio/cards/proposal-panel.tsx`, `studio/clips/highlight-proposal-panel.tsx`, `studio/subtitles/correction-panel.tsx`다. 기존 frame/token/Button/React 선언을 재사용하며 컴포넌트·테마·직접 SDK 정책을 새로 복제하지 않는다.

### 유지할 상태·오류·동시변경 계약

| 상태/입력 | 기존 구현이 제공하는 계약 |
|---|---|
| Idle/미등록 | 조회 전 기본 verdict 없음. SDK profile이 없으면 Unsupported/disabled이고 품질 성공을 생성하지 않는다. |
| Inspecting → Ready | typed inspect 한 응답의 exact target/proposal/output revision/SHA와 source/generated/draft/accepted text, criteria 전체 id/version/purpose/clauses/identity·evidence를 함께 보유한다. 조회 완료는 자동 선택/승인이 아니다. |
| 명시 선택/입력 | 같은 inspection을 별도 선택한 뒤 usable/needs_changes/deferred를 고른다. note/declared reviewer는 필수, reviewer ID/claimed Unix ms는 선택이다. 선언은 인증 신원이 아니고 claimed/observed 시각을 구분한다. |
| Recording | 현재 output revision과 exact target·criteria·evidence를 사용하고 SDK CAS/저장 정책을 복제하지 않는다. 기존 worker/registry/ticket/ACK/FIFO/NodeLease를 사용하며 transport pending이면 편집·save/accept·다른 record를 함께 막는다. |
| edit/save/accept/source/scope 변경 | 기존 controller invalidate로 inspection/선택/편집 판정 필드를 해제한다. 진행 ticket은 폐기하지 않고 drain/ACK한다. 과거 응답·다른 owner/token·종류 불일치로 새 대상이나 UI를 되살리지 않는다. |
| RevisionConflict/NeedsInput | 알려진 pre-publication 오류는 typed 오류·remediation을 보존하고 재조회/재선택한다. 이전 criteria/출력으로 자동 제출하지 않는다. |
| publication_may_exist / unknown | NeedsInput이라도 publication_may_exist=true가 우선이다. 원 decision ID/body를 보존하고 새 decision 생성·자동 retry를 금지한다. immutable history 대조 또는 재조회/재선택 후 원 ID/body의 명시 재시도만 허용한다. |
| Recorded/history | 실제 immutable 저장 receipt와 target/output revision/SHA/decision SHA를 표시한다. 동일 request readback으로 확인하며 단순 history 개수/technical acceptance/old confirmed를 현재 품질 승인으로 바꾸지 않는다. history NotChecked는 새 evidence 재검증 성공이 아니다. |
| close/cancel | UI 수명 종료가 이미 전달된 mutation의 미실행/rollback을 뜻하지 않는다. 원 host의 cancel/terminal/ACK/cleanup 경계를 유지하고 결과 미확인을 성공으로 감추지 않는다. |

typed3endpoint는 기존 [wire 인수](../../sots/verification/application/accepted-output-review-wire-2026-10-20.md)를 그대로 사용한다. 기준의 초안 문서와 실제 SDK catalog/criteria identity를 구분하고 UI에서 version/hash·사람 정답·모델 승인·자동 적용을 발명하지 않는다.

### 다음 정확 범위 제안

현재 품질 컴포넌트/DTO/controller/등록/bridge/세 TSX의 **신규 제품 source 변경 필요는0**이다. 이번 범위는 이 기존 계획과 [제품 등록·런처 인수표](presentation/proposal-product-registration.md)의 reachable 기록만 정정한다. source 재구현·통과한 headless27/런처2 재실행은 하지 않는다.

다음 코드 작업은 품질판정 기능 추가가 아니라 기본 Player의 Native shell/route·동일 client/LocalHost/playback 수명 통합이다. source 범위는 해당 계약과 public native video slot의 실제 준비 상태를 확정한 뒤 별도 배정한다. 현 `main.rs`/`application/launch.rs`의 default mode를 NativeProposal로 단순 치환하거나 명시 request를 가짜로 채우는 변경은 제안하지 않는다. 실제 사용자 인수는 GUI 보류가 해제된 뒤 원 검사 대상/입력/receipt를 고정해 진행하고, 사람이 실제로 내린 판정과 합성 시험 verdict를 별도 근거로 남긴다.

## 2026-10-20 UI 인계 준비 — wire18 소스 대조

### 현재: C21 실제 제품 mount·기능 검증 인수

이 절이 아래 source-only/미mount 작성 기록보다 최신이다. C는 승인된 exact21/신규2이며 `runtime/coordination/ui-output-review-product-c1/{paths.txt,pins.json,source/,before.json}`에 동결했다. pin SHA256은 `1A10ED1F569D814424AAF9ACA5B0F5002EEA3C662358C68465589F7DBEE1EF59`다. **카드·교정·하이라이트 세 원본 화면이 이제 같은 품질 판정 컴포넌트에 실제로 연결됐다.** 공통 Rust helper가 prefixed props/callback과 기존 NodeLease/FIFO를 준비하고, 초기/live props를 맞춘다. ProductWindow는 Native8을 선택하며 factory가 실제3 capability를 확인한 뒤 기존 pending controller를 활성화·재투영한다. legacy5는 disabled/Unsupported다. 새 decision ID는 infrastructure의 OS CoCreateGuid 패턴으로 공급하며 새 의존성·clock/process/counter 보장·claimed/observed time 정책 복제는 없다.

자동 기능 gate는 `native-proposal-window` feature의 실제 ProductWindow/원 frame·InspectorDock/실제 CSS·assets/NativeDocumentHandle 및 같은 SDK worker를 사용했다. 원문·초안을 실제 Native DOM 문자/controlled textarea와 대조한 뒤 입력→저장→명시적 채택→전체 기준 조회·명시 선택→필수 메모/선언 검토자·선택 claimed time→Deferred 기록→history를 실행했다. 문서 복구는 공개 DocumentRepairSelection 타입과 전체 UTF8 digest/LF/escape·source_ref를 사용해 repair→typed 교정으로 전환하고 원 source/malformed raw를 보존했다. 매 단계 현재 callback path를 조회하고 host.operation_pending을 drain했으며 OwnedProductCleanup의 workerStopped/unack0까지 확인했다. GUI View를 여는 observer는 금지한 headless 실행이다.

| 실행된 범위 | 실제 관측 |
|---|---|
| 새 실제 제품 flow | 4 pass /129525ms/exit0: Cards, cue 교정, Highlights, document repair→typed 교정. 0 fail/ignored/filtered |
| 초기/live bootstrap / 교정 projection | 12 pass /54788ms, 5 pass /2813ms |
| 기존 원 review / Inspector dirty·focus·identity·atomic export 실패 복구 | 2 pass /31659ms, 4 pass /49033ms. 다른 frame5/과거 전 계약은 반복하지 않음 |
| root default/native all-target strict | 23081/21264ms/exit0 |
| bridge native all-target strict | 23586ms/exit0 |
| root/bridge cargo fmt --check | 2817/3312ms/exit0 |

실제 행위 시험 합계27이며 통과한 product4를 변경 없이 반복하지 않았다. `runtime/quality-review/ui-output-review-product-c1/`의 원10 receipt는 모두 terminal/exit0, 원 handle 모두 종료·기록20 PID 부재 `final-owned=[]`, 실제 시험 바이너리4 identity를 남겼다. 검증 B8/A4/routerr4/Meari1df 기초에 exact21만 overlay하고 내용 hash/mtime21을 대조했다. 최종 live/snapshot21 hash 차이0·보호 snapshot549 차이0이며 heavy와 제품 snapshot을 PM에 명시 반환했다. index/stage/commit/push0, 다른 SDK/Meari 후보/gitlink 혼합0이다.

이 gate는 **세 작업의 정상 제품 기능 흐름** 근거다. 자동 합성 입력을 실제 사람 품질 승인·모델 표본 성공으로 세지 않는다. GUID 실패와 mounted product의 unknown/cancel/stale 전체는 C에서 새로 실행한 범위가 아니며 앞선 B/SDK 분리 계약 또는 소스 검토를 참조한다. 수동 GUI/스크린샷/Native-Web 실제 시각 비교·전체13화면 시각 일치·새 player.exe 배포 artifact build는 미수행이다. 아래의 ‘C 미연결’ 문장은 각 과거 단계 당시의 잔여 기록으로 한정한다.

### PM의 SDK gate 인수 후 최소 source-only 작성

PM은 wire4/schema2/fresh CLI10(96 프로세스)/storage16/legacy7+3/default·native strict/fmt gate 종료와 source18 hash delta0을 전달했다. 이는 SDK 인수 근거이며 아래 UI 소스의 컴파일·실행 성공으로 전이하지 않는다. 첫 편집 전 exact5 source와 이 기존 계획1을 PM에 전달했고 동일 범위 source-only 배정을 받았다.

| 작성한 파일 | 책임과 현재 범위 |
|---|---|
| `src/presentation/web_ui_adapter/proposals/mod.rs` | 아래 두 presentation 모듈 등록. |
| `src/presentation/web_ui_adapter/proposals/pending_host.rs` | 기존 async host의 typed request/response에 inspect/record/history 추가. 기존 dispatch·ticket·poll·provider ACK·cancel·close 경로 재사용. 새 기능의 미등록 capability는 Unsupported. 기존 필수5 capability를 새8로 강제하지 않는다. 큰 새 DTO는 boxed variant로 전달. |
| `src/presentation/web_ui_adapter/proposals/output_review_state.rs` | 동일 inspection 전체 보유, 조회 뒤 명시 선택, 기본 verdict 없음, note/reviewer 입력, 원 output revision 전달. 요청별 owner token과 operation으로 다른 view/이전 응답/다른 응답종류를 거부. 외부 invalidation은 ticket을 버리지 않으며 늦은 응답으로 대상을 되살리지 않는다. 기록 결과 미확인은 원 요청을 보존하고 history 대조 또는 재조회·재선택 후 동일 ID/body의 명시 재시도만 제공한다. SDK hash/criteria/CAS/저장 정책은 호출하지도 복제하지도 않는다. |
| `src/presentation/web_ui_adapter/proposals/output_review_panel.rs` | SDK가 준 target·같은 state의 원문/생성 원문/저장 초안/채택 출력·전체 criteria·evidence observation·기록 이력·선언/관측 시각을 표시용 props로 투영. `historyNotChecked`는 history의 NotChecked에서만 파생한다. |
| `src/presentation/web_ui/components/proposal-output-review.tsx` | 기존 Button·inspector/evidence/input 토큰 스타일을 재사용하는 세 task 공통 선언. technical acceptance와 별도 영역, 선택·세 verdict·필수 메모/검토자·선택 ID/claimed time·기록·history·동일 요청 재시도 action 선언. JavaScript 정책/SDK 호출 없음. |

상태 모듈 안에 교차 view token, 잘못된 응답종류의 pending 소유권 유지, invalidation 뒤 history의 비선택/NotChecked, 이전 token 재사용·verdict 기본값 금지 회귀4를 작성했다. 이력은 빈 합성 구조만 사용하며 실제 사용자 품질 승인 fixture는 없다. **시험은 아직 실행하지 않았다.** 직접 rustfmt/차이 검토만 수행하며 Cargo/GUI는 Meari heavy 반환 후 별도 배정이다.

source5 사본은 `runtime/coordination/ui-output-review-thin-r1/{paths.txt,pins.json,source/}`에 고정했다. pins JSON SHA256은 `DEF2AF8918273BAF69B753206209992023248F282CD7EC57E088A5CCFD5B117D`다. 직접 rustfmt --check4와 tracked diff --check를 통과했다. 신규 TSX의 실제 Meari 원 module graph/Button/useId 포함 lower·props bind는 **미검증**이며 Rust 포맷 성공으로 대체하지 않는다. 다음 집중 시험 후보는 `tools/player-ui-sdk-bridge/tests/output_review_panel.rs` 하나이며 원 TSX details/summary/controlled textarea/currentTarget.value/Boolean props 논리/string repeat, 정상 inspect→명시선택→세 verdict, unknown 동일 ID/body 재시도/다른 target·criteria 거부, 전체 state/target/criteria·claimed/observed·NotChecked 투영을 포함해야 한다. 이 시험 파일 작성·실행은 별도 배정 전이다.

### exact6 후속 실행 관측 — r3 기능 gate 인수

위의 미실행 표시는 최초 source5 작성 시점이다. 이후 PM이 시험1 추가와 exact6/snapshot/단일 heavy를 배정했다. 신규 시험은 `tools/player-ui-sdk-bridge/tests/output_review_panel.rs`이며 원 source5와 별도로 r2 exact6을 보존했다. PM/AI 소스 검토가 발견한 `NeedsInput + details.publication_may_exist:true`는 게시 전 거부로 분류하면 안 되는 P1을 state1에서 수리했다. 이 표식을 코드 분류보다 우선해 unknown·원 error/details·attempted body를 보존한다. false/부재의 실제 사전 NeedsInput 분류는 유지한다.

첫 실행은 시험의 `ProposalFiles::open(root)` 참조 전달 오류 E0308로169778ms/101에 종료됐다. 원 로그·원 r2 source를 보존하고 원 핸들 종료/프로세스 부재 후 `&root` 한 줄만 수정했다. 최종 exact6 r3는 `runtime/quality-review/ui-output-review-thin-r2/{source6-r3.json,source-r3/}`이며 JSON SHA256 `467656FC2E11C2ADD01D10461D6F89CCD92C802A8CE0BFA1FBF96FA0FDE4D2BB`다.

| 최종 관측 | 결과 |
|---|---|
| 원 TSX 실제 Meari graph/Button/useId lower·props bind/재투영, canonical typed SDK 합성 자료의 세 task 명시 선택·필수 입력·세 verdict, 전체 투영, unknown/history/동일 body retry·target/criteria 변경 거부·late/다른 view 응답 | 집중8 pass /96926ms/exit0 |
| 상태 모듈 inline 회귀 | 4 pass /143692ms/exit0. 다른 workspace lib의 filtered0은 추가 통과 시험으로 세지 않음 |
| root default/native all-target clippy -D warnings | 각각2038ms/27512ms/exit0 |
| bridge native feature all-target clippy -D warnings | 77783ms/exit0 |
| root/bridge cargo fmt --check | 각각4178ms/4262ms/exit0 |

동결 snapshot에 추가 exact6만 overlay했으며 보호541과 최종 live/snapshot exact6 hash 차이는0이다. 원 실행 핸들8 모두 종료·receipt terminal, launcher/child16 PID 부재 `final-owned=[]`, 실제 시험 바이너리 hash는 `test-binaries.json`에 있다. 기존 cache·canonical runner·affinity3/BelowNormal/offline locked j1/testthreads1을 사용했다. 단일 heavy와 snapshot을 PM에 명시 반환했다. index/stage/commit/push/GUI/미디어/추론0이다.

이후 PM이 root index를 최종 exact6 커밋에 한정 위임했다. 재해시·빈 index·exact staged paths·diff check 후 `9eec7bbdff059edf02b0dc3b52143c7ca34ee341`로6경로를 커밋했다. commit 설명에 미mount/async5 잔여를 명시했고 index empty를 확인해 PM에 즉시 반환했다. 다른 SDK/WIP/spec 추가·push는0이다. 앞 문장의 index 미사용은 gate 실행 구간 관측이다.

publication 가능 오류는 합성 SDK 게시 후 오류 shape 전달 또는 미게시 불확실 shape 주입이며 실제 파일 adapter readback 실패 재현으로 세지 않는다. 이 gate의 원 graph lower/props bind 성공을 Native 실제 render·Web 실행·시각 동등성으로 확대하지 않는다. 실제 사람 품질 승인·모델 표본 추가 성공0이다. 제품 mount/controller/async 새3명령 등록은 아래 gap으로 남는다. 다음 A는 manifest/async adapter/tests와 기존 canonical contract 파일의 **exact4 source-only** 확장이다. legacy BYTES/기존 소비자 identity는 유지하고 async8만 별도 profile을 사용한다. B 같은 controller/worker·typed callback 수명, C 세 원본 panel 및 실제 mount를 이어서 배정한다.

### controller B8 r2 실행 관측과 C 후속

PM 승인으로 B 최초6에 공통 `InteractionController` relay를 소유한 `document_controller.rs`를 추가했고 실제 통합시험1도 추가해 exact8이 됐다. 파일은 `src/assembly/meari/async_proposals/registration.rs`, `src/presentation/web_ui_adapter/proposals/{pending_controller.rs,document_session.rs,output_review_intents.rs,callback_adapter.rs,intent_queue.rs,document_controller.rs}`, `tools/player-ui-sdk-bridge/tests/output_review_controller.rs`다. `output_review_intents.rs`는 물리적으로 proposals 아래에 있지만 명시 path module로 `proposals::intent_queue::output_review_intents`가 소유한다. 공개 action/field/ID 포트는 `pending_controller`에서 재노출한다.

기존 register는 Native5를 유지하며 register_output_reviews만 Native8을 선택한다. controller의 한 ActiveKind가 기존 proposal token 또는 quality token을 보유하고 같은 host/ticket/poll/cancel/terminal ACK/close를 사용한다. quality 입력도 기존 FIFO의 NodeLease·entry/UTF-8 byte budget·text coalescing에 들어간다. Immediate 기본 경로는 canonical Unsupported를 error details에 담아 명시 거부한다. session_mut/save/accept/close가 target을 invalidate하고 unknown attempted는 보존한다. 잘못된 전달 reply는 이미 ACK된 terminal 관측을 보존한 unknown으로 전환하며 mutation을 재시도하지 않는다. bootstrap/화면 mount는 아직 이 단계에 포함하지 않았다.

동결 최초8 pin `8331582155C594BC96DA2DE7FBAD43BB9E13CD02CEF4873CC5644FD4CF7A9190`과 source는 `runtime/coordination/ui-output-review-controller-b1/`에 있다. 첫 실제5시험은4 pass/1 fail/162795ms101이었다. 공개 support가 inner client와 Counted를 별도로 반환하는데 worker에 inner를 넣어 thread 계측0 vs3 단언이 실패했다. 원 handle/PID 종료 뒤 **시험1 한 줄만** 실제 worker client를 Counted로 연결했고 단언·제품7·예산을 유지했다. 최종8은 `runtime/quality-review/ui-output-review-controller-b1/{input8-r2.json,source-r2/}`이며 pin `E9B7F96F1A140C960AFEB6191EC0397FBDFF8C1314EA07694DAE6E51C1E3EB56`이다.

| 실제 gate | 결과 |
|---|---|
| 새 Native8/실제 SDK/HostCallbacks·CallbackAdapter/공통 FIFO/controller 통합 | 5 pass /23734ms/exit0. 같은 worker·명시 선택·동일 요청 retry/history·edit invalidation·stale/close/ACK·budget/coalesce/retired lease·Immediate Unsupported |
| 기존 pending controller | 9 pass /105628ms/exit0. 기존 save/accept/follow-up inspect·취소·ACK 재시도·소유권·공통 queue 동작 유지 |
| root default/native all-target strict | 15932/48295ms/exit0 |
| bridge native all-target strict | 62950ms/exit0 |
| root/bridge fmt | 3154/3551ms/exit0 |

snapshot은 검증 A4/SDK router r4/기존 Meari1df 기초이며 보호553 차이0, 최종 live/snapshot8 차이0이다. 내용 hash와 갱신 mtime8/r2 test1, 실제 시험 이름·개수·바이너리 identity를 보존했다. 원8 receipt 모두 terminal·기록16 PID 부재 `final-owned=[]`이며 원 실행 handle을 모두 닫고 heavy/snapshot을 PM에 명시 반환했다. SDK router의 과거0시험 cache 실행을 이 통과 수에 포함하지 않는다. 실제 사람 품질 승인·실제 파일 readback 실패 재현·수동 GUI·Native/Web 시각 비교·index/push/모델 실행0이다.

다음 C는 승인 exact21에서 세 원본 화면에 같은 공통 component·prefixed props/callback·기존 NodeLease를 연결하고 초기/live props와 repair→typed 전환을 맞춘다. ProductWindow만 Native8을 선택하며 legacy5는 disabled/Unsupported를 유지한다. fresh decision ID는 기존 OS GUID 생성 패턴을 재사용하는 Rust infrastructure adapter에서 공급하고 UI는 주입 포트만 호출한다. clock/process/counter로 재시작 충돌 방지를 보장하거나 claimed/observed time 정책을 복제하지 않는다. 실제 SDK/headless Native에서 세 task의 원문 비교→사람 수정→명시 채택→품질 기록/history를 하나의 제품 흐름으로 검증하기 전에는 **사용 가능 UI 완료가 아니다**.

### 등록 A4 작성과 별도 실행 관측

아래는 작성 시점 기록이다. 이후 별도 배정된 A4 gate에서는 source pin `D204987303FFF4694BFFC9412012860A9583727201CD1C009A2D8AC246A387A4`의4파일만 검증 UI6/SDK 기초 snapshot에 overlay했다. 작성 중인 live B7/SDK router 및 Meari producer 변경은 섞지 않았다. `runtime/quality-review/ui-output-review-registration-a1/`에 집중7 pass+기존 actual-nine1 ignored(80953ms), 기존 manifest1 pass(1315ms), root default/native all-target strict(32941/24492ms), fmt(2647ms)의 원5 receipt와 로그를 보존했다. 모두 terminal/exit0이며 반복 검증은 하지 않았다. 다른 workspace lib filtered0은 추가 통과 시험으로 세지 않는다. opt-in assembly 확장은 기존 bridge 소스/계약을 변경하지 않아 추가 bridge 실행은 하지 않았다.

최종 live/snapshot4 hash 차이0·snapshot 보호547 차이0·launcher/child10 PID 부재 `final-owned=[]`와 실제 root 시험 바이너리 `test-binary.json`을 기록했다. 원 핸들 모두 종료 후 단일 heavy와 snapshot을 PM에 명시 반환했다. 이 구간 index/stage/commit/push/GUI/추론0이다. 기존 Native4/5 계약 보존·새 Native8 opt-in/권한/동일 worker/취소 terminal ACK의 기능 gate이며 실제 사람 품질 승인·B controller/C mount/Native-Web 시각 검증은 아니다. **이후 아래의 미실행 표시는 A4 최초 작성 당시로 한정한다.**

PM이 heavy를 Meari A에 교대하고 제품 snapshot을 동결한 뒤 승인된 live exact4만 수정했다: `src/assembly/meari/proposal_manifest.rs`, `src/assembly/meari/async_proposals/{mod.rs,tests.rs}`, `packages/media-contracts/src/plugin_contracts/proposal_v1.rs`. 이전 파일은 `runtime/coordination/ui-output-review-registration-a1/before/`, 새 사본은 `source/`와 `pins.json`에 보존한다.

- 기존 canonical `BYTES`, Immediate4 및 기존 async5의 contract identity·capability 집합을 유지한다. 같은 파일에 별도 `OUTPUT_REVIEW_PROFILE_BYTES`를 추가해 명령8·record output revision·typed target/criteria/명시 메모·검토자·시각·scope/grants/cancel/ACK·불확실 결과 경계를 기술한다. provider hash는 실제 source 변경을 계속 반영하며 기존 provider hash를 가짜로 고정하지 않는다.
- 새 `output_review_proposal_manifest()`/`output_review_adapter()`만 Native8 opt-in을 제공한다. 기존 `async_proposal_manifest()`/`adapter()`와 `AsyncProposalRegistration::register()`는 계속 Native5다. 공유 `adapter_profile`은 같은 execution attachment/typed SdkPort/registry pump를 사용한다. 새 request/reply identity는 canonical output-review schema source와 구분된 role로 계산한다.
- 기존 시험 파일에 회귀4를 추가했다: legacy4/5와 opt-in8의 계약·권한 구분, 실제 typed inspect/record/history+기존 inspect의 같은 worker 스레드, 새3명령 권한 미부여 시 SDK 실행 전 거부, 취소 ticket의 원 terminal/ACK 정리 뒤 scope 교체·이전 완료 거부. 합성 Deferred는 사람 품질 승인/모델 표본 성공으로 세지 않는다. 기존 pending/initial-attach 회귀는 그대로 둔다.

직접 rustfmt --check4와 scoped diff --check만 통과했다. **Cargo·새 시험4 실행·snapshot overlay는0이며 source-only**다. 이 opt-in adapter는 제품 등록 경로가 아직 선택하지 않으므로 새3명령이 실제 UI에서 호출 가능하다고 표시하지 않는다. B에서 등록 선택·동일 pending controller/worker·typed mailbox/NodeLease·불확실 결과 수명을, C에서 세 원본 panel/초기 props/재투영/원 graph inventory·실제 mount를 연결한다. Native/Web 실제 시각 비교는 계속 별도 미검증 gate다.

**후속 연결 gap:** 기존 세 proposal panel에 mount하지 않았고 callback registration/NodeLease·bounded mailbox/입력 문자열의 Rust typed 변환/decision ID 공급/pending controller·document commit 연결도 아직 없다. 따라서 현재 제품에서 판정 UI를 사용할 수 있다는 뜻이 아니다. 후속 exact scope에서 같은 document/registry/worker의 수명과 dirty 편집 충돌을 연결하고 초기 props/재투영 동일성·오류·기록 결과 미확인 회귀를 확인해야 한다. `ProposalView.quality_decision_known`과 technical Save/Accept 동작은 변경하지 않았다.

실제 Native/Web 시각 비교는 계속 **미실행**이다. 동일 story/state/viewport/scale/Pretendard·JP/font asset/token/source revision의 두 실제 renderer 결과로 비교하는 별도 gate는 [기존 후속 계획](presentation/visual-fidelity-follow-up.md)에 유지한다. 이 소스 작성·포맷 검사로 기능·시각·사람 품질 승인을 완료 처리하지 않는다.

아래는 **최초 인계 준비 당시**의 소스 대조 기록이다. SDK 동결 `runtime/coordination/sdk-output-review-wire18-r1/pins.json` SHA256 `55E7D2D73DBF24CF021490E49B6A8BBBB12AAA30EDA9FA5565C73A42499127EF`를 확인했다. 당시에는 SDK CLI gate 미인수·제품 source/Cargo/GUI/새 이미지·Pages 변경0이었고 별도 배정을 기다렸다. 이후 SDK 인수와 UI source-only 작성 범위는 위 후속 관측에 구분했다.

### 확인된 계약과 화면 경계

- `proposals.inspect_output_review_target`의 **동일 응답** `state/target/criteria_document/criteria_identity/evidence_observation`을 하나의 선택 단위로 보관한다. 이전 proposal 조회의 state와 새 target을 섞거나 UI에서 output hash·criteria identity를 재계산하지 않는다. target의 proposal/task/draft·output revision/hash format·SHA/source/generated identity/historical reference를 접어서라도 모두 확인할 수 있게 한다. 기준은 purpose와 모든 clause의 ID/how_to_review/expected, version/content SHA까지 제공한다.
- `proposals.record_output_decision`은 선택한 target·criteria identity, 명시 verdict `usable/needs_changes/deferred`, 필수 note·declared reviewer label, 선택 reviewer ID·claimed time을 전달한다. `expected_revision`은 선택한 **output revision**이다. 기록 시각은 응답의 adapter `observed_record_time`으로만 표시하며 입력 claimed time이나 인증 신원으로 대체하지 않는다. 숫자 revision/time은 wire decimal 문자열을 유지한다.
- 기존 `ProposalActions`의 “명시적 채택”은 technical acceptance다. 품질 판정은 별도 공통 영역과 별도 명시 제출이며 채택 성공으로 usable·사람 편집·모델 품질 승인을 생성하지 않는다. 기존 `ProposalSource`의 저장 historical 참조와 session evidence observation도 구별한다.
- `proposals.output_decisions`의 기록과 `current_evidence: not_checked`를 함께 표시한다. presentation 상태 후보 `historyNotChecked`는 이 응답에서만 파생하며 새 wire 필드가 아니다. 이력의 존재를 현재 historical 검증 성공으로 표시하지 않는다. 각 기록의 exact target·criteria document·verdict/note/reviewer/claimed·observed time/decision SHA를 보존한다. `accepted_output_only`, automatic apply false, model sample 집계 false 경계는 그대로다.

### 정확한 presentation 책임·파일 후보 — 아직 작성하지 않음

| 책임 | 기존 연결점 / 최소 신규 후보 |
|---|---|
| SDK typed 조회·기록·이력 호출 | `src/presentation/web_ui_adapter/proposals/registry_host.rs`, `pending_host.rs`, `pending_controller.rs`. 기존 application client/registry/worker와 오류 보존 경로를 사용하며 application 정책·저장 adapter를 UI에 복제하지 않는다. 기존 Save/Accept pending과 합칠 구체 타입 범위는 gate 후 확정한다. |
| 별도 품질 선택·입력·상태 | 신규 후보 `src/presentation/web_ui_adapter/proposals/output_review_state.rs`: inspection 전체를 보유하고 사용자 입력/요청 중/재조회 필요 상태만 관리. `session.rs`의 draft/dirty·technical receipt 소유권을 대체하지 않는다. |
| 공통 props와 callback 연결 | 신규 후보 `src/presentation/web_ui_adapter/proposals/output_review_panel.rs`, 기존 `panel_model.rs`, `correction_panel.rs`, `bootstrap.rs`, `callback_adapter.rs`, `intent_queue.rs`, `document_factory.rs`, `window_bridge.rs`. 실제 필요한 파일만 후속 exact scope로 배정한다. 기존 NodeLease·bounded mailbox·같은 document/session·commit 후 상태 승격을 유지한다. |
| 세 task 공통 시각 영역 | 신규 후보 `src/presentation/web_ui/components/proposal-output-review.tsx`; 기존 `studio/cards/proposal-panel.tsx`, `studio/clips/highlight-proposal-panel.tsx`, `studio/subtitles/correction-panel.tsx`에 동일 컴포넌트 연결. 기존 공용 Button/입력 컴포넌트·inspector section/footer·토큰을 재사용하고 별도 화면/테마/독립 창을 만들지 않는다. |

### 필수 상태와 남은 인수 조건

| 상태 | 요구 동작 |
|---|---|
| unavailable / accepted output 없음 | capability unsupported 또는 NeedsInput을 원 오류와 구분해 표시. technical 채택을 자동 수행하지 않는다. |
| inspection 요청 중 / ready | 다른 대상의 이전 inspection을 제출 가능 상태로 유지하지 않는다. ready에서 state의 원 source/generated/저장 draft/선택 acceptance와 target 및 전체 기준을 함께 제시한다. |
| 명시 verdict 입력 / 기록 중 | verdict 기본 usable 선택·자동 제출 없음. note/reviewer 필수, claimed time 선택. pending 중 중복 제출·편집 충돌은 기존 transport 수명 관리에 연결한다. UI 필수 입력 안내가 SDK 검증을 대체하지 않는다. |
| stale / 기준·근거 오류 | RevisionConflict·NeedsInput·Unsupported 등 canonical 오류를 보존한다. 저장 중 자동 latest 재연결/자동 재시도 금지. 재조회 후 사용자가 정확 대상을 다시 선택하고 제출한다. |
| 결과 미확인 / 기록 성공 / historyNotChecked | transport 결과가 불명확하면 성공·실패를 발명하지 않고 이력 재조회로 확인한다. decision ID와 동일 요청의 재시도 의미는 SDK 계약을 따르며 새 ID로 자동 중복 생성하지 않는다. 성공은 반환 기록 범위만 표시하고 이력은 현재 evidence 미검증 표시를 유지한다. |

현재 gap은 기존 proposal props/typed callbacks/pending 상태가 이 세 명령을 연결하지 않는다는 점이다. `ProposalView.quality_decision_known`을 historical 유무·acceptance·이력 개수만으로 true로 바꾸지 않는다. 먼저 SDK gate의 typed 응답·오류·fresh 영속/read-back·동일 요청 재시도 관측을 인수하고, UI 배정에서 exact 파일과 bounded 입력 예산을 고정해야 한다. 테스트용 합성 기록은 실제 사람 품질 승인·원 모델 추가 성공 수에 포함하지 않는다. 실제 Native/Web 시각 확인은 별도 실행 배정 및 기존 시각 검토 규칙을 따라야 하며 이 문서 대조로 완료 처리하지 않는다.

2026-10-20 중앙 배정 설계안. **미구현·미검증이며 SDK 후속 인수 요청**이다. 정확한 새 endpoint 이름·DTO·저장 schema는 SDK가 확정한다. [사람 검토 인수표](../../sots/player/infrastructure/inference/research-followups/model-quality-final-review-readiness.md)의 편집 output 판정 gap만 다룬다. 기존 historical 검증·원 candidate 판정·technical acceptance 구현을 대체하지 않는다.

현재 세 기능의 원문 비교→수정→명시 채택 payload/기대 실패는 기존 [proposal API의 시나리오 정본](../../sots/player/application/api/proposals.md#2026-10-20-세-기능의-sdkcli-인수-시나리오)에 둔다. 그 구현된 technical 경로와 아래 미구현 quality decision 확장을 구분한다.

task별 `criteria_id/version/task/purpose/clauses`의 미인수 내용 초안은 [채택 출력 검토 기준](../../sots/player/application/accepted-output-review-criteria.md) 한 문서에 둔다. catalog/판정 저장 구현이나 사용자 품질 승인이 아니다.

## 대상 식별과 불변식

| 항목 | 후속 계약 요구 / 현재 한계 |
|---|---|
| 판정 대상 종류 | 원 `review package/item candidate`와 `accepted output`을 명시적으로 구분한다. accepted만으로 human-edited라고 표시하지 않으며 편집 이력·내용 변화·사람 행위 관측은 별도로 구분한다. 동일 task·문구·source라는 이유로 한 대상이 되지 않는다. 기존 candidate API는 그대로 유지한다. |
| 정확 output 식별 | proposal ID, 채택한 draft revision, receipt output revision, task, canonical output hash와 hash 형식 version을 함께 결합한다. 현재 receipt.output의 typed payload를 대상으로 하며 rendered media/파일 bytes hash라고 부르지 않는다. 미저장 working draft·현재 latest 포인터는 불변 판정 대상이 아니다. |
| canonical hash | SDK가 고정한 직렬화·문자열/숫자/배열순서·schema/hash version으로 domain/application에서 계산하고 저장된 receipt.output에서 재계산한다. 호출자가 보낸 hash만 신뢰하지 않는다. 기존 project capture hash는 전체 capture 대상이므로 output hash로 대체하지 않는다. 검토자·note·시각을 output hash 입력에 섞지 않는다. |
| source/lineage | receipt의 source ID/revision/SHA와 원 proposal source/generated identity를 결합한다. historical이면 companion SHA/bytes/item을 보존하고 원 review package/item/candidate asset과의 관계는 실제 검증된 연결만 기록한다. companion SHA와 package SHA를 혼동하지 않는다. 없는 historical/current job·attempt·project revision·runtime dtype는 absent로 유지한다. |
| 명시 decision | target 종류와 위 불변 식별, decision ID, usable/needs_changes/deferred, 필수 note, 검토 기준 version/내용 identity를 저장한다. 판정은 그 output/조건에만 한정한다. note가 같거나 기존 candidate usable이어도 새 명시 행위 없이 decision을 생성하지 않는다. |
| 기술 채택과 품질 | 기존 proposals.accept와 receipt는 technical output 선택이다. 품질 판정으로 소급 변경하지 않는다. usable도 자동 apply·모델 admission·파일 export·실제 미디어 적용을 허용하지 않으며 자동 적용 false 경계를 유지한다. |

검토 화면은 원 source, 불변 원 candidate, 저장 draft, 선택한 acceptance.output과 각각의 revision을 구분한다. 저장 참조 표시는 UI의 현재 독립 수리 범위다. plain DTO에 `verified` 그림자 bool을 만들거나 `historical.is_some()`으로 품질 상태를 생산하지 않는다. canonical inspect 당시의 검증 관측과 output quality decision은 별개 필드/근거 범위다.

## 저장·stale·tamper 경계

1. 제시 단계에서 canonical bound inspect 성공 응답과 명시 선택 receipt의 식별을 얻는다. 판정 저장 시 동일 canonical state·대상 output/hash/source/lineage를 서버 측에서 다시 확인하고 그 관측과 판정 게시 사이의 writer/CAS 경계를 SDK가 정의한다. 기존 동기 verifier가 단순 표시된 참조로 대체되지 않는다.
2. 초안 수정·다른 output 채택·source revision 불일치로 선택 대상이 바뀌면 조용히 latest로 재연결하지 않고 stale을 거부한다. 재조회 후 사용자가 명시적으로 대상을 다시 선택한다. 과거 불변 output의 재검토를 허용할지는 SDK가 별도 확정하며, 허용 시에도 exact historical target을 읽는 계약 없이 latest로 대체하지 않는다.
3. output bytes/typed payload·hash·target revision·source SHA·historical item 교환, 필요한 evidence 누락/변조는 판정 성공으로 반환하지 않는다. 실패로 추가 decision/publication이 없어야 하며 원 오류·cleanup 상태를 보존한다. hash는 무결성 대조이며 모든 저장소를 재작성하는 공격에 대한 서명/trust root를 새로 제공하는 것은 아니다.
4. 판정은 불변 append/read-back으로 관리한다. 동일 decision ID의 재요청은 동일 target·verdict·note·기준이면 기존 기록을 반환하는지, 불일치면 충돌시키는지 SDK가 명시한다. 현재 미지원 idempotency/revision context를 기존 reviews 포트에 임의 추가하지 않는다. 판정 변경은 원 기록 덮어쓰기 대신 새 명시 기록과 대상 관계로 남긴다.
5. fresh CLI read-back에서 decision ID/hash, exact target identity/hash/source/lineage, verdict/note/기준과 기록 메타데이터가 일치해야 한다. project capture/reopen에 포함할 범위는 SDK가 확정하며 포함 전에는 품질 판정이 project 저장에 보존된다고 주장하지 않는다.

## 검토자·시각과 기존 기록 호환

최소 후속 메타데이터는 사용자가 선언한 검토자 label/ID와 저장 adapter가 관측한 기록 시각·시각 기준이다. 선언된 검토자 이름은 인증된 신원 증명이 아니며 로컬 wall-clock 시각은 신뢰된 외부 시각 증명이 아니다. 요청자가 주장한 검토 시각이 필요하면 저장 관측 시각과 구분한다. OS 사용자와 실제 검토자가 같다고 자동 추정하지 않는다. Windows Hello·암호학적 본인인증을 필수 차단 흐름으로 추가하지 않는다.

현재 `RecordReviewDecision`/`ReviewDecision`에는 reviewer/time·edited target 필드가 없다. 기존 candidate package/decision SHA·bytes·해석·CLI 호출은 그대로 보존하고 일괄 재작성하지 않는다. 새 대상은 version된 별도 확장 경계로 설계하며 legacy 기록의 reviewer/time은 absent다. 기존 candidate decision을 편집 output decision으로 migration하거나 과거 이름·시각을 발명하지 않는다. 이전 실행 파일과 새 schema의 호환성은 실제 지원 범위를 SDK가 명시한다.

## derived package 대안과 품질 기준

편집 결과를 새 package의 candidate asset으로 고정하면 기존 reviews 포트를 재사용하는 방안은 검토할 수 있다. 그러나 현재 proposal→derived package 생성/계보 결합 포트는 없다. 현재 `ReviewResult.condition`은 `ModelCondition`이 필수이므로 human-edited output에 원 모델 조건을 그대로 붙이면 사람 수정을 모델 성능으로 집계할 위험이 있다. 사람 편집을 표현하는 명시 대상/출처 계약과 원 candidate 관계가 확정되기 전 이 대안을 구현 완료로 표시하거나 가짜 모델 ID/revision/precision을 만들지 않는다.

작업별 기준은 인수표의 correction 의미 보존·허용 편집, cards supporting cue·중복·길이, highlights 선정 목적·근거를 사용한다. 기존 candidate 자동검사 통과를 편집 output에 전이하지 않고 새 대상에 적합한 구조/근거 검사를 구분한다. 판정 usable만으로 자동 prerequisites 통과를 가정하지 않는다. 같은9표본의 수정본을 독립 새 모델 표본으로 집계하지 않는다.

## SDK 인수와 검증 순서

### SDK 최소4단계안의 AI 독립 검토 — 설계 인수 전

검토 대상은 [SDK 최소 단계 제안](accepted-output-review-sdk.md)이며 코드/시험 결과가 아니다. 현재 acceptance만 쓰기 대상으로 삼고 exact target+output hash, 단일 writer reservation, candidate API/bytes 보존, declared/observed 기록을 분리하는 방향에는 이견이 없다. 아래는 이 계획의 기존 요구를 구현 전에 구체화할 누락/미확정 항목이다. endpoint/schema를 여기서 확정하지 않는다.

| 우선 확인 | 구체적인 누락·위험과 SDK 결정 요청 |
|---|---|
| 채택과 실제 편집 여부 | 현재 accept는 생성 결과가 typed 유효하면 별도 edit 없이도 가능하다. 그러므로 accepted output을 모두 human-edited라고 이름 붙이면 실제 편집을 발명한다. 대상은 우선 accepted output으로 식별하고, 편집 이력/내용 변화/사람 행위의 무엇을 관측해 provenance를 표시할지 구분해야 한다. 내용 동일은 무편집 증거가 아니며 edit 호출도 실제 사람의 키 입력 증거가 아니다. 미관측은 unknown으로 둔다. |
| 원 candidate의 실제 lineage | 최소 target 목록은 source/historical reference를 담지만 generic proposal의 원 generated identity와 historical companion→원 review package/item/candidate asset 연결을 어떤 canonical 조회에서 확인하는지는 열려 있다. 원 generated.task/raw bytes identity를 immutable proposal 이력에서 해석할지 명시하고, 실제 연결된 package/item/asset만 표시해야 한다. companion digest를 package digest로 대체하거나 caller가 넣은 모델 조건으로 lineage를 채우지 않는다. |
| criteria content의 획득·범위 | version+content identity 결합은 타당하나 제공 경로가 미확정이다. 저장 adapter/application이 실제 criteria bytes와 task·편집 목적을 확인할 방법을 먼저 정해야 한다. caller hash만 비교하거나 원candidate 기준을 편집output에 무조건 상속하면 기준 바꿔치기/완화가 가능하다. 재제시·read-back에서 같은 기준 내용이 읽히고 판정의 평가 범위가 고정돼야 한다. |
| 판정 보존과 재검증 관측 | SDK가 열어 둔 read-back historical 재검증 범위는 저장된 판정의 존재/대상과 현재 evidence 가용성·검증 관측을 구분해 결정해야 한다. 이후 evidence가 없어져도 과거 기록을 미존재·다른 대상·새 성공으로 바꾸지 않는다. 원 오류와 읽기 지원 범위를 명시하고 검증 관측을 영구 true로 저장하지 않는다. |
| 모델 집계 분리 | ModelCondition 미복제만으로 집계 오염이 자동 방지되지는 않는다. 결과 소비자가 원candidate sample 판정과 accepted-output 판정을 별도 target 종류/집계 범위로 구분해야 한다. 사람이 수정했는지 unknown이거나 원candidate와 bytes가 같아도 새 모델 실행/독립 품질 표본/원모델 추가 성공으로 합산하지 않는다. |

declared reviewer/optional claimed time/adapter observed time 분리와 동일 재요청의 observed time 보존안에는 이견이 없다. 선언/관측의 출처를 read-back에서도 유지하면 되며 인증 신원·신뢰 시각·Windows Hello를 새 필수 흐름으로 요구하지 않는다. 스킬 `modootoday-free/ai-quality-plus/llm-output-eval-suite`의 case별 평가대상·기준 고정/집계 slice 구분을 이번 검토에 적용했다. 이 검토는 SDK의 P3 wake 구현과 병행 품질 구현을 요청하는 것이 아니다.

### 구현 인수용 사례 — 아직 실행하지 않음

아래 P와 Q, revision2/3/4는 설계용 상징값이며 실제 사용자 자료·실행 결과가 아니다. P의 draft2를 technical accept해 output3을 얻고, 그 typed output hash를 H3라 부른다. 오류의 wire code와 새 endpoint 이름은 SDK가 확정하되 거부/불변 결과를 먼저 고정한다.

| 사례 | 입력·경합 | 필수 결과 |
|---|---|---|
| exact output 정상 판정 | P/draft2/output3/H3/source/lineage/기준C를 제시한 뒤 명시 decision D 저장 | fresh read-back의 target·hash·기준·verdict/note가 일치. 원 generated/source/binding·technical receipt 불변. 자동 apply false |
| 같은 내용의 다른 대상 | Q가 P와 동일한 typed output H3를 갖지만 P의 decision을 Q에 요청 | output hash 동일만으로 대상 동일 처리 금지. proposal/revision/source/lineage까지 대조해 교환 거부 |
| 선택 뒤 수정 | P/output3 제시 후 별도 edit가 draft4를 게시하고 acceptance를 해제 | 원 선택을 latest draft4로 재연결하지 않음. 현재 대상에 대한 CAS 거부, 새 decision0. 과거 대상 지원은 별도 명시 계약 없으면 불가 |
| 기준 변경 | 동일 output3에 대해 제시된 C 대신 다른 기준C2로 저장 요청 | 기준 identity 불일치 탐지/재제시. C의 승인을 C2 승인으로 변환하지 않음 |
| hash·payload 불일치 | H3를 주장하면서 receipt.output의 한 값을 바꾸거나 다른 hash version을 전달 | canonical 재계산/지원 version 검사에서 거부. 원 기록과 publication 수 불변 |
| source·historical 교환 | 유효한 다른 source revision/SHA 또는 companion/item을 섞음 | 각각의 결합 검증에서 거부. storedref가 유효한 형식이라는 이유로 성공하지 않음 |
| 관측 뒤 evidence 손실 | 제시 때 inspect 성공 후 저장 전에 companion/source/candidate가 누락·변조됨 | 저장 경계에서 재검증해 거부. 이전 성공 관측으로 우회하지 않음. cleanup 미완료는 소유권/오류와 별도 보존 |
| decision 재요청·충돌 | D와 동일한 요청 재전송, 이어 같은D에 다른 target/note/verdict/기준 전달 | SDK가 확정한 중복 반환 계약을 검증하되 불일치 요청이 기존 decision을 덮거나 새 대상으로 재결합하지 않음 |
| declared reviewer·관측 시각 | 선언 label/ID, 선택적 claimed time 및 adapter observed time 저장 | 선언/관측의 종류가 read-back에 유지. claimed time을 저장 관측 시각으로 대체하지 않으며 인증된 신원·신뢰시각 표시0 |
| legacy 읽기·모델 집계 | 기존 package/item candidate decision을 새 구현에서 조회 | 원 bytes/SHA·candidate 대상·sample_only 유지, 없는 reviewer/time/edited target 발명0. accepted-output 판정은 편집 여부·내용 동일 여부와 무관하게 원 모델 후보의 추가 성공 수에 합산0 |

단위 시험은 순수 식별/hash/version·상태 전이를, 저장/SDK 시험은 atomic publish/CAS·오류 무게시를, fresh CLI는 실제 설정·영속·재조회 wire 계약을 맡는다. 같은 기능을 GUI에서 반복 검증하지 않는다. 이 표의 시험 구현·실행은 중앙 배정 이후이며 합성 입력을 사람 품질 승인으로 세지 않는다.

- SDK: 대상 표현·canonical hash version·CAS/원자 게시·legacy 호환·declared reviewer/observed time·read-back·오류 계약을 먼저 인수한다. AI는 모델 조건/품질 대상 구분을 검토하며 UI는 지원된 typed 결과를 표시한다.
- 예약 슬롯 이후 Rust domain/SDK 시험과 fresh CLI로 정상 판정/read-back, stale revision, 다른 proposal/receipt/hash/source/item 교환, evidence 누락/동일크기변조, 중복 decision 충돌, legacy candidate 기록 bytes 보존을 검증한다. GUI 기능 중복 시험은 하지 않는다.
- 품질 판정·technical acceptance·실제 apply/export·model/license/실정밀도 admission을 분리한 결과를 남긴다. verifier 설치/배포와 프로젝트 자체 license 선택은 별도 잔여다.

현재 수행은 계획 작성·source 계약 대조뿐이다. 새 코드/endpoint/Cargo/브라우저/원자료 복사/추론/다운로드/worker 빌드/사용자 판정 저장은0이다. 적용한 modoo.today `modootoday-free/ai-quality-plus/llm-output-eval-suite`의 본문 근거와 선택 판단은 연결된 인수표에 기록했다. task별 기준/실패 분리만 사용하며 새로운 평가 실행이나 LLM judge를 추가하지 않는다. 계획의 SDK 인수 전 구현을 시작하지 않고 전체 목표를 완료 처리하지 않는다.
