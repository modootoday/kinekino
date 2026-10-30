---
id: evidence:kinekino/20261029114756
title: Native 오디오 연결 및 실행 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native 오디오 연결 및 실행 관측

[준비 관측](native-audio-preparation-2026-10-19.md)은 미등록 시점의 기록이다. 이 문서는 후속 root 연결과 실제 실행을 구분한다. 인수 기준은 [오디오 계획](../../../plans/player/native-audio-checkpoint.md)이다. 증거 루트는 `runtime/quality-review/native-audio-cli-r1`이며 첫 실패 로그를 덮어쓰지 않는다.

## 연결한 책임

domain의 AudioIntent는 volume과 explicit muted를 분리한다. application의 handoff core는 media 확인, 전체 WebView mute 확인, native queue 접수, 실제 mute·volume·pause 관측을 분리한다. 같은 application coordinator와 intent owner를 SDK NativeControlPort와 기존 Rust presentation이 소비한다. COM과 mpv 프로세스·IPC는 infrastructure adapter가 처리한다. presentation은 상태 표시와 effect 구동을 연결한다.

전환 중 설정은 `deferred_until_handoff`이며 native queue 접수인 `accepted`와 다르다. 두 receipt 모두 completion은 `unobserved`다. 다른 instance, 변경 전 plan과 A→B→A 의도 변경은 기존 계획으로 native queue에 들어가지 않는다. 종료가 시작되면 추가 접수를 차단하고, process·IPC·stdout·stderr 종료 proof 뒤에만 WebView 원상복원을 시도한다. 실패한 종료·복원은 소유권을 유지하며 명시적 close로 정리만 재시도한다. 원 오류와 정리 오류를 따로 보존한다.

`playback.audio_handoff`는 공통 coordinator가 게시한 관측과 `published_age_ms`를 반환한다. configured legacy control port에서만 지원하며 관측 미게시 상태는 not_found다. 기본 SDK나 이 coordinator를 연결하지 않은 native session host에서 지원을 가장하지 않는다. 저장된 사용자 mute 설정이나 실제 청취 품질을 입증하는 명령은 아니다.

Browser는 같은 generation Cell을 native WebView audio adapter와 공유한다. 문서 세대 포화는 0으로 폐쇄하며 재사용하지 않는다. Web origin 전환은 문서에 결합하고, 독립 local native 재생은 background WebView 전체에 lease를 걸어 새 navigation에서도 침묵을 유지한다. 기술용 provider WebView는 별도 역할로 로드 전에 고정 mute를 설치한다. 새 JavaScript 정책을 추가하지 않았다.

## 실행 결과

- 첫 root check: 24771ms/exit0, 미연결 경고를 보존했다. 첫 focused는 실제 23pass다.
- linked check 첫 실행은 MpvControlHandle의 trait 연결 누락으로 exit101이었다. 고정 inherent submit으로 forwarding하고 r2 exit0, 후속 r3 18705ms/exit0이었다.
- focused r2: 62071ms/exit0, 30pass. r3: 43413ms/exit0, 33pass·0fail·0ignored·245filtered, 테스트 본문 0.02s. 동시 요청, 실패한 접수의 의도 비승격, ABA, 다른 instance 정리 거부, queue ack만의 준비 완료 거부, stop/restore 실패와 명시적 재시도, 문서 변경 뒤 전체 view 침묵을 포함한다. synthetic adapter/proof 테스트이며 실제 COM/mpv 관측이 아니다.
- Clippy 첫 실행: 24936ms/exit0. 이후 CLI actual driver와 SDK query 테스트를 추가했으므로 최종 strict 결과를 별도로 인수한다.
- 전체 cargo test: 336160ms/exit0, 371pass·0fail·18ignored. API query port/discovery/schema/strict payload 검사도 포함한다. 드라이버 수정 뒤 같은 full 호출에서 cargo fmt를 실행하고 runner를 시작했다. 시작 전 별도 snapshot은 없으며 이후 consumed Rust 입력은 동결했다. `source-final`은 후속 보존 snapshot이다.
- 최종 Clippy: 3509ms/exit0, fmt check: 1616ms/exit0, bins: 1345ms/exit0, jobs harness 재빌드: 856ms/exit0. 재빌드한 jobs와 CLI/GUI/runner를 final-bin에 복사하고 SHA를 보존했다.
- actual-first: 2281ms/exit101. 시드 해시 환경변수를 잘못 지정해 `Explicit episodes seed SHA: NotPresent`로 미디어 실행 전에 실패했다. source/bin은 바꾸지 않고 `EPISODE_SEED_RESULT_SHA256`로 고쳤다.
- actual-r2: 97539ms/exit0, 실제 고정 case `native_episode::actual_resolve_and_owned_native_lifetime` 1pass·0fail·15filtered, 본문97.51s. 새 `actual-cli-r2`에서 resolve→Next/Previous→options→PNG 4초/전체 decode→mute/volume→play progress→close/reopen→EOF→post-host Cancelled 검사가 통과했다.

