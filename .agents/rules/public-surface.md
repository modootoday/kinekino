---
paths:
  - "README.md"
  - "docs/**"
  - ".spec/**"
  - "THIRD_PARTY.md"
---
# 공개 표면 규칙 [AGENTS.md; 용어 목록은 사용자 결정 2026-11-03 전달, 근거 문서 없음]
- `.spec`·추적 파일·커밋 메시지에 비공개 흔적을 넣지 않는다. 금지어 목록은 공개 저장소에 두지 않는다. PR마다 필수 체크 `Public surface`(CI 시크릿 `KINEKINO_DENY_TERMS`, 선택 추가 목록 `KINEKINO_DENY_TERMS_EXTRA`와의 합집합)가 검사하고, 로컬 사본(`.agents/local/`)으로 사전 검사할 수 있다. [2026-11-16 개정]
- 공개 문서는 `.spec.local`을 링크하지 않는다. 커밋 메시지는 일반적인 표현.
- 문서는 `spec` CLI로만 만든다(`spec context` → `spec new … --write`), 파일명 직접 작성 금지.
- 흔적 후보(호스트 약칭·계열 토큰, 개발 호스트의 절대 경로 패턴, 작업 머신 지칭어)와 정밀 스캔 도구는 `.agents/local/scan.md`(추적 안 함)에 있다. 일반어와 겹치는 적중은 문맥으로 판정한다. [사용자 결정 2026-11-03 전달, 근거 문서 없음]
- 일반 표현: 호스트 → "개발 호스트"/"dev host", 저장소 경로 → `<repo>`, 그 밖의 로컬 경로 → `<private-path>`(상세는 .spec.local, 공개 문서에서 링크 금지). [사용자 결정 2026-11-03 전달, 근거 문서 없음]
