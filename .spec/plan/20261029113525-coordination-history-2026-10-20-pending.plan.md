---
id: plan:kinekino/20261029113525
title: Pending 구현 조율 이력 — 2026-10-20
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Pending 구현 조율 이력 — 2026-10-20

이 문서는 SDK S1 최종 인수 직전 원문을 보존한 이력이다. 현재 담당·실행 배정·인수 상태는 [현재 조율](current-coordination.md)을 따른다. 아래 대기/미실행 표현을 최신 상태로 사용하지 않는다.

2026-10-20. 구현·관측·미검증을 구별한다. 제품 전체 완료와 사람 품질 승인은 아직 아니다. 누적 원문은 [이번 구현 이력](coordination-history-2026-10-20-provenance.md), [이전 인계 이력](coordination-history-2026-10-20.md), [회차·커서 이력](transition-cursor-coordination-history-2026-10-19.md)에 보존했다.

## 담당과 현재 작업

| 담당 | 인수한 근거 | 현재 책임 |
| --- | --- | --- |
| PM | 원 receipt·지원 범위·소유 정리 대조 | 공유 파일·자원·다음 인계 관리 |
| SDK | historical 및 S1 기존 집중 인수·최신 source 별도 보존 | 최신 실행/권한 폐기/재등록·실제5cap/actual9 bound3task 검증. 현재 단일 heavy |
| UI | bootstrap 전체61·strict/fmt·inventory54d53203…a219·최종 반환 인수 | 범용 typed async codec/위임·presentation 소비 소스 구현. 실행 없음 |
| Meari | Native17/13/41/doc1(E0382)·strict/fmt·최종 BAA63732…9487C/delta[]/프로세스[] 인수 | headless 문서 마무리·source 동결. OS ignored 실행 없음 |
| AI | 기존 모델 결과·provenance·품질 계약 조사 | 새 추론/다운로드 보류. 제품 연결 후 최종 품질 검토 |

Rust 통일·단일책임·계층 구조를 유지한다. GUI/CLI는 같은 SDK/application/domain을 사용하고 기능 검증은 CLI가 우선이다. 부모/기존 파일을 여러 담당이 동시에 수정하지 않는다. 실행·공유 입력 반환은 terminal·소유 정리·명시 인계 뒤 인정한다.

## 현재 배정: 공통 Pending 실행과 Native driver 연결

UI 인계 통신 잔여: 대기 중 UI thread로 보낸 후속 메시지는 앱의 `Cannot steer conversation ... without an active turn id`로 세 번 실패했다. 메시지 수신/새 배정 성공으로 집계하지 않는다. UI는 직전 성공 인계에서15파일 후보 동결 및 SDK최종pin/배정 대기를 명시했다. 사용자에게 해당 UI 세션에 한 번 입력해 활성화해 줄 것을 요청했으며 SDK 검증은 계속 진행한다. 새 세션 생성이나 UI 소유 구현을 PM이 임의 인수하지 않는다.

현재 단일 heavy는 SDK로 돌아갔다. Meari default141865ms/0·17pass, host23913ms/0·13pass, window lib73026ms/0·41pass, strict54295ms/0, doc-r29926ms/0·실제 E0382 진단1pass, fmt680ms/0을 PM이 원문 대조했다. 최초 doc53354ms/0도 보존하며 r2 변경은 preparation doc의 기대 오류 명시뿐이다. source-after/source-r2 inventory BAA637328CECE7F6C05E4DE0B3562C863DFB0B3727123E9779ACFBB25369487C, 최종 delta[]·소유 프로세스[]·manifest/lock 불변과 종료된 gate를 확인해 인수했다. SDK는 최신 source-r3→focused/실제 정책→고정 actual9/3task→영향 검증을 수행한다. UI15파일 후보 b16a1f59…e9e06은 아직 미컴파일이며 generic5/Native/Core/manifest와 함께 소비 전 동결한다. 실제 OS child/wake·IME·전체 시각과 S2는 잔여다. 아래 Meari 실행 배정은 이 반환 이전 이력이다.

최신 실행 배정은 Meari다. SDK가 실행 중 gate 없음·관련 프로세스0을 명시 반환했고 PM도 cargo/rustc/resource-runner 조회 결과 없음을 확인했다. Native checkpoint의 승인14파일·manifest/lock 불변 입력에서 default lifecycle/host→window lib/doc→window all-targets strict/fmt를 전용 target·중앙 resource-runner로 검증한다. 실제 OS ignored 시험은 아직 배정하지 않았다. SDK/UI는 source/시험 작성만 병행하고 Native/Core/manifest를 수정하지 않는다. 아래 SDK heavy 배정 기록은 이 반환 이전 이력이며 최신 변경의 회귀는 아직 미실행이다.

