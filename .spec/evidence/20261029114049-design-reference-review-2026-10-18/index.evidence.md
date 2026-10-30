---
id: evidence:kinekino/20261029114049
title: 전체 선택 디자인 재검토 — 2026-10-18
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 전체 선택 디자인 재검토 — 2026-10-18

## 범위와 사실

[manifest](../../../artifacts/design/manifest.json)의 selected_studio 19장과 viewing 6장을 원본 이미지로 다시 열어 검토했다. 이는 13개 화면군의 25개 시안 상태다. manifest의 32개 파일 중 나머지 7개는 refined 이전 대안이며 현재 구현 기준에 섞지 않는다. 파일 hash 원장은 manifest를 유지한다. manifest의 과거 review-required 표기는 방향 승인과 실제 렌더 인수가 별개임을 뜻하며, 전 화면 시각 검증 완료로 변경하지 않는다.

## 선택 원본과 매핑

경로는 `.spec/artifacts/design/` 기준이다. 전체 구성을 관측한 결과는 [화면 구성 정본](../../player/presentation/shared/screen-composition.md)에 반영했다.

| 화면 | 선택 원본 | 관측한 핵심 차별점 |
|---|---|---|
| VIEW | v2/viewing/native-player-start-v2.png | 중앙 원형 표식, outlined 시작 동작, 검정 면, 전체 폭 seek |
| VIEW | v2/viewing/native-player-playback-v2.png | 회차 목록·현재 행 강조·분리된 하단 transport |
| VIEW | v2/viewing/native-player-fullscreen-v2.png | 무제목줄 영상, 상단 안내와 하단 overlay |
| VIEW | v2/viewing/native-player-skip-v2.png | OP/ED 범위, 우하단 skip, 확인 상태 |
| VIEW | v2/viewing/native-player-web-v2.png | 분류 탭·검색·작품 목록·차단 상태 |
| SEG 보조 | v2/viewing/native-player-studio-v2.png | 초기 제작 진입 구도. 탐색/기능 세분화는 selected_studio 우선 |
| SEG | v2/studio/skip-settings.png | OP/ED handle, 구간 표·최근 작업·큐 펼침 |
| SUB | v2/studio/subtitles.png | 파형·cue 편집 도구·원문/교정안 |
| SUB OCR | v2/studio/ocr-refined.png | 영상 ROI, 감지/낮은 신뢰도, 우측 텍스트 편집 |
| SUB 교정 | v2/studio/correction.png | 제안 비교·근거·수락/기각 |
| SUB 변환 | v2/studio/convert.png | 원본/출력 문법 두 패널과 손실 안내 |
| OBJ | v2/studio/subject.png | 도구 rail·mask overlay·피사체 목록·추적 확인 구간 |
| CLIP | v2/studio/short.png | 세로 crop와 원본, 영상/자막/음성 track |
| CLIP 추천 | v2/studio/short-recommend-refined.png | 후보 thumbnail·선정 근거·행별 동작 |
| CLIP 일괄 | v2/studio/short-batch.png | 회차 표와 계획 요약 |
| FRAME | v2/studio/raw-frame.png | 이전/현재/다음 frame·PTS·원본 정보 |
| FRAME 편집 | v2/studio/snapshot-refined.png | crop/텍스트 경계, filmstrip, 레이어 |
| CARD | v2/studio/cards.png | 세로 thumbnail rail·카드 canvas·alt text |
| A2D | v2/studio/asset2d.png | checker·8 handle·pivot·레이어·원본 참조 |
| PROJ | v2/studio/projection3d-refined.png | 분리 평면·XYZ·상대 깊이·카메라 |
| PROJ 패키지 | v2/studio/projection-package.png | 파일 tree·thumbnail·유효성 표 |
| SAVE | v2/studio/download-refined.png | 트랙 선택과 미확인 호환성 |
| MODEL | v2/studio/models-refined.png | 기능별 목록·미선정 모델·revision/라이선스 |
| JOB | v2/studio/queue-refined.png | 상태 filter·선택 작업 단계·입력 필요 진단 |
| OUT | v2/studio/export.png | 요약과 저장 확인 분할·충돌 해결 |

## 재발 원인과 정본화 결정

기존 계약은 기능과 필드 중심이며 구체적인 편집 구조가 누락됐다. 그 상태에서 일반 미리보기+표를 모든 화면에 적용하고 토큰을 조정해 시각 일치로 접근했다. 따라서 카드 rail, 변환 비교, crop, scene, 캡션 같은 핵심 구조·세부를 놓쳤다. 테스트의 render 성공은 이 구조의 일치를 검증하지 못한다.

선택 시안끼리도 탐색 폭, 제목줄 높이, 선택 배경, 탭 모양, 초록/amber 절약 모드, 큐 높이와 아이콘이 다르다. 모든 차이를 그대로 복제하면 일관성이 악화된다. 공통 프레임·선택·탭·폰트·캡션은 공유 계약으로 정규화하고 각 화면의 편집 구조는 보존한다. 생성 이미지의 잘못된 글자, 임의 50/100자 제한, 표시 샘플 수치·성공 상태, 특정 모델 가용성은 기능 규격이 아니다. 썸네일의 시스템 글꼴 예시도 UI Pretendard 정본을 바꾸지 않는다.

원본 raster pixel과 논리 UI 단위를 혼동하지 않는다. 창 크기 비율만으로 모든 치수를 확대하지 않고 제목줄·컨트롤·문자 같은 공통 앵커를 정규화한 뒤 요소별 geometry를 비교한다. 숫자를 실제 측정하지 않은 항목은 측정 완료로 적지 않는다.

## 검증 한계와 후속 게이트

이번 기록은 25장 전체 구성의 육안 재검토다. 전 요소의 좌표 측정·2x/4x crop·실제 렌더 차이표가 모두 완료된 것은 아니다. 기존 캡션/탭/탐색/변환 검사기 확대 관측은 부분 근거다. 새 A2D WIP는 fmt/test/clippy 통과만 확인됐고 실제 시각 검증은 하지 않았다. 전체 화면 일치·접근성 인수·실제 모델 결과 검증은 미완료다.

구현 재개 전 해당 화면마다 요소 ID, 참조 영역, 논리 치수, 공통 컴포넌트, SDK 필드, 상태, 좁은 배치, SVG/raster 자산, 미결정을 작성한다. 구현 후 같은 요소의 1:1 및 2x/4x 비교에서 선·모서리·여백·정렬·문자·상태·hit 영역을 따로 판정한다. 누락된 영역을 기능 미구현과 혼동해 삭제하지 않는다. 상세 후속 작업은 [계획](../../../plans/player/presentation/visual-fidelity-follow-up.md)에서 추적한다.
