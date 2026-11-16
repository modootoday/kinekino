---
paths:
  - "crates/**"
  - "docs/**"
---
# 제품 구조 결정 (AGENTS.md에서 옮김, 근거 ADR 20261029113521·20261030050042)
- GUI와 AI 친화 CLI는 같은 application 유스케이스와 domain 정책을 쓴다. 기능을 GUI 이벤트나 CLI 명령에만 구현하지 않는다.
- Meari를 소비하는 React/TSX Web UI를 병행하며, Web/Native는 Meari React Router 호환으로 경로·탭·데이터 전달을 공유한다. 독자 router를 전체 호환으로 표시하지 않는다.
- 제품·환경별 기능은 Meari 공통 플러그인 계약으로 확장한다. 제품 정책은 공통 application/domain에 둔다.
- 표시 제품명·출력 파일명·패키지 메타데이터는 한 책임의 제품 식별 설정에서 관리한다. ID는 표시 이름에 의존하지 않는다.
- 기존 비 Rust 코드는 이관 대상으로 추적하고, 대체 검증 전 삭제하지 않는다.
- 모델은 HF revision·라이선스·정밀도를 고정한다.
