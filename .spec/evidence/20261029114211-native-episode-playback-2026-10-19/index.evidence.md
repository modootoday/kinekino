---
id: evidence:kinekino/20261029114211
title: 실제 회차의 native 프레임·제어·소유 종료 CLI 인수
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 회차의 native 프레임·제어·소유 종료 CLI 인수

## 고정 실행과 검사

`runtime/quality-review/native-resolution-cli-r3`는 [등록 r1 및 r2 이력](native-resolution-cli-2026-10-19.md) 뒤 stdout/stderr 수집을 완성한 구간이다. source-product에 root src/tests/examples/assets/Cargo/lock을 보존했다. 전체 외부 컴파일 입력·프로세스 메모리를 봉인했다는 주장은 아니다.

fmt0·workspace/all-targets strict14342ms0, 전체295pass18ignored(root202pass1ignored)/315544ms0, 두 bin1633ms0이다. Cargo의 실제 jobs-d1505f40f71d4673.exe와 --list의 `native_episode::actual_resolve_and_owned_native_lifetime`을 대조했다. 고정 actual은1pass/44.39초/15filtered, runner44418ms/launcher95736/child74084/terminaltrue/exit0이다. 모든 실행은 affinity3·두 CPU·BelowNormal16384·자식 상속 및 offline/locked/-j1·기존 cache를 유지했다.

| 파일 | SHA256 |
| --- | --- |
| player-cli.exe | fdbf0abf4ab764614ca4613df0ae2e5a935cc27d3dec52699eac35c1f41c8756 |
| player.exe | b523abbf85702a74c05a348a98146afd73af906708f7c4c389f5fe776ee393fd |
| jobs-harness-final.exe | f88d2da4d1a608ca4204c680551b3adb6705347c5ab57aae24c8b32921f471fc |
| resource-runner.exe | 692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463 |
| mpv.exe | f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790 |
| Cargo.lock | 50323b23fb1bbf496e3ad42be71b4ccbdc3e1f38de1de5a5154a1ba5d78da5f7 |

실행 전/복사 전후 CLI SHA는 기존 Rust fixture가 검사했다. 새 모델 추론·다운로드·장시간 재생·GUI 시각 승인은 수행하지 않았다. r2의 첫 Clippy9641ms101와 첫수리8240ms101, 수리 strict16951ms0/전체295·317882ms0는 원로그·source-repair를 보존한다. r3 actual에 맞춰 원 deadline이나 assertion을 완화하지 않았다.

## 새 resolve와 첫 native 인스턴스

private 원문은 `runtime/quality-review/native-resolution-cli-r3/actual-cli`다. 기존 강화 resolve 인수를 새 root에서 실제 재실행했다. 같은 actual episode의 schema2 결과 `0f12a36d7b134bae3a07249682161375461f648f1381c3081fb4d5d89876e9c6`, master artifact `c613b02c73b6999007ded28b720df8801f2b3fca2b848758e0797e60dad8a8a8`, source descriptor `13fcce4f093f940c7ea7f2fafa47c35fe115ebb5213122cb643d0239f437bb5b`다. wire Referer absent·master variant1/ref1/segment0·미암호화 metadata를 재검증했다. raw 전체 CLI 재읽기·fresh CLI typed inspect·tamper 복사본 거부·합성 legacy typed Unsupported/원문 보존도 통과했다. provider EOF는 browser_worker_exited=true/completed_total=1이다.

이어 명시 native host에서 고정 CLI playback.open으로 engine instance1을 시작했다. 중복 open은 revision_conflict로 거부했고 기존 instance를 유지했다. 독립 typed EndpointClient 관측은 connected=true/paused=false/idle=false/error없음,1920x1080/audio_channels2/duration_us1420002080과1초 이상 시간 진행을 확인했다. 마지막 초기 진행 표본은1172567us다. 음량은0으로 시작했고 실제 청취는 검증하지 않았다.

## 프레임과 네 가지 제어

CLI `production.capture_frame`은 현재 instance1의 프레임을 `native-episode-cli/native-frame.png`로 게시했다.1920x1080/4319338bytes/SHA256 `22e4077d9d9654abd00e8ac711f40a9c7443780231cabad71966f5b92a1e4331`다. 제품 PNG 전체 디코드 후 테스트가 별도 파일 SHA와 PNG 픽셀 디코드를 확인했다. SDK와 PM은 저장 PNG를 직접 열어 어두운 성과 불빛의 영상 장면을 확인했다. 단순 창·검은 placeholder의 크기 정보만으로 성공한 gate가 아니다.

