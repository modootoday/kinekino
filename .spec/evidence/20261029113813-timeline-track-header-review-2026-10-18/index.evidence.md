---
id: evidence:kinekino/20261029113813
title: CLIP-12 트랙 헤더 SVG 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CLIP-12 트랙 헤더 SVG 검토

2026-10-18. 기준 short.png의 트랙 label rail274,724,78,170을4배 확대했다. 파생물 `runtime/design-inspection/clip-track-labels-r50-4x.png`. 영상 filmstrip·사각T·음표의 형태와 라벨의 행 중앙 정렬을 확인했다. 기존 CLIP은 텍스트만 표시했다.

## 공용 구현

직접 작성한 track-video.svg/track-subtitle.svg/track-audio.svg를 assets/design/icons에서 관리하고 Icon의 의미별 매핑에 등록했다. 24viewBox/white1.75stroke/round cap·join 규약이다. TimelineTrackHeader는 Video/Subtitle/Audio typed kind별 glyph와 용어를 소유한다. icon20·rail80·left8·gap6·body13·TEXT 색을 기존 공통 토큰으로 적용하고 부모 clip과 rail clip을 교차한다. 별도 선택·음소거 기능을 암시하지 않는 표시 컴포넌트다.

## 실제 실행

`runtime/review-runs/ui-track-header-20261018-r50/player.exe`, PID89792, --ui-demo의 명시적 DesignSample. 기존 가상 모니터3500,100에서1440×900/760×480을 검토했다. 크기는 데모의 기존 선택 기능을 사용했으며 VM·DPI·모니터 구성은 변경하지 않았다.

- CLIP1440: 세 SVG가 렌더되고 각 영상/자막/오디오 라벨과 수직 중앙에 정렬됐다. label rail 뒤에서 기존 filmstrip grip·자막 block·파형·재생선 위치가 유지됐다.
- CLIP760: 본문 하단으로 스크롤하여 세 아이콘과 오디오 전체 라벨이 rail80 안에 잘리지 않고 보이는 것을 확인했다. 고정 헤더/푸터로 번지지 않았다.
- 설정 패널 열기→Escape 닫기 후 본문 스크롤 위치·트랙 헤더·52초 재생선을 유지했다. 입력 감지로 거절된 시도는 새 상태 관찰 후 재시도했으며 제품 실패로 판정하지 않았다.
- Empty760: 세 트랙의 icon/label 구조는 유지되고 반복 이미지·구간·grip·파형·재생선·가짜 시간눈금이 제거됐다. 연결 필요 문구와 비어 있는 track만 남았다.

화면은 도구 반환 이미지로 관측했으며 별도 저장 screenshot을 주장하지 않는다. 캡션 닫기 후 player/mpv 프로세스 조회가 비어 있음을 확인했다.

## 검사·남은 범위

최초 컴파일의 Ui 가변 참조 오류를 수정했다. 수정 후 cargo fmt --check, 전체 cargo test(80pass/8ignored), cargo clippy --all-targets -- -D warnings와 player 빌드가 통과했다. 검사와 실제 시각 검토는 별도 근거다.

실제 미디어 track 목록/숨김/음소거, 다국어 label resource·긴 번역, 타임라인 선택·편집 상태는 미검증/미구현이다. 이 변경을 CLIP 전체 시각 일치 완료로 표시하지 않는다.
