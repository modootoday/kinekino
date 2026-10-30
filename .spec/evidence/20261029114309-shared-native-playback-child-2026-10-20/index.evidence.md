---
id: evidence:kinekino/20261029114309
title: Shared NativeChild의 고정 producer 소비 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Shared NativeChild의 고정 producer 소비 검증

2026-10-20. [조립 계획](../../../plans/player/native-playback-child-assembly.md), [기존 router 검증](native-parent-router-2026-10-20.md). 증거 디렉터리 `runtime/quality-review/sdk-shared-native-child-r1/`.

## 입력과 최종 상태

기초는 C21+b0c+검증된 launcher2 제품 snapshot `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/public-p3-parent-r1`이다. Meari 1df→49443cf7e55585937d5a0314826f5642565cbed4 고정 overlay53과 SDK bridge2+child3만 반영했다. 최신 UI를 독립 a3 checkout 전체로 덮지 않았다. 자막 package7/API6·PNG export·미검증 WebBC는 섞지 않았다.

원 SDK5 pin6E5F8449439A1A9792767EC143867DA94E7A3FF9943062504EF95D2763435864/union58 pin2D2E092361E252DD981E018A311105AE3C799567BC2F4F3E0F78E229CD74ACBD 및 before58 원문/신규13 absent를 보존한다. source hash 유지 상태로 overlay58 mtime을 갱신한 뒤 시작했다. 이 gate에는 0-test 관측이 없으며 실제 개수를 확인했다.

default strict의 feature-off libtest에서 NativeChild 전용 bridge3가 dead_code로 101/65299ms였다. 원 입력/로그/receipt를 보존하고 해당3 메서드에 consumer와 동일한 `cfg(feature="native-proposal-window")`만 추가했다. r2는 router/mod.rs 속성3줄뿐이며 새 allow/expect·feature-on 본문 변경은 없다. feature-on 기존 회귀는 반복하지 않고 최종 strict에서 양 feature의 모든 target을 컴파일했다.

최종 SDK5는 `runtime/coordination/sdk-shared-native-child5-r2/pins.json` SHA256 **FC7BF70F94EF7B0A8705A3B1DD936B5E3CBB806AEEBC152D7774782445C13946**. repair.diff SHA06AD12546AC58622969DCEC1E355A5C0AD447B9466A082340E65AF548B0B8669. 최종 r2-union58.json SHA95618D9F71902A623C56FEBC67F4CF270952C4F1FF61FED31BC3062B64C6A267. 고정 producer overlay manifest SHA69808B828C0C959F4B05606100988687D4C9AC6CF42DDD8BD2F7D69F32C75F0B/patch SHAA99CFDAD452FE3B3A8454279712E70CAB39AF76AA076CD6723D4D6CBC48A1F13은 불변이다.

## 실제 결과

| 입력/검사 | 실제 결과 | 원 receipt 시간 |
| --- | --- | --- |
| SDK5 r1 native child | 7 passed | 165241ms |
| 같은 native router | 12 passed | 2434ms |
| 같은 native binding | 6 passed | 4784ms |
| 같은 dispatcher | 4 passed | 3357ms |
| 같은 launcher | active2 passed, 기존1 ignored | 15238ms |
| 최종 r2 root default strict | all-target -D warnings, 0 | 40640ms |
| 최종 r2 root native strict | all-target -D warnings, 0 | 49605ms |
| 최종 r2 root fmt | 0 | 3052ms |
| 최종 r2 UI product | 4 passed, 0 filtered/ignored | 531481ms |
| 최종 r2 bridge native strict | all-target -D warnings, 0 | 190034ms |
| 최종 r2 bridge fmt | 0 | 3684ms |

