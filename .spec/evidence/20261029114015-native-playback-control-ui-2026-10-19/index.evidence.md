---
id: evidence:kinekino/20261029114015
title: GUI 공통 네이티브 재생 제어 연결
status: active
created: 2026-10-29
updated: 2026-10-29
---
# GUI 공통 네이티브 재생 제어 연결

2026-10-19. [SDK 계약 검증](../application/native-playback-control-2026-10-19.md)을 따른다. 기존 화면·토큰·서랍 작업은 유지하고 presentation 경계에서 입력과 engine lifecycle을 연결했다. 실제 조립은 main 담당 세션이 수행하며 UI 세션은 main.rs를 수정하지 않았다.

## 구현 경계

기존 run/run_with_client/run_with_native_status signature와 preview를 유지하고 run_with_native_playback에 optional binder를 추가했다. PlayerApp은 client와 바인더를 별도로 보유한다. 같은 native_control_channel 쌍의 port는 SDK, binder는 lifecycle owner에 전달해야 한다. 정확한 인자는 runtime/coordination/ui.md에 인계했다.

바인더가 있으면 GUI toggle·절대 seek·volume·startup pause를 typed client로 전송한다. 관측된 pause에 따라 Play/Pause를 명시하며 pause property 미관측 시 요청하지 않는다. seconds→정수 us 변환은 NaN/무한대·정밀도 범위를 검사한다. 바인더 없는 기존 호출만 typed control handle 호환 경로를 사용한다. 바인더 있는 SDK 오류를 legacy로 우회하지 않는다.

stop/교체/Web전환과 Drop은 바인딩을 먼저 해제하고 소유 engine을 정리한다. 새 local 선택 후 launch 전 이전 engine을 중지하되 파일 대화상자 취소는 이전 engine을 유지한다. bind 실패 시 새 소유 Mpv를 drop한다. 초기 mute=false는 포트 범위 밖의 기존 adapter 명령으로 남으며 오류를 표시한다.

receipt는 accepted/unobserved와 요청·provider 경로를 진단으로 기록할 뿐 native state를 갱신하지 않는다. 기존 status publish는 실제 snapshot을 유지한다. volume 편집 버퍼는 다음 update에서 실제 volume property로 동기화한다. 지원 범위 밖의 관측 음량은 안내하며 임의로100%로 합성하지 않는다. 접수 실패는 transport 위 Warning과 자동 숨김 고정으로 표시한다.

## 검증 진행

UI target cargo check가 통과했다. 새 Rust seek 변환 검사는 소수 초→정수 us·0·NaN/무한대/음수/범위 초과를 포함한다. 마지막 소스 기준 cargo fmt --check,cargo test --target-dir runtime/parallel-build/ui -j1,cargo clippy --all-targets --target-dir runtime/parallel-build/ui -j1 -- -D warnings가 통과했다. 라이브러리77 passed/1 ignored와 통합 테스트를 포함하며 외부 미디어 ignored 검사는 실행하지 않았다. 실제 GUI 관측은 아래와 구분한다.

## 실제 r129 관측

소유 player PID65608,1180×760. `runtime/review-runs/ui-native-control-20261019-r129/events.jsonl`과 `stderr.log`를 보관했다. 초기 검토 폴더에 mpv가 없어 launch 실패 안내를 확인했다. 기존 target/debug/mpv.exe를 복사하고 SHA256 `F8BF7E8FD8D6696E35441C55091498060E4594F724A132FD6C46579341BC0790` 일치를 확인한 뒤 기존 원본 Sintel.2010.720p.mkv를 열었다. 다운로드/추론은 수행하지 않았다.

모든 아래 요청 로그는 shared_provider=true,acceptance=accepted,completion=unobserved였다. 실제 적용 근거는 별도 native-playback property 관측과 화면이다.

| 입력 | 요청/관측 |
|---|---|
| startup Play·일시정지 클릭 | engine1 Play 후 Pause 접수, pause_observed=true/paused=true 확인·play glyph로 변경 |
| 정지 중 seek rail 클릭 | position_us53960277 요청, 실제 time53.958·paused=true, 바뀐 영상 프레임 확인. 프레임 정확도/원본 sourcePTS 동일성 보장 아님 |
| 볼륨 팝업 Home | SetVolume0 접수 후 실제 volume0.0·zero glyph 확인 |
| 팝업 Right | SetVolume1 접수 후 실제 volume1.0·일반 음량 glyph 확인 |
| 재생 클릭 | Play 접수 후 paused=false,time65.75→66.792→67.833 진행과 영상 변화 확인 |
| 파일 대화상자 취소 | 기존 화면/정지 상태 유지. 새 engine 접수 없음 |
| 같은 파일 재선택 | 이전 mpv PID13020이 사라지고 새 PID113864 하나만 존재. engine2 Play/Pause 접수 후 paused=true,time41.208,volume1.0 관측 |

새 선택 직후 한 capture에서 영상 영역이 밝게 비었으나 다음 관측은 정상 새 영상이었다. 지속 실패나 전체 전환 시각 일치로 해석하지 않는다. 파일 대화상자가 부모보다 크게 열려 자동화의 이름 필드 좌표가 범위를 벗어났고 set_value도 거절됐다. 목록의 관측한 행 클릭→Down→Enter로 파일을 선택했다. 이 기록은 대화상자의 접근성/크기 문제 해결을 뜻하지 않는다.

caption 닫기 후 player/mpv CIM 목록이 비었다. 기존 가상 모니터 [3440,0,7280,2160] 안의 [3500,100] 위치를 사용하고 VM·DPI·모니터 구성은 변경하지 않았다. UI target build가 통과한 실행본이며 main 담당의 같은 channel 조립이 포함됐다. 이후 공통 의존성 feature 변경을 포함한 마지막 전체 test/clippy도 통과했다. 전체 시각 일치나 명령별 ACK 완료를 검증한 것으로 확대하지 않는다.

## 남은 범위

마지막 공통 의존성 feature 상태의 cargo build --bin player --target-dir runtime/parallel-build/ui -j1도 exit0이었다. 검토에 사용한 r129 실행본은 보관하고 새 빌드를 해당 증거 위에 덮어쓰지 않았다.

실제 pause/play·seek·volume와 소유 engine 교체/종료는 위 표본으로 확인했다. 실제 큐64 포화·preconnect/closed/poisoned binder 오류 Warning·Web 전환·legacy no-provider 실행·preview의 새 실행 회귀는 미검증이다. SDK fixture 시험과 실제 GUI 오류 관측은 구분한다. 명령별 ACK correlation/외부CLI host IPC는 없다. 장시간 재생·모든 decoder/미디어·Web 제어 SDK 이관은 이번 연결 범위가 아니다. VM·DPI·사용자 앱과 기존 동결 CLI/AI 근거는 변경하지 않는다.
