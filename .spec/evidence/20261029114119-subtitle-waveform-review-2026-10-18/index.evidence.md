---
id: evidence:kinekino/20261029114119
title: SUB 파형 시간축과 선택 구간 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# SUB 파형 시간축과 선택 구간 검토

2026-10-18. 대상 SUB-03b, 공통 timeline_ruler 및 waveform. 모든 자막 하위 화면의 완료 기록이 아니다.

## 시안과 실제 기준

기준 `.spec/artifacts/design/v2/studio/subtitles.png` 영역(279,579,899,91)을 Rust visual-review로 3배 확대했다. 파생물 `runtime/design-inspection/sub-waveform-r60-3x.png`에서 회색 파형 면·중앙선·amber 선택면/세로 grip·독립 재생선·아래 시간 라벨을 확인했다.

r60(PID105900,1440×900, `runtime/review-runs/ui-sub-waveform-20261018-r60/player.exe`) 실제 ASR는 검정 직사각 안에 얇은 파형 막대와 정규화된 재생선만 표시했다. 시간 눈금과 선택 범위 표현이 없었다. baseline 확인 후 소유 창 닫기 버튼으로 종료했고 player/mpv 조회 결과가 비어 있었다.

## 수정 책임

- waveform은 duration/position/optional selection을 정수µs로 받아 같은 timeline_ruler::x 변환을 사용한다. content FIELD·border1·radius5, body56와 아래 ruler32, 중심선·회색 amplitude·amber 반투명 선택면과 폭4 grip을 토큰으로 조합한다.
- view projection은 실제 선택된 stable row의 start/end가 TimeUs일 때만 구간을 전달한다. 선택되지 않은 첫 cue를 자동 선택하거나 없는 시간을 발명하지 않는다. 재생 위치는 preview.position_us를 그대로 사용한다.
- 아래 라벨은 timeline_ruler::paint_below로 공용화했고 기존 filmstrip/CLIP의 위쪽 ruler API는 유지했다.
- amplitude는 기존의 명시적 DesignSample fixture다. 실제 오디오 디코딩이나 선택 구간 편집 기능을 구현한 것이 아니다. source duration에 비해 짧은 cue는 좁게 표시되며 시안의 임의 시간 비례를 맞추기 위해 범위를 늘리지 않는다.

## 수정본 실행

r61(PID116228, `runtime/review-runs/ui-sub-waveform-20261018-r61/player.exe`)을 기존 가상 모니터에 실행했다.

- 1440×900 ASR: FIELD 면·둥근 얇은 외곽선·회색 amplitude/중심선·아래 시간 눈금(00:00/05:00/10:00/15:00/20:00)이 표시됐다. 미선택 때 선택면은 없다.
- 두 번째 cue(25–40초)를 클릭하면 표 선택과 파형 왼쪽의 두 grip/선택면이 함께 표시됐다. Down으로 세 번째(50–65초)에 이동하면 선택면도 옮겨지고, 재생선은 기존 52초 위치를 유지했다. 선택과 재생 위치를 하나로 합치지 않았다.
- 760×480: 본문 스크롤 후 파형·눈금·표에 도달했다. 세 번째 cue 클릭 후 Home으로 첫 번째(0–15초)를 선택하자 파형 시작점으로 선택면이 이동했다. source duration 대비 짧은 cue는 폭이 작고 시작점 grip은 body clip 경계에 놓인다. 근접 grip의 분리 가독성과 확대 기능은 후속이다.
- 설정을 열고 Escape로 닫아도 ASR·선택·본문 스크롤 위치가 유지됐다.
- 760 교정 탭: 공통 파형·선택면·눈금이 표시됐다. 이것은 전체 길이 파형이며 교정 전용 확대 구현의 증거가 아니다. Empty 전환 시 amplitude·선택면·재생선·시간 숫자가 사라지고 오디오 파형 연결 필요/시간 범위 연결 필요 안내와 화면 골격이 유지됐다.
- 최종 fmt 검사·전체 cargo test·Clippy all-targets warnings deny·player 빌드 통과. lib64 pass/1 ignored, CLI2·계약4·identity1·frame2 pass/6 ignored·playback3·SDK parity2·subtitle9 pass/1 ignored. headless 전체 탭/fixture 통과는 실제 전 화면 검토의 대체 근거가 아니다.

검토 후 소유 r61 창의 닫기 버튼으로 종료했고 player.exe/mpv.exe 조회 결과는 비어 있었다. VM·DPI·모니터 구성을 변경하지 않았다.

## 남은 범위

ASR 자체 Empty 재검토, LongLabels, 980/1180, 기존 filmstrip/CLIP 위쪽 ruler의 실제 회귀는 이번 표본에 포함하지 않았다. 교정 선택 구간 확대 timebase, 실제 PCM/min-max 데이터, 줌/편집 handle, cue 표 checkbox/번호·열 폭, 전용 transport와 subtitle overlay는 별도 미완료다. 범위 drag·hover/pressed/focus는 미연결이므로 편집 지원으로 표시하지 않는다.