UI bootstrap의 최종 반환을 인수하고 [S1 계획](historical-import-pending-integration.md)의 구현을 배정했다. SDK는 application 실행 수락/worker/terminal/control과 root assembly, UI는 player-ui-plugins의 범용 async port/codec·cancel/shutdown 위임과 presentation 소비를 소유한다. shared Rust 계약·정확 파일 경계를 먼저 상호 대조하고 패키지의 root 역의존·정책/두 번째 SDK 복제를 금지한다. 대상은 create/inspect/edit/accept/import5개이며 project 동기 잔여를 숨기지 않는다. queued 취소와 in-flight Requested 뒤 실제 결과를 구별하고 S2 강한 취소/게시 reservation·S3 실제 OS wake는 별도 인수다. HP01~HP10은 Meari 감사의 P01~P10과 구분·매핑한다.

단일 heavy는 SDK에 배정했다. UI/Meari는 소스/시험 작성만 병행하고 cargo/GUI 프로세스를 실행하지 않는다. Meari는 승인 Native window/driver/lifecycle/preparation 경계에서 실제 parent·Ready/geometry/loss/removal·child-first barrier를 연결한다. 공통 coordinator/native_host 변경이 필요하면 먼저 PM과 소비 범위를 조율한다. Core/다른 crate/모든 manifest·lock/제품 파일은 변경하지 않는다. SDK root의 Native 비소비 구간을 활용하고 Native/consumer gate 직전에는 해당 소유자에게 해석 가능한 checkpoint와 freeze를 요청한다.

### S1 계약과 추가 소유 범위

후속 factory 실패 검토에서 parent.close의 worker 종료가 같은 owner 재조립을 막는 gap을 확인했다. SDK·UI에 문서 registration/독점 attachment의 quiescent 해제와 ExecutionOwner 최종 close/join을 분리하는 수정을 배정했다. Pending/미수락 terminal/ACK 잔여는 원 scope·부분 owner와 함께 Busy로 남긴다. 해제만으로 제품 정리 완료를 주장하지 않고 정상 close의 단일 부모가 최종 worker 종료까지 책임진다. Immediate 자동 정리는 보존하며 exact SDK port·공유 정본·재조립/정상 종료 회귀는 담당 합의 및 구현·검증 후 인수한다. 아직 구현 통과가 아니다.

초기 loading 순서는 trusted canonical/명시 선택 검증→immutable 준비 상태의 소유 인수→plain operation ACK→같은 ExecutionOwner의 actual document scope attachment/등록으로 고정한다. full mount 안의 등록 전에 operation ledger가 비어야 하므로 full mount 이후 ACK하는 중간 제안은 대체됐다. ACK 실패는 mount 전에 ACK만 재시도하고, mount 실패는 보존된 초기 결과로 조립만 재시도한다. SDK submit·inspect 재실행으로 복구하지 않으며 이 초기 수락은 renderer·시각·사람 품질 ACK가 아니다.

SDK worker-r2는54988ms/0·6pass/0ignored, generic-first는1295ms/0·10pass/0ignored로 PM이 원 receipt와 전체 대상 시험 목록을 확인했다. 이는 fake SDK scheduling 및 실제 Core registry/controlled port의 집중 증거이며 실제 제품 assembly·세 task·OS 연결은 후속이다.

UI는 presentation 미등록3파일 checkpoint48c51e3c…8c48/시험source5개를 인계했다. 다음 source 배정은 같은 [UI 계획](presentation/typed-async-sdk-s1.md)의 document_controller 조립, queue 공통 drain, session pending retaining, bridge 외부 pump/실제 commit, 교정 공통 투영 및 factory inspect-free 내부 mount 추출이다. 기존 Immediate 호환을 보존하며 generic5파일/SDK/Native/manifest는 동결한다. 최초 factory inspect가 ModuleProgram 생성 전에 동기 실행되는 gap은 SDK plain execution의 trusted loading/원 terminal ACK/같은 owner handoff 계약으로 SDK·UI가 합의하며 가짜 scope나 두 번째 worker로 우회하지 않는다. parent export·consumer 실행은 아직 배정 전이다.

