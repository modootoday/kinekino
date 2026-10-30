---
id: evidence:kinekino/20261029113754
title: CLIP 하위 화면 순환·AI 프리뷰 면 보완
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CLIP 하위 화면 순환·AI 프리뷰 면 보완

2026-10-19. short-recommend-refined.png 전체와 하단 transport(258,511,879,65)4배 확대를 검토했다. 파생물은 `runtime/review-runs/clip-preview-transport-r141-4x.png`다.

## 기준 실행과 차이

r140 `runtime/review-runs/ui-clip-cycle-20261019-r140/player.exe`, PID104620,1440×900을 실제 실행했다. 수동의 세로 crop grid/원본16:9/영상·자막·오디오3트랙, 추천의 세로 이미지/후보3행, 일괄의 표/계획 요약을 순환 확인했다. 첫 수동 전환에는 이미지 loading spinner가 보였고 다음 캡처는 정상 자산이었다. 추천은 검정 전체 폭 면/하단 transport가 빠져 이미지가 본문 바탕 위에 떠 있었다. 일괄 표는 card 가용 폭보다 좁게 왼쪽에 모여 있고 수동 crop에서는 인물이 왼쪽 잘리는 현재 cover 결과도 관측했다. 이 두 항목은 이번 수정 범위 밖이다. r140 caption 종료 뒤 수정본을 실행했다.

## 공용 변경

MediaPreview::unavailable를 추가하고 AI추천에만 적용했다. 전체 폭 VIDEO_SURFACE/THIN border/CONTROL radius 안에 기존9:16 artboard320과 CHROME transport를 조합한다. 공용 Play/Volume/Fullscreen32·MediaRail·미상 시간·명시9:16·padding6/gap8을 사용한다. clip-local 재생 계약이 없으므로 시간/완료 상태를 만들지 않고 조작은 비활성/이유를 유지한다. 후보 데이터·선택 정책·수동/일괄 구조는 바꾸지 않았다.

## 수정본 실제 관측

r141 `runtime/review-runs/ui-clip-preview-20261019-r141/player.exe`, PID81480. 기존 가상 모니터3500,100 사용, VM·DPI·모니터 구성 변경 없음.

- 1440×900: 검정 전체 폭 프리뷰 면과 가운데 세로 이미지, 아래 CHROME transport/미상 시간/중성 seek/볼륨/9:16/전체화면을 확인했다. 후보2 클릭→End로3의 선택·focus와 자동 스크롤이 이어졌다. 자산은 후보가 공유하므로 이를 서로 다른 영상 재생으로 주장하지 않는다.
- 수동 탭은 수정본 진입 과정에서도 세로 crop/원본/3트랙을 유지했다. 일괄 수정본 실제 재방문은 하지 않았다.
- 760×480: 본문 scroll로 transport 전 항목에 접근했고 폭 밖으로 밀리지 않았다. 설정 drawer→Escape 후 같은 AI추천/본문 위치가 유지됐다. 초기 viewport에 전체 프리뷰가 들어간다는 판정은 하지 않는다.
- Empty: 이전 이미지/후보행이 사라졌고 검정 면/소스 연결 안내/비활성 transport와 디자인 예시0개 표시는 유지됐다.
- caption 종료 후 CIM에 player.exe/mpv.exe가 없었다.

## 검사·남은 범위

fmt--check·전체 cargo test(lib86pass/1ignore+기본 통합/doc)·clippy all-targets-Dwarnings·player 빌드가 통과했다. UI target runtime/parallel-build/ui,-j1 사용.

수동 crop의 의미와 preview transport, 일괄 표의 전체 폭 열 배치, 추천 inspector 상단 여백·시안 세부 옵션, 실제 clip-local PTS/재생·저장·후보 생성, LongLabels/NeedsInput/980/1180·모든 hover/focus 상태는 남는다. 기능 미연결을 시각 구조 생략의 이유로 삼지 않되 아직 전체 CLIP 시각 일치는 완료가 아니다.
