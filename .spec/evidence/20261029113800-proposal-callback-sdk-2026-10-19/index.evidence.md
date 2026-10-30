---
id: evidence:kinekino/20261029113800
title: Proposal callback·실제 SDK headless22 gate
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Proposal callback·실제 SDK headless22 gate

2026-10-19 중앙 슬롯에서 fmt·offline 독립 lock 동기화·locked test22·strict Clippy가 종료0이다. 원문은 runtime/quality-review/player-ui-callback-sdk-r1의 gate-r1/최초실패, retry-map-type/최종검사에 보존했다. Component parser/renderer·실제 TSX 입력·제품 document 단일 registry·시각 인수는 포함하지 않는다.

## 고정 입력과 실행

prepared-22-r1/source-pins.json과 실행 직전 대조에서 모든 소유 입력이 일치했다. 원본 snapshot은 유지했다. 최신 root lock은 전후 `50323b23fb1bbf496e3ad42be71b4ccbdc3e1f38de1de5a5154a1ba5d78da5f7`, 독립 tool lock은 offline 동기화 후 `2c1d2145cc612286d165436c0d00253c06378dbe1db62e0075e24a7a7e5b8d21`이다. 동기화는 신규 package0이며 root lock을 수정하지 않았다. upstream API/source pin은 gate-r1/upstream-pins.json, 최종 변경 source pin은 retry-map-type/final-source-pins.json이다.

검증된 resource-runner SHA `692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463`를 사용했다. receipt에서 affinity3/logical_cpus2/priority16384(BelowNormal)/자식 계열 적용을 확인했다. 공유 native target·-j1·RUST_TEST_THREADS=1·offline/locked를 유지했다. 최종 test compile19.63초/runner24639ms, Clippy9.70초/runner9897ms다. 모두 terminal=true/exit_code0이며 완료 직후 슬롯과 소비 API 동결을 반환했다.

## 범위와 실제 결과

| 묶음 | 결과·범위 |
|---|---|
| callback_sdk1 | 실제 HostCallbacks→callback registry→enqueue adapter→bounded queue→dispatch 반환 뒤 controller→기존 ProposalReviewHost/SDK/ProposalFiles, 명시 저장·채택·refresh·새 SDK 재열기 |
| proposal_assets3 | 기존 실제 raw9개 SDK 회귀. 품질 승인과 구별 |
| proposal_view14 | 기존 상태·필드·controller9 + bounded queue4 + 실제 Meari callback port/synthetic controller1 |
| sdk_files4 | 기존 실파일 변환/SDK오류/문맥·host 분리 회귀 |

실제 SDK callback case는 `93920-1792393854813529100`이다. calls.json에서 edits1/accepts1/inspect_attempts3/sdk_inspects2를 직접 확인했다. 세 번째 수가 다른 것은 post-accept inspect 실패1회를 host wrapper에서 명시 주입했기 때문이다. 실제 SDK 장애로 표현하지 않는다. dispatch-store-pins.json과 통과 assert는 dispatch 동안 파일별 SHA·canonical 상태 불변, drain 뒤 명시 mutation, refresh 중 저장소 불변·mutation 재전송0을 확인했다.

restored.json은 새 SDK/ProposalFiles로 재열었으며 source SHA/raw/receipt 전체가 controller의 최종 상태와 같았다. 문서3·receipt draft2/output3·automatic_application_enabled=false를 직접 대조했다. 원문 source.json과 generated.txt는 변경되지 않았다. technical fixture이며 generation provenance/quality는 unknown, quality_approval_recorded=false다.

callback registry와 기존 headless ProposalBridge의 SDK registry는 별개다. 두 경로를 연결한 실제 SDK port 검증이며 실제 제품 document 단일 registry·node/event guard를 입증하지 않는다. source_callback_lowering_verified/renderer_verified는 false다. 합성 QueueHost의 호출 수 검사도 실제 renderer 결과로 확대하지 않는다.

## 최초 실패와 좁은 수리

첫 compile은 tests/support/callback_fixture.rs의 BTreeMap 키 타입 추론 E0282로 종료101이었다. 같은 형태인 proposal_view.rs의 선언도 함께 명시 `BTreeMap<String, QueueBinding>`으로 고쳤다. 첫 진단·두 수정 전 source는 gate-r1에 보존했다. 제품 정책·scope 검사·시험 범위를 줄이지 않고 같은 슬롯에서 fmt/full22/strictClippy를 통과했다. 그 외 source 수리는 없었다.

후속은 [동일 TSX renderer 인계](../../player/presentation/shared/proposal-renderer-handoff.md)와 [제품 조립](../../../plans/player/presentation/proposal-product-registration.md)을 따른다. 원본 source와 callback/node guard 인계 후 실제 document 경로로 검증하며 전체13화면군의 시각·키보드·패널 인수는 별도로 남는다.
