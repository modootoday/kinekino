---
id: evidence:kinekino/20261029113834
title: Default chooser D 실제 source 여정 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Default chooser D 실제 source 여정 검증

2026-10-24. PM 단독 shared S/T 배정으로 private existing5 후보를 검증했다. canonical 채택·stage·commit은 PM 소유이며 이 기록은 제품 채택을 뜻하지 않는다. 기능 범위는 원 default ProductFrame choicesNone 조립에서 같은 SDK/NativeRuntime을 사용하는 source 입력·실제 retained frame 여정이다.

## 최종 입력과 결과

- root `67c6215a1a831967090cd5de6d55077f91995bb9`, Meari `b532fe0eed06d8526b755b11bc36416070244ef7`, lock `8F2C1D56C8A2C22D36D9E57A285D727C7EE9C7D2D1160902B66CC6630B042C67`.
- 최종 source5 `runtime/development/default-chooser-journey-d-document-ref-repair1/source5.json` SHA256 `C44586185B692330C74D01DBD6C154AD2F2FF6F5EA2AD229F136CDE98FCDEC93`. 원 existing5 및 단계별 before/raw/delta/leaf/HANDOFF를 보존했다.
- 시험 입력 `default-chooser-journey-d-r1/GATE-tests6.json` SHA256 `0ACDF2B5D01D227D15464381CF67FDF91631427203C89BD67457B19E2D503714`: ordinary 영향2 → 신규 ignored D3 → 기존 ignored Card SourceReturn1.
- 최종 BIN `runtime/verification/default-chooser-journey-d-r1/bin/default-chooser-journey-d-r4-tests.exe` SHA256 `8DD26C81EC18354D5C252463E12FA1093888FD49BF9C49D2990F674963C90E1E`, Cargo fresh=false, features native-observer-test-support/native-proposal-window. 실제 producer/lib/profile 및 FileID는 41-fixed-bin-r4.json, 원JSON은 40-lib-no-run-r4.stdout.
- 37 fmt, 38 default all-target clippy -D warnings, 39 native feature all-target clippy -D warnings, 40 native lib no-run JSON 모두 exit0. 실제 list/ignored와 정확6 등록을 대조했다. root 전체 cargo test를 실행했다고 표시하지 않는다.
- 44 Correction 선행1은 actual1pass/terminal0이며 중복이다. 45~50 최종 exact6은 각각 actual1passed/0failed/0ignored, terminal0. 최종 고유6=ordinary2+신규 D3+기존 Live1이며 선행 중복1을 합산하지 않는다. 원결과 해시는 54-final-results6-r4.json.

각 heavy 직전 RAM free16GiB/GPU free3072MiB/C free12GiB/writer0 입장, 원 resource-runner conservative2 CPU2/mask3/BelowNormal/descendant Job, offline/buildjobs1/testthreads1을 유지했다. 원 supervised child total20초/cleanup reserve2초/run18초, frame expiry15초/frame64/event80/payload4096은 확대하지 않았다.

## 실제 입증 범위

각 child의 동일 SDK/store에 원 create로 문서2·문서당 항목2를 seed했다. TrustedLiveChoices를 D에 주입하지 않고 원 ProductFrame choicesNone을 사용했다. 매 단계 현재 module callback/RadioGroup item을 다시 찾아 원 Action::Activate로 List→문서A→두 번째 항목→Open→원 current inspection/ACK→실제 entry proof/frame→clean active→실제 source VIEW→새 Viewing proof/frame→release→단1회 fresh redraw 요청/consume→다른 실제 submission ticket→새 List/키 generation→문서B→첫 항목→재Open→두 번째 실제 entry proof/frame→clean active→원 close를 검증했다. Card·Highlight·Correction(cue A→document-ref B) 세 task를 각각 독립 child로 확인했다.

최종 child 원기록은 53-child-original-r4와 53-child-summary-r4.json에 원 경로·사본·해시·family identity와 함께 보존했다.

| 최종 여정 | child 내부 ms | event 수 | entry frame / Viewing frame |
|---|---:|---:|---:|
| Card | 6478 | 36 | 2 / 1 |
| Highlight | 8891 | 36 | 2 / 1 |
| Correction cue→document | 7772 | 37 | 2 / 1 |

event 수는 실제 저장 관측 수이며 총 렌더 frame 수로 바꾸지 않는다. 각 D에서 요청/consume/새 실제 ticket 흔적3, 원 seed create2와 여정 list2/inspect4를 구분했다. Save/Accept/CAS/텍스트 수정 요청은 추가하지 않았다. 최종 세 child 및 기존 Live1은 run_ok/same_runtime/closed=true, 원 cleanup WorkersQuiescent(Closed), 실제 child exit0/Family Empty/forcedfalse와 원 감독 protocol 검증이 통과했다. 선행 Correction은 별도6,539ms/event37이다.

Correction은 실제 active Stored draft edits[index].source_ref와 선택 입력 ref·원 seed/current inspect identity를 대조한다. repair 전용 document_selection은 별도 관측이다. 현재 retained frame에서 editor 값과 실제 ProposalSource module origin의 정확1개 sourceRefs li 전체 텍스트가 선택 draft ref JSON과 정확 일치해야 한다. GeneratedRaw 전체문자열이나 다른 edit ref의 존재를 양성 근거로 사용하지 않았다. 성공한 실제 li/ref/stamp/editor 기록은 원 LiveInputDisplayed event에 있다.

