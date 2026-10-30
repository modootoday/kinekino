---
id: evidence:kinekino/20261029114310
title: Fresh r145 실행파일 pin·CLI/native 실제 검증과 봉인
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [application]
---
# Fresh r145 실행파일 pin·CLI/native 실제 검증과 봉인

## 새 실행이 필요한 이유와 독점 fixture

r137은 실제 동작 관측은 통과했지만 실행 시점 executable SHA가 없었다. 그 [기존 봉인과 한계](native-gui-endpoint-evidence-seal-2026-10-19.md)는 변경하지 않는다. 이번 r145는 사용자가 명시 승인한 **새 실행 근거**이며 r137 PID/endpoint/trace/바이너리 귀속을 재사용하거나 소급 보완한 것이 아니다.

UI 담당자의 기존 검사·r144 시각 검토가 끝난 뒤 fresh normal GUI r145와 paused Sintel을 인계받았다. GUI119792, mpv117412(parent119792), 실행 디렉터리는 `runtime/review-runs/ui-local-host-pin-20261019-r145/`였다. endpoint의 PID/start와 실제 Windows process/image를 비교했다. `fixture-validation.json`은 GUI start FILETIME134368251857549820, mpv start134368252326225859 및 실제 image path 검증·source write/delete guard를 기록한다.

UI는 입력/종료/해당 실행파일 교체를 보류했고 시험 중 source도 고정했다. SDK 담당은 main/presentation/SDKcontract를 수정하거나 GUI를 자동화하지 않았다. Source는 고정 local Sintel.2010.720p.mkv만 허용하고 read-share-only handle로 쓰기·삭제·교체 open을 차단했다. 모든 mutation 직전 mpv pipe의 실제 PID·source path를 확인하고 동일 engine ID를 유지했다. 공개 SDK의 media revision CAS를 구현하거나 검증한 것은 아니다.

## 실행 전·후 파일 바이트 pin

`executables-before.json`은 첫 native 관측/CLI/control 전에 네 파일의 original/copied/executed path, bytes, SHA256을 기록했다. 각 source-before/source-after/copy hash·bytes가 같아야 하며 source와 copy의 read-share lock을 시험 종료까지 유지했다. `executables-after.json`에서 복원 후 네 source/copy가 모두 그대로임을 재검증했다.

| 역할 | bytes | 실제 pin SHA256 |
|---|---:|---|
| GUI + LocalHost | 71995904 | `ee68bf1e24a6ec4f641cfe1f469b06dda1bcbbd7ef0af445147699fee0feae58` |
| mpv | 121051136 | `f8bf7e8fd8d6696e35441c55091498060e4594f724a132fd6c46579341bc0790` |
| copied CLI | 2844160 | `594961f8b4166f42692d1d28e03bccaeac4545dd1b9a4b720c3b09fd70d3d67f` |
| test harness | 2122240 | `f1914b232865011e3be234a56613815a01b91e7228fe88d1ebe1d6ddf8d4b0aa` |

CLI는 SDK target 원본이 아니라 **fresh evidence의 binaries/player-cli.exe만 실행**했다. 각 child 생성 직후 QueryFullProcessImageNameW로 실제 image가 그 copy인지 확인하고 PID·verified_copied_cli·SHA/bytes 연결을 21개 exit JSON에 저장했다. 예: [008-seek-exit](../../../evidence/player/native-gui-endpoint/r145-native-cli-pin-20261019-a/run/008-seek-exit.json)의 child103268은 위 copied CLI hash/bytes에 연결된다. std::process::Command에 명시된 경로만으로 성공을 추정하지 않았다.

scope는 **통합 작업 전부터 복원 후까지의 실행파일 바이트와 process-image 경로**다. GUI/mpv/test harness는 인계/실행 중인 process의 image file을 preflight에서 snapshot했다. GUI/test loader 시작 전 capture, process memory 전체 attestation, exact compiler inputs, signature chain을 검증했다고 주장하지 않는다. source snapshot은 여전히 참고 자료다. r137의 historical_run_digest_recorded=false와 이번 fresh executable_pins_verified=true를 혼동하지 않는다.

## 21 copied CLI 요청과 실제 native 관측

실제 `player-cli --endpoint --input -`의 initial status, capabilities, playback.status/4controls schema5개, seek/status, volume/status, play/status, pause/status 및 restore pause/status·volume/status·seek/status의 **21회 모두 exit0/request_id 일치/stderr 없음**이었다. 현재 user/IL/elevation 경계와 실제 host PID/start 확인을 우회하지 않았다. 중간 fallback·재전송·임의 shell·GUI 입력 자동화가 없다.

제어와 복원 receipt7개는 모두 **accepted / unobserved / engine_instance_id="1"**이었다. 별도 owned mpv pipe 연결의 읽기 전용 get_property(path/pause/time-pos/volume/duration/idle-active/mute)와 SDK NativeObservation status를 각각 저장했다. get_property 응답은 적용 완료 ACK가 아니며 아래는 후속 실제 property 관측이다.

| 단계 | pause | position | volume | mute |
|---|---|---|---:|---|
| initial | true | 8.583s | 80 | false |
| seek 후 | true | 9.583s | 80 | false |
| volume 후 | true | 9.583s | 40 | false |
| play 후 | false | 10.125s | 40 | false |
| pause 후 | true | 10.250s | 40 | false |
| 150ms 안정성 확인 | true | 10.250s | 40 | false |
| restored | true | **8.583s** | **80** | **false** |

