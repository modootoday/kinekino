---
name: test-first-slice
description: kinekino 조각(큐의 한 행)을 테스트 선행으로 구현할 때. 실패 테스트 → 구현 → 게이트 → 인계 보고 순서.
---
# 테스트 선행 조각 구현
1. `spec context <바꿀 경로>`로 관련 plan·ADR·SoT를 읽는다.
2. modoo.today skills에서 도메인 스킬을 찾는다(`modoo-skill-lookup` 스킬).
3. 실패하는 테스트만 먼저 쓰고 실패를 확인한다(출력 요약 보관).
4. 최소 구현 → 새 테스트 통과 → 전체 게이트(rules/build-and-test.md).
5. 커밋하지 않는다. 아래 형식으로 끝낸다.

## 인계 형식
- 바꾼 파일 목록 / 새 테스트 이름 / 게이트 결과(숫자) / 시안과 다른 점 / 남은 일 / 적용한 스킬(출처·판단)
