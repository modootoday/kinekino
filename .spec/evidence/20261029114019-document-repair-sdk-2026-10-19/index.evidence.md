---
id: evidence:kinekino/20261029114019
title: 실제 R3 문서 수동 수정 Native 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 R3 문서 수동 수정 Native 인수

2026-10-19. 제품 `studio/subtitles/document-repair.tsx`와 공통 `verification/proposal-review-intent.tsx` 두 원문 모듈을 실제 Meari parser→ModuleProgram→InteractiveDocument로 소비했다. SDK·Meari 소유 담당의 입력 동결 뒤 실행했으며 root 제품 등록이나 GUI 창은 포함하지 않는다.

## 실행과 입력

독립 consumer는 `tools/player-ui-sdk-bridge`, 실제 테스트는 `tests/document_repair_sdk.rs`다. `runtime/quality-review/player-ui-document-repair-r1/gate-r1`에 owned source snapshot·shared/owned/root Rust pins와 stdout/stderr/receipt를 보존했다. 검증된 resource-runner SHA256 `692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463`, affinity3/2 logical CPU/BelowNormal/descendants job, 기존 cache, offline/locked/-j1/RUST_TEST_THREADS=1을 사용했다.

Native 선택 테스트는 1passed/0failed/0ignored, receipt exit0/142641ms, 테스트 본문0.89s다. 이 선택 실행의 컴파일·기능 수리는 없었다. 전체 회귀 최초 실행은 테스트 helper `support/typed_review_case.rs:194`에서 Result에 ok_or를 호출한 E0599로19332ms/101이었다. gate-r1의 원 stderr와 typed-review-case-before-result-fix.rs를 보존하고 해당 호출만 `module_projection()?`로 수정했다.

retry-result/full-test는102339ms/101이다. R3/기존 Native·callback·actual3·view14를 합친20pass 뒤 원본 Card/Highlight intake1이 실제 map 미지원으로 실패했다. 기본 fail-fast로 미실행된 sdk_files는 별도로4pass/0failed/0ignored,2616ms/0을 확인했다. 합산24pass/1fail/0ignored이며 전체 green이 아니다. strict Clippy(all targets,-D warnings)는29664ms/0, cargo fmt --check는246ms/0이다. 원본 intake를 삭제·ignore·flatten하지 않았다.

독립 lock SHA256 `c4091684061e724f1aee2f79fa7f7bff4f7dcd54f38d473c1af432dfa1d141dc`, root lock `50323b23fb1bbf496e3ad42be71b4ccbdc3e1f38de1de5a5154a1ba5d78da5f7`를 유지했다. gate-r1/final-owned-pins.json과 shared/root Rust 전후 pins·차이 배열을 보존했으며 측정한 shared/root Rust 차이는0이다. 수정한 helper 최종 SHA는 `2b66a822a3f9f205126671e4956699bdf82ef9db7428394853ebd52510878a27`이다.

Meari의 허용된 미등록 keyed_reconciliation.rs 추가는2026-10-19T11:26:46.7783567Z에 관측되며 파일 metadata를 보존했다. 부모 source_modules.rs에 등록이 없고 실제 meari-add7253f05337ea9 dep-info에 해당 파일이 없는 것을 확인했다. 소비한 source-after inventory SHA는 `3a2295a3c7d478842314b1ecb7700f9bc01e0d632784ccfc41694fcbb375361e`다. 이 gate는 직접 parser/ModuleProgram/Native 경로이며 builder tree hash를 호출하지 않는다. 전체 Meari 트리가 전기간 불변이었다고 주장하지 않는다. 최초 빈 비교 결과 출력의 파일 생성 누락도 보존하고 명시 배열로 다시 출력했다(input-boundary.json).

실제 R3 package `659b3cb6757259d8e770f6d3c96c4a77dafbab43c6ea540bcc7e0c58e4eb0197`의 명시 item `gemma270-correction`을 기존 inspect_review/read_review_text_asset로 읽었다. 단일 candidate asset임을 검사하며 source companion과 raw를 구별한다. source_import의 canonical adapter로 DocumentSourceSnapshot을 만들고 NeedsRepair/draft=None/import_error 상태로 시작한다. evidence_verified는 패키지 증거 무결성 확인이며 후보의 품질 승인이 아니다.

## 확인한 경로

실제 Native disabled 시작/초안 입력 거부, 원문 번호0→InvalidSelection·선택/draft 없음·SDK 호출0, 번호1 명시 선택, 별도 수동 시작→저장 draft 없음, Unicode `명시 문서 수정 日本語 🦀` 입력→저장 호출 없음, 명시 Save/Accept/Refresh를 확인했다. SDK 호출은 edit/inspect/accept/inspect/inspect 순서이며 accept 후 첫 store load에 기술 Timeout을 주입한다. 저장·CAS·receipt 검증은 기존 SDK에 위임한다.

잘못된 Document 작업 Cards/Highlights와 Editable+draft=None은 ReviewSession 생성 시 canonical validation에 의해 InvalidSnapshot으로 거부된다. working draft가 생긴 뒤 재시작·재선택은 ExistingDraft로 거부된다. UI에 별도 저장 정책을 복제하지 않는다.

실제 case `runtime/quality-review/player-ui-document-repair-r1/10996-1792409164444556400`의 selection/restored/source-import JSON을 보존했다. 선택 참조는 source SHA `b282db497133af492690d5d8726549cbafa7dd203517476e64e915c8101b2e27`, outer pointer `/messages/0/content`, extraction `asr_request_after_no_think_lf_v1`, inner pointer `/0`다. 새 SDK/store 객체로 재열고 전체 canonical state·raw·historical import_error·선택 참조·다국어 수정 텍스트를 비교했다. document revision3/receipt draft2/output3, automatic application=false다.

## 남은 범위

같은 테스트 process의 새 SDK/store 재열기이며 fresh CLI process 검증은 아니다. 스크립트로 명시한 기술 선택·수정이며 실제 사용자의 품질 승인이 아니다. generation_provenance_bound/quality_decision_known=false를 유지한다. GUI 시각·폰트·전체13화면·Web·root 등록·실제 provenance 누락/손상/불일치 오류는 인수하지 않았다.

Card/Highlight 원본 graph의 keyed map 미지원과 이 scalar 경로는 구별한다. 실제 intake case `runtime/quality-review/player-ui-review-graph-intake-r1/56968-1792409376706699800`의 양쪽 최초 진단은 components/proposal-source.tsx538..609 unsupported(`only string/bool literals and props.name supported; hooks/effects/expressions unsupported`)다. [유형별 후속 계획](../../../plans/player/presentation/review-type-native-follow-up.md)을 따른다.
