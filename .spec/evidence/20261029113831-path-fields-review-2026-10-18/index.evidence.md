---
id: evidence:kinekino/20261029113831
title: 공용 저장 경로 필드
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공용 저장 경로 필드

2026-10-18. [검사기 계약](../../player/presentation/shared/inspector-controls.md), FRAME/CLIP/SUB/SAVE/OUT의 기존 시각 계약을 따른다. 일반 텍스트 입력과 경로 선택의 의미를 분리한다.

input::unavailable_path는 입력 높이32·유동 경로 영역·간격8·고정32 폴더 버튼을 공용 Taffy detail_sidebar로 구성한다. 기존 직접 작성 Folder SVG와 공용 control_icon을 사용한다. 경로 선택 필요를 표시하고 실제 picker가 연결되지 않은 버튼은 비활성과 사유를 유지한다.

FRAME2탭, CLIP 수동/일괄, SUB demo 변환, SAVE inspector, OUT 중앙 저장 위치에 같은 컴포넌트를 적용했다. 실제 SDK 변환의 기존 picker 경로는 변경하지 않는다. 형식/파일명·실제 경로 validation·충돌 정책은 별도 책임이다. 비활성 폴더 glyph를 실제 선택 성공으로 표현하지 않는다.

자동 검사·실제 실행 결과를 확인하기 전 전체 경로 필드의 시각 통과를 선언하지 않는다. 실제 폴더 선택·취소·쓰기/충돌·모든 폭과 상태는 미검증이다.

## r31 실제 관측

fmt/test/clippy/build 통과(77 passed,8 ignored). player0개 확인 후 r31(PID29160, runtime/review-runs/ui-all-pages-20261018-r31/player.exe)을 소유 가상 모니터3500,100·1440×900에서 실행했다. VM·DPI·모니터 구성 변경은 없다.

FRAME 원본, SAVE, OUT, CLIP 수동 inspector 하단, SUB demo 변환을 실제 열어 경로 선택 필요 입력과 폴더 SVG가 같은 높이/우측 고정 배치를 유지함을 확인했다. CLIP은 본문 scroll로 출력 경로에 접근했고 footer는 고정됐다. OUT 중앙 경로 필드는 넓은 카드 안에서 유동폭을 사용했다. 다른 탭·좁은 창·hover tooltip 및 실제 picker 동작은 이번 실기 범위가 아니다. 검토 후 캡션 닫기로 종료했다.