duration888.032s, idle=false 및 같은 Sintel source를 유지했다. seek/restore tolerance는100ms, pause stability tolerance는50ms이며 실제 복원값은 처음과 같았다. 프레임 정확도·원자적 CAS·per-command completion ACK·endurance·성능 측정으로 확대하지 않는다. launch 인수의 --mute=yes를 실제 mute=true로 간주하지 않았고 **실제 initial/restored false**를 기록했다.

test 실행23.22s, **1 passed / 0 failed**. 시나리오와 복원 후 해시 검증까지 성공했으며 UI 담당자에게 lease를 반환했다. UI가 실제 창의 눈산 frame/Play glyph/00:08·14:48/volume rail과 mute=false를 재관측했다. 최초 caption 입력 뒤 CIM 공백을 프로세스 종료로 보고한 관측은 지연 shell 출력을 끝까지 확인하지 않은 오류로 정정됐다. 후속에서 잔존 r145를 다시 활성화·관측하고 실제 caption 닫기를 수행했으며 새 창 목록에서 r145가 사라짐을 확인했다. 이후 r146도 닫아 UI가 보고한 최종 r145/r146 창 목록은[]였다. 창 소멸을 OS PID 종료의 독립 증명으로 확대하지 않는다. [UI 정정 관측](../presentation/local-host-pin-gui-review-2026-10-19.md) 참조. SDK 담당은 종료·후속 IPC 호출을 하지 않았고 복원/4exe 검증 결과는 변경되지 않았다.

## 원본·봉인본·고정 anchor

runtime 원본: `runtime/native-gui-endpoint/r145-native-cli-pin-20261019-a/`.

정본 봉인: `.spec/evidence/player/native-gui-endpoint/r145-native-cli-pin-20261019-a/`.

format2 [manifest.json](../../../evidence/player/native-gui-endpoint/r145-native-cli-pin-20261019-a/manifest.json)은 **36,397 bytes**, SHA256은 다음이다.

```text
89205291bc7f6609f223ef806792fc6fa8913474da9cd179f1bc183ce6915cc7
```

96 entries +manifest/sidecar2: run81파일(21 req/resp/exit63개, metadata/native14개, binary4개), support source6개, seal tool source9개다. `executable_pins_verified=true`; before/after4pin, copied child21image, receipt/property/restoration을 저장 바이트에서 검사했다. 원본·복사·재독 SHA/bytes를 비교하고 fresh staging을 verify한 뒤 final로 게시했으며 기존 경로가 있으면 덮어쓰지 않는다. 복사본 read-only 속성과 외부 digest pin은 WORM/전자서명을 대신하지 않는다.

원본 PID/endpoint·source·binary가 없어도 저장 byte verify가 가능하다. 새 actual 시험은 lease 반환·후속 창 닫기를 마친 fixture-r145.json을 재사용하지 말고 새 소유 GUI/lease/PID/start/run ID와 별도 승인이 필요하다. r137 원본/봉인 및 historical identity 한계는 그대로다.

## 재실행·검증 명령과 일반 gates

실제 실행에서만 승인된 fixture 환경 변수를 지정했다. 기존 r145 lease는 만료됐으므로 아래 actual 명령은 새 승인 fixture로만 실행한다. PID 소멸 여부를 재사용 허가로 해석하지 않는다.

```text
cargo test --locked --offline --target-dir runtime/parallel-build/sdk -j 1 --test native_gui_endpoint -- --ignored --nocapture --test-threads=1
```

현재 봉인 byte만 재검증하는 명령은 GUI/IPC를 실행하지 않는다.

```text
cargo run --locked --offline --manifest-path tools/native-gui-evidence/Cargo.toml --target-dir runtime/parallel-build/sdk-evidence -j 1 -- verify-pinned r145-native-cli-pin-20261019-a --manifest-sha256 89205291bc7f6609f223ef806792fc6fa8913474da9cd179f1bc183ce6915cc7
```

양성 external-pin verify exit0. 64개의0을 external pin으로 제공한 negative gate는 **exit1 / fresh manifest external SHA mismatch**로 거부했다. 이미 완료된 `seal-pinned r145-native-cli-pin-20261019-a`는 기존 final 경로를 덮어쓸 수 없다. 상세 interface는 tools/native-gui-evidence/README.md를 따른다.

- 일반 SDK: `cargo test --locked --offline --target-dir runtime/parallel-build/sdk -j 1 --quiet`: **123 passed / 0 failed / 9 ignored**, exit0. lib86, CLI unit2, cli_contract4, cli_request_identity1, frame_capture2, local_host9, native hash unit1, playback_control4, playback_status3, sdk_capability_parity2, subtitle_conversion9 passed. 공유 UI source의 당시 집합이다.
- actual ignored1 pass는 위 기본 ignore와 별도 보고하며 중복 합산하지 않는다.
- root cargo fmt --check / clippy --locked --offline --all-targets --target-dir runtime/parallel-build/sdk -j1 -- -D warnings: exit0.
- 독립 evidence tool fmt/test3/clippy all-targets -Dwarnings: exit0, -j1 전용 target.

root 모델/엔진 compile과 분리했고 모델·추론·DPI/VM/Pages·다른 사용자 앱·SDKcontract/main/presentation 편집은 수행하지 않았다. 다음 scope는 root의 새 실행 근거 검토·중앙 고정 anchor 감사이며 이 문서가 과거 r137 compile/run provenance를 재귀속하지 않는다.
