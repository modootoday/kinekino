---
id: evidence:kinekino/20261029113936
title: 일반 창 크기 조절 실제 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 일반 창 크기 조절 실제 검토

2026-10-18. 시청 화면의 좁은 창 검토를 위한 선행 확인이다. VM·DPI·모니터 구성을 변경하지 않았다.

## 구현과 검사

공용 `presentation/window_frame.rs`에 egui BeginResize 요청 경로를 추가했다. edge6/corner12 토큰을 사용하고 캡션126×32 영역을 보호한다. 최대화·전체화면·popup 동안 비활성한다. 우상단 모서리는 캡션 보호를 위해 대상에서 제외했다. 제작·시청·데모가 같은 컴포넌트를 호출한다.

geometry 단위 검사, cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings와 player 빌드가 통과했다. 이는 실제 OS 크기 조절 성공의 증거가 아니다.

## 실행 관측

검토 실행은 `runtime/review-runs/ui-window-frame-20261018-r46/player.exe`, PID29440, 기존 가상 모니터 위치3500,100이다. 시작 창1180×760에서 우하단1178,758→758,478 드래그를 수행했다. 직후와 재관찰 모두1180×760으로 유지됐다. 따라서 새 경로의 실제 동작은 실패/미확인 상태이며 완료로 판정하지 않는다. 입력 전달 시점과 egui 반응 및 backend 명령 소비를 분리해 후속 진단한다.

캡션 닫기1158,16 입력 후 창이 사라졌고 player/mpv 프로세스 조회 결과가 비어 있음을 확인했다. 다른 세션의 창을 조작하지 않았다.

## r47 입력 경계 진단과 전체화면 검토

같은 가상 모니터의 r47 검토 실행 PID115248에서 우하단1174,754→754,474 드래그를 수행했다. 검토 로그 `runtime/review-runs/ui-window-frame-20261018-r47/stdout.jsonl`의 elapsed_ms18915에 SouthEast BeginResize 요청과1180×760 frame을 기록했다. 직후 window-metrics와 실제 재관찰은1180×760이다. egui 입력→요청 생성 경계는 확인했고, 요청 이후 네이티브 동작 또는 자동화 입력 시점의 문제는 미확정이다. winit0.30.13 구현은 Windows WM_NCLBUTTONDOWN 경로를 사용하며, 해당 API 문서는 호출 직전 좌측 버튼 눌림을 전제로 한다. 이 사실만으로 winit 결함이나 도구 결함을 단정하지 않는다.

`pressing_bottom_corner_requests_native_resize` 검사는 PointerMoved+Primary press를 egui에 공급하고 SouthEast 명령 출력을 확인한다. 최종 fmt --check, 전체 cargo test, all-targets Clippy -D warnings가 통과했다. 실제 resize 성공을 대체하지 않는다.

별도 실제 미디어 검토로 Ctrl+O에서 Sintel.2010.720p.mkv를 열고 Space로8.542초에 정지했다. 전체화면은 기존3840×2160 가상 모니터 내에서 실행했다. 볼륨80% popup의 label·rail이 native 영상에 가리지 않고 표시됐다. 첫 Escape는 popup만 닫고 fullscreen3840×2160을 유지했다. 두 번째 Escape는1180×760 일반 창으로 복귀했다. P로 회차 패널을 연 뒤 볼륨 popup을 열어 label·rail이 완전하게 보이는 것을 확인했다. Escape 후 popup만 닫히고 회차 패널과 정지8초 상태가 유지됐다. popup 동안 영상/패널의 사용 높이를64 줄이는 기존 대응도 관측했다.

캡션 닫기로 r47을 종료한 뒤 player/mpv 프로세스 조회가 비어 있음을 확인했다. 좁은760 창과 재생 중 idle 자동숨김은 이번 실행에서 확인하지 않았다.

## 남은 검증

실제 드래그 크기 조절 원인 진단·수정, 760 창의 볼륨 popup과 영상 HWND 가림, 재생 중 fullscreen idle 전환, 제작 화면 resize 회귀는 미검증이다. 이전 r44의 일반 창 볼륨 검토를 이 범위의 성공 근거로 확대하지 않는다.

추가 시각 불일치: 정지 상태 fullscreen에서 제목줄·제작·캡션이 계속 표시됐다. VIEW-09a의 전체화면 제목줄 숨김 계약과 다르다. popup/Escape 검증 통과를 fullscreen 시안 일치 완료로 해석하지 않는다. 후속 수정은 fullscreen overlay와 일반 창 chrome을 분리하고 실제 재생·정지 양쪽을 재검토해야 한다.
