---
id: evidence:kinekino/20261029114003
title: R2 공통 live mount — headless 검증·보호 반환
status: active
created: 2026-10-29
updated: 2026-10-29
---
# R2 공통 live mount — headless 검증·보호 반환

2026-10-24 KST. PM의 유일 heavy 배정으로 기존 S/T와 원 resource-runner conservative2를 사용했다. 기준 root09fc7518407798be1480f17aa81f2efa1bdef8d3, Mearib532fe0eed06d8526b755b11bc36416070244ef7, lock8F2C1D56C8A2C22D36D9E57A285D727C7EE9C7D2D1160902B66CC6630B042C67. [계획](../../../plans/player/presentation/live-mount-r2-headless-gate.md), [원 로그·manifest](../../../../runtime/verification/ui-live-mount-r2/STATE.md), [R1 선행 정본](live-entry-r1-2026-10-24.md)을 구분한다. root stage/commit/push와 canonical 수정은0이다.

## 입력과 수정 계보

원 R2 source12 6A0A5864는 불변이다. 채택 R1 fixture를 한 leaf에 상속한 successor1 C5BD3B83 exact12로 입장했다. 기존10/new2 current canonical/S preimage 일치, SDK·Media A·R1을 보존했다. 각 추가 수리는 PM이 원 실패와 diff를 읽고 별도 승인한 뒤 반영했다.

| revision | effective manifest SHA256 | 변경·원 실패 |
|---|---|---|
| successor1 | C5BD3B83776EA4D149422AC460FE19B98E526985C656C1A580B37960252ECB8E | 원 exact12, 02 strict E0500 |
| repair1 | 7F29618124A3D5E152F8D08631E1EFE4136C063CAA3F79528D8B43CCA712496E | return_viewing 같은 cards Rc 로컬 보유. 04 기존 시험 import/literal 오류5 |
| repair2 | B6F64A64ED8D5164E7FCA563D28AB626E6463F49FCD182879D18D82ECA7E463C | mount_card 시험 import와 기존 literal2 completion:None. 기존 시험2를 더한 effective14 |
| diagnostic1 | FB518A1987DA8AB7F2784A9396CAC426A624938B6703608D533A356C89C52AA4 | 원16 직접 unwrap의 runtime/typed error를 보존하고 반드시 실패. 23에서 실제 Commit props 오류 확인 |
| repair3 | 17DEA552DEC5A617AE9BFD04CEF1BF3831535A7E88F80EA986939CC2472810B8 | gate_props가 미선언 key를 추가하던 제품 결함 수정. 기존 bool만 변경·key 집합 보존. 38은 시험의 full stamp 가정 오류 |
| repair4 | 6C54939C0A8F189EA723AA77196BA7CCCA9DA9DA34B0CB06B461DB68129CCDF4 | 동일 문서·owner·revision/무commit과 정상 turn+1 구분. 114는 기존 시험의 주입 위치 가정 오류 |
| repair5 | 5007ED3672C99AF04430A1F74BE1DBDE12F217A821672B3502268AC2BC4C6F7C | mount_card/tests.rs 한 leaf의 원 복귀 transaction 주입 및 panic-safe cleanup |

최종 입력은 runtime/development/ui-live-mount-r2-repair5/source14.json이다. 원 exact12와 추가 기존 시험2의 경계를 유지하며, 별도 실제 frame fixture11은 합성하지 않았다. source/delta/before/patch/HANDOFF는 각 runtime/development/ui-live-mount-r2-*에 보존했다. 최종 tested-source14의14 hash 일치. 최초 콘솔 mismatch14는 PowerShell 비교의 RHS를 property 대신 literal `expected`로 쓴 집계 오류였으며, 원 저장 hash는 일치한다(tested-source14-check.json). 소스 불일치나 자동 수리의 근거로 사용하지 않았다.

## 실제 검사와 결과 귀속

최종119 fmt/120 strict/121 CargoJSON no-run 통과. strict는 kinekino-player all-targets, features native-observer-test-support,native-proposal-window, -D warnings; Cargo는 locked/offline/j1, thread1, CPU2/BelowNormal/descendant Job. 입장 RAM34743132160B, GPU6276MiB, C111114555392B/writer0이고 후속 heavy resource 기록을 보존했다.

CargoJSON의 실제 library kinekino_player/profile.test/src/lib.rs producer를 고정했다. 최종 BIN5270B3314AC73B15A2888F20AD9A5F86C7A5496DC32668A415328237F058BE1F, 174873088B/freshfalse, list1145/ignored68. 기본 player.exe/alias를 시험 실행 근거로 사용하지 않았다.

고유74개가 원 stdout에서 각각1 passed/0 failed/0 ignored다. 일반29·literal --ignored45이며, exit0만으로 통과를 세지 않았다. 신규14, R1 원8, 기존 영향15, 추가 직접 영향37을 중복 제거했다. 전체 library/workspace/CLI/다른 ignored/OS window/GUI 재실행은0이다.

- source6C549/BIN96CE5227B0345271DB4267D43224AF340871A61F4916BAAAEA5AAF65D61274EB에서69개 통과. 원114 한 개 실패와 잔여4 미실행을 보존했다.
- repair5는 시험 한 body/helper만 바꾸며 제품13과 다른 시험 body는 불변이다. PM이 영향 없는69 반복을 배정하지 않았다. 최종5007ED/BIN5270에서125 원114,126–129 잔여4,130–131 같은 leaf의 기존2를 실제 실행해7개 통과했다. 원69 중2는 중복 재검증이다.
- final74-results.json은 이전 소스67+최종 소스7로 구성한다. retained69-results.json/final7-results.json/원stdout/receipt를 연결하며 최종 BIN에서74개 전부를 재실행했다고 주장하지 않는다.

