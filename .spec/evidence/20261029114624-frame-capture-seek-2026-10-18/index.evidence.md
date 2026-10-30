---
id: evidence:kinekino/20261029114624
title: 원본 PTS 프레임 캡처 입력 seek 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 원본 PTS 프레임 캡처 입력 seek 검증

2026-10-18. 변경 범위는 `src/infrastructure/frame_capture/**`, `tests/frame_capture.rs`와 이 검증/coordination 문서다. application/domain/API/CLI runner/GUI/Cargo/모델 도구는 변경하지 않았다.

## 문제와 보존한 관측

기존 어댑터는 원본 처음부터 `select=gte(t, requested_seconds)`까지 디코드했다. root의 기존 CLI SHA256 `805b5b1ff481f9351e4ed8bb95f52a76ec9971151dce57f8b813d368487662ac`에서 Sintel PTS `820000000`과 기본 30000ms 요청은 timeout/noPNG였다. 원본 실패 파일 `runtime/ai-spike/results/ocr-natural-credit-capture-request.json`, `ocr-natural-credit-capture-response.json`은 수정하거나 재실행하지 않았다. 이번 검증은 작은 합성 미디어만 사용했다. 실제 820초 요청의 성공·성능은 root의 후속 검증 항목이다.

## 공식 근거와 구현

