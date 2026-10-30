---
id: plan:kinekino/20261029113921
title: Viewer selector CPU gate — 배정 전 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Viewer selector CPU gate — 배정 전 준비

상태: PM이 후계4301757F의 독립 재읽기 결과(잔존P2 해소/새 확정P1P2없음)를 인수하고 정확6 CPU gate를 배정했다. Media C2 OS4 반환 E779FF51을 읽고 인수했다. 현재 배정은 아래 CPU/복구/반환 범위이며 actual N/C는 별도다. 기존 통합 계획과 cache-alias-preservation/원 CONTROL-PROTOCOLED44를 재사용하고 새 framework/target/download/전수 cache 재해시·중복 backup0이다.

## 핀·적용 경계

selector private 후계 root는 `runtime/development/default-native-viewer-window-selection-r1/revisions/r2/`다. effective8 `4301757FA932982A5DF090697CDA68FD56CED9916D46D8B7D5939360F39FEDA4`, source2 `262F19AD36F8F78988335B1C27B7FB8742B591C21DCCA64EE012E76F1A890287`, expected20 `50169931EC3AFA99E2F45B44500C5A221F67B6BD44BECD7D086AFA312D8B4E37`(상위 root 명단 불변), canonicaldeltaAD6FD8B7/parentdelta229B70AB다. 원r1 81E302A0은 미실행 보존하며 통합에 사용하지 않는다. EDEF parent witness/ED427 Meari는 불변이다. 추가 후계는 원핀/실패를 보존하고 인수 후 명시 교체한다.

| 적용 경로 | 후보 | 실제 S 입장·복구 기준 |
| --- | --- | --- |
| tests/native_product_launch/default_launch.rs | EDEF raw leaf4F6E4111 | 원7A8854A4 existing1; canonical E3로 복구하지 않음 |
| tests/native_product_launch/default_launch/window.rs | selector r2 leaf65B28ACC | 원1BC90166 existing1 |
| tests/native_product_launch/default_launch/tests.rs | selector r2 leaf2C27D5EF | 원4E1A981F existing1 |
| tests/native_product_launch/default_launch/file_guard.rs | prerequisiteA71B92F4 | 원부재 new1; 최종 literal 제거 |
| packages/meari/packages/meari-native/src/infrastructure/window/startup/runner.rs | ED427 source2의 원 r1 raw F6C313AE | 원87FE8F4D existing1 |
| packages/meari/packages/meari-native/src/infrastructure/window.rs | ED427 r2 raw FA2DACB5 | 원630791C4 existing1 |

총6은 SDK 변경3+prerequisite new1/Meari existing2이며 기존5/신규1이다. 각 소유자와 실제 S preimage의 bytes/mtime/FI/원부재를 따로 보존한다. exact paths 이외 전체 canonical sync는 하지 않는다. effective8의 다른4는 readonly pin 대조만 한다. shared S에 적용할 후보별 개별 경로는 각 source2/effective8 manifest를 읽어 결정하며 Meari r1 window로 되돌리지 않는다.

소스 준비 중 HEAD를 새로 읽어 root abb63c8ed1778d185603b2ca62ada9eb016973dd/library f85a68df7a6a8fee239d9655895ee15d5eaa5510를 관측했다. 이후 PM은 git log/merge-base 직접 확인으로 7fb53c9가 f85 조상임을 확인해 전달했다. 입장에서 HEAD/index/lock와 실제 S 입력을 다시 확인하고 차이만으로 source를 전체 sync하지 않는다. S6081/T589는 과거 입장 숫자이며 새 전체 inventory 결과가 아니다.

최신 보호는 Media C2 type 반환1BDBB84C/SOTB6C014E9 및 OS 반환/SOTE779FF51의 `runtime/verification/owned-test-runner-c2-r1/carry-fixed144.json`, SHA `B760BC6E19C2E4DB8A80FBAF02B56C381770B677FBC12FECEA0E24BAE7FA284C`와 SDK23이다. C2 normal alias/producer B014C8BC, 새 lib unit88192259, 기존 tools unit8BA73689를 구분한다. native/debug 아래 owned-test-runner.exe, deps/owned_test_runner.exe, deps/owned_test_runner-810aae3316a0ab46.exe, deps/owned_test_runner-b5b41c25c605e891.exe의 namespace4를 현재 반환 상태로 보호하며 옛 normal02DD로 되돌리지 않는다. 정확 귀속은 owned-namespace-exe4.json/T-original-tool3-after.json/cargo-artifacts.json을 재사용한다. root의 player/CLI/H producer3는 C2와 별도다.

