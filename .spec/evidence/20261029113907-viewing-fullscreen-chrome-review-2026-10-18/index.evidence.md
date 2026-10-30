---
id: evidence:kinekino/20261029113907
title: VIEW-09a 전체화면 일반 제목줄 제거
status: active
created: 2026-10-29
updated: 2026-10-29
---
# VIEW-09a 전체화면 일반 제목줄 제거

2026-10-18. 이전 r47에서 발견한 제목줄·제작·캡션 노출을 수정하고 실제 재검토했다. 전체화면 전체 시안 일치 완료는 아니다.

## 기준과 변경

기준 `.spec/artifacts/design/v2/viewing/native-player-fullscreen-v2.png` 전체와 상단0,0,1672,90을 검토했다. Rust visual-review로2배 확대하여 `runtime/design-inspection/fullscreen-top-r48-2x.png`에 파생물을 만들었다. 기준에는 일반 창 제목줄/캡션이 없고 좌측 작품명 배지와 우측 Esc 안내가 영상 위에 있다.

`player_app.rs`는 컨트롤 표시 여부와 별도로 fullscreen일 때 일반 toolbar 패널을 생성하지 않는다. 정지·popup으로 컨트롤이 고정돼도 제목줄을 다시 보이지 않는다. 일반 창/제작 프레임은 기존 책임을 유지한다. `.agents/rules/window-chrome.md`에 이 구분을 명시했다.

## 실제 관측

실행 `runtime/review-runs/ui-fullscreen-chrome-20261018-r48/player.exe`, PID70024. 검토는 기존 가상 모니터 위치3500,100, 일반1180×760 및 전체화면3840×2160에서 했다. VM·DPI·모니터 구성 변경 없음.

Ctrl+O로 실제 Sintel.2010.720p.mkv를 열었다. Space로00:12에 정지한 뒤 우하단 전체화면 버튼을 눌렀다. 전체화면에서 제목줄·제작·최소화/최대화/닫기 영역이 사라졌고 하단 컨트롤은 유지됐다. Space 재생 후에도 제목줄은 노출되지 않았다. 입력 없는 후속 화면에서 하단 컨트롤도 숨겨졌다. stdout.jsonl의 elapsed_ms78226 window-metrics는 fullscreen=true, controls_visible=false, client3840×2160을 기록하고 이후 native-playback은 paused=false/time22.583을 기록했다.

Escape로 일반1180×760에 복귀하여 제목·제작·캡션과 하단 컨트롤이 다시 표시되는 것을 확인했다. 재생은00:47로 진행됐다. 캡션 닫기로 종료했다. 종료 후 player/mpv 프로세스 조회가 비어 있음을 확인했다. 화면 근거는 도구 반환 이미지의 직접 관측이며 별도 저장 screenshot을 주장하지 않는다.

## 검사와 남은 차이

cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings, player 빌드 통과. 전체 테스트80개 통과/8개 ignored이며 실제 UI 관측과 구별한다.

작품명 배지·Esc 안내와 하단 영상 위 그라데이션 합성은 미구현이다. 현재 native HWND 위 자유로운 egui overlay 합성이 검증되지 않았으므로 상단 공간을 예약한 가짜 overlay로 대체하지 않았다. 현재 하단 컨트롤은 별도 패널이며 기준의 영상 위 합성과 다르다. popup pin/회차 패널이 열린 상태의 이번 제목줄 변경 회귀, narrow760, 제작 resize, 실제 드래그 resize는 별도 검증/수정이 남는다.