[FFmpeg 공식 옵션 문서](https://ffmpeg.org/ffmpeg.html)의 `-ss`, `-accurate_seek`, `-seek_timestamp`, `-copyts`, `-start_at_zero`를 대조했다. 입력 `-ss`는 앞선 seek point로 이동하며, `-noaccurate_seek`는 그 지점부터의 preroll을 보존한다. `-seek_timestamp 1`은 파일 시작 offset을 더하지 않고 절대 timestamp로 해석한다. `-copyts`는 원본 offset을 유지한다.

- 요청에서 2000000us를 뺀 값과 원본 start의 최댓값을 seek 지점으로 계산한다. 원본 start 이하의 요청 구간은 seek하지 않는다. 나머지는 `-seek_timestamp 1 -noaccurate_seek -ss seconds.microseconds`를 `-i` 앞에 전달한다. `-copyts`를 유지하며 시간 원점을 옮기지 않는다.
- ffprobe의 원본 비디오 `time_base=num/den`을 읽고 선택 경계를 `ceil(request_us * den / (num * 1000000))` tick으로 계산한다. `select=gte(pts, threshold),showinfo`로 소수 초 비교의 경계 오차를 피한다. overflow·0 time base·double 정수 정밀도 범위 밖 경계는 unsupported로 거부한다.
- showinfo의 실제 time base가 probe와 유리수로 동등한지, 실제 원본 PTS·PNG 크기가 계약을 만족하는지 검증한 후만 원자적으로 새 파일을 게시한다. receipt는 계속 원본 ticks/time_base/정수 microseconds를 반환한다.
- 기존 고정 argv, file-only protocol whitelist, forced mov/matroska demux, mov 외부 drefs 금지, 로컬 소스 guard, 16GiB/픽셀/출력 크기 제한, overwrite 금지, 기본30000ms·최대120000ms 및 단일 deadline을 유지했다. 실패 시 child kill/wait와 pipe reader join 경로는 변경하지 않았다.

2000000us는 고정 timestamp preroll이다. 실제 demuxer는 더 앞선 keyframe을 선택할 수 있으므로 디코드 시간이 2초분으로 제한된다는 의미가 아니다. 인덱스가 없거나 GOP가 길거나 손상된 소스는 같은 deadline 안에서 실패할 수 있다. deadline 연장이나 전체 재디코드 fallback은 추가하지 않았다. 기존 format duration 기반 범위 추정의 컨테이너별 정밀도도 이번 변경의 검증 범위 밖이다.

## 실제 검증

로컬 FFmpeg/FFprobe는 모두 `6.1-essentials_build-www.gyan.dev`였다. 테스트·fixture 작성·판정은 Rust다.

`seek_matches_independently_enumerated_first_frame_and_pixels`는 96×64, 12fps, 5초 합성 H.264 소스 6종을 생성한다: 0-start CFR MP4, +5초 MP4, +5초 MKV, frame drop VFR MKV, 48-frame 긴 GOP MP4, open GOP MP4. B-frame과 2개 이상 keyframe을 실제 ffprobe에서 확인하고, VFR은 서로 다른 frame 간격을 확인한다. 0-start MP4는 음수 DTS와 비음수 PTS의 공존도 확인한다.

각 소스에 상대 PTS `0`, `250000`, `583333`, `583334`, `3000000`, `4250000`, `4500001`의 7개 요청을 적용했다. 전체 42개 포트 요청이 통과했다. 독립 ffprobe 전체 프레임 목록에서 정수 유리수 비교로 **첫** 해당 프레임 index를 고르고 ticks/time_base/actual_us를 정확히 비교한다. 참조 PNG는 seek와 timestamp select 없이 처음부터 디코드하여 `select=eq(n,index)`로 만든다. 모든 출력의 RGB24 PNG 디코드 픽셀이 참조와 일치했다. 각 소스의 마지막 요청은 실제 CLI의 full Request stdin과 기본 timeout으로도 실행하여 6개 CLI의 ticks/time_base/픽셀 일치를 확인했다.

음수 시작 fixture는 정상 합성 MP4의 ctts를 Rust에서 signed composition offset으로 변환해 생성했다. muxer가 음수 시간을 보정하는 단순 encoder offset 방식은 fixture 조건을 충족하지 않아 채택하지 않았다. 최종 fixture는 독립 ffprobe의 음수 stream start와 음수 decoded frame PTS를 실제 확인하며, capture는 unsupported/noPNG로 거부한다. 음수 DTS를 가진 정상 0 PTS B-frame 소스는 위 성공 검증에 포함된다.

`deadline_kills_and_reaps_child`는 Rust 테스트 실행 파일을 자식으로 띄워 exclusive 파일 핸들을 잡고 30초 대기하게 한다. 어댑터의 2초 deadline에서 Timeout을 받고 5초 이내 반환 및 Windows exclusive 핸들 재획득으로 종료·회수를 확인했다. 기존 native 회귀는 실제 CLI/typed client 일치, timeout/noPNG, overwrite/hardlink 보호, 동시 출력 단일 승자, 16GiB 초과 거부, URL/playlist/잘못된 signature 및 음수·숫자 JSON PTS 거부를 재확인했다.

실행 명령과 결과:

```text
cargo fmt --check
cargo test --locked --offline --target-dir runtime/parallel-build/sdk-j2 -j 2 -- --test-threads=1
cargo test --locked --offline --target-dir runtime/parallel-build/sdk-j2 -j 2 --test frame_capture -- --include-ignored --skip actual_local_sintel_frame_and_response_are_saved_for_review --test-threads=1
cargo test --locked --offline --target-dir runtime/parallel-build/sdk-j2 -j 2 --test frame_capture seek_matches -- --ignored --nocapture
cargo clippy --locked --offline --target-dir runtime/parallel-build/sdk-j2 -j 2 --all-targets -- -D warnings
cargo build --locked --offline --target-dir runtime/parallel-build/sdk-j2 -j 2 --bin player-cli
```

모두 통과했다. 전체 기본 테스트는 76 passed/8 ignored이며, 별도 native frame suite는 7 passed/실제 Sintel1 filtered out이었다. 마지막 테스트 전용 CLI 동등성 assertion 추가 후 seek test와 fmt/clippy를 다시 통과했다. 기존 52초 Sintel ignored test와 외부 자막 corpus ignored test는 이번 범위에서 실행하지 않았다. GUI/모델/실제820초 성능 완료를 뜻하지 않는다.

## 인계할 코드와 CLI 식별

SHA256, 최종 검사 시점:

| 경로 | SHA256 |
| --- | --- |
| `src/infrastructure/frame_capture/mod.rs` | `576acb8722f2ecddf22919940bbe26d29937b0383acde820eb2b10441c279c02` |
| `src/infrastructure/frame_capture/probe.rs` | `0e6ddca7a2d45f8cce30198f11627ff2e38e3c8575da3a81e89c3706af5ecd1a` |
| `src/infrastructure/frame_capture/seek.rs` | `2f463141b2751bd9db8343bb6cccf97c007be0898f8f6251684d7e80558674bf` |
| `src/infrastructure/frame_capture/process.rs`（본체 유지, 테스트 추가） | `96f83ba0c57de3eb6bb9e0e37cceeda4ded04265a2170f9a358964c259978ace` |
| `src/infrastructure/frame_capture/png_validation.rs`（변경 없음） | `e463a604d5e3881dff695049ac42026f1d342af94618d468ac9b8fc2c7ad00e8` |
| `tests/frame_capture.rs` | `ee9683f56f66f0e06a82fcc13f441855dd65fc585a216dc2ff2a17ffe421103d` |
| `runtime/parallel-build/sdk-j2/debug/player-cli.exe` | `dc2b265922b9913e47b92b08699017ebc26a3de8f2cce5cdde78faacf943849c` |

원본 실패 응답을 보존한 채 root가 이 CLI hash를 확인하고 새로운 요청·응답·PNG 경로로 820초 요청을 재검증한다. 이 문서는 그 후속 결과를 선행하여 성공으로 기록하지 않는다.