최신 공유 입력은 UI export 포함 다섯 파일 checkpoint `runtime/coordination/ui-typed-async-s1/plugin-checkpoint.json`(SHA-256 `f469a872ece03c8c41ab4f2a534658a97cba0e94de2a7aa2227c7f05cdea8d2b`)이다. PM이 다섯 실제 파일 해시와 export를 대조했다. caller `AsyncCodecBudget`·checked 총복사 예약·Failed message/details envelope 사전 측정을 보완했으며 SDK가 이 입력을 소비해 generic 시험·assembly 연결을 검증한다. 다섯 파일은 해당 gate 종료까지 동결하고 UI는 별도 미등록 presentation 소스를 계속 작성한다. 아래 미등록 네 파일 관측은 이 checkpoint 이전 이력이다. SDK의 새 EncodingMeasurement 구분과 serialization 실패 fixture도 첫 5시험 이후 변경이므로 다시 검증해야 한다.

첫 application worker 집중 검사는 `runtime/quality-review/sdk-execution-s1-r1/worker-focused-first.*`에서 56725ms/0·5pass/0ignored, fmt-first는2857ms/0으로 PM이 원 receipt/stdout/stderr를 확인했다. fake ApplicationClient로 queue/실행 취소 구분·ACK 전 보존·포화·foreign owner·알림 실패·Drop 뒤 retained handle·큰 결과 fault를 검증한 범위다. 실제 제품 다섯 capability·assembly·CLI 재검증·strict·전체 영향 검사는 아직 완료로 집계하지 않는다.

공유 계약은 [SDK 실행 계약](../../sots/player/application/api/sdk-execution.md)의 actual-ticket `AsyncSdkPort<I,O>`와 비소비 `Arc<Result<O,ApiError>>`로 합의했다. SDK assembly가 operation mapping과 typed 변환 캐시를 소유하고 UI는 actual ticket 및 accepted-but-unacknowledged 상태를 보존한다. queued 취소의 canonical Cancelled와 실행 중 Requested를 구별한다. invalid/oversize terminal은 원 결과와 Core pending을 보존하고 close Busy로 남으며 명시 복구는 잔여다. generic 신규 네 파일은 직접 rustfmt 파싱 checkpoint를 UI가 인계했으나 아직 미등록·미컴파일이며 실행 성공으로 집계하지 않는다.

Meari 추가 내부 변경은 `application/window_lifecycle/{authority,coordinator}.rs` 및 필요한 `state.rs`에 한정 승인했다. 생성/활성/현재 cleanup attempt의 scoped 권한과 callback 없는 Drop revoke를 구분하며 공개 raw getter와 Drop 중 SDK/child callback을 금지한다. 기존 lifecycle 회귀와 foreign/stale/reentrant 부정 시험을 준비하고 Native 소비 전 parseable checkpoint를 인계한다. SDK 첫 gate는 Native 비소비 application 범위이며 단일 heavy 배정을 유지한다.

## 최근 인수: UI bootstrap 등록과 소비 검증

SDK 최종 반환을 인수하고 실행 슬롯을 UI에 배정했다. 준비 bootstrap 두 파일·기존 proposals/mod.rs의 최소 등록과 필요한 별도 consumer 시험만 변경한다. canonical document3/binding1·legacy None·잘못된 조합·품질 구분과 실제 원본 ModuleProgram/factory 소비를 인수한다. SDK/Native/plugin/모든 manifest·lock/기존 승인 fixture는 동결한다. D consumer 캐시에서 집중→영향 전체/strict/fmt를 실행하고 root 소비 경계가 바뀌면 필요한 scoped 검사를 명시한다. 실제 GUI/브라우저 실행은 없다.

최종 결과는 집중199624ms/0·11순수+2소비 표본, 최종 consumer37486ms/0·61pass/0ignored·strict16611ms/0·fmt349ms/0이다. 경고는 공개 test module과 실제 factory budget assertion으로 최소 보완했고 억제/기존 factory 수정은 없었다. inventory54d53203…a219의 소유4파일·외부 SDK/manifest 불변과 명시 반환을 PM이 인수했다. [실행 근거](../../sots/verification/presentation/bootstrap-preparation-2026-10-20.md). 실제 제품 런처 미연결·correction/repair 새 bootstrap 소비/독립 품질 표시·원 후보 decision 조회·import/Pending·OS/전체 디자인 잔여는 유지한다.

