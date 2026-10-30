---
id: evidence:kinekino/20261029114052
title: 명시 선택 bootstrap 소비 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 명시 선택 bootstrap 소비 검증

2026-10-20 KST. 순수 canonical snapshot→초기 props와 실제 원본 Card/HL ModuleProgram/factory 소비의 기능 검증이다. 제품 런처·실제 창·전체 디자인·실제 historical package 검증 완료가 아니다.

## 변경 경계

- `src/presentation/web_ui_adapter/proposals/bootstrap.rs`: canonical StoredProposal.validate, 명시 proposal/document revision/source ID·revision·SHA 및 선택 index/reference 대조. Card/HL/cue correction/document correction/NeedsRepair 초기 props. SDK·store·worker·registry mutation 없음. callback은 실제 opaque handle을 위조하지 않는 reference다. canonical StoredProposal 전체를 borrow하여 historical을 복제·폐기하지 않는다.
- `bootstrap/tests.rs`: 순수 변환 11개. 두 번째 항목·참조 순서·시간 None·모든 선택 identity stale·잘못된 target/index/reference·NeedsRepair 미선택/raw/import_error·document correction·document3/binding1·legacyNone·잘못된 version/binding·원본 binding과 수정 accept output/품질 미판정 분리를 확인했다.
- `proposals/mod.rs`: bootstrap 및 의존 panel_model의 최소 공개 등록. root presentation은 이 adapter를 아직 등록하지 않는다. 이번 새 컴파일 경계는 별도 player-ui-sdk-bridge consumer이며 제품 main/lib/manifest/기존 factory/bridge/SDK/Native를 변경하지 않았다. 제품 런처 변경이 없으므로 root 전체396 회귀를 반복하지 않았다.
- `tools/player-ui-sdk-bridge/tests/bootstrap_preparation.rs`: 기존 원본 Card/HL source inventory·factory·실제 ProposalSdkRegistration/공통 SDK를 재사용한다. 명시 두 번째 항목 props 일치와 mount/close, 잘못된 props 거부 3개. 선택 index1은 기술 시험 입력이며 제품 기본값이 아니다. 새 제품 fixture/default 경로를 추가하지 않았다.

## 실행 근거

원문은 `runtime/quality-review/ui-bootstrap-r1/`에 보존했다. runner SHA256 `692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463`, 전용 cache `runtime/build-cache/sdk-product-parent-consumer`. 모든 cargo는 offline/locked/-j1, 시험 thread1, runner affinity3(2 CPU)/BelowNormal16384/descendants job을 사용했다. GUI/브라우저/모델/다운로드/DPI/VM 실행 0이다.

| 실행 | 결과·원 receipt |
|---|---|
| 집중 `cargo test … bootstrap` | 13pass/0fail/0ignore, exit0, 199624ms. selected.* |
| consumer 전체 최초 | 61pass/0fail/0ignore, exit0, 30097ms. full.* |
| 최종 source strict `cargo clippy … --all-targets -- -D warnings` | exit0, 16611ms. strict.* |
| 최종 source consumer 전체 | 61pass/0fail/0ignore, exit0, 37486ms. final-full.* |
| `cargo fmt … --check` | exit0, 349ms. fmt.* |

컴파일/시험 실패는 없었다. 최초 집중 실행에서 private test module로 가져온 기존 공개 factory/helper의 dead_code 경고가 발생했다. 원문과 `consumer-before-public-test-modules.rs`를 보존하고 기존 소비 시험과 같이 test module 공개로 최소 수리했다. 첫 전체 실행에는 host parts의 제한 필드 경고가 남아 `consumer-before-budget-assertions.rs`를 보존한 뒤 기존 factory가 전달한 registry_pending/queue entries/text budget을 실제 assertion으로 검증했다. 경고를 억제하거나 기존 factory를 수정하지 않았다. 최종 strict·전체·fmt는 동일 최종 source 기준이다.

## source 및 정리

변경 전 12파일의 inventory와 사본은 `before-inventory.json`, `before/`에 보존했다. 최종 13파일 inventory는 `final-inventory.json`, 변경4파일 사본은 `final-source/`다. 최종 inventory SHA256은 `54d5320303a5275bf94a954a2a5eb3987f1f247ff4400814b54a51b4ab93a219`다. 전후 hash 차이는 bootstrap·child tests·proposals/mod.rs의 소유3파일이며 새 소비 시험1파일이 추가됐다. inventory에 포함한 SDK canonical source·assembly·root/consumer Cargo manifest/lock·root presentation source는 동일하다. 기존 SDK 소유3 fixture는 수정하지 않았다.

모든 runner receipt는 terminal/exit0이며 마지막 관측에서 cargo/rustc/link/resource-runner 실행은 없었다. store 시험 표본·최초 경고 원문·source 사본은 증거로 보존하고 다른 앱을 종료하지 않았다. source/단일 heavy 슬롯을 PM에 명시 반환한다.

## 검증 한계와 후속

v3 fixture의 canonical 구조 검증은 실제 companion reader 무결성/원 후보 verdict의 검증이 아니다. historical binding은 품질 승인 필드가 아니며 accept output에 원본 quality를 상속하지 않는다. Card/HL qualityLabel은 caller의 미판정 문구를 유지한다. correction/repair 독립 provenance 표시와 원본 decision 조회는 별도 계약이 필요하다. 실제 원본 ModuleProgram/factory 소비는 Card/HL 표본이며 correction/repair의 새 bootstrap 소비 연결·런처·OS 입력·실제 terminal wake·13화면 시각 인수는 남아 있다.

import capability와 bound inspect/edit/accept 모두의 비동기 verifier 연결은 이번 변경에 포함하지 않았다. SDK 공통 execution/terminal/cancel/게시/assembly와 UI 범용 async codec·cancel/shutdown·presentation 소비를 후속 배정에서 조율한다. 기존 Core 공개 API를 재사용하고 plugin package의 root SDK 역의존을 만들지 않는다.
