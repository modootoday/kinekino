---
id: plan:kinekino/20261029113817
title: 원본 Card·Highlight SDK 소비 인수 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원본 Card·Highlight SDK 소비 인수 준비

최신 실행은 [원본 Card·Highlight Native SDK 인수](../../../sots/verification/presentation/original-review-sdk-2026-10-19.md)로 완료했다. 선택2/전체27passed·failed0·ignored0, strict/fmt0, 코드 수리0이며 아래 준비/미실행 문장은 당시 이력이다. 이 기능 gate 완료는 제품 root/window·freshCLI·사람 품질·전체 시각 인수의 완료가 아니다.

상태: Meari Repeat 최종 checkpoint와 SDK 입력 동결 후 실행할 준비 계획. `tests/original_review_sdk.rs`에 두 유형의 top-level SDK fixture·typed props 조립을 추가했다. 아직 컴파일·실행 전이며 소스 helper 준비·본체 parser/mount 성공은 제품 SDK 경로 성공이 아니다. 현재 새 cargo/GUI/IAB 실행은0이다. [제품 유형별 상태](review-type-native-follow-up.md), [factory 계약](../../../sots/player/presentation/shared/proposal-document-factory.md)을 따른다.

## 준비 입력과 fixture

원본 Card/Highlight 각3모듈은 verification의 card-proposal/highlight-proposal source-graph inventory를 사용한다. 공통 ProposalSource의 details/summary와 keyed sourceRefs.map, ProposalActions 및 원본 콜백·독립 field는 그대로 둔다. ReviewCase.sources/parse/mount/bindings/exercise가 같은 원문과 실제 Native API를 소비한다. typed refs는 공개 runtime::IntoValue의 Vec<String>로 전달한다. 이 후속 helper 변경은 마지막 R3 gate 이후 미컴파일 상태다.

첫 연결 회귀는 명확히 기술 fixture로 분류한다. 기존 ReviewCase.seed의 Card 앞/뒤 또는 Highlight 제목, canonical cue-a/cue-b 두 원문과 두 source_refs, None 시간, 명시 source revision/SHA를 실제 PlayerSdk/CreateProposal/ProposalFiles에 넣는다. raw는 typed seed를 직렬화한 입력으로 보존하며 create 뒤 Valid draft에서 시작한다. 실제 모델 결과나 R3 원 candidate 품질로 표시하지 않는다. 이전 실제 R3 Cards/Highlights SDK assets 회귀는 계속 별도 유지한다.

파일 저장 어댑터는 기존 Native 회귀와 같이 ProposalFiles에 위임하고 accept publication 후 첫 inspect load Timeout만 한 번 주입한다. SDK client/context factory/단일 registry/RegistrationHandle/RegistryProposalHost/ProposalDocumentSession은 기존 제품 경로를 재사용한다. 별도 CAS·receipt·idempotency·검증 정책을 만들지 않는다. immutable source_refs·raw·원문과 새 SDK/store 재열기 결과를 artifact로 보존한다.

## 실제 인수 순서

| 단계 | 실제 확인·보존 |
|---|---|
| 원본 intake | 최신 세 모듈 bytes/hash와 실제 parser 결과. 실패하면 최초 file/code/span을 그대로 보존하고 원본을 단순화하지 않음 |
| Native mount | 실제 module scope factory/registry 공유, refs 두 개의 순서·각 행 본문, disclosure 유지, disabled Save/Accept 및 typed callback 연결 |
| 독립 입력 | Cards front 변경 뒤 back·refs 불변, 이어 back 변경 뒤 front·refs 보존. Highlight title 변경 뒤 두 refs·원문/None 시간 보존. Unicode 본문 유지 |
| deferred mutation | Native callback enqueue·dispatch 반환 전 canonical store 불변, drain에서만 working draft 변경, 명시 Save까지 SDK mutation 없음 |
| stale/reproject | old field/drained callback을 보관하고 fresh field props/export 교체 뒤 stale 거부. 실패 props 적용은 last-good/old lease 유지. stale input 뒤 Save가 연쇄 실행되지 않음 |
| 명시 Save/Accept | SDK edit/inspect 후 revision과 typed payload 대조, accept/inspect Timeout 뒤 확정 receipt를 유지하고 mutation 재실행 금지 |
| 명시 Refresh | inspect만 재시도해 document3/receipt draft2/output3·automaticfalse 확인. capability sequence edit/inspect/accept/inspect/inspect 유지 |
| close/reopen | Native callbacks→UI session→SDK unregister, old handle 거부. 같은 process의 새 SDK/store로 canonical snapshot/raw/refs·receipt 재열기 비교 |

위 경로를 실행할 top-level fixture 조립과 typed props 생성은 `tests/original_review_sdk.rs`에 작성했다. 각 유형 별도 테스트는 실제 SDK/client/store와 원본 scope factory를 조립하고 Native input/Activate→dispatch/drain→fresh props를 사용한다. 매 입력 뒤 두 참조의 실제 text run이 각각1개인지 확인하며 잘못된 sourceRefs 누락 재투영 실패 뒤 old export 생존·성공 뒤 old handle stale·close 뒤 현재 handle stale를 검사하도록 준비했다. 아직 실제 실행하지 않았다. first Card/Highlight 선택은 이 기술 fixture의 명시 singleton이며 제품 factory의 사용자 선택 계약을 대체하지 않는다. 기존 exercise 함수나 새 테스트 소스를 성공으로 집계하지 않는다.

## gate·동결·경계

본체 최종 green 및 SDK 명시 동결을 받은 뒤 owned snapshot과 pins를 고정한다. 동결 범위는 root Cargo/lock/SDK application·domain·소비 infrastructure, Meari 등록 Core/Native/paint·manifest/lock, media-contracts/media-subtitles/player-ui-plugins, src/assembly/meari/proposals.rs, 제품 web_ui_adapter/proposals, 소비 TSX/inventory, tools/player-ui-sdk-bridge manifest/lock/src/tests다. 미등록 파일 추가와 등록 compile input을 구별하고 전체 tree 불변을 임의 주장하지 않는다.

중앙 단일 heavy 슬롯·검증된 runner(2CPU/BelowNormal/descendants)·기존 cache·offline/locked/-j1/RUST_TEST_THREADS1을 유지한다. 원본 두 유형 실제 SDK 선택 실행→기존 R3 NeedsRepair 포함 전체 cargo test --no-fail-fast→strict Clippy --all-targets -- -D warnings→fmt --check 순서다. 새 fixture 컴파일 최초 실패를 보존하고 UI 소유 범위만 최소 수리한다. shared 입력 변경은 owner에 먼저 조율한다. 마지막 전체 테스트의 기본 fail-fast로 잔여 sdk_files를 따로 실행했던 이력을 반복하지 않도록 전체 target을 끝까지 수집한다.

종료 시 원본 bytes/pins/첫 실패 및 수리/각 receipt와 실제 통과·실패·ignored 수를 정본화하고 슬롯·freeze를 명시 반환한다. 실제 player.exe root host 등록·window/event loop hook·fresh CLI·project binding·원 candidate와 수정 output 품질·전체13화면/시각/virtual monitor 인수는 별도 미완료다.
