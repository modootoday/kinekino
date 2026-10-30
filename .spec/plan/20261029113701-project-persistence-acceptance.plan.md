---
id: plan:kinekino/20261029113701
title: Project persistence implementation and acceptance
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Project persistence implementation and acceptance

패키지 분리 checkpoint의 CLI·비GUI 검증은 완료했고, focus 단독 검사·고정 실행본 인계 후 SDK 담당이 프로젝트 구현을 이어간다. 아직 projects 명령 지원을 주장하지 않는다. [projects 계약](../../sots/player/application/api/projects.md)과 [현재 API 계획](plan-006-sdk-gui-cli.md)을 따른다. jobs/native/catalogue 후속 범위는 유지한다.

## Boundaries on lease return

- Domain owns project revision, selected source locator, captured proposal snapshot, working-copy state and validation. It must not import application/API errors, Request, storage or UI.
- Application owns capture against ProposalStore revision, nonempty context, project CAS and ProjectStore port. It imports shared contracts errors directly; Request dispatch stays in the API façade.
- Infrastructure owns local non-reparse rooted directories, exclusive writer handle, bounded hash-addressed immutable records, fsync and incomplete/corrupt publication errors. Shared filesystem machinery is a named responsibility, not a generic utils module.
- Main/CLI inject explicit project storage. Typed client methods are the only UI path. Existing proposal Source1/Source2 and accepted snapshots remain unchanged.

## Required scenarios

1. Default backend remains unsupported, including absent payload, and capability/schema agree in CLI and object-safe client.
2. Empty watching/proposal context is invalid. A declared local-video or canonical episode-page selection persists as locator-only; it neither opens a URL nor claims current media/position/source PTS verification.
3. Create with an actual9 proposal ID and exact saved revision captures source/raw/draft/acceptance and canonical snapshot SHA. Unknown/mismatched/repeated proposal refs fail before reserving an ID.
4. A working copy persists separately from captured accepted output. Correct task/source kind and bounds are mandatory. Empty text/unselected refs remain needs_input; valid and invalid working copies do not become an acceptance or mutate the original proposal.
5. A fresh CLI and a fresh SDK adapter read the same complete project revision without the proposal backend. Source/raw/captured result and working-copy bytes are unchanged.
6. Save requires expected project revision. Competing saves for the same revision have exactly one publication; stale or ambiguous-response retries cannot silently create another revision. Inspect recovers the stored outcome.
7. Historical inspect returns immutable prior content. A later proposal edit does not change earlier project snapshots. Cross-root latestness/transactions are not claimed.
8. Missing/extra/torn/invalid-hash files and noncontiguous revisions fail closed; earlier state is not silently substituted. Oversized state fails before reserving a project identity/revision.
9. Root escape, UNC/reparse storage roots, unknown fields/schema and invalid selectors are rejected. Saved locator metadata never launches a shell/player.
10. 기능 판정은 공통 SDK/고정 CLI의 저장·재시작·CAS·손상 거부 근거를 사용한다. GUI는 별도 복사 fixture와 고정 실행본에서 typed source/work/result/충돌 상태의 표시·입력 보존·API 연결 UX를 검토한다. 동일 기능을 GUI에서 반복 시험하거나 GUI 자동화로 CLI 미연결 기능을 대체하지 않는다.

11. `origin: saved`는 CAS 기준 프로젝트 snapshot의 ID/capture SHA를 그대로 재사용한다. proposal backend 없는 fresh CLI에서 buffer를 바꾸고 저장·재조회한다. create의 saved 입력, 없는 ID, 다른 SHA는 거부한다. live 조회 실패를 saved로 대체하지 않는다. record3MiB/working 합계512KiB 초과는 ID 예약 전에 거부한다.

## Gate and evidence order

After packages phase1/2 verification returns the source/native target lease: implement the source contract, run fmt/test/clippy/bins, pin both binaries, execute actual CLI create/read/save/conflict/corruption/restart, then hand a copy to UI. Preserve any failed checkpoint separately. No new model inference, download, benchmark, VM/DPI or user-app termination is required.
