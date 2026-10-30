---
id: evidence:kinekino/20261029114225
title: 검증 회차의 native 수명 SDK 등록 checkpoint
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 검증 회차의 native 수명 SDK 등록 checkpoint

## 후속 고정 구간

`native-resolution-cli-r2`는 native current-frame PNG·private stderr 원문·상세 response schema와 실제 harness를 등록했다. 첫 strict9641ms101(unnecessary_unwrap), 첫 수리8240ms101(harness 직렬화 참조 누락), 두 파일 source-repair 뒤 fmt0·strict16951ms0·전체295pass18ignored(root202pass1ignored)/317882ms0다. 실제 실행은 하지 않았다. 이전 r1 원본·실패·수리 구간을 소급해 바꾸지 않는다.

공식 mpv의 기본 stdout 로그 근거를 확인해 stderr-only 보존을 전체 로그 보존으로 표시하지 않았다. `native-resolution-cli-r3/source-product`는 stdout/stderr 각각512KiB drain·원본별 SHA/bytes와 두 worker join을 포함한 후속 snapshot이다. [최종 r3](native-episode-playback-2026-10-19.md)는 fmt0·strict14342ms0·전체295/18ignored315544ms0·두bin1633ms0 뒤 실제1pass44.39초다. 아래 r1 bin/미디어 없는 EOF는 별도 고정본이며 r3 실제 회차 검증의 근거를 소급하지 않는다.

## 고정 검사

`runtime/quality-review/native-resolution-cli-r1/source-product`는 root src/tests/examples/assets와 Cargo/lock의 고정 snapshot이다. 전체 외부 빌드 입력을 봉인한 manifest로 확대하지 않는다. application playback_session, 명시 native owner, 고정 mpv executable/network argv, 공통 SDK·typed client·`player-cli --serve-native`를 등록했다.

fmt0, workspace/all-targets strict Clippy20215ms0, 전체293pass17ignored(root200pass1ignored)/336902ms0, 두 bin944ms0이다. 첫 strict 수리 없이 통과했다. runner는 affinity3·두 logical CPU·BelowNormal16384·자식 상속, Cargo는 offline/locked/-j1·기존 cache를 사용했다.

| 파일 | SHA256 |
| --- | --- |
| player-cli.exe | 4193ba8b56320529c62c1a82de9270657d52eb5bde9bb0497bff4b750aa38aeb |
| player.exe | d14eeab0916bee348e875d2e6fd3511e2608bf0ba7615a7390b3023aaf4825fb |
| resource-runner.exe | 692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463 |
| Cargo.lock | 50323b23fb1bbf496e3ad42be71b4ccbdc3e1f38de1de5a5154a1ba5d78da5f7 |
| 명시 mpv.exe | f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790 |

## 실제 host 시작과 EOF

고정 CLI에 r3 private catalog, 명시 mpv 경로/SHA, stdin EOF를 제공했다. host-eof receipt는5443ms/exit0/launcher82012/child83924다. native_host_ready의 playback_verified=false 후 native_host_stopped/host_lifetime_ended=true를 기록했다. 이때 engine을 열지 않았으므로 종료 receipt는 null이다. executable pin·transport 시작·미디어 없는 host EOF 증거이며 child mpv/IPC 종료·decode 증거가 아니다.

## 구현 범위와 미검증

open은 저장 artifact ID만 받아 공통 결과 reader가 원문 해시·소유 증거·HLS를 재검증한 뒤 native adapter로 전달한다. 소유 active 인스턴스가 있으면 교체하지 않는다. 네트워크 모드와 Referer 존재 여부를 분리하며 Absent는 header 옵션 생략, Empty는 wire 동등성 미검증으로 Unsupported, URL은 기존 정책을 통과한 값만 별도 argv로 전달한다. 셸·외부 옵션·모델 다운로드는 없다.

close/control의 인스턴스 검사는 같은 잠금 안에서 수행한다. 일반 close 후 새 open은 허용한다. host shutdown의 영구 closed gate는 이미 시작된 SDK 호출의 늦은 open도 막는다. close 성공은 소유 child wait와 IPC worker join 뒤 반환하도록 구현했다. OS try_wait/kill/wait 오류는 성공으로 반환하지 않고 Active를 유지한다. IPC write await 때문에 join 시간을 보편100ms라고 주장하지 않는다. 실제 hung 관측은 없다.

관측은 최신 스냅샷 한 개만 유지하며 공개 engine/status는 URI·토큰·stderr 원문을 반환하지 않는다. open/process_started와 control/accepted·completion=unobserved, 별도 status/time/frame 관측은 서로 다른 증거다. 현재 전체 테스트는 새 invalid/expired 입력 차단, 전송 헤더 구분, 닫힌 owner의 IPC 종료를 포함하지만 실제 native 프로세스 수명 전체를 증명하지 않는다.

## 다음 실제 인수

미등록 `runtime/coordination/native-resolution/actual_cli.pending.rs`는 새 실제 resolve→고정 CLI open→typed SDK 시간/크기/오류 관측→CLI pause→실제 pause 관측→CLI close→새 open→stale ID 거부→EOF 초안이다. 아직 컴파일·실행하지 않았으며 프레임/디코드 증거를 추가하기 전 playback 성공 gate로 사용하지 않는다. 차기 구간은 stderr 실패 원문 보존, native frame 증거, 상세 response schema와 실제 수명·제어·EOF 인수다. 속도·트랙·회차 전환·참조 media의 독립 검증과 나머지 자막/제작/AI 전체 goal을 유지한다.

[후속 계획](../../../plans/player/native-resolution-handoff.md)과 [실제 manifest 검증](player-manifest-wire-cli-2026-10-19.md)을 구별한다. 이 checkpoint는 전체 재생 또는 제품 완료가 아니다.
