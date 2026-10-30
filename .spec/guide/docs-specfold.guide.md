---
id: guide:kinekino/docs-specfold
title: 문서 관리(specfold) 사용법
status: active
created: 2026-10-29
updated: 2026-10-29
---

# 문서 관리(specfold) 사용법

설계·결정·계획·근거 문서는 모두 저장소 루트의 `.spec/`에서 specfold(`spec` CLI, 0.2.6 이상)로 관리한다. `docs/spec/`의 옛 경로와 id는 `.spec/_aliases.tsv`로 계속 해석된다.

| 내용 | 타입 | 예 |
| --- | --- | --- |
| 지금 유효한 규칙 | sot | [[sot:kinekino/conventions]], [[sot:kinekino/project-status]] |
| 결정과 이유 | adr | `.spec/adr/` |
| 작업 계획과 진행 기록 | plan | `.spec/plan/` |
| 검증·측정 기록 | evidence | `.spec/evidence/` (옛 `docs/spec/sots/verification`) |
| 조사 | research | `.spec/research/` |
| 디자인 시안 | design | `.spec/design/` |

- 코드를 바꾸기 전: `spec context <path>`
- 새 문서: `spec new <type> --title "…"` (디렉토리를 직접 고르지 않는다)
- 커밋 전: `spec check --staged` 또는 `cargo xtask spec-check`
- 알려진 기존 결함(이 저장소에 없는 meari/runtime 경로 인용)은 `.spec/_baseline.json`에 있고, 새 결함만 실패한다.
