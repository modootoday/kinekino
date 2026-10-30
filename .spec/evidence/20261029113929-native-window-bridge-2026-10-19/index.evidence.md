---
id: evidence:kinekino/20261029113929
title: 제품 NativeWindowHost bridge r2 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 NativeWindowHost bridge r2 검증

2026-10-19 KST 실행, 2026-10-20 KST 정본 확정. [bridge 계획](../../../plans/player/presentation/native-window-bridge-gate.md)과 [admission 계약](../../player/presentation/shared/proposal-native-admission.md)의 새 consumer를 실제 Rust SDK·proposal store·Meari NativeHostRuntime으로 실행했다. 실제 OS 창·driver wake·키보드/IME·제품 root 등록은 포함하지 않는다.

## 입력과 첫 실패

`runtime/quality-review/player-ui-native-window-bridge-r2/gate-r1`에 r2 준비 owned5/source·전체 Rust/consumer/manifest 전후 pins와 원 로그/receipt를 보존했다. prepared-owned-diff는[]다. 새 SDK API SHA는 `96a9a83bbd1048918759a1b9c24775bf72b7944c452756235062f9115f92e51a`이며 이전 factory/포인터 시점 API를 재사용한 것이 아니다. Meari/SDK 담당의 명시 동결 뒤 실행했다.

검증된 runner692c227…c463의 affinity3/2CPU/BelowNormal/descendants job·기존 target/cache·offline/locked/-j1/RUST_TEST_THREADS1을 사용했다. 최초 선택은75719ms/101, document_factory17 중15pass/2fail/0ignored였고 cargo 기본 fail-fast로 panel2는 미실행이었다. capacity accepted2≠1 및 단건 초과 accepted1≠Rejected의 원문과 before-budget-fix source3개를 보존했다.

원인은 factory 초기 큐의 budget 기본8/8192와 bridge에 별도로 넘긴1/4096이 달라 후속 재투영에서만 새 예산이 적용된 배선이었다. assert를 완화하지 않고 factory가 실제 사용한 PanelBudget 하나를 private Mounted→Parts→Bridge로 이전했다. from_factory의 독립 budget 입력을 제거하고 시험은 factory input에서 예산을 선택한다. 외부 SDK/Meari source 변경은 없다.

수리 후 선택 --no-fail-fast는23050ms/0, factory/bridge17+panel2=19pass/0fail/0ignored였다. 이후 같은 테스트 수 안에서 재투영 다음 세대의 entries1/단건4096 예산 불변 검사도 추가하고 전체 회귀에 포함했다. 최종 전체 --no-fail-fast는119312ms/0·46passed/0failed/0ignored, strict --all-targets -- -D warnings는21757ms/0, fmt --check는310ms/0이다. 이 전체/strict/fmt는 동일 최종 source를 소비했다. 별도 lint 수리는 없었다.

46개는 callback1/factory+bridge17/R3 Document1/Review Native1/CardHL2/panel2/실제 SDK assets3/view14/원본 intake1/sdk_files4다. 첫 실패는 지우거나 expected-success로 바꾸지 않았고 새 단일 budget 배선과 재투영 후 불변 검사를 실행해 인수했다.

gate-r1/final-source5·final-owned-pins.json inventory SHA는 `4b2c97b027d611b92199e1b0c85dea9ab5a1341970ad2ac038a99581564c9c50`이다. prepared r2 `c609a23b…d9b4f`와 최종 수리 입력을 구별한다. shared Rust delta4행은 owned factory/bridge의 두 파일 전후이며 외부 SDK/Meari delta0·manifest delta0이다. owned delta에는 위 두 파일과 테스트 source의 변화만 있다. 원 source/수리 diff·전후 pins·소유 프로세스[]를 보존했으며 모든 runner receipt는 terminal=true다. 새 GUI/browser는0이다.

## 실제 검증 범위

NativeWindowHost의 admit는 실제 session callback enqueue만 수행하고 after_dispatch에서 registry/doc guard 밖에 SDK/local draft를 drain한다. exact host stamp/실제 source scope를 확인하고 suffix가 남아 있으면 export 교체를 미룬다. try_reproject의 동기 receipt를 반환해 driver가 두 번 적용하지 않는다. 명시 edit/save/accept·다음 Completion의 비재실행, foreign host·항목 거부, 마지막 source 유지와 다음 callback, 종료 veto·parent 재시도·forced incomplete·닫힌 factory panel 재개방 거부를 선택 실행에서 확인했다.

단건 초과는 validation에서 영구 거부하고 controlled props를 복원한다. 5000bytes 두 field/8192bytes 합산은 첫 accepted1/remaining1 뒤 drain과 재입장 accepted2/remaining0으로 구별했다. 같은 adversarial synthetic batch의 옛 field generation은 별도 Stale 거부되며 이를 모든 multi-field 편집 성공으로 확대하지 않는다.

공유 registry limit1 시험은 실제 factory 한도와 같은 private 값을 사용한다. 포화 중 opaque foreign handle/자체 초과 text는 검증에서 거부하고, 정상 Save만 suffix로 남긴다. 실제 기술 Pending adapter의 CancelRequested 뒤에도 accepted0/SDK counter0, terminal Cancelled 뒤 명시 Completion에서 Save1회 admission과 SDK edit1/inspect1을 확인했다. 다음 Completion의 호출 counter는 그대로다. counter는 제품 SDK adapter의 ticket/context factory에서 관측하며 snapshot 불변만으로 호출0을 추론하지 않는다.

parent pending의 취소 요청은 완료가 아니고 terminal 전 close는 완료되지 않는다. NativeRuntime이 제공한 callback revoke receipt와 UI thread consumer checkpoint를 보존하며 session 성공 뒤 parent만 재시도한다. host/handle은 Rc를 포함해 같은 UI thread에 남고 checkpoint에 Send/Sync를 요구하지 않는다. 강제 OS destroy 요청은 best effort 뒤에도 ForcedIncomplete다. 정상 UI QueueCallbackAdapter는 Immediate이며 방어 unexpected UI Pending 분기를 정상 fixture 성공으로 꾸미지 않았다.

## 남은 경계

입력은 공개 document API의 기술 입력이며 실제 OS 키보드/IME가 아니다. terminal 뒤 Completion을 명시 호출한 headless 수직이며 실제 parent backend→NativeWindowWake→OS driver 연결은 후속이다. SDK parent lease의 root 등록·실제 NativeWindowDriver/제품 창·eframe loop/공유 GL·fresh CLI/project/provenance·전체13화면 시각·커서/음소거/청취·seek stall은 각각 별도 인수를 유지한다.
