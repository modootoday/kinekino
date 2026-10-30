---
id: evidence:kinekino/20261029113743
title: Proposal 단일 registry·실제 SDK22 gate
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Proposal 단일 registry·실제 SDK22 gate

2026-10-19 RegistryProposalHost와 기존 actualSDK callback 회귀를 단일 registry 구성으로 실행했다. 기존22개 전부·strict Clippy·fmt 종료0이며 추가 수리는 없었다. 실제 InteractiveDocument/Native·Web listener/TSX 입력/시각 인수는 포함하지 않는다.

prepared-single-registry-r1/source-pins.json의8개 입력이 실행 직전 모두 일치했다. runtime/quality-review/player-ui-single-registry-r1/gate에 대조·upstream/source 입력 pin·stdout/stderr·resource receipts를 보존한다. root lock은 전후 `50323b23fb1bbf496e3ad42be71b4ccbdc3e1f38de1de5a5154a1ba5d78da5f7`, 독립 tool lock은 `2c1d2145cc612286d165436c0d00253c06378dbe1db62e0075e24a7a7e5b8d21`로 유지됐다. offline workspace 동기화도0이다.

resource-runner의 자기/자식 affinity3·logical_cpus2·priority16384(BelowNormal), 공유 native cache·-j1·RUST_TEST_THREADS=1을 사용했다. test compile1분32초/runner97949ms, Clippy10.69초/runner10905ms, fmt263ms이고 각 receipt가 terminal=true/exit0이다. 완료 즉시 슬롯과 root/Meari 소비 동결을 반환했다. root Cargo/API·Meari core는 UI가 수정하지 않았다.

## 단일 registry 관측

tests/callback_sdk.rs가 StateProgram의 scope로 registry를 한 번 생성하고 실제 SDK proposal adapter와 UI intent adapter를 함께 등록한다. callback helper는 registry를 주입받으며 생성하지 않는다. RegistryProposalHost도 같은 Rc/SDK registration/context slot을 주입받는다. callback dispatch가 반환된 뒤 queue drain이 relay를 호출하고 canonical Request 검증·context slot borrow 해제 뒤 SDK registry dispatch를 실행한다. manifest와 ticket context helper는 기존 consumer를 재사용했다.

기존 분리 registry checkpoint는 [이전22 gate](proposal-callback-sdk-2026-10-19.md)로 보존한다. 같은 회귀를 단일 registry로 전환했으며 test 수를 늘리거나 synthetic 대체하지 않았다. 전체 결과는 actual callback SDK1 + actual raw9를 소비한3 + view14 + sdk_files4 =22다.

새 실제 case는 runtime/quality-review/player-ui-callback-sdk-r1/98864-1792396873897790600이다. calls.json은 edits1/accepts1/inspect_attempts3/sdk_inspects2이며, post-accept inspect1은 기존 명시 기술 주입이다. dispatch-store-pins.json의 registry_topology는 single_registry_headless다. dispatch 중 store 불변→drain 뒤 명시 mutation·inspect→refresh 재전송0→새 SDK 재열기/원문 SHA·raw·receipt 비교 assert가 통과했다. restored.json의 문서3/receipt draft2/output3·automaticfalse도 직접 대조했다.

## 남은 실제 document 경계

이 fixture의 scope는 실제 StateProgram에서 생성했지만 InteractiveDocument가 아니다. 실제 renderer node/event/disabled 검사·CallbackBindings export 교체·Native/Web listener·원본 TSX 동작·GUI checked-present를 입증하지 않는다. registry_topology=single_registry_headless를 제품 실제 document 완료로 확대하지 않는다. [renderer 인계](../../player/presentation/shared/proposal-renderer-handoff.md)와 [SDK 제품 등록](../../../plans/player/presentation/proposal-product-registration.md) 순서를 유지한다. 전체13화면군/모든 하위탭의 시각·키보드·패널 검증은 남는다.