## 최근 인수: historical provenance 영속 연결

[구현 계획](historical-provenance-product-checkpoint.md), [API 계약](../../sots/player/application/api/historical-provenance.md), [실행 정본](../../sots/verification/application/historical-provenance-product-2026-10-20.md)을 따른다. 기존 actual9 원문/결과만 사용하며 모델 재실행은 없다.

새 historical document3/binding1은 source snapshot1/2와 구분하며 binding을 단일 immutable proposal publication에 포함한다. 일반 기존 proposal의 None 필드는 생략해 bytes/capture SHA를 유지한다. 단일 binding 제거는 거부하되 version·이력·checksum 전체 재작성에 대한 서명/trust root를 주장하지 않는다. inspect/edit/accept 재요청·project live/saved/reopen에서 실제 증거를 재검증한다.

제품은 tools source에 의존하지 않는다. 명시 경로·SHA·workspace·package로 고정한 Rust verifier 프로세스 어댑터를 사용한다. 설치/설정이 없으면 지원으로 표시하지 않는다. packages library 승격과의 비교는 구현 계획에 있다. stdout/stderr 한도·deadline과 실제 cleanup 완료를 구별하며 미완료 owner를 보존한다.

PM이 직접 확인한 실행은 다음과 같다.

- reader actual9: 4069ms/0·정확1시험 안의9개 재구성. reader owner4pass/2ignored·strict37364ms/0. fake 실패 주입과 실제 OS 권한 실패를 구별한다.
- binding check29934ms/0·집중60510ms/0(root8pass/2ignored 및 별도 jobs 필터 시험). legacy bytes/capture SHA와 실제 파일 저장 회귀를 포함하며 fixture reader 범위를 유지한다.
- 실제 CLI cli-first107497ms/0·정확1시험/본문31.11s. 9import/fresh inspect와 correction/cards/highlights 각각 기술 edit·foreign/stale 거부·명시 accept/retry·자동 적용false·원문/binding 불변·project capture/reopen, scratch 누락/변조/교환/부분 게시 거부를 확인했다. 고정CLI8065a232…c6266과 별도 PID/reaped 기록을 대조했다. 사람 품질 승인·실제 카드 렌더/영상 적용은 아니다.
- binding strict35531ms/0·최종 fmt1411ms/0·root full255986ms/0(전체396pass/22ignored, root lib296pass/4ignored)·consumer full406266ms/0(47pass/0ignored)·consumer strict45445ms/0·fmt437ms/0·bins3122ms/0을 원문 확인했다. 최종 GUI b7cb99ee…8d9f0·CLI8065a232…c6266·SDK inventory b2c92d62…cdd35와 승인 fixture의 None4줄만 변경된 diff·manifest/lock불변·소유[] 및 명시 반환도 PM이 인수했다. 별도 실제 CLI/reader 성공과 일반 회귀의 ignored를 구별한다.

## 공유 source 경계

SDK는 domain binding·application import/API/CLI·실제 reader/store·project capture와 Rust harness를 담당한다. UI 소유 `tools/player-ui-sdk-bridge/tests/{panel_preparation,proposal_view}.rs`와 `src/presentation/studio/proposals/tests.rs`는 StoredProposal literal의 None 추가만 SDK에 승인했다. 기존 assert/데이터/예산 변경은 없다.

UI bootstrap source는 `src/presentation/web_ui_adapter/proposals/bootstrap.rs`와 `bootstrap/tests.rs`이며 consumer에 등록·검증됐다. canonical snapshot·명시 선택을 초기 props로 변환하며 정책·SDK 호출·숨은 첫 항목 선택은 없다. 최종 inventory54d53203…a219를 기준으로 하고 준비 당시 미등록 해시는 이력에 보존했다. root presentation은 adapter를 아직 등록하지 않으므로 제품 런처 연결 성공으로 집계하지 않는다.

Meari 공통 Native 인수 inventory는20e21d11…c4bc1이다. lifecycle17·기존 headless host13·strict-r210476ms/0·fmt-r2529ms/0, 최종 delta[]/소유[]를 인수했다. 최초 strict의 fixture type_complexity는 역할 타입별칭만 보완했다. [실행 근거](../../../packages/meari/.spec/evidence/verification/window-lifecycle-common-2026-10-20.md). 이전 consumer 반환 뒤 현재 driver 소스를 배정했으며 새 변경은 아직 이 결과의 지원 범위가 아니다.

