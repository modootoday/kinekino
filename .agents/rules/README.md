# 규칙 색인 (정본)
프로젝트 컨벤션의 정본. 에이전트 메모리·루틴 프롬프트에 있던 규칙도 여기로 옮겨 다른 도구도 읽게 한다. 각 줄 끝 [근거]. "근거 문서 없음"(사용자 결정) 항목도 추적한다. 개발 호스트 경로·명령·비공개 용어·운영 기록은 `.agents/local/`(추적 안 함, 색인 `local/README.md`)에 두고, 추적 규칙에는 일반 표현과 포인터만 둔다. [2026-11-16 원 님 지시]
| 파일 | 로드 |
|---|---|
| git-and-release.md | 항상 |
| routine-operations.md | 항상 |
| dev-tools.md | 항상 |
| build-and-test.md | *.rs, Cargo.* |
| host-plugin-boundary.md | crates/, src/ |
| product-architecture.md | crates/, docs/ |
| public-surface.md | README, docs/, .spec/, THIRD_PARTY |
| visual-review.md | UI·assets·시안 (PotPlayer, 재생목록, 작품 탐색 버튼) |
| brand.md | UI·assets·시안 |
| ../local/*.md (추적 안 함) | 항상(있으면). 추적 규칙이 가리키는 로컬 값 |