실제 미디어 driver에서 explicit mute true→volume13.25→volume0 동안 mute 유지→explicit mute false와 volume0의 독립 관측이 통과했다. `mute-observations.json`은 muted=true/volume0, `volume-observations.json`은 muted=true/volume13.25, `unmute-at-zero-volume-observations.json`은 muted=false/volume0이며 모두 paused=true/connected=true/has_error=false였다. strict boolean 실패 exit2/invalid_argument와 stale instance exit5/revision_conflict도 실제 CLI로 통과했다. 청취 품질이나 GUI Web→native 전환의 실제 검증으로 확대하지 않는다.

`closed.json`의 instance3은 engine/IPC/stdout/stderr 종료가 모두 true다. EOF 종료 receipt의 instance4도 네 종료값이 true, provider browser_worker_exited=true, completed_total=2, pending_transition_job=Ok(null), host_lifetime_ended=true였다. runner44428/harness33264는 종료 뒤 부재였으며, 고정 mpv 경로와 이번 evidence 실행 경로의 소유 process inventory는 `owned-processes-after.json`의 `[]`다.

`volume-restore-observations.json`에서도 muted=true/volume0을 별도로 관측했다. 볼륨 복원이 음소거를 해제하지 않았다는 실제 property 근거다.

고정 SHA256:

| 입력 | SHA256 |
|---|---|
| player-cli.exe | 74a5eff8264ff19204f4b5359d6b677eb7c21831fd9e91c47f90e4597dc75bde |
| player.exe | 2e3eb7b2e6862a93c8865f4d3f618c1283479fc8377a4806103a95546b8e6da2 |
| jobs harness | d29acdaf5cfcc181e3ab5c610419f3ff8f6cb26aafe73c8cab907aa8467573f0 |
| resource-runner.exe | 692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463 |
| mpv.exe | f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790 |
| src/application/api/mod.rs | 96a9a83bbd1048918759a1b9c24775bf72b7944c452756235062f9115f92e51a |

Cargo/lock·lib/main·presentation parent는 기존 pin을 유지하며 `shared-input-pins.txt`에 기록했다. Meari/root 제품 factory는 추가 등록하지 않았다. 후속 UI의 독립 미등록 adapter/tests는 이번 root full 검사의 소비 입력이 아니다.

## 남은 검증

실제 WebView2 native 속성 readback과 같은 coordinator를 통한 실제 Web/native handoff 성공·실패·WebView lease 정리를 확인해야 한다. 위 실제 CLI native process/worker 정리와 고정 pins는 인수했지만 WebView handoff 인수를 대체하지 않는다. GUI embedded/fullscreen/transport의 시각 일치나 제품 전체 완료로 확대하지 않는다. mpv package 추출·libmpv·모델 다운로드/추론은 이번 연결에 포함하지 않았다.
