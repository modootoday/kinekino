---
id: evidence:kinekino/20261029113903
title: 시청 패널·제작 본문 스크롤 후속
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 시청 패널·제작 본문 스크롤 후속

2026-10-18. [시청 요소 계약](../../player/presentation/viewing/visual-elements.md), [제작 구성](../../player/presentation/shared/screen-composition.md)을 적용한다.

## r19 제작 본문

기존 모든 전용 화면의 본문 scroll ID가 dedicated-screen 하나였던 소스 문제를 발견해 workspace/subview별로 분리했다. 수정 후 test/clippy/build 통과, fmt는 최초 검사에서 줄바꿈 차이를 발견해 수정하고 재검사했다.

가상 모니터 3500,100에서 r19(PID105556, runtime/review-runs/ui-all-pages-20261018-r19/player.exe)를 하나만 실행했다. 760×480에서 SEG 본문을 구간 표까지 내린 뒤 SUB로 이동하니 SUB는 위쪽 영상에서 시작했다. SEG로 돌아오면 기존 아래쪽 구간 표 위치가 유지됐다. 이는 두 workspace의 실제 관측이며 모든 하위 탭·상태의 보존을 입증하지 않는다. 캡션 닫기로 종료했다.

## r20 실제 시청 앱

데모 종료·player0개 확인 후 r20(PID35772, runtime/review-runs/ui-viewing-20261018-r20/player.exe)을 --ui-demo 없이 실행했다. 기본 URL은 about:blank, 위치는 같은 가상 모니터이며 실제 창1180×760이다.

원형 시작 표식·제품명·파일 열기/작품 탐색 동작·하단 미상 시간 --:-- / --:--·비활성 재생/seek를 확인했다. P로 회차 패널을 열면 영상 작업 영역이 줄고 시작 그룹이 그 영역 중앙으로 이동했다. Escape는 패널만 닫았다. 소스가 없으므로 실제 재생·seek·음소거·자동숨김·재생 fullscreen의 증거가 아니다.

패널 header가 제목 옆 일반 닫기 버튼인 차이를 발견했다. 공통 Close SVG·hit32를 우측 끝에 정렬하고 panel 폭260/min180/max360을 토큰으로 옮겼다. 목록 본문 scroll ID도 명시했다. 수정 전 r20은 캡션 닫기로 종료했다. 수정 후 실제 관측은 후속으로 기록한다. VM·DPI·모니터 구성은 변경하지 않았다.

## r21 패널 수정 재검증

수정 후 fmt/test/clippy/build 통과(71 passed, 외부 의존5 ignored). r21(PID99624, runtime/review-runs/ui-viewing-20261018-r21/player.exe)을 같은 위치에서 실제 앱으로 실행했다. 창1180×760·무소스 상태다.

P로 연 패널 header의 우측 끝 X SVG를 확인했다. 마우스가 창 캡션 닫기 위에 남아 있던 툴팁은 배경 클릭 후 사라져 패널 아이콘과 구별했다. 패널 X(창 좌표1156,52)를 실제 클릭하니 패널만 닫히고 시작 화면·transport는 유지됐다. 기본/클릭 동작 관측이며 모든 hover/pressed/Tab focus 인수는 아니다. 검토 후 창 캡션 닫기로 종료했다. 실제 영상 재생·회차 목록 선택·전체화면·긴 제목 및 모든 시작 동작 상태는 계속 미검증이다.
