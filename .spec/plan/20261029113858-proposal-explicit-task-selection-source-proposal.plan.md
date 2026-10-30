---
id: plan:kinekino/20261029113858
title: 세 작업의 명시 typed 선택 — 다음 비겹침 소스 제안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 세 작업의 명시 typed 선택 — 다음 비겹침 소스 제안

## 후속 구현 상태

PM exact3 승인 후 runtime/development/ui-project-task-choices-r1에 private 작성·동결했다. source3 SHA7058FDCF49482C3DDC0F589768F3332E652D97A653967C3B0D827D44ED1E7AA3 / before3 B6A91BB34014C92C9EAB0648A922E5CC97CA6534C1BE3CDE48EB192A16A513FA, 기존1/new2/preimage 불일치0. 작성6/compile0/actual0, direct leaf3 fmt/check만 통과했다. 해당 HANDOFF.md에 exact 시험·의미·미연결 범위를 기록했다. 아래 제안의 route/scoped adapter는 계속 미구현·별도 배정이다. JOB source46과 자동 합성하지 않는다.

2026-10-23 root8014701/libraryb532 기준. JOB r5 동결 뒤의 읽기 제안이며 구현·실행0이다. 기존 proposal-default-route-gap-audit.md를 재사용한다. 이 제안은 선택 값만 작성하며 Viewer route/실행 조립은 별도 배정한다.

## 첫 배정 후보 exact3

source3 독립 읽기 검토에서 새 확정 P1/P2 없음. generation은 화면 교체마다 원 caller가 새로 공급해야 하며, 독립 프로젝트에 같은 generation을 재사용한 key의 출처를 이 모듈이 인증하지 않는다. 향후 callback 연결은 resolve 전에 원 owner/scope를 검증해야 한다. 작성6/compile0/actual0을 유지한다.

- 신규 src/presentation/web_ui_adapter/proposals/project_task_choices.rs: 저장 프로젝트에서 Card/Highlight/CueCorrection/DocumentCorrection의 명시 표시 행과 typed 선택을 만들고 원 generation의 key만 resolve한다.
- 신규 src/presentation/web_ui_adapter/proposals/project_task_choices/tests.rs: 혼합 종류·원 identity 보존·잘못된/이전 generation key·빈 항목·index overflow·명시 선택 회귀.
- 기존 src/presentation/web_ui_adapter/proposals/mod.rs: 같은 native-proposal-window cfg의 모듈 등록만 한다. JOB source46와 겹치지 않는다. 기존 Card adapter/capture 계약은 유지한다.

NativeProposalSelection/Target은 기존 application 계약을 그대로 소비한다. 행은 원 capture의 proposal_id/draft_revision/source_id/source_revision/source_sha256와 명시 항목 index 및 cue_id/source_ref를 보존한다. key는 원 frame generation에 묶는다. 첫 항목 자동 선택, callback에서 authority 역직렬화, capture→live 권한 승격은 없다. live_inspection_selection은 SDK inspect를 위한 입력이며 편집 권한 증명이 아니다. DocumentRepair의 명시 source_ref는 별도 선택 입력이 필요하므로 단순 첫 텍스트를 추정하지 않는다. 그 UI 입력과 연결 전에는 repair 미선택 상태를 표시한다.

선택 이후의 fresh inspect는 기존 StoredProposal{document,acceptance}와 기존 bootstrap/controller가 판단한다. stale/deleted/unsupported 응답은 그대로 반환하며 SDK 정책이나 CAS/accept를 이 모듈에 복제하지 않는다. 원문·생성안·사람 수정안·명시 채택의 책임을 유지한다.

## 후속 별도 소유 제안

위 선택 모듈 인수 후 신규 correction/highlight scoped adapter를 기존 src/presentation/web_ui_adapter/proposals/scoped_panel.rs 및 document_factory/bootstrap의 공통 조립으로 설계한다. 이 후속은 factory/controller/worker를 새로 복제하지 않고 기존 ProductWindow 세 패널과 ProposalOutputReview를 재사용한다. Viewer SUB/CARD/CLIP route wiring·복귀·재개 inspect의 exact 경로는 해당 연결을 읽은 뒤 별도로 제시한다. 현재 Card-only capture surface를 모든 종류의 쓰기 권한으로 확대하지 않는다.

## 검증과 한계

첫 범위는 Rust 순수 선택 정책 시험과 기존 Card 영향 회귀 후보이며 실제 SDK/Native/window 검증은0이다. Cargo 실행은 PM 배정 뒤 단일 gate에서 fmt/test/clippy 및 실제 시험 목록을 기록한다. 이후 route 통합은 실제 NativeHostRuntime/원 SDK 응답/오류/owner 정리를 별도 입증한다. 기존 CLI/GUI 과거 증거는 재사용 근거이며 현재 연결의 실제 결과로 세지 않는다. seek/clip/SRT/모델·미디어 기능, 기본 런처 전환, 수동 시각 인수는 이 범위에 없다.

적용 출처: modoo.today modootoday-free/design-delivery-plus/frontend-first-journey의 공통 typed port와 기존 컴포넌트 재사용; modootoday/software-qa/test-fixture-and-mock-build의 원 identity/owner 및 합성 정책·실제 실행 증거 분리. 기존에 읽은 본문을 적용했으며 타 언어 개발 코드는 추가하지 않는다.