전체 runner receipt123은 terminal, exit0=117/exit101=6이다. 원 실패02/04/16/23/38/114를 삭제·성공으로 덮지 않았다. 원16/23/38은 original cleanup complete=true. 원114는 기존 시험 panic으로 ordered cleanup 인수 근거가 없으며, 프로세스 terminal과 정리 완료를 혼동하지 않는다. 수정125는 original_viewing_return_closeout complete=true/primary_panic=false/cleanup_panic=false를 실제 기록했다.

## 복귀 fixture의 정확한 한계

원114의 immutable callbacks borrow는 full runtime의 앞선 Jobs mutable guard에서 실패하므로 Card 해제 뒤 export failure라는 가정이 틀렸다. 제품 guard를 바꾸지 않고 첫 fault만 원 advance_viewing_return transaction seam에서 관측한다. 정확 UnsupportedCallback/viewing_return_error/receiptNone/cancel RecoveryBlocked/경쟁 attachment 성공을 함께 검사한다. private Exporting은 직접 enum 관측이 아니라 원 호출·retry 경로의 추론이다. 그 뒤 경쟁 SDK attach failure·명시 retry·full runtime commit/read·원 LocalHost close는 유지한다.

runtime 생성 뒤 body는 catch 밖의 원 runtime/competing attachment로 보호한다. body panic 뒤에도 기존3s cleanup을 시도하고 원 panic/typed cleanup error/pending/원 progress를 보존한다. cleanup 실패 carrier는 ManuallyDrop으로 원 owner를 보존하며 가짜 complete/Drop 종료를 만들지 않는다. setup 조립 전 보호와 panic 분기의 실제 fault injection은 이번 인수에 없다. 정상125 cleanup만 실제 입증했다.

## 보호·반환과 캐시 예외

S 기존5349 bytes/UTC mtime diff0. 기존12 복원·신규 소스2 exact leaf만 제거했다. 새 시험 store267은 ui-capture-guarded-stores155/current-card-mount30/jobs20/live-entry34/live-mount28로 보존했다. root tracked1365/library493/index2/fixed95 bytes/mtime diff0, alias4 bytes/mtime/FileID diff0. 새 고정 product BIN5개도 원 hash와 일치한다. 임의 clean/touch/cold target/전체 복사0. 최종 owned process 관측은[]이며 모든 실행 session은 terminal이다.

**known cache2의 입장 bytes 보호는 실패했다.** T575개 중572개 bytes/mtime 불변, library test producer1은 DAD4721D→5270B331로 변경됐다. 별도로 media-vad-worker-470928e8aa35bb24의 build_script_build-470928e8aa35bb24.exe/build-script-build.exe 두 경로가 입장2D2094C73DC9FA25162BFA1B2466F7C5C5E0390ED79E637914F6E86D6BC6CCAA에서10B560FEA3D8094098A43C8594B543D1A630B754ACF1D6F0B6BE8C5A4CC2B6CE로 바뀌었다. current mtime2026-10-23T15:42:34.2327483Z는 첫 strict02의 Compiling media-vad-worker 구간에 있다. 하지만 strict는 CargoJSON producer 기록이 아니며 후속07 JSON custom-build는 다른 cc2fc3 경로를 가리킨다. 이 두 경로의 exact current producer는 미확정이다.

입장 시 known2의 hash/mtime/FileID만 기록하고 원 bytes backup을 만들지 못했다. 따라서 이번 자료만으로 원 bytes 복원은 불가능하다. 현재 bytes2를 cache-exception2에 별도 복사했고 전후 FileID(084a16→101332), 같은 current FileID, hardlink 목록 조회 Error50도 원문 보존했다. 두 경로의 같은 FileID 관측과 전체 hardlink 열거 성공은 구분한다. 과거2D2094 producer 미귀속 이력도 해결되지 않았다. all_cache_protected=false/known2_restored=false이며 임의 복원·mtime 재작성·다른 cache 덮기·추가 build는 하지 않았다. 이 예외를 포함해 반환하며 다음 gate는 현재 cache를 새 baseline으로 고정해야 한다.

## 미인수 범위와 적용 스킬

이 결과는 원 SDK owner·공통 panel·typed route의 headless 연결/입력·오류·close 범위다. actual Some checked frame/entry 관측→활성→새 Viewing proof/원 startup notifier 양성은 [별도 fixture 계획](../../../plans/player/presentation/live-mount-r2-fixture-gates.md)에 남는다. R3/R4·사용자 화면 진입·OS compositor·키보드/IME·13화면 시각 인수·DPI/VM·SDK 목록/Media B 합성0. 사용자 demo 종료0.

실제로 읽은 modoo.today `modootoday-free/design-delivery-plus/frontend-first-journey`의 원 typed port/공용 컴포넌트/미지원 구분과 `modootoday/software-qa/test-fixture-and-mock-build`의 원 setup/body/error/cleanup carrier·source/model/actual/revision 구분을 적용했다. PowerShell은 파일·hash·metadata·Cargo/runner orchestration에만 사용했고 새 제품 정책·시험 로직은 Rust다.
