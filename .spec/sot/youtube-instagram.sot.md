---
id: sot:kinekino/youtube-instagram
title: YouTube 커뮤니티·Instagram 카드뉴스
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# YouTube 커뮤니티·Instagram 카드뉴스

`빈 템플릿/선택 프레임/선택 자막 구간 → 카드 순서 → 문구/레이아웃 → 플랫폼별 미리보기 → 이미지 묶음` 흐름. 원문 인용과 요약문을 구분하고 자동 요약에는 회차/시간/cue 근거 연결을 유지한다. 카드 순서 변경·복제·삭제/되돌리기, 표지/본문/마지막 카드, 통일 폰트·팔레트·페이지 번호, 긴 한글 문구 넘침 감지를 제공한다.

공통 내용에서 플랫폼별 캔버스를 파생한다. 비율 변경 시 글자 크기·줄바꿈·피사체 크롭을 재검토하고 단순 전체 이미지 왜곡 확대를 하지 않는다. 결과는 `01-cover.png`, `02-....png` 등 순번 이미지, 게시문 초안 `.txt`, 대체텍스트 초안, 편집 가능한 프로젝트로 묶는다. 문구·프로젝트는 로컬 저장하며 플랫폼 게시 성공을 표시하지 않는다.

구현·실제 CLI 검증 범위는 [accepted Card review PNG export 정본](card-review-export.md)을 따른다.
