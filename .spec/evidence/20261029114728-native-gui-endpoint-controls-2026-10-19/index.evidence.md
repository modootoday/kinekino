---
id: evidence:kinekino/20261029114728
title: Owned normal GUI → 공유 SDK → CLI native control 실제 검증
status: active
created: 2026-10-29
updated: 2026-10-29
tags: [application]
---
# Owned normal GUI → 공유 SDK → CLI native control 실제 검증

후속 [정본 evidence 봉인·저장 byte 재검증](native-gui-endpoint-evidence-seal-2026-10-19.md)이 runtime 원본과 `.spec/evidence` 복사본을 구분해 연결한다. 아래는 당시 실제 실행 관측이며, 실행 전 hash가 없던 과거 CLI/source의 byte/compile provenance를 새 봉인으로 소급 확정하지 않는다.

## 범위와 소유권

UI 담당자가 전용 normal GUI r137과 local Sintel을 준비하고 paused 상태·입력 보류 lease를 명시했다. GUI PID83612, child mpv PID7368, 두 실행파일은 `runtime/review-runs/ui-local-host-20261019-r137/` 안의 player.exe/mpv.exe였다. endpoint PID/start generation을 실제 Windows process handle과 비교했다. root/UI의 최신 main LocalHost/typed native control 조립을 사용했으며 SDK 담당은 **src/main.rs·presentation·Cargo·model-spike를 수정하지 않았다**.

시험은 `tests/native_gui_endpoint.rs`의 명시 ignored test와 책임별 support 모듈(fixture/cli/native/scenario)이다. `PLAYER_NATIVE_GUI_FIXTURE` JSON이 없으면 실행되지 않는다. `fixture_kind=owned-normal-gui-sintel-v1`, exclusive_control=true, run ID, GUI/mpv PID·실행파일, endpoint, mpv pipe, 정확한 Sintel source를 요구한다. 일반 사용자 앱·미디어를 자동 탐색하거나 재사용하지 않는다.

미디어 고정은 UI 입력 보류 lease, Sintel read-share-only 파일 handle(쓰기/삭제/교체 open 차단), GUI/mpv 실제 executable·생존 검사, 모든 변경 직전 별도 native pipe의 source path 확인, receipt의 동일 engine-instance ID 확인을 조합했다. 확인 실패 시 추가 mutation/restore를 거부한다. 공개 SDK에는 media revision CAS가 아직 없으므로 비협조적 외부 GUI 입력과 원자적으로 경쟁하는 것을 보장하지 않는다. 이 시험은 합의된 owned fixture에 한정한다.

## 실제 전송과 독립 관측

모든 변경은 실제 `player-cli --endpoint ENDPOINT --input -` subprocess의 공통 Request로 전송했다. CLI를 통한 GUI 조작 자동화, 임의 OS command, direct mpv seek/play/pause/set_property는 사용하지 않았다. client/host는 기존 same-user/same-integrity/elevation 경계를 그대로 통과했다. [권한 경계 근거](local-host-privilege-boundary-2026-10-19.md)를 참조한다.

독립 관측은 별도 owned mpv pipe 연결에서 **읽기 전용 get_property**만 사용했다. Windows GetNamedPipeServerProcessId가 mpv7368인지 확인하고 path/pause/time-pos/volume/duration/idle-active/mute를 조회했다. GUI가 게시한 playback.status DTO와 mpv의 실제 property를 각각 저장했다. 이 연결은 앱의 property observer·command receipt 연결과 별개다.

CLI 21회 모두 JSON request_id 일치·exit0·stderr 없음: initial status, capabilities, playback.status/4control의 schema 5개, seek/status, volume/status, play/status, pause/status, restore pause/status·volume/status·seek/status다. 제어와 복원 receipt 7개는 모두 **accepted / unobserved / engine_instance_id="1"**이었다. 이를 적용 완료 응답으로 바꾸지 않았으며, 아래 결과는 후속 property 관측이다.

| 단계 | 실제 mpv pause | engine position | volume | 관측 |
|---|---|---|---|---|
| 원래 상태 | true | 9.625s | 80 | Sintel duration888.032s, idle=false, mute=false |
| CLI seek 후 | true | 10.625s | 80 | SDK status 및 독립 property 일치 |
| CLI volume 후 | true | 10.625s | 40 | mute=false 그대로 |
| CLI play 후 | false | 10.958s | 40 | seek 위치보다333ms 실제 진행 |
| CLI pause 후 | true | 11.083s | 40 | 추가150ms 뒤에도11.083s로 정지 |
| 복원 후 | true | **9.625s** | **80** | mute=false·같은 Sintel 그대로 |

