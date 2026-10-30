---
id: evidence:kinekino/20261029114050
title: 공통 제안 controller 실제 SDK gate
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공통 제안 controller 실제 SDK gate

2026-10-19 중앙 배정 슬롯에서 `tools/player-ui-sdk-bridge`의 현재 source를 검사했다. fmt·offline 자체 lock 동기화·최종 locked test16개·strict Clippy가 종료0이다. 실제 TSX callback·renderer·시각 인수·모델 품질 승인은 포함하지 않는다.

## 실행 경계

인계받은 `runtime/quality-review/frame-document-cli-r1/final-bin/resource-runner.exe` SHA256은 `692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463`다. JobObject의 자기/자식 affinity3, logical_cpus2, priority16384(BelowNormal)를 각 receipt에서 확인했다. 공유 native target, offline/locked, cargo -j1, RUST_TEST_THREADS=1을 사용했다. 사용자 앱·VM·DPI·모델을 조작하지 않았다.

원문은 `runtime/quality-review/player-ui-proposal-controller-r1`에 보존한다. `gate`는 최초 실행, `retry-store-fix`는 시험 준비 수리 후 실행, `final-gate`는 Clippy 권고 수리 후 최종 실행이다. 최종 test runner 34088ms, compile29.55초, strict Clippy3.60초/runner3779ms다. 최종 receipt는 모두 terminal=true/exit_code0이다. 완료 직후 슬롯과 소비 source 동결을 반환했다.

root lock SHA256은 전후 `477b61ddf828c185697fb0b6d5fa35231ee14b3b38670e8690a73331faf4fa14`로 유지됐다. UI 독립 lock은 SDK의 windows-core 직접 의존 인계 후 offline 동기화하여 최종 `65f05f3c2a311b8a01b56dda247ba9a6ea619f5efd1d5f3c781057f4160960f2`다. 최종 adapter/controller/test source SHA는 final-gate/source-pins.json에 둔다. root SDK/기존 assembly/Meari core 정책을 UI에서 수정하지 않았다.

## 검사 범위

| 검사 | 실제 입증 |
|---|---|
| proposal_assets3 | 기존 실제 raw9개를 canonical SDK·같은 registry로 create/inspect/명시 기술수정/stale CAS/채택/재열기. 원문·SHA·NeedsRepair·automaticfalse 보존 |
| proposal_view9 | 기존 상태4 + 필드2 + refresh상태1 + controller 실패주입1 + Windows 실제SDK controller1 |
| sdk_files4 | 기존 변환 실파일·오류·요청 문맥·동시 host 분리 회귀 |

실제SDK controller는 ProposalFiles에 기술 fixture를 생성하고 ProposalReviewHost→기존 ProposalBridge→Meari PluginRegistry→ApplicationClient를 통해 필드 수정/저장/fresh inspect/명시 채택을 실행했다. 최종 문서3·receipt draft2/output3, automatic_application_enabled=false다. stored.json은 각 소유 시험 디렉토리에 남기며 기술 fixture를 사람의 품질 승인으로 표현하지 않는다.

실패주입 controller 검사는 edit와 accept 각각 첫 inspect Timeout 후 명시 refresh를 실행했다. 각 경우 mutation1회·inspect2회, 동일 mutation 재전송0회 assert가 통과했다. 이 호출 횟수는 synthetic host 관측이며 실제 네트워크/TSX 입력 관측으로 확대하지 않는다. 별도 상태 검사는 refresh 재실패·오래된 inspect 거부·수정본/확인된 receipt 보존을 확인했다.

카드 앞/뒤와 하이라이트 제목·source_refs를 묶는 ReviewItem과 기존 TextInput의 session/generation 경계도 검사했다. CARD/CLIP TSX 패널·CSS·원본 시안의 시각 일치가 이 Rust 검사로 입증되지는 않는다.

## 실패와 좁은 수리

첫 실행은 actualSDK controller 시험이 store 디렉토리를 생성하지 않고 ProposalFiles::open을 호출하여 NotFound(os error2)로 실패했다. 제품 오류나 이전 ParentDir guard 문제와 구별한다. 해당 store 폴더 생성 한 줄을 추가했다. 원실패 로그와 `gate/proposal_view.before-store-fix.rs`를 보존했고 synthetic 대체/skip하지 않았다.

재실행16개는 통과했으나 strict Clippy의 chunks_exact_to_as_chunks가 실패했다. 경고를 억제하지 않고 고정2개 묶음을 as_chunks::<2>()로 변경했다. `retry-store-fix/fields.before-as-chunks.rs`와 진단을 보존하고 최종 full16/Clippy를 다시 통과했다.

## 남은 제품 연결

[공통 입력 계약](../../player/presentation/shared/proposal-view-input.md)과 [CARD/CLIP source 기록](card-highlight-proposal-source-2026-10-19.md)을 따른다. callback opaque handle API의 source 인계는 받았으나 TSX lowering/renderer guard/Native·Web dispatch는 아직 인수하지 않았다. 해당 기능 보완 뒤 동일 fixture/원본 패널로 입력·저장·refresh를 검증한다. 전체13화면군·모든 하위탭의 시각/키보드/패널 인수는 남는다.
