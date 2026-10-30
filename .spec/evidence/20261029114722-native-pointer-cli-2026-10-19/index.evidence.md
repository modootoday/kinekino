---
id: evidence:kinekino/20261029114722
title: Native 포인터 결함 수리와 실제 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native 포인터 결함 수리와 실제 관측

사용자가 영상 hover 때 커서가 사라지는 현상을 수정 대상으로 확정했다. 이 checkpoint는 [전환 r2 인수](native-transition-cli-2026-10-19.md) 이후 소스이며 해당 pass를 수정 코드에 상속하지 않는다. runtime 근거는 `runtime/quality-review/native-pointer-cli-r1`이다.

## 수리

기존 owned network_plan과 legacy spawn 양쪽의 `--input-cursor=no`를 제거하고, infrastructure/mpv/pointer_policy의 고정 `--input-cursor=yes`·`--cursor-autohide=no`를 공유한다. 엔진은 포인터 이벤트를 받고 자체적으로 커서를 숨기지 않는다. raw 외부 실행 옵션·UI 전용 상태 분기를 추가하지 않았다. 제품 fullscreen 유휴 숨김·transport 접근 정책을 mpv 기본값에 위임하지 않는다.

Pinned mpv의 --no-config --list-options는 cursor-autohide 기본1000ms/fs-only=no/input-cursor=yes를 보고했다. 공식 [mpv pointer 옵션](https://mpv.io/manual/stable/#options-input-cursor)과 [자동 숨김 옵션](https://mpv.io/manual/stable/#options-cursor-autohide)의 의미를 대조했다. 이 기본값만으로 사용자가 본 즉시 숨김의 단일 원인을 확정하지 않는다.

## 검사 기록

두 논리 CPU·BelowNormal·descendant Job·offline/locked/-j1·단일 테스트 thread·기존 cache를 유지했다. 값은 wrapper 표시 시간이 아닌 원 receipt 기준이다.

| gate | 결과 |
| --- | --- |
| check-first | 12,114ms / 0, all-targets |
| focused-first | 25,090ms / 0, native launch pointer 회귀1pass |
| strict-first | 29,288ms / 0, all-targets -D warnings |
| full-first | 237,518ms / 0, 330pass/0fail/18ignored |
| strict-final | 1,692ms / 0 |
| harness-final | 6,959ms / 0 |
| fmt-gate | 10,170ms / 0, fmt --check |
| bins-final | 3,113ms / 0 |
| actual-first | 94,311ms / 0. 본문94.26초, 1pass/0fail/15filtered |

전체 검사 후 실제 포인터 어댑터가 먼저 창 캡션에서 영상으로 진입하도록 검사 단계만 보강했다. 제품 pointer args는 그대로이며 위 최종 strict/harness/fmt/bin은 보강된 소스다. 고정 harness의 exact case는 native_episode::actual_resolve_and_owned_native_lifetime, 명시 NATIVE_POINTER_EVIDENCE=1로 포인터 관측과 기존 실제 gate를 함께 실행한다. source-final은 src/tests/Cargo 입력 보존본이며 모든 외부 환경을 봉인한 provenance가 아니다.

## 실제 커서 관측

`actual-cli/native-episode-cli/native-pointer-observations.json`에서 Rust Windows 어댑터는 제품 title·정확한 pinned mpv 실행 파일 경로로 visible window 하나를 선택했다(PID78888). foreground 요청 수락 true, client1920×1080, origin(760,171)이었다. 원 위치를 RAII로 보존하고 아래 OS 상태를 기록했다.

| 단계 | 위치 | CURSOR_SHOWING / handle / 대상 창 |
| --- | --- | --- |
| 창 캡션에서 진입 준비 | 1720,155 | true / 존재 / 일치 |
| 영상 진입 후100ms | 1720,711 | true / 존재 / 일치 |
| 추가1,500ms 대기 | 1720,711 | true / 존재 / 일치 |
| 영상 안 이동 후100ms | 2200,711 | true / 존재 / 일치 |
| 아래 가장자리 후100ms | 1000,1231 | true / 존재 / 일치 |
| 원 위치 복원 | 2664,771 | true / 존재 / 일치 |

이것은 실제 standalone native 영상 창의 OS 포인터 표시 관측이다. 제품 GUI embedded 영역·fullscreen·transport overlay의 실제 접근·수동 시각 일치·전체 화면 품질 검토를 대신하지 않는다. 고정 PNG 미디어 캡처는 OS 커서를 포함하지 않으므로 커서 증거로 사용하지 않는다.

같은 실행에서 Next/Previous resolve·old1→new2→new3 commit·영상/오디오/시간 진행·전체 PNG decode·consumed handle/stale ID 거부·rate/track·pause/seek/volume/play·close3/reopen4/EOF4·종료 후 open Cancelled까지 통과했다. Next는 artifact269e41ce3dc0c2c0bcba5b5be3f1630cad098f669a3a5e973f8874cb59a236b3/PNG1920×1080·9,194,033bytes·SHAfcf62cda18584d3c23aab61336e0e4ec9508ec6033a8e5ac0b78f46ba94be987, Previous는 artifact2183de1cf68694a7a61593c31a573caee3b989ade320bbfb42faa690b406d18c/PNG1920×1080·5,581,145bytes·SHA0ddb9cf4c8bfd22832dd32fd50b420b6b5c2e7c3d8e5d5cce124fa488f39d650다. 기존4초 publication·해상도·검증 조건을 유지했다.

native-stop receipt는 engine4/IPC/stdout/stderr 모두 종료 true, diagnostics complete=true/truncated=false/read_failed=false, provider browser worker 종료 true, jobs completed2/cancelled0/interrupted0, pending transition null이다. runner117504/harness111512/최초mpv78888은 종료됐고, `owned-processes-after.json`은 관련 mpv/fixture 실행 파일/profile 프로세스 []다. 실행 후 고정 CLI/GUI/harness SHA도 동일했다. 첫 전환 checkpoint에서 보존한 seek 후 정지 관측의 원인을 해결한 증거로 해석하지 않는다.

## 고정 입력

| 입력 | SHA256 |
| --- | --- |
| player.exe | fa3566cfccecc2dac1bba02d99efc952d428f74f8c5a9d669f9ef8af18737a7d |
| player-cli.exe | 8493545882af29996897049fc2c59beb7cb9f4590e25eef32299349f5eca7ebd |
| jobs harness | f9b43b9e4bb936b0d4f8cbd28c3e5382ebb5912d41afe480e4d9ae9350564cdb |
| mpv.exe | f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790 |
| runner | 692c22751fac7f9999f2d833571dab6605afd58e66711370494e62dd8a62c463 |
| Cargo.toml | 03ee9a12b41a32758e63998653d7888dfb0443dafe9fcf66766209b746539a34 |
| Cargo.lock | 50323b23fb1bbf496e3ad42be71b4ccbdc3e1f38de1de5a5154a1ba5d78da5f7 |

무음은 이번에도 CLI의 명시 volume0 설정이며 청취 성공을 주장하지 않는다. legacy GUI 초기 mute의 handoff 불확실성은 [오디오 감사](native-audio-handoff-audit-2026-10-19.md)와 [후속 계획](../../../plans/player/native-pointer-and-audio.md)에 분리해 수리를 계속한다. 커서 standalone 관측 인수 이후 SDK는 heavy 슬롯과 등록 root 입력의 소비 동결을 중앙에 반환하며, 오디오 공통 계약은 다음 별도 검증 전까지 저자원 소스 준비로 진행한다.
