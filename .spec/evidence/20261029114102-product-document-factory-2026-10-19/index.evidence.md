---
id: evidence:kinekino/20261029114102
title: 제품 document factory·명시 패널 선택 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 document factory·명시 패널 선택 검증

2026-10-19. [준비 gate](../../../plans/player/presentation/product-document-factory-gate.md)의 owned6 새 소스와 선언9검사를 실제 독립 Rust consumer로 실행했다. 이전 Native SDK27과 구별한다. root 제품 등록·Native window host 연결·실제 GUI/청취·전체 시각 인수는 포함하지 않는다.

## 입력과 실행

근거는 `runtime/quality-review/player-ui-product-factory-r1/gate-r1`의 source·prepared-input-diff[]·owned/shared Rust/manifest pins와 stdout/stderr/receipt다. 준비 inventory SHA `25e9f654f13185e1ea738fb06a7d08f64bf0d3bd1d274c6cc6e86ee2cae325bc`와 owned6의 실제 차이는0이었다. SDK API SHA는 `732cec0b522431837f728fea480d37a446b7378a770fe89cab0b595b4d444567`, Meari native-host-port-r1/source-final-input 및 source-final-after inventory는 둘 다 `978122c7ece12427569a337dfd86e3723d2c6c0dac0a161c8e7c944d0a11a422`다.

검증된 runner SHA `692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463`, affinity3/2CPU/BelowNormal/descendants job, offline/locked/-j1/RUST_TEST_THREADS1·기존 shared target/cache로 실행했다. 선택 gate는131586ms/exit0, factory7passed/0failed/0ignored·1.53s와 panel2passed/0failed/0ignored·0.03s다. 최초 선택과 전체의 컴파일/기능 수리는0이다.

전체 --no-fail-fast는121788ms/exit0,36passed/0failed/0ignored였다. 기존27(callback1/R3 Document1/Review Native1/CardHL2/actual assets3/view14/intake1/sdk_files4)에 새 factory7/panel2를 더한 실제 수다. 이후 최초 strict23288ms/101은 parent.close 중첩 조건문 collapsible_if와 test closure 반환 type_complexity의2개 지적이었다. 원 stderr와 source2개를 gate-r1/*before-clippy.rs에 보존했다.

동일 의미의 let-chain과 TestRegistry/TestContext type alias로만 최소 수정했고 allow/기능 정책 변경은 없다. gate-r1의 두 fix.diff에 원 차이를 보존했다. 수정 뒤 strict는11637ms/exit0, 영향 선택9는16254ms/exit0(factory7+panel2, failed0/ignored0), fmt는310ms/exit0이다. 전체36은 수정 전 입력, 새9/strict/fmt는 최종 입력 근거이며 마지막 전체36을 수정 뒤 다시 실행했다고 주장하지 않는다.

final-source6과 final-owned-pins.json을 보존했고 최종 inventory SHA는 `129c2029019389ff77d29efdad6615c81ce70d4d199da4dcefe63c9fa5b564c1`이다. shared Rust delta의2행은 동일 UI owned factory 파일의 수정 전/후이며 외부 SDK/Meari shared delta0·manifest delta0이다. owned delta에는 factory source와 test helper의 두 파일 변화만 있다. shared/manifest 전후 pins·prepared와 final pins를 구별한다. 확인한 모든 runner receipt는 terminal=true, owned-processes-after=[]이며 새 GUI/browser는0이다.

## 인수한 경계

원본 Card3모듈·실제 SDK/ProposalFiles·외부 parent registration을 사용해 factory를 조립했다. source identity·명시 index·초기 props의 실제 session projection 대조, ModuleProgram 실제 root scope=Native callback factory scope=registry/session scope를 확인했다. parent는 같은 client/registry/context에 등록하며 새 제품 SDK/store나 가상 NativeHost API를 만들지 않는다. SDK pending parent lease를 root에 등록해 소비한 것은 아니며 시험 Lease는 동일 registration 수명 경계의 기술 fixture다.

factory7은 정상 조립·session 뒤 parent close1회, source/item/초기 props 불일치의 parent 등록 전 거부, parent handle 기술 실패와 정리 Busy 보존·재시도, 정리 증거 Active→NativeAndSessionClosed와 parent 실패 뒤 재시도 보존, 실제 registry pending→cancel Requested→여전히 Busy→fixture terminal Cancelled→정리 완료, renderer preflight 기술 실패 뒤 retired cleanup Busy·borrow 반환 뒤 재시도/실패 handle stale·old export 생존, 중복 UI epoch stage 등록 실패의 원PluginError/부분 owner 보존을 확인했다.

panel2는 복수 Card 중 명시 두 번째 선택의 독립 front/back·전체 refs 순서와 해당 원문 순서, 범위 초과·다른 session·편집 후 stale 거부, 최신 명시 선택 재발급·Unicode·첫 항목/raw 보존을 확인했다. 선택 index를 stable domain ID로 취급하지 않는다.

새 new_retaining이 부분 owner를 호출자에 보존하는 반면 호환 new는 error만 반환하고 partial Drop의 cleanup 오류를 재시도할 수 없다는 한계는 유지한다. 제품 factory는 retaining 경로를 사용한다. callback 정리 증거는 NotCreated/Unconfirmed와 실제 관측한 Native/session 완료를 구별하며 정상 OS CloseReceipt로 확대하지 않는다. 취소 Requested는 완료가 아니고 borrowed registry 해제만으로 pending이 사라지지 않는 실제 동작을 검증했다.

## 한계와 후속

현재 factory/panel은 제품 부모 mod/root 미등록이며 독립 테스트가 path로 소비했다. source session의 retaining constructor/정리 보완은 독립 consumer 전체에 적용된다. Meari 신규 window driver/AfterDispatchHost/host stamp/OS surface API 연결을 이번 gate에서 실행한 것은 아니다. 기술 fixture source/typed raw는 실제 모델 출력·품질 승인이 아니며 same-process SDK/store 재열기와 fresh CLI/project/provenance 인수는 별개다. 제품 창·volume/mute·커서·전체13화면 시각·가상 모니터 검증은 별도 후속이다.
