---
name: modoo-skill-lookup
description: 설계·구현·검증 작업 시작 시 modoo.today skills에서 도메인 스킬을 찾아 본문을 읽고 적용 판단을 기록할 때.
---
# modoo.today 스킬 조회
연결: modoo-skills 도구(skills.modoo.today; skill_find·skill_describe·skill_read·skill_catalog)를 쓴다. 조회가 실패하거나 도구가 없으면 건너뛰고 인계 보고에 `skill: 조회 불가`만 적는다.
도구(읽기 전용): `skill_find`(인자 `intent`), `skill_describe`, `skill_read`, `skill_catalog`. `skill_call`·`skill_bundle`은 루틴에서 쓰지 않는다.

| 작업 | intent 예 | 2026-11-05 시험 조회로 확인된 후보 |
|---|---|---|
| 테스트 선행 구현 | "rust test-first regression test for player controller" | 적합한 스킬 없음(무관한 결과만) — `skill: 없음` 기록 |
| 버그 분류 | "decide whether a failing regression test is flaky or a product defect" | `modootoday/software-qa/defect-triage-evidence` |
| 공개 표면 | "review a public repository README and docs for stale versions and leaked private terms" | `modootoday-free/security-ops-plus/public-release-leak-audit` |
| 설계 문서 | "write a design plan or architecture decision record for a feature before implementing it" | `modootoday/spec-authoring/plan`, `modootoday/spec-authoring/adr-authoring` |
| 리뷰 | "code review finding verification" | (미확인, 조회해서 확인) |
| 빌드·CI | "build cache correctness audit" | `modootoday-free/build-performance-plus/build-cache-correctness-audit` |

전체 카탈로그: https://skills.modoo.today (511개, `skill_catalog`는 페이지 단위).

- 이름만 보고 채택하지 않는다. 본문을 읽고 이 저장소 규칙(Rust 전용, 호스트·플러그인 경계)과 맞는 부분만 쓴다. 다른 언어 예제를 코드로 들여오지 않는다.
- 기록(인계 보고와 해당 plan Record에 한 줄): `skill: <id> | 적용: <무엇을> | 버림: <무엇을, 이유>`. 맞는 스킬이 없으면 `skill: 없음 (질의: …)`.