실제 행동 시험35개, 기존 ignored1은 통과 개수에서 제외한다. dependency 업그레이드가 있어 product4/launcher2를 집중 소비 회귀로 실행했다. product4는 카드/교정/문서수리/하이라이트 각각 실제 ProductWindow+Native Inspector+SDK/worker/callback을 거쳐 원문 비교→수정→save→accept→inspect/select→Deferred record/history를 수행했다. GUI View observer는 실행을 금지한다. 전체 root 시험을 실행했다고 주장하지 않는다.

## 소유와 관측

원 parent lease를 UI child가 유지하고, 자체 cleanup task를 만들지 않고 router의 원 RetirementHandle/task를 공유한다. 전체 CleanupStamp+reason+signal 검증, 같은 attempt 관측, 실제 no-inflight terminal 실패 후 유효한 새 request만 fresh wake로 retry한다. Pending과 ObservationUnavailable은 성공/terminal 실패로 합치지 않는다. unreadable owner 오류와 기존 Engine/Incomplete typed 진단을 분리 보존하고, 실제 원 joined 결과만 Complete로 연결한다. parent_lost는 고정 binding의 owner 확인+atomic revoke만 한다.

작성·실행된 alias Drop 시험은 coordinator가 machine을 계속 소유하는 경우이며 NativePlaybackChild 최종 Drop/NativeParentLease/OS surface 증거가 아니다. child의 직접 거부 시험은 Closed뿐이며 Full/Faulted는 없다. unknown late signal은 같은 attempt다. old-attempt/A-B-C는 별도 기존 router gate를 참조한다. CleanupMachine이 유일한 attempt driver라는 trusted assembly 전제를 유지한다. NativeParentLease와 OS parent↔route 결합은 constructor가 OS 인증하는 것이 아니다.

실행은 canonical runner SHA692C22751FAC7F9999F2D833571DAB6605AFD58E66711370494E62DD8A62C463/절대 cwd·manifest/offline locked j1/testthreads1/affinity3/BelowNormal/descendant job으로 직렬 수행했다. root native cache와 기존 bridge cache만 사용했다. 미확인 외부 cargo는 종료·간섭하거나 소유 실행으로 집계하지 않았다. modoo postcondition/replay 스킬의 코드 pin·실제 개수·직접 파일 상태 대조, rust-pro의 opaque ownership/borrow와 lint-and-validate의 수정 뒤 재검증을 적용했다.

## 마감 근거

- independent/frozen/snapshot SDK5 일치, union58 차이0, producer394 차이0, 보호527 차이0.
- 원 실패101 포함 receipt12 모두 terminal. 기록된 launcher/child24 PID 모두 부재, final-owned=[]이다.
- root5는 수정하지 않았고 root Meari gitlink는 1df44812c11e61b7e8c000324e35ce461b0a53b0 그대로다. root index/stage/commit/push 변경0. 별도 자막 root commit과 이 snapshot의 차이는 의도된 분리다.
- 실제 root libtest `.../native/debug/deps/kinekino_player-c38450de6d906d68.exe`, SHA74A6B916F1D775C1ED728C4591659DBCECD0E5E66B496C7B4BE3FDEC0D0E10B4/152290816bytes. r1 feature-on child/router/binding/dispatcher/launcher 실행 identity다.
- 실제 product binary `.../sdk-product-parent-consumer/debug/deps/output_review_product-bfc72df4bd9bb393.exe`, SHA4165A351F78C73596674B65DCADF588689AB37073668A03340B64917B2AC2BAA/64536064bytes. r2 producer 소비 product4 identity다. 절대 경로는 각 binary.json에 기록했다. 옛 덮어쓴 binary bytes나 새 player.exe 빌드를 보존했다고 주장하지 않는다.

heavy와 snapshot 소유권을 PM에 명시 반환한다. source 통합/커밋은 별도 PM 배정이다. 실제 OS parent/slot rect·clip·focus·HWND Destroy/main/mpv/미디어·수동 시각 인수는 미검증이다. router 모듈 한 곳의 기존 dead_code 유예는 main trusted assembly 연결 때 재검토한다. 모델·DPI·VM 보류를 유지한다.
