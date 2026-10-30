---
id: evidence:kinekino/20261029113841
title: 원본 Card·Highlight Native SDK 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원본 Card·Highlight Native SDK 인수

2026-10-19. `tools/player-ui-sdk-bridge/tests/original_review_sdk.rs`가 제품 원본 Card/Highlight 각3모듈을 실제 parser→ModuleProgram→InteractiveDocument→제품 session/controller→SDK/ProposalFiles 경로로 소비했다. [gate 계획](../../../plans/player/presentation/original-review-sdk-gate.md), [제품 factory 계약](../../player/presentation/shared/proposal-document-factory.md)을 따른다. 제품 root 등록과 실제 window 실행은 포함하지 않는다.

## 입력과 실행

SDK와 Meari 소유자의 동결 응답 후 `runtime/quality-review/player-ui-original-review-sdk-r1/gate-r1`에 소스 snapshot·owned/shared Rust/manifest pins·stdout/stderr/receipt를 보존했다. Meari r11 source-after SHA256은 `10ec036aae146ae7ee4fe2b409e0f9d9b04f9eb680b4fa69bfdd25ff20dac3d5`, SDK api/mod.rs는 `55216af45998dc09adfee69e7da97942bcc7643732b87a9e0ac6b3e46c36e4f9`다. pin 목록의 옛 api.rs 경로 경고는 api-mod-corrected-pin.json으로 수정 보존했으며 전체 src Rust inventory에는 처음부터 실제 파일을 포함했다.

검증된 runner SHA `692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463`, affinity3/2CPU/BelowNormal/descendants job, 기존 target/cache·offline/locked/-j1/RUST_TEST_THREADS1을 사용했다. 선택 gate는2passed/0failed/0ignored·테스트1.13s, runner82923ms/exit0이다. 전체 --no-fail-fast는27passed/0failed/0ignored·122962ms/exit0, strict Clippy --all-targets -- -D warnings는12107ms/exit0, fmt --check는288ms/exit0이다. 이번 gate의 컴파일·기능·형식 수리는0이다.

전체27은 callback1/R3 Document1/기존 Review Native1/원본 Cards·Highlights2/실제 SDK assets3/view14/원본 graph intake1/sdk_files4다. R3 gate 때의24pass+map intake1fail과 이번 원본 변경 없는 성공을 구별한다. parser-only intake case `83628-1792412640280810400`의 양쪽 status는 parser_verified=true이고 그 테스트 자체의 native/sdk/visual_verified는 false다. 실제 Native/SDK 성공은 별도 original_review_sdk의 두 테스트가 입증한다. 성공 intake에는 first-diagnostic.json이 없으며 status/inventory/원문/typed-reference-value를 보존했다.

gate-r1의 shared-rust-pins 전후 비교0/manifest pins 전후 비교0, final-owned-rust-pins와 owner-checkpoint pins를 보존했다. 신규 테스트 source SHA는 `ba5f5e7b229d449203829be72bd11558109e7481d7c764dfa6ce29433e09b35e`, 독립 lock은 `c4091684061e724f1aee2f79fa7f7bff4f7dcd54f38d473c1af432dfa1d141dc`, root lock은 `50323b23fb1bbf496e3ad42be71b4ccbdc3e1f38de1de5a5154a1ba5d78da5f7`이다. 모든 runner receipt는 terminal=true이고 확인한 소유 launcher/child 생존 목록은 빈 배열이다. 새 GUI/IAB는0이다.

## 실제 검증한 동작

원본 entry는 studio/cards/proposal-panel.tsx와 studio/clips/highlight-proposal-panel.tsx이며 공용 proposal-source/proposal-actions를 그대로 소비했다. sourceRefs의 keyed map·details/summary를 삭제하거나 flatten하지 않았다. 공개 Vec<String>::IntoValue로 cue-a/cue-b를 넘겼고 Native text run에서 각 참조가 정확히1개인지 최초 mount와 매 성공 재투영 뒤 검사했다.

명시 singleton 기술 fixture를 실제 SDK create로 저장했다. Cards의 앞/뒤 독립 다국어 입력, 앞면 수정 직후 뒷면 불변, 최종 두 필드·refs 보존을 확인했다. Highlights의 다국어 제목과 두 참조·원문 시간 None 보존을 확인했다. 입력 callback dispatch 전후 canonical store가 같은지 비교하고 dispatch 반환 후 drain에서 working draft를 변경하며 명시 Save 전에는 SDK capability 호출이 없는지 검사했다.

sourceRefs prop을 누락한 재투영은 실패하고 old export는 유효했다. 올바른 새 props/bindings로 성공 교체한 뒤 old export/실제 입력 callback handle은 stale로 거부됐다. Save/Accept/Refresh는 edit/inspect/accept/inspect/inspect 순서이며 accept 후 첫 store read에 기술 Timeout을 주입했다. refresh는 mutation을 재실행하지 않는다. close는 당시 유효한 handle도 거부했다.

같은 process의 새 SDK/store로 재열고 전체 canonical state·원 raw·두 참조·다국어 최종 텍스트·None 시간을 비교했다. document3/receipt draft2/output3·automatic application=false이며 quality_decision_known/generation_provenance_bound=false를 유지한다. 별도 CAS·도메인 검증 정책은 만들지 않았다.

선택 gate artifact는 `cards-42404-1792412509030086100`, `highlights-42404-1792412509618534700`이며 위 runtime 디렉토리에 original-modules/source/restored/sdk-calls JSON을 보존했다. SDK calls는 plugin ticket factory의 관측이며 테스트의 read-only 직접 inspect는 이 목록에 포함하지 않는다.

전체 회귀 재실행의 두 실제 case는 `cards-26628-1792412632684849200`, `highlights-26628-1792412634683780900`다. 이 반복은 계획한 전체 회귀이며 소스 수리 후 재시도나 새 모델 추론이 아니다.

## 한계

명시 기술 cue/typed raw fixture이며 실제 모델 결과나 사람의 품질 승인이 아니다. 실제 R3 SDK assets 회귀·Document NeedsRepair 회귀와는 별도다. 같은 process 객체 재생성이며 fresh CLI process/project binding 검증이 아니다. 실제 player.exe 등록·window/event loop·전체13화면·시각 확대 비교·폰트/asset fidelity는 미완료다. 본체의 동적 행/키보드/접근성/Web 실행 근거를 이 제품 SDK gate에서 재검증한 것으로 확대하지 않는다.