## 다음 제품 연결과 잔여

1. 현재 SDK 전체/consumer/최종 인수와 새 저장 형식·오류·호환 범위를 확정한다. 같은 입력의 불필요 반복 전체 검사는 하지 않는다.
2. UI bootstrap 등록·시험과 제품 명시 선택 연결을 인수한다. [historical UI 연결](presentation/historical-import-ui-connection.md)에 document3/binding1 회귀·정확한 품질 표시·추가 import capability/action·동기 reader 비동기 실행의 잔여를 구분했다. 현재 Native proposal plugin은 기존4cap이며 새 import 지원으로 집계하지 않는다. [기능/GUI 인수표](presentation/proposal-product-registration.md)는 기능을 CLI에, 실제 wake/입력/시각을 GUI에 둔다.
3. [단일 창 우선 경로](../../adrs/player/presentation/native-top-level-window.md)는 Meari 최상위 창+범용 child lease의 검증이며 최종 backend/기본 런처 전환은 미확정이다. [최소 수직](presentation/native-window-minimal-vertical.md), [Meari 수명 계약](../../../packages/meari/.spec/sots/integration/native-window-lifecycle.md)을 연결한다. driver 발급 single-use pre-ready 준비객체→동일 coordinator/waker/runtime→consume attach를 우선 설계한다. sink 설치 전 Ready/child 노출 금지, early completion 보존, foreign/late binding 거부와 부분 실패 owner 반환이 필요하다. 아직 실제 driver/OS child 구현·W01~13 전체 인수는 없다.
4. 동기 verifier를 Immediate UI-thread adapter에 연결하면 지연될 수 있다. [Pending 경계 감사](../../../packages/meari/.spec/evidence/reviews/plugin-pending-sdk-boundary-2026-10-20.md)에서 Core terminal 검증/Closing drain 재사용과 현 TypedSdkPlugin의 무조건 AlreadyTerminal 취소·동기 relay 제약을 구별했다. Pending binding만 추가해 실제 worker 지원으로 표시하지 않는다. 공통 scheduling/typed Pending·실제 terminal wake는 미구현이며 Rc document/registry/GL을 worker로 보내지 않는다. 기존 방어 시험을 실제 provider 비동기 성공으로 세지 않는다.
5. CLI 기능 뒤 전체13화면/하위탭·React routing/플러그인·SVG/폰트/토큰·상태 일관성을 실제 실행과 승인 시안 확대1:1로 검토한다. 조작 검토는 사용자 최신 지시에 따라 메인 모니터·소유 앱1개로 한다.
6. 실제 provider 오디오 전환·청취, 첫 seek stall 원인, 제작/export/apply·모델 기능·화자 분리 등 기존 전체 범위는 각 기능별 계획에 남는다. 품질 검토는 제품 연결 뒤 AI 담당과 진행한다. 원 candidate의 verdict를 수정 output에 상속하지 않는다.

## 자원과 최근 인수 링크

[자원 예산](resource-budget.md)의 단일 process family·affinity3/2CPU·BelowNormal·descendants Job·cargo -j1/offline/locked·시험thread1을 유지한다. root는 C의 프로젝트 native cache, consumer는 `runtime/build-cache/sdk-product-parent-consumer`, Meari는 자체 `packages/meari/target`을 사용한다. 다른 workspace/feature graph의 해시 없는 산출물을 섞지 않는다. formatter도 다른 담당 dependency 전체를 임의 순회하지 않는다.

사용자 ChatGPT/Terminal/다른 앱과 기존 cache/registry는 종료·삭제하지 않는다. Pages·VM·DPI·실제 시스템 절전·새 장시간 재생 시험·AI 추론/다운로드는 보류한다. 자원 대기를 source 작업 중단이나 goal 완료로 확대하지 않는다.

이전 인수는 [SDK parent 등록](../../sots/verification/application/sdk-product-parent-2026-10-20.md), [UI bridge](../../sots/verification/presentation/native-window-bridge-2026-10-19.md), [실제 WebView 오디오](../../sots/verification/application/native-audio-webview-2026-10-20.md), [커서](../../sots/verification/application/native-pointer-cli-2026-10-19.md)를 참조한다. mpv 동봉·adapter 분리·libmpv/source submodule은 [권장안](mpv-package-boundary.md)이며 채택/배포 완료가 아니다.
