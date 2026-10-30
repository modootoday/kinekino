---
id: evidence:kinekino/20261029113921
title: SEG Web source의 시안 대응
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SEG Web source의 시안 대응

2026-10-19. 기준 skip-settings.png 전체와 기존 확대 seg-range-detail-2x.png(원본275,575,895,280/2배)를 직접 열어 [SEG 시각 계약](../../player/presentation/studio/segments-visual.md)을 대조했다. crop 좌표는 내부 정밀 bounds가 아니다. source 작성 기록이며 실제 renderer/시각 인수로 세지 않는다.

| 요소 | 기준 | 작성 source | 미검증 |
| --- | --- | --- | --- |
| SEG-01a | 제목·영상·filmstrip/ruler·표 | segments/workspace.tsx의 독립 media/timeline/candidates slot | 미디어·실제frame/PTS·파싱·렌더 |
| SEG-05b | 종류·시작·끝·길이·상태·수정/삭제 | candidate-table.tsx, stable id·selected·종류 label+색 dot·독립 상태 badge | collection/callback 일반 source binding·actual SDK |
| OP/ED | amber/보라 구별, 후보와 확인은 문자로도 표시 | opening=기존 accent, ending=기존 native PURPLE_300(#bca0eb), 상태는 host 문자열 | 영상 overlay·range fill/handle·실제 색 일치 |
| 표/버튼 | time 고정열·행32 이상·hit32·둥근badge | 공통time폭132, compact Button32/icon20, badge24/max240·공통border/radius | dot10은 논리 후보 토큰, 실제 glyph/서체/간격 검토 필요 |

source는 SDK가 준 label과 capability·이유를 소비하며 임의 구간·confidence·확인 상태를 생성하지 않는다. 컬렉션이0일 때 empty 안내도 host 입력으로 받는다. 영상/filmstrip은 이 source에서 샘플을 자동 생성하지 않는다. 선언 route/source 경로를 등록 완료로 간주하지 않는다. compact Button CSS를 subtitle 전용 파일에서 components.css로 옮겨 SEG가 subtitle 스타일을 암묵 의존하지 않도록 했다.

modoo.today design-to-code-fidelity-diff의 expected/built/token/verdict 방식과 기존 palette를 적용했다. 새 GUI·브라우저·VM/DPI·추론 실행은 하지 않았다. 현재 우선 gate는 실 SDK 변환 bridge이며 이 source의 시각 폴리시나 수동 편집 추가를 선행 gate로 확장하지 않는다. SEG 수동편집/검사기·timeline와 나머지 화면군 요구는 후속으로 유지한다.