## 원 실패와 최소 후계

원 FFCD/BDEE의 12 Card는 실제 entry frame/Active 뒤 identity 관측이 다음 turn을 기다리던 시험 위치 오류로 실패했다. 원 child101/Empty/nonforced/정상 WorkersQuiescent(Closed), ordinary2pass를 12-failure-checkpoint.json E26AF1F0과 원raw에 보존했다. active committed 경계의 기존 cfg(test) probe 위치로 읽기 호출만 이동했다.

후계 B49B/A01D의 21 Card는 실제 Viewing retained frame 뒤 fresh frame 요청이 없어 원18초에서 child124/forcedtrue로 종료됐다. 21-failure-checkpoint-r2.json D7961F57을 보존했다. Family Empty/감독 물리적 Complete/원 supervisor owner 없음과 SDK worker/ACK/inner cleanup 미확인은 구분한다. 이 실패를 정상 cleanup으로 바꾸지 않는다. 다음 후계는 실제 release에서 pending1→기존 redraw hook 단1회 consume→다른 실제 ticket의 current frame 뒤 원 List를 입력하도록 했다.

후계087823/FB96의 Card 선행 및 최종 ordinary2/Card/Highlight는 통과했지만 35 Correction은 두 번째 retained entry까지 도달한 뒤 repair 전용 선택 상태를 draft item ref로 비교해 실패했다. 정상 Closed/Empty/nonforced 및 원raw4case를 35-failure-checkpoint-r3.json 73D418E3에 보존했다. 최종 C445는 실제 draft item ref와 현재 exact li/editor로 관측 계약을 바로잡았다. 세 원 실패/BIN/부분 결과를 최종 성공수에 합산하거나 삭제하지 않았다. 실패마다 후속을 중지하고 PM의 정확 후계 승인을 받은 뒤 새 BIN으로 검증했다.

## 복원·보호·반환

SDK 반환 slot A04E7568/S5944/T588/carry129 A5238F92 및 RESTORE-INCIDENT.md를 먼저 읽었다. 승인된 canonical launcher7 exact7만 별도 preimage로 보존·S 동기화했고 그 뒤 S5945를 이번 복원 baseline으로 고정했다. D overlay와 분리한 기록은 01-sync-preimage7/02-preimage5/00-admission.json이다.

51-restored5-r4.json에서 기존5 bytes/UTC mtime 복원 완료. 승인 launcher7 동기화는 유지한다. root1383/library493/index·manifest·lock6/protected6/fixed129/S기존5945는 bytes·mtime 및 해당 FileID 변경0이다. protected 실사본6도 해시 일치한다. alias에 직접 Copy 덮어쓰기·unlink·복구를 하지 않았다. 현재 alias/producer FileID 쌍과 hardlink list Error50 한계는 alias-link-boundary4 및 56-after 기록에 있다. 전체 링크 membership을 열거했다고 주장하지 않는다.

S 신규76은 실제 감독 여정11개 전용 runtime root에 귀속해 55/58 manifest로 보존했다. 최종 S6021은 after-S6021.json이다. T 기존588 중2(libtest kinekino_player/player_ui_plugins) 변경은 최종40 CargoJSON producer에 정확 귀속했고 신규exe0이다(57-T-changed-producers2.json/after-T588.json). 기존 fixed129에 원/후계 고정BIN4개를 추가한 carry-fixed133.json SHA256 `02661FE39CC06E3B48222FCE2CD044FD7D5F85761AB763BEE3CF5B0CAB4C55A6`을 보존한다.

SDK의 과거 alias 복원 사고에서 bytes/mtime는 복구됐지만 alias4 FileID가 바뀐 예외와 과거 known2 2D2094→10B560 producer 미귀속/원backup 부재 예외는 유지한다. 이번 admission 이후 보호6 불변과 혼동하지 않으며 all_cache_protected=false다. 임의 cache clean/복구/target 교체0이다.

59-recorded-pid-observations/59-final-pins-process: 원 parent·child PID22개 현재 부재, writer0, root/library/lock 재확인. canonical/index/stage/commit/push0이다. 원 controller/worker cleanup은 성공한 child의 원 결과 범위에서만 인정한다. 원21 forced child의 미확인 SDK cleanup은 여전히 미확인이다.

## 미검증과 스킬

일반 player.exe noargs launcher 도달성, OS pointer/key/IME/접근성·수동 시각 일치·웹 전체 호환·실제 미디어·CLI 전체·AI 기능은 이 fixture에 합산하지 않는다. 기술 source action과 실제 retained frame은 해당 여정만 입증한다. 별도 CLI/SDK 기능 gate를 GUI로 대체하지 않았다.

이 세션에서 modoo.today 본문을 읽은 `modootoday/software-qa/test-fixture-and-mock-build`의 원 fixture acquire/teardown·입력/실제결과·원 실패 보존과 `modootoday-free/design-delivery-plus/frontend-first-journey`의 실제 진입 경계·공통 source 기준을 적용했다. `%USERPROFILE%/.codex/skills/rust-pro/SKILL.md`의 Rust typed Result/소유권·차용 기준을 구현·native strict 검증에 적용했다. 새 비 Rust 제품/시험 정책 코드는 없다.

PM 인수: 원최종6결과/child5 raw해시를 대조하고 exact5를688a99cbab9a8d371a33d0d3716c34a52f2949b7로채택했다. 범위한계·과거실패/캐시예외 유지.
