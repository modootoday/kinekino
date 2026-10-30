---
id: evidence:kinekino/20261029113945
title: 실제 재생 볼륨·창·패널 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 재생 볼륨·창·패널 검토

2026-10-18. VIEW-07a 부분 개선과 시청 회귀 기록이다. 실제 소리 출력/별도 mute/전체 시안 인수를 검증한 기록은 아니다.

## 참조와 문제 관측

native-player-playback-v2.png의 volume영역(973,910,210,48)을 Rust visual-review로4배 확대했다. 파생물 runtime/design-inspection/view-volume-r43-4x.png. speaker glyph와 가는 rail/원형thumb의 기본 표현을 확인했고0상태는 공통 의미 규칙으로 보완한다.

r43: runtime/review-runs/ui-viewing-volume-20261018-r43/player.exe, PID29508. 로컬 runtime/ai-spike/source/Sintel.2010.720p.mkv를 Ctrl+O로 열어 실제 프레임/14:48 길이를 확인하고 Space로00:24 일시정지했다. volume rail Home→0인데 일반 speaker glyph가 유지됐다. popup을 열면 label 위쪽이 native 영상에 가려지고 아래 rail만 보였다. 이는 mpv HWND와 egui 합성 경계의 실제 관측이다. Escape 후 caption 닫기로 종료했다.

## 수정 책임

volume_control 공용 컴포넌트에 icon/현재값 tooltip/inline와popup Rail을 모았다. 직접 작성한 volume-zero.svg로0값과 양수값을 구분한다.0을 별도 mute state로 대체하지 않는다. inline72/popup180/clearance64를 token으로 정의하고980 미만에는 popup만 둔다.

transport는 해당 popup이 열린 동안 상단64 공간을 확보하고 기존 native surface layout으로 영상 rect를 줄인다. 닫히면 공간을 제거한다. 별도 플랫폼 호출이나 libmpv 통합을 추가하지 않았다. 일시적 영상 resize라는 비용을 숨기지 않고 [렌더링 정본](../../player/presentation/shared/rendering.md)에 기록했다.

## 수정 후 실제 관측

r44: runtime/review-runs/ui-volume-control-20261018-r44/player.exe, PID88440,1180×760. mpv.exe는 기존 검토 r43의 신뢰된 동일 실행 파일을 복사했다. 기존 가상 모니터 위치3500,100만 사용, VM/DPI/모니터 구성 변경 없음.

- 시작 화면 popup80% 전체 label/rail을 확인했고 Escape로 닫자 시작 그룹/영상 영역이 복원됐다.
- 같은 로컬 파일을 열어 실제 프레임/시간을 확인하고 Space로00:25 일시정지했다. inline rail Home→0에서 zero speaker SVG가 표시됐다. popup의0%와 rail이 영상에 가려지지 않았다.
- popup rail 클릭→50%에서 열린 상태를 유지하고 양수 speaker로 바뀌었다. Escape 후 원래 영상 rect와 paused00:25 상태가 복원됐다. 소리의 물리 출력은 측정하지 않았다.
- 최대화 caption 클릭→기존 가상 모니터3840×2160, restore 겹친 사각 glyph 확인→복원 caption 클릭→1180×760. 정착 후 영상/transport·paused시간이 유지됐다. 크기 변경 직후 capture의 잔상과 정착 화면을 구분한다.
- P→회차 패널과 source 제목/목록 연결 안내, native 영상 contain 축소. Escape→패널만 닫히고 영상 복원.

실제 화면은 도구 반환 이미지로 관측했으며 저장 캡처를 주장하지 않는다. r47 후속에서 정지 상태 fullscreen popup과 Escape 우선순위 및 회차 패널 보존을 [실제 확인](viewing-window-resize-review-2026-10-18.md)했다. popup 첫 표시 순간의 모든 frame·최소화·760/980 실제 popup·fullscreen 재생 중 idle·다른 native menu 합성·실제 회차 목록/재생 전환·별도 mute command는 미검증이다. 단기 공간 확보를 자유로운 overlay 합성이나 시안 완전 일치로 판정하지 않는다.

## 검사·종료

cargo fmt --check·cargo test -j2(78통과/8ignored)·cargo clippy -j2 --all-targets -- -D warnings·cargo build -j2 --bin player 통과. r44 소유 mpv PID108412/parent88440을 확인했고 caption 종료 후 player.exe/mpv.exe 프로세스 부재를 별도 조회했다.
