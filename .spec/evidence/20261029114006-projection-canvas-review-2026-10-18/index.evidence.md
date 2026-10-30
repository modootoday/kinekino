---
id: evidence:kinekino/20261029114006
title: PROJ 캔버스 배치 재검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# PROJ 캔버스 배치 재검토

2026-10-18. PROJ-05/07의 부분 개선 기록이다. 전체 PROJ 인수 완료가 아니다.

## 기준과 변경 전

projection-package.png의 XYZ영역(275,489,102,95)·toolbar(668,151,182,50)을 기존 Rust visual-review로4배 확대했다. 파생물 runtime/design-inspection/proj-axis-r35-4x.png·proj-toolbar-r35-4x.png. XYZ는 좌하단, 도구는 우상단4개이며 확대는 돋보기+ 모양이었다.

r35: runtime/review-runs/ui-proj-review-20261018-r35/player.exe, PID108180,1440×900에서 깊이/패키지 두 탭을 관측했다. 기존 toolbar는 canvas 밖 가로 텍스트 줄, XYZ는 우상단, 격자는 한 방향 사선이었다. 평면 크기가 원본 비율과 무관했다. 초기에 포인터 입력이 화면 전환으로 이어지지 않아 상태/활성화를 재관측했고, Tab 포커스 후 새 클릭에서 PROJ로 진입했다. 입력 도구의 user-input 감지와 제품 응답을 구분하며 원인은 확정하지 않는다. 캡션 닫기로 종료했다.

## 공용 변경

toolbar::canvas_controls는 OBJ rail과 Frame/토큰 외형을 공유하고 우상단에4개32단위 버튼을 Taffy row로 배치한다. move-view.svg·zoom-in.svg를 직접 작성했다. 모두 미선택/비활성이며 실제 시점/카메라 명령을 제공한 것으로 표시하지 않는다.

projection_scene은 원본 asset 비율을 사용하고 격자 두 방향을 하반부에 clip한다. XYZ 좌하단 위치는 space::SCENE_AXIS_LEFT/BOTTOM 토큰이다. 임의 추론/mesh/GPU 렌더러 기능을 추가하지 않았다.

## 첫 수정 후 관측과 보정

r36: runtime/review-runs/ui-proj-canvas-20261018-r36/player.exe, PID107240,1440×900. 두 탭 모두 캔버스 내부 우상단4도구·좌하단XYZ·교차 격자를 관측했다. 패키지 유효성표6행이 전부 보였다. 평면의 뒤쪽 위 모서리 잘림이 남아 있어 네 평면 그룹 중심을 기준으로 배치를 보정했다. r36을 닫고 재빌드/재검토한다. 이 표본으로 최종 보정 후 배치를 판정하지 않는다.

기존 가상 모니터 위치3500,100만 사용하며 VM/DPI/모니터 구성 변경 없음. 실제 화면은 도구 반환 이미지로 관측했고 저장 캡처를 주장하지 않는다.

## 최종 보정 후 실제 관측

r37: runtime/review-runs/ui-proj-fit-20261018-r37/player.exe, PID41856.1440×900에서 깊이/패키지 모두 뒤쪽 위 모서리와 앞쪽 아래 모서리가 canvas 내부에 들어온 것을 확인했다. 우상단4도구/좌하단XYZ는 평면과 별도 영역에 유지된다. 패키지에서 Left로 깊이, Right로 패키지 복귀를 실제 확인했다.

760×480 패키지에서4도구가 canvas 우상단에 유지되고, body scroll로 좌하단XYZ와 scene 아래로 적층된5파일 목록에 접근했다. 설정을 연 뒤 Escape로 패널만 닫히고 패키지/본문 scroll 위치가 유지됐다. viewport 밖 부분이 본문 scroll로 가려지는 것과 scene 자체의 모서리 잘림을 구분한다. 최종 r37은 caption 닫기로 종료했다.

## 검사 결과

초기 수정의 전체 cargo test는78통과/8ignored였다. 최종 중심 보정 후 fmt --check·clippy --all-targets -- -D warnings·player build는 통과했다. 최종 전체 cargo test에서는 subtitle_conversion::object_safe_client_on_worker_matches_cli_typed_conversion의 SDK/CLI capabilities 응답 비교1건이 실패했고, 같은 테스트 단독 재실행은 통과했다. SDK 세션은 동시 변경이 없다고 확인했다. FFmpeg discovery deadline/부하에 따른 차이 가능성은 추정이며 actual 필드 diff 원인은 미확정이다. 해당 테스트나 SDK 구현은 수정하지 않았다. 전체 검사를 무조건 통과했다고 보고하지 않는다.

후속 cargo test --test subtitle_conversion은9통과/1ignored로 통과했다. player.exe 프로세스 부재를 종료 후 별도 조회했다. 반복성 원인은 여전히 미확정이다.

## 남은 항목

depth 원본 inset/legend/카메라 설정, 파일별 glyph/metadata/thumbnail 전환, typed scene/depth/camera/package 계약, target 검증/실제 저장, custom GPU callback은 미완료다. 비활성 도구의 hover 사유·상태별 caption·전 화면 1:1 인수는 미검증이다.