직전 SDK 통합 반환94CAACA6의 fixed H/player/CLI/PDB5와 진단 N 반환DDD25064의 원 입력6도 기존 기록·독립 사본을 재사용해 보호한다. 과거 alias FI 변경/known2/all_cache_protected=false를 유지하며 실제 경계 차이는 기록 후 PM과 조정한다.

## 기존 순차 gate

매 호출 fresh RAM16GiB/GPU3072MiB/C12GiB/writer0, 기존 resource runner CPU2/BelowNormal/descendant Job/offline locked j1/thread1, 동일 root 공유 S/T를 사용한다. opt-in은 MEARI_NATIVE_DIAG_CLOSE=0이다.

1. 별도 lease·Media 반환·독립 검토 핀을 인수하고 정확6 preimage→후보 배치 핀을 기록한다. 직접 필요한 변경 대상만 새 사본을 만들고 기존 보호 사본은 재사용한다.
2. 기존 `cargo fmt --all -- --check`, default all-target strict, native-observer-test-support all-target strict를 순차 수행한다. 최초 오류를 보존하며 정책/단언/기한/owner를 완화하지 않는다.
3. 기존 `cargo test --offline --locked -j 1 -p kinekino-player --test native_product_launch --features native-proposal-window --no-run --message-format=json`의 실제 H producer/package/target/profile/features/executable/fresh를 기록하고 별도 고정 H를 만든다. 목록은 **31/ignored11 예상**이며 실제 대조 전 확정하지 않는다.
4. 위 expected20 정확 명단(기존17+새3)을 같은 고정 H에서 name/--exact/--test-threads=1로 각1회 실행한다. 기존17의 admission 본문 보강을 포함한다. ignored OS11은 실행하지 않는다. 결과는 각 stdout/receipt의1pass/terminal0로 확인하고 기존 통합17pass를 대체 근거로 쓰지 않는다.
5. normal product/CLI는 실제 JSON의 target.kind=bin/profile.test=false 산출물로 구분해 고정한다. normal 둘이 없으면 기존 명령 `cargo build -p kinekino-player --bins --features native-proposal-window --offline --locked -j 1 --message-format=json`1을 별도 lease 범위에 명시한다. Cargo fresh만으로 현재 source 귀속/복구된 alias의 bytes를 보장한다고 가정하지 않는다. producer와 alias/실제 bytes가 충돌하면 원 기록을 보존하고 PM에 인계하며 fingerprint 삭제나 강제 cache 수리를 하지 않는다. 옛4224/harness를 제품으로 대체하지 않는다.
6. 새 normal exe/PDB producer를 독립 보존한 뒤 player/CLI alias4는 기존 backup의 검증된 새 파일→정확 unlink/replace로 원 bytes/mtime를 복구한다. C2 normal alias와 섞지 않는다. producer bytes/mtime/FI 유지와 alias FI 변경 예외를 분리한다. existing5 원 bytes/mtime/FI와 new1 원부재를 복구하고 입력/SDK23/carry144·C2 namespace4/소유 producer·원 실행 handle/regular EOF를 좁게 감사한다. SOT·slot-return을 발행한다.

## CPU 이후 별도 actual 순서

CPU 반환 인수 뒤 원예산 N→C를 별도 배정받는다. 새 고정 H와 해당 normal product/CLI를 쓰며 기존 독립 A/B/config·sameH host/controller/operator·State pre/Run1/State post 계약을 재사용한다. 원 argv0/30·5/case1, opt-in parent witness/child stderr의 관측 경계를 유지한다. N에서 정상 viewer 선택/CloseRequested/원 physical Closed·Empty/EOF가 실제 입증되지 않으면 C로 자동 진행하지 않는다. 실패 시 원 owner/first cause를 인계하고 추가 Close/Run/retry/자동 Abort/기한 연장0이다.

C는 기존 healthy store의 새 owned copy와 해당 새 CLI readonly inspect 사전 검증을 재사용한다. CLI 실패나 N 미해결을 GUI로 우회하지 않는다. 현재 N/C 실제 재실행은0이며 selector 수리가 제품 shutdown 전체 해결을 뜻하지 않는다. custom popup/다른 backend 지원과 전체 style 동일성은 주장하지 않는다.

이번 세션에 읽은 modoo.today code-change-review-verification9f73865f/필수 verdict-table9d9619a8·fixturea393f15f·plan-authoring31459a74/참조091d2f8d의 원 오류·자원/복구·미검증 구분을 적용했다. 정확 전체 출처 SHA는 selector HANDOFF/parent witness 정본에 보존했다.
