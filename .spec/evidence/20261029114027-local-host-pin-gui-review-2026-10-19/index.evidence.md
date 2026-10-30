---
id: evidence:kinekino/20261029114027
title: 새 일반 GUI의 SDK 실행 파일 고정 검증 인계
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 새 일반 GUI의 SDK 실행 파일 고정 검증 인계

2026-10-19. r137을 재사용하지 않고 새 일반 GUI r145를 준비했다. 제품 코드를 수정하지 않았으며, 앞선 r144 배치 정렬 수정 뒤 fmt/test/clippy/build 통과 실행 파일을 사용했다.

## UI 관측과 소유 범위

`runtime/review-runs/ui-local-host-pin-20261019-r145/player.exe`, GUI PID119792, 자식 mpv PID117412. 기존 가상 모니터3500,100의1180×760 창. VM·DPI·모니터 구성 변경 없음. Ctrl+O 파일 대화상자에서 실제 Sintel.2010.720p.mkv를 선택했다. 첫 반환 캡처의 시작 안내/검정 면은 영상 로드 성공으로 취급하지 않았다. 후속 캡처에서 실제 영상 프레임·제품/미디어 분리 제목·Pause glyph·00:03/14:48을 확인했다.

Pause 클릭 뒤 Play glyph와00:08/14:48 표시를 확인했다. events.jsonl의 native-playback은 paused=true,pause_observed=true,time8.583,duration888.032,volume80,error=null이었다. 프로세스의 mpv 명령행은 mute=yes다. 실제 IPC 속성 독립 확인과 CLI 결과 검증은 SDK 시험 결과를 기다린다.

## 인계와 남은 검증

SDK 세션에 run_id `r145-native-cli-pin-20261019-a`, GUI/자식 실행 경로·생성 시각·endpoint·IPC·source·원래 재생 상태를 인계했다. 시험 중 UI 입력/종료/소스·복사 실행 파일 교체를 보류하는 독점 입력 lease를 제공했다. SDK의 실행 파일 해시/잠금·copied CLI child image·native 속성 및 복원 결과는 아직 미확인이다. 실행 파일 고정의 성공을 GUI 캡처만으로 판정하지 않는다. lease 반환 후 최종 화면 재관측과 소유 프로세스 종료를 별도로 확인한다.

인계 후 상태 재확인: CIM에서 GUI119792와 자식117412가 같은 r145 경로로 살아 있는 것을 확인했다. SDK chat snapshot은 진행 중이었다. 이 관측은 검증 완료를 뜻하지 않으며 새 player를 중복 실행하지 않았다.

## 실제 시험 결과와 최종 화면

SDK가 actual gate PASS(23.22초)와 lease 반환을 알렸다. raw `runtime/native-gui-endpoint/r145-native-cli-pin-20261019-a/report.json`의 scenario_passed/restoration_passed=true, 두 error=null을 직접 확인했다. executables-after.json의 all_source_and_copy_hashes_unchanged=true, 각4exe copied_bytes_verified=true를 확인했다.21요청과 copied CLI child image 검증은 SDK의 시험 범위다.

native-restored.json은 pause=true,time-pos8.583,volume80,duration888.032,mute=false였다. 실제 initial/restored mute=false 유지가 SDK에서 확인됐으며, 앞서 관측한 launch 옵션 mute=yes를 실제 mute 상태로 확대하지 않는다.

입력 반환 뒤 같은1180×760 창을 다시 관측했다. 인계 전과 같은 눈산 프레임, Play glyph,00:08/14:48,볼륨 rail을 확인했다. 전체화면·다른 media·색상 정확도·전 화면 시각 인수로 확대하지 않는다. caption 닫기를 수행했으며 이후 프로세스 조회 결과는 별도 확인한다.

caption 입력 후 CIM 조회에서 소유 PID119792/117412가 출력되지 않아 두 프로세스 종료를 확인했다. 이후 배치 배지/구분선 변경의 UI target(-j1) 검사 파이프라인 session51048을 시작했다. 이 후속 검사는 진행 중이며 r145의 검증 결과와 구분한다.

정정: 위 종료 확인은 shell session의 지연 출력을 끝까지 확인하지 않은 잘못된 판정이었다. 후속 CIM과 창 목록에서 r145가 남은 것을 확인했다. r146 검토 창이 잠시 함께 열린 뒤 r145를 활성화해 실제 시청 화면을 새로 관측하고 caption 닫기를 다시 수행했다. 이어 새 창 목록에서 r145 두 창이 사라지고 r146만 남은 것을 확인했다. 앞선 ‘종료 확인’을 근거로 사용하지 않는다. SDK의 원상복구 관측 결과 자체는 변경하지 않는다.
