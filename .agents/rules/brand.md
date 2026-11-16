---
paths:
  - "assets/**"
  - "crates/**/presentation/**"
  - "crates/**/ui/**"
  - ".spec/artifacts/design/**"
---
# 브랜드 자산 [ADR 20261031111316-ui-mockups-and-mascot-brand; 경로 세부는 사용자 결정 2026-11-03 전달, 근거 문서 없음]
- 아이콘은 브랜드 원본(위치는 `.agents/local/brand.md`, 추적 안 함)의 실제 마스코트에서 파생: 3D `model3d/out/kinekino-mascot.glb`, 최신 렌더 `model3d/shots/v8`, 2D `logo/`.
- 로고는 눈 없는 첫 프레임.
- `favicon.svg`(고양이 실루엣)·`brand.svg`(재생 삼각형)에서 파생 금지.
- 빈 상태·오류 화면은 설계(ADR 위)에 기록된 마스코트 상태로 매핑.
- 브랜드 원본의 로컬 경로는 공개 파일에 적지 않는다.
