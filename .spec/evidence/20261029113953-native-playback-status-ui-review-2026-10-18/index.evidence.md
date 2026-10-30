---
id: evidence:kinekino/20261029113953
title: GUI 실제 네이티브 상태 읽기 검토
status: active
created: 2026-10-29
updated: 2026-10-29
---
# GUI 실제 네이티브 상태 읽기 검토

2026-10-18. [SDK 메모리/계약 검사](../application/playback-status-contract-2026-10-18.md)와 별도로 실제 player/mpv 연결을 확인했다. UI 소유 구현은 presentation이며 main reader/SDK/writer 조립은 root 세션이 수행했다. [표시·수명 정본](../../player/presentation/shared/sdk-playback-observation.md)을 따른다.

## 변경과 검사

기존 run/run_with_client/PlayerApp::new 호출을 유지하고 optional publisher 진입점 및 builder를 추가했다. update_native의 실제 snapshot publish, stop/source launch 전 clear를 연결했다. 전용 inspector early return으로 접근 불가능했던 SDK 연결 정보를 실제 workspace의 접힌 공용 영역에서 열 수 있도록 했다. 같은 client의 typed 수동 조회만 사용하며 DesignSample UI는 이 영역을 추가하지 않는다.

최종 cargo fmt --check, cargo test, cargo clippy --all-targets -- -D warnings, player 빌드가 통과했다. 실제 engine 관측과 단위/합성 fixture를 구별한다. 첫 typed 호출에서 RequestContext 누락 컴파일 오류를 수정한 뒤 최종 검사를 수행했다.

## 실행 식별과 실제 관측

실행 파일 `runtime/review-runs/ui-native-status-20261018-r53/player.exe`, PID57432, SHA256 `097A328581FE7FBE800703502333AD3A71EC33485385907BEDE6DA441727263D`. 기존 가상 모니터3500,100에서 일반1180×760와 제작1440×900을 사용했다. VM·DPI·모니터 구성 변경 없음.

1. 미디어 없는 상태에서 제작→SDK 연결 정보→수동 새로고침을 수행했다. GUI 지원 명령에 playback.status가 표시되고 결과는 게시된 네이티브 관측 없음이었다. 가짜 idle/paused 기본 snapshot을 표시하지 않았다.
2. 시청 복귀 후 Ctrl+O로 기존 Sintel.2010.720p.mkv를 열었다. 실제 영상과14:48 길이를 확인하고 Space로00:51에서 정지했다. 입력 감지로 거절된 첫 Space 시도는 새 상태 관찰 후 다시 실행했다.
3. 제작 복귀 후 같은 SDK 새로고침에서 connected=true, paused=Some(true), idle=Some(false), position_us=51875000, duration_us=888032000, volume=Some(80), video_size=1280×544, audio_channels=Some(6), has_error=false, published_age_ms=3이 표시됐다. stdout.jsonl의 실제 native-playback도 time51.875/duration888.032/1280×544/6channels와 pause_observed/idle_observed=true를 기록했다. 위치를 source PTS로 판정하지 않는다.
4. 소유 mpv PID77484의 실행 경로가 r53/mpv.exe이며 ParentProcessId57432임을 fresh CIM 조회/조건문으로 확인한 후 그 PID만 종료했다. 이는 검토용 엔진 종료 오류 주입이며 사용자/다른 세션 프로세스에는 입력하지 않았다. 이후 GUI SDK 새로고침은 게시된 네이티브 관측 없음으로 돌아갔다. 즉 실제 engine 종료→stop_native→clear 이후 이전51.875초 snapshot이 읽히지 않았다. 새로고침 전 화면은 명시된 수동 조회 캐시를 유지했다.
5. 캡션 닫기로 player를 종료한 뒤 player/mpv 조회가 비어 있음을 확인했다.

화면은 도구 반환 이미지로 직접 관측했으며 별도 저장 screenshot을 주장하지 않는다. 진단 JSONL은 위 실행 경로 stdout.jsonl에 있다. raw path/error를 새 SDK DTO에 넣거나 세션 전체를 직렬화하지 않았다.

## 미검증과 시각 목표

읽기 경로의 실제 연결/관측 없음/paused snapshot/엔진 종료 clear를 확인했다. playing DTO 별도 조회, GUI 정상 정지 동작, 회차/소스 교체 race, property 미관측 시점의 실제 null, 권한/lock 오류,760 진단 영역·긴 값·접근성은 별도 검증 대상이다. CLI는 host IPC 미연결로 이 GUI 채널을 읽지 않으며 기존 provider 없음 계약을 유지한다.

진단 값의 Some/VideoSize 표기는 현재 기술 진단 표현이다. 제품의 공통 읽기 필드 표현으로 정리할 여지가 있다. 이 통합은13개 화면군과 모든 하위 탭의 시각 완전성 완료를 뜻하지 않는다. 기존 타임라인·FRAME 간격/편집 canvas·전체화면 overlay·native resize 등의 남은 차이를 유지한다.