캡처 전후 관측 시각은1214278–3424822us이며 정확한 source PTS나 해당 프레임의 정확한 presentation 시각으로 표시하지 않는다. frame_time_exact/subtitles_included/window_capture/color_parity_verified는 false다. 같은 출력 경로의 재캡처는 revision_conflict였고 파일 SHA 불변을 확인했다.

| CLI 명령 | 별도 native 관측 |
| --- | --- |
| pause | paused=true |
| seek | paused=true/position_us10000000 |
| set_volume |13.25 관측 뒤0 복원 확인 |
| play | paused=false/position_us10556944, 재개 후500000us 초과 진행 |

receipt의 accepted/completion=unobserved를 적용 완료로 바꾸지 않았다. 이후 pause를 다시 관측하고 close했다. 각 제어 관측은 같은 instance1에 묶였고 원5초 관측 deadline을 유지했다. native 초기 진행30초, capture4초, 원 resolve30초도 유지했다.

## close·reopen·stale ID·EOF

잘못된 instance ID의 pause는 revision_conflict다. instance1 close는 소유 engine·IPC·stdout·stderr worker 종료를 모두 true로 반환했다. close 후 playback.engine은 NotFound다. 같은 artifact를 다시 open한 instance2는 새 ID를 받았고 늦은 instance1 close는 revision_conflict로 거부돼 instance2를 유지했다. 이 재개방은 process start/수명 증거이며 두 번째 decode나 다른 회차 전환까지 검증한 것은 아니다.

instance2를 소유한 host에 stdin EOF를 보내 native_host_stopped/host_lifetime_ended=true와 engine/IPC/stdout/stderr 종료를 확인했다. 별도의 명시 host를 시작하되 엔진을 열기 전에 종료한 뒤 남겨 둔 SDK clone에서 open하면 Cancelled인 assertion도 통과했다. 일반 close 뒤 open 허용과 host 종료 뒤 영구 차단을 구별한다.

진행 중 read-only CIM 도구 출력에서 mpv PID82772/parent120824를 관측했다. 별도 JSON snapshot을 시도할 때 이미 종료돼 파일이 생성되지 않았으므로 완전한 parent chain의 파일 봉인 근거로 확대하지 않는다. 이후 launcher95736/harness74084 및 관측한82772/120824는 Get-Process 조회에 없었다. 소유 종료 인수의 중심은 Rust가 보유한 Child wait와 각 worker join 및 실제 close/EOF receipt다.

## 원문 출력과 범위

instance1 진단 artifact `bcc9991e7a98b754cfb4caf74847cac5b66994b2c99ebfeeb84363f1700830ef`는9059bytes/body SHA `92907bfc525ddd25fff4d76b9741715816fbea135abe94a9fc350e4ebf61625c`다. fresh CLI catalog.read_raw가9059bytes/next_offset=null로 전체 wrapper를 재읽었다. stdout6308bytes/SHA `454a5ba646c34884e2179e93ab24d280efc2168e16493cf7c51d844b7be2f96a`, stderr0bytes/빈값 SHA다. 두 stream을 수집해야 한다는 실제 증거이며 complete=true/truncated=false/read_failed=false다. 원본 바이트는 private wrapper의 base64에 보존했다. native_engine_diagnostic의 HTTP status는 null이며 HTTP 응답으로 표시하지 않는다.

instance2 EOF 진단 artifact `46fce84d9b4caa70238386f4c619a171de62617f18ec27ac3d3b2768685ec98b`는644bytes/body SHA `e51a4bbc1e006caeabde0f7f8dde3d7aad3ce72a8f4ec3b1a4a81e47adbc83c3`다. 두 stream0bytes/complete=true이며 빠른 종료 전 두 번째 미디어 로드 성공을 주장하지 않는다. 상한은 stream별512KiB다. 전체 내부 로그·미발행 메시지를 모두 보존한다는 뜻은 아니다.

저장 master 결과의 referenced_resources_verified/media_decoded/native_playback/live_session_capability/url_expiration_verified=false는 역사적 manifest 계약 그대로 유지한다. 이번 별도 native 시도는 그 시점의 실제 프레임·재생 시간·제어·소유 종료를 입증한다. 참조 playlist/segment/key 원문의 독립 보존·전체검증, 전체 회차 재생·장시간 안정성·청취·색상/PTS/GUI 일치·다른 회차·모든 카테고리/화면·모델 품질로 확대하지 않는다. rate/tracks/previous/next·GUI 이관과 나머지 제작/AI 등 [전체 계획](../../../plans/player/plan-006-sdk-gui-cli.md)은 계속 남는다.

[application 계약](../../player/application/native-episode-playback.md)과 [후속 계획](../../../plans/player/native-resolution-handoff.md)을 연결한다. 전체 제품 goal은 완료되지 않았다.
