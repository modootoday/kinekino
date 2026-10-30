---
id: evidence:kinekino/20261029114207
title: 공통 네이티브 제어 GUI 조립
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공통 네이티브 제어 GUI 조립

2026-10-19. main에서 native_control_channel의 port를 기존 PlayerSdk.with_playback_control에, binder를 presentation::run_with_native_playback에 전달했다. 기존 status reader/publisher와 preview 분기를 유지했다. main에는 입력 정책·COM·제어 wire를 넣지 않았다. [SDK 계약](native-playback-control-2026-10-19.md)과 [GUI 구현/실기](../presentation/native-playback-control-ui-2026-10-19.md)를 따른다.

provider의 supported는 기능 제공 여부이며 현재 엔진 readiness나 적용 완료가 아니다. CLI 기본 구성은 이 provider가 없어5개 지원을 유지한다. GUI의 file/frame/status/control provider 구성은 이 호스트 dependency 가용 시10개 지원이며 미디어 미선택/미연결은 해당 readiness 오류를 반환한다. Play/Pause/Seek/Volume receipt는 계속 accepted/unobserved이고 실제 상태는 별도 snapshot으로 읽는다.

## 조립 회귀와 근거

root의 fmt check·전체 cargo test·clippy all-targets -D warnings는 SDK target/-j1에서 exit0였다.104passed/0failed/8ignored로 lib77·CLI2·contract4·identity1·frame2·control4·status3·capability2·subtitle9다. 외부 media7개와 내부 subprocess fixture1개의 ignored 범위를 유지하며 새로 실행했다고 하지 않는다. 이 checkpoint는 이후 host IPC 구현의 전체 검증이 아니다.

[당시 main 사본](../../../evidence/verification/native-control-assembly-2026-10-19-r1/main.rs) SHA256은 `f639a23f67a23adbad6bc6b3383c2f8522e223a6b1a0388de3b9bc425975968f`다. 같은 폴더에 fmt/test/clippy 원문을 보존했다. 소스 사본을 전체 compiler input attestation으로 소급하지 않는다.

UI 담당의 r129 원시 이벤트도 [보존](../../../evidence/verification/native-control-assembly-2026-10-19-r1/ui-r129-events.jsonl)했고 root가 shared_provider=true·accepted/unobserved·engine1/2와 별도의 실제 native property를 재확인했다. events SHA256은 `7a5636e5b1958f124c8689df59b1d8454b718b9550c8f8c716a0c65b74185efc`다. 실제 UI 조작/화면 검토는 담당 세션의 관측이며 root가 다시 조작한 시험으로 표현하지 않는다. Pause/Play·seek·volume0/1·파일 취소/재선택·engine 교체/종료의 범위와 일시적 빈 영상·미검증 오류는 위 GUI 정본을 따른다. aggregate acknowledgement를 명령별 완료로 사용하지 않는다.

## 남은 범위

현재 사용자 host IPC는 SDK 담당이 별도 구현·권한 경계 검증 중이고 main에 아직 조립하지 않았다. 실제 CLI→GUI→mpv 적용은 후속 gate다. 큐 포화/연결 전/닫힌 채널 Warning·Web 전환·legacy no-provider 실기 및 모든 시각 일치는 미검증이다. VM·DPI·새 장시간 시험은 수행하지 않는다.
