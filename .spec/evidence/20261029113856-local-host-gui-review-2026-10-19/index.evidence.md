---
id: evidence:kinekino/20261029113856
title: LocalHost 시청 GUI 통합 준비·실행 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# LocalHost 시청 GUI 통합 준비·실행 관측

2026-10-19. r137은 최신 UI target 빌드에 root 소유 LocalHost main 조립을 포함한다. 이번 UI 세션은 main/SDK 파일을 수정하지 않았다. 전용 경로 `runtime/review-runs/ui-local-host-20261019-r137`에 player.exe와 기존 trusted target/debug/mpv.exe를 복사하고 --record-validation --validation-position=3500,100으로 normal GUI를 실행했다. demo가 아니며 stdout events.jsonl/stderr.log를 같은 디렉터리에 보존한다.

GUI PID83612, child mpv PID7368이다. 기존 가상 모니터의1180×760 창에서 Ctrl+O로 기존 `runtime/ai-spike/source/Sintel.2010.720p.mkv`를 열었다. 실제 영상과 제목/14:48 duration/시간 표시/transport를 확인했다. 초기 검정 영상 영역 뒤 실제 첫 장면이 표시됐으며 첫 프레임 지연 시간은 계측하지 않았다. Pause 클릭 후 play glyph로 바뀌었고 로그는 paused=true,pause_observed=true,time9.625,duration888.032,volume80.0을 기록했다. receipt는 accepted/unobserved이며 실제 상태 관측과 별도로 기록됐다.

local-host-ready에 is_running=true/named_pipe/current_user_same_privilege와 endpoint가 기록됐다. endpoint/owned PID·경로·mpv IPC·원본 경로를 SDK/root 세션에 전달했다. SDK가 실제 CLI 상태/네 제어 및 mpv read-only 교차 검증을 준비하며, UI는 해당 시험/복원 완료 인계까지 입력·미디어 변경을 보류한다. endpoint 문자열은 런타임 로그를 기준으로 하며 영속 설정으로 저장하지 않는다.

SDK 세션이 실제 endpoint 네 제어 시험 PASS와 복원 완료를 인계했다. 원본 증거는 `runtime/native-gui-endpoint/r137-native-cli-20261019-a/`의 요청/응답·독립 mpv 조회·report.json에 있다. UI 세션은 복원 후 실제 창에서 동일 Sintel 장면·00:09/14:48·Play glyph·볼륨 rail을 확인했다. 로그는 paused=true,time9.625,volume80.0,error=null이었다. caption 닫기 후 CIM에 player.exe/mpv.exe가 없었다. 이를 전 화면 시각 일치나 장시간/교체 스트레스 성공으로 확대하지 않는다. VM·DPI·모니터 구성·사용자 권한은 변경하지 않았다.
