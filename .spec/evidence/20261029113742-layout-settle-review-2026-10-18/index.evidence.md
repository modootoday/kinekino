---
id: evidence:kinekino/20261029113742
title: 데모 레이아웃 안정화 조사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 데모 레이아웃 안정화 조사

2026-10-18. [색상표 검토](color-swatches-review-2026-10-18.md)의 r24에서 FRAME 썸네일 진입 후 request_discard 경고가 후속 화면 조회에도 남았던 문제를 조사했다. 경고를 끄거나 max_passes를 늘리지 않는다.

## 분리된 근거

로컬 egui_taffy0.14 소스의 Taffy recalculation은 레이아웃 dirty/크기 변경 때 request_discard를 호출한다. egui0.36.2 request_discard는 같은 run의 추가 패스를 요청하며, 그 자체로 후속 event-loop repaint를 예약하지 않는다. 실제 PlayerApp의 제작 분기는100ms repaint를 요청하지만 오프라인 PreviewApp에는 같은 후속 요청이 없었다.

headless regression thumbnail_layout_settles_after_tab_transition을 추가해 원본4프레임→썸네일16프레임을 연속 렌더했다. 관측된 discard reasons 수는 [11,0,0,0,12,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]였다. 이 조건에서 무한 반복이 아니라 측정 후 안정화한다는 근거다. 이미지 로더를 사용하는 실제 GUI의 모든 전환/폭에도 같은 수가 나온다는 의미는 아니다.

## 수정

PreviewApp은 레이아웃 discard 요청을 관측하면 다음 repaint를 예약하고, 요청이 사라진 뒤 안정화 repaint를 마친다. 마지막 경고/측정 화면이 입력을 기다리며 남지 않도록 하는 event-loop 보완이다. 상시 timer·max_passes 변경·debug 경고 숨김은 없다. 제품 재생/application 정책은 변경하지 않았다.

수정 후 fmt/test/clippy/build와 실제 FRAME 탭 전환을 재검증하기 전 시각 해결을 선언하지 않는다. 다른 화면·이미지 로드·창 resize의 반복 레이아웃은 별도 관측 대상으로 유지한다.

## r25 실제 재검증

fmt/test/clippy/build 통과. 현재 전체 tests는75 passed·외부 의존7 ignored였다(동시 SDK 작업으로 테스트 수 증가). 새 thumbnail 안정화 regression도 통과했다.

player0개 확인 후 r25(PID112132, runtime/review-runs/ui-all-pages-20261018-r25/player.exe)를 소유 가상 모니터3500,100·1440×900에서 실행했다. SEG→FRAME 원본→썸네일 클릭 후 화면에 경고가 남지 않았고, Left로 원본→Right로 썸네일 재진입 후에도 경고 없는 최종 화면을 직접 확인했다. 프리뷰·색상표·고정 footer는 유지됐다. VM·DPI·모니터 구성을 변경하지 않았다. 검토 후 캡션 닫기로 종료했다.

이는 해당 전환/폭의 실제 재검증이며 모든 화면·폭·이미지 로드의 layout convergence나 CPU idle 측정 완료가 아니다. 상시 repaint가 없다는 소스 구조와 실제 CPU 사용률 검증을 구분한다.