test의 seek/restore 허용 오차는100ms이며 실제 이번 복원값은 원래 position과 같았다. `published_age_ms`는 SDK publication age이지 native event timestamp가 아니므로, target의 변화와 별도 mpv property를 함께 확인했다. per-command mpv ACK correlation·지속 completion/job·프레임 정확도/미디어 전체 재생 품질을 검증한 것이 아니다.

## 근거 보존과 복원

- immutable evidence run: `runtime/native-gui-endpoint/r137-native-cli-20261019-a/`.
- [최종 report](../../../../runtime/native-gui-endpoint/r137-native-cli-20261019-a/report.json): scenario_passed=true, restoration_passed=true, errors=null.
- [initial native](../../../../runtime/native-gui-endpoint/r137-native-cli-20261019-a/native-initial.json), [seek native](../../../../runtime/native-gui-endpoint/r137-native-cli-20261019-a/native-after-seek.json), [playing native](../../../../runtime/native-gui-endpoint/r137-native-cli-20261019-a/native-during-play.json), [restored native](../../../../runtime/native-gui-endpoint/r137-native-cli-20261019-a/native-restored.json).
- `001..021` request/response/exit JSON, fixture.json 및 CLI executable identity, initial-status.json, volume/pause/stability native JSON도 같은 디렉터리에 보존했다. 재실행은 새로운 run ID·fresh owned lease를 요구하며 기존 근거를 덮어쓰지 않는다.

actual ignored invocation: `PLAYER_NATIVE_GUI_FIXTURE=runtime/native-gui-endpoint/fixture-r137.json`를 명시한 뒤 `cargo test --locked --offline --target-dir runtime/parallel-build/sdk -j 1 --test native_gui_endpoint -- --ignored --nocapture`. **1 passed / 0 failed**, test execution18.66s였다. 이 fixture의 프로세스는 이미 종료됐으므로 그대로 재실행하면 소유/생존 확인에서 거부되어야 한다.

시나리오 성공·실패와 관계없이 원래 paused/time/volume 복원을 시도하고 report에 결과를 분리한다. 단, source/owned process 검증이 깨진 경우 다른 미디어에 복원을 보내지 않는다. 뜻하지 않은 Rust unwind도 catch한 뒤 복원을 수행한다. CLI subprocess만 자체 deadline8초로 관리하며 사용자의 GUI/mpv를 종료하지 않는다.

복원 후 UI 담당자가 실제 창의 동일 Sintel·00:09/14:48·Play glyph·volume rail과 native log paused=true/time9.625/volume80/error=null을 교차 확인했다. 이후 UI 담당자가 캡션 닫기로 자신의 GUI를 종료했고 player/mpv가 없음을 회신했다. [UI 측 관측](../presentation/local-host-gui-review-2026-10-19.md)을 참조한다. SDK 담당은 GUI 종료·키보드/마우스 입력을 하지 않았다.

## 최종 검사와 남은 범위

최종 `cargo fmt --check`, `cargo clippy --locked --offline --all-targets --target-dir runtime/parallel-build/sdk -j 1 -- -D warnings`는 exit0으로 통과했다. `cargo test --locked --offline --target-dir runtime/parallel-build/sdk -j 1 --quiet`도 exit0, **121 passed / 0 failed / 9 ignored**였다(lib85, CLI unit2, cli_contract4, cli_request_identity1, frame_capture2, local_host9, playback_control4, playback_status3, sdk_capability_parity2, subtitle_conversion9 passed). 공유 workspace UI의 기존 시험 추가를 포함한 실행 당시 집합이다.

새 실제 GUI 시험은 기본 실행에서는 ignored1개이며, 위 명시 실행에서 **1 passed**로 별도 검증했다. 기존 외부 미디어 ignored gate를 실행하거나 기본 ignore를 성공으로 합산하지 않았다. 첫 compile에서 evidence sync 오류의 &str/String 반환형 불일치 한 곳을 수정 후 실제 실행·전체 검사·clippy를 통과했다.

다른 사용자 앱/재생상태, 별도 Medium/Limited process의 elevated GUI 접속, media revision CAS, remote/non-Windows, DPI/VM/Pages, AI worker/model 획득·추론, 새 장시간 시험은 이번 결과에 포함하지 않는다. 일반 CLI in-process 및 기존 unsupported/권한 실패 gate는 기존 시험을 유지한다.
