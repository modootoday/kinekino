---
id: plan:kinekino/20261029113911
title: Native proposal UI canonical façade 등록
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native proposal UI canonical façade 등록

2026-10-20 PM 제품 등록2단계 배정이다. SDK 등록1단계의 root5파일 inventory `36914414b13f3c05d0b1ee32f3983203dee533bb85071cf9d7db3201b7545a8b`를 실제 재hash해 차이0으로 인수했다. UI 소비19 기준 `26335921b555b8732b3609f9d22a7ad7dda93b9f5e3d4998111381818774f35c`의 구현 본문을 유지하며 부모 선언·consumer import·feature 경계만 등록한다.

## 경계와 정확 대상

`src/presentation/mod.rs` → Windows/native-proposal-window 조건부 public web_ui_adapter, 신규 `web_ui_adapter/mod.rs` → proposals, `proposals/mod.rs` → 같은 조건의 document_factory/initial_panel/window_bridge/window_startup을 등록한다. application/SDK/Native 정책을 옮기지 않는다. root가 tool을 import하지 않는다.

consumer feature `native-proposal-window`는 root의 같은 feature를 전달한다. feature-on의 proposal_view와 integration factory/initial/bridge/startup/panel_model·notifier/assembly 조립은 root의 동일 모듈을 re-export하며 별도 path 정의를 함께 컴파일하지 않는다. feature-off는 기존 독립 path 소비를 유지한다. 기존 subtitle_conversion probe는 이번 Native canonical 등록 범위 밖의 독립 소스 경계를 유지한다.

소유 파일은 presentation 부모3개와 consumer Cargo/lock/lib 및 관련 integration import·공개 행동 회귀뿐이다. root Cargo/lock/lib/infrastructure/assembly/main/launch와 UI19 구현 본문·CSS/SEG/CLIP/Native/Core/SDK core는 동결한다. 이전 파일10개는 `runtime/quality-review/product-native-registration-step2-r1/before/`에 보존했다.

## 검증과 비공개 필드 대체

처음 consumer on compile은 locked lock 갱신 필요로657ms/exit101이었다. 원로그를 보존하고 consumer cwd의 offline workspace resolution1회를 수행했다. 새 package0이며 lock diff는 root kinekino-player의 meari-native 의존 간선1줄이다. 이후 locked 검증을 재개한다.

bootstrap_preparation의 PanelHostParts.registry_pending/budget 필드는 pub(super)이므로 외부 root consumer에 공개하지 않는다. feature-off의 기존 필드3단언을 유지한다. on/off 공개 행동 매핑은 다음과 같다.

| 기존 단언 | 공개 행동 근거 |
|---|---|
| registry_pending=8 | canonical_registry_eight_pending_slots_resume_without_duplicate_save: 실제 registry pending8을 점유해 Save admission을 막고 terminal 이후 한 번 재개. 기존 limit1 회귀도 유지 |
| budget.entries=8 | canonical_eight_entry_budget_preserves_ninth_callback_until_resume: 첫8개 수용·9번째 suffix 보존·ResumePending 뒤 commit |
| budget.text_bytes=8192 | 기존 native_host_bridge_aggregate_text_bytes_preserve_suffix_until_queue_drain: 5000+5000 bytes의 합산 초과를8192한도로 나눠 drain/재개. 기존 single oversized 거절도 유지 |

단순 private field 읽기 제외를 회귀 동등으로 간주하지 않는다. 새 행동 시험은 실행 결과를 별도 검증 정본에 기록한다. consumer feature-on에서 사라지는 중복 모듈 단위 시험은 root feature-on lib test에서 소비하고 통과 수를 신규 성과로 합산하지 않는다.

중앙 runner/단일 heavy, offline/locked/-j1/affinity3/BelowNormal Job/RUST_TEST_THREADS1을 유지한다. root는 SDK가 인계한 기존 `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/native`, consumer는 기존 `runtime/build-cache/sdk-product-parent-consumer`를 재사용한다. on compile→root on lib·소비 on/off 회귀와 strict/fmt→canonical 실제3task·종료/회복 OS 영향 검증→최종 pin/owned[] 인계 순서다.

## 후속 launch helper 인계

현재 UI production entry는 신뢰한 factory 입력으로 명시한다. Cards=`studio/cards/proposal-panel.tsx`, Highlights=`studio/clips/highlight-proposal-panel.tsx`, Correction=`studio/subtitles/correction-panel.tsx`와 공유 proposal-source/proposal-actions/text-comparison inventory를 사용한다. 현재 BootstrapPanel::Correction.entry()의 verification 경로를 production 기본값으로 쓰지 않는다.

다음 UI helper는 application의 plain launch selection을 BootstrapSelection으로 한 번 변환하고, 고정 product entry/inventory·labels·budget·asset 입력으로 WindowStartupInput을 구성해야 한다. 기존 SDK client/context/ExecutionOwner/immutable preparation을 그대로 이전하고 initial 준비 실패·입력 실패·NativeRunOutcome의 반환 owner를 보존한다. 이번에는 helper/main/launch 구현을 시작하지 않는다. DocumentRepair의 미선택 None 준비와 Some(source_ref) 초기 미지원도 임의 fallback 없이 인계한다.

### SDK DTO와 DocumentRepair 초기 준비의 차이 — 후속 배정 전 관측

2026-10-20 PM 인계에 따르면 SDK는 DocumentRepairSelected DTO의 full DocumentTextRef roundtrip을 구현했고, bootstrap::project_initial은 Some(reference)를 찾아 props를 생성한다. 반면 initial_panel의 초기 prepare match는 DocumentRepair { source_ref: Some(_) }를 TargetMismatch로 거부한다. 이는 DTO의 기능 범위를 축소할 근거가 아니라 후속 UI 초기 준비/공통 correction projection을 맞춰야 하는 구현 공백이다. 이 문구는 PM이 인계한 현재 관측이며 SDK 최종 DTO pin 인수·새 UI 구현/시험 통과가 아니다.

SDK 최종 DTO 인계와 정확 파일 해제 후 selection의 full reference를 검증해 초기 공통 correction projection에 전달하고, Some의 명시 선택과 None의 명시 미선택을 모두 지원해야 한다. 항목0/첫 source/None fallback으로 Some을 대체하거나 DTO를 축소하지 않는다. 초기 props와 후속 live props가 같은 선택·원문 ID/전체 reference·revision/SHA 및 저장 정책을 유지하는지 UI 구현과 함께 검증한다. 현재 source/cargo/GUI는 동결하며 이 계획 보완만 수행했다.

## 등록2단계 인수·슬롯 반환

[최종 검증 정본](../../../sots/verification/presentation/product-native-registration-step2-2026-10-20.md)에 원 실패·수리·단일 heavy 위반/회수와 이후 순차 검증을 분리했다. root on 제품lib347/workspace lib395, root off 전체426, canonical consumer on55/off93, root 및 consumer on/off strict·두 fmt와 실제 canonical OS9가 통과했다. 중복 UI24/initial7*2의 root 고유31 이동과 private3단언의 공개8/8/8192 행동 대체를 이름/결과로 연결했다.

최종11 inventory SHA256 `215b94167313293829674568b504154ec4349a27a67de9e92f446d9df4d73332`, 최종source/보호입력 차이0, owned[]를 확인하고 단일 heavy를 PM에 반환한다. root main/launch와 UI19 구현 본문은 승인된 registration/import 외 변경하지 않았다. 다음 정확 배정 전 새 제품 helper나 launch 구현을 시작하지 않는다. 전체 제품·시각13화면·물리 입력 목표는 미완료다.
