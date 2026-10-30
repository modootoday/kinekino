---
id: evidence:kinekino/20261029113629
title: Rust 화자 평가기 준비 및 합성 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Rust 화자 평가기 준비 및 합성 검증

2026-10-18. **독립 도구 `tools/diarization-evaluation` 구현·합성 검증·빌드 보존을 완료했다. 실제 AMI 평가나 모델 추론은 수행하지 않았다.** root가 실제 코퍼스 취득·crop·모델 실행·최종 DER/JER 실측을 소유한다. 원본 Python/Perl scorer와의 실행 수치 parity는 미검증이다.

## 구현과 책임 경계

[독립 Cargo manifest](../../../../tools/diarization-evaluation/Cargo.toml)는 자체 `[workspace]`를 사용하므로 제품/root Cargo manifest·lockfile을 바꾸지 않는다. 새 소스·테스트·빌드 증거 보존 로직은 모두 Rust다. 코퍼스/가중치 다운로드·모델 load/infer·Python/Perl 실행·기존 model-spike/제품/UI/동결 증거 변경은 없다. 빌드에 필요한 Rust crate source 세 개(pathfinding/integer-sqrt/deprecate-until)는 Cargo가 취득했다. 이를 모델/코퍼스 취득으로 세지 않으며 Rust 의존성의 checksum은 자체 Cargo.lock에 고정돼 있다.

| 계층 | 책임 |
|---|---|
| `src/domain/time.rs`, `intervals.rs` | decimal 초→정수 microsecond, 반열림 구간, 같은 화자 union, crop/UEM 교집합 |
| `src/domain/assignment.rs` | 고정 `pathfinding=4.16.0` Hungarian wrapper, finite/크기/overflow 범위 검사; `ordered-float=5.5.0` JER 비용 |
| `src/domain/metrics.rs` | continuous DER, dscore-style10ms JER, 서로 다른 전역 mapping 및 오류 components |
| `src/application.rs` | 한 파일의 검증·평가·provenance report 유스케이스 |
| `src/infrastructure/` | bounded local I/O·SHA256·WAV·RTTM/UEM·stdout 해석, 새 출력 및 build snapshot |
| `src/presentation.rs` | CLI 인자 해석·결과 표시 |
| `src/main.rs` | CLI 조립/실행만 |
| `tests/synthetic_*.rs` | 실제 corpus가 없는 합성 known-answer·실패·CLI·snapshot 검증 |

Hungarian은 [pathfinding 공식 API](https://docs.rs/pathfinding/4.16.0/pathfinding/kuhn_munkres/index.html)의 maximum/minimum을 사용한다. 탐욕 매핑을 'optimal'로 표시하거나 reference 화자 수를 모델 oracle 입력으로 제공하지 않는다. 테스트는 wrapper의 비탐욕 예와 독립 수식으로 계산 가능한 matrix 기대값도 확인한다.

## 실행 계약

평가 CLI는 다음 **8개 named option을 각각 한 번씩** 받는다. 모든 파일 경로는 기존 로컬 드라이브의 절대 경로여야 한다. Windows UNC/network/device 경로를 거부하며 입력 파일의 canonical 경로와 출력 parent도 확인한다. 파일/화자 ID는 1~128자의 ASCII 영숫자 및 `_ - . :`로 제한한다.

| option | 값과 의미 |
|---|---|
| `--reference` | 원본 시간 축 SPEAKER RTTM의 절대 경로 |
| `--uem` | 원본 시간 축의 명시적 UEM 절대 경로; silence도 평가 영역에 남김 |
| `--file-id` | RTTM/UEM 모두에서 정확히 일치해야 하는 file ID |
| `--source-start-us` | 원본 crop 시작 offset, unsigned integer microseconds |
| `--duration-us` | 실제 입력 WAV의 정확한 crop 길이; 지원 범위 10,000~60,000,000us |
| `--hypothesis-stdout` | 새 prototype의 stdout JSON 파일 절대 경로 |
| `--audio` | prototype가 실제 입력한 local mono16kPCM16 crop WAV 절대 경로 |
| `--output-new` | 존재하지 않는 새 JSON 파일의 절대 경로. parent는 기존 디렉터리여야 함 |

예전 `.stdout.log` 등 이름을 추측해 선택하지 않는다. root가 생성한 **실제 stdout 경로**를 사용한다. `--reference`/`--uem`에는 원본 annotation을 그대로 주고 source offset은 한 번만 적용한다. root의 AMI 사례는 `file-id=ES2004a`, source-start-us=0, duration-us=60000000이다. 실제 참조/fixture 경로와 SHA는 root의 acquisition/corpus manifest가 정본이며 이 도구 준비 작업에서는 해당 실제 파일을 읽지 않았다.

검증은 입력당 읽기 상한(RTTM8MB/UEM1MB/stdout8MB/WAV4MB), 한 파일 source time≤1일, 최대32화자, 최대4096개의 평가 interval/hypothesis span 및 최대6000 JER grid point로 제한된다. RTTM을 읽을 때 원본 최대100,000줄의 각 행도 crop 밖이라는 이유로 검증을 생략하지 않는다. 실제 benchmark 전체 corpus를 처리하는 일반 배치 scorer라고 주장하지 않는다.

### WAV와 stdout identity

WAV는 실제 header상 sample_rate16000/channels1/PCM integer16이어야 하며 모든 sample을 끝까지 읽어 frame 수를 확인한다. `frames×1,000,000 = duration_us×16000`이 정확히 맞아야 한다. 파일 전체 SHA256이 stdout 최상위 `input_sha256`과 일치해야 한다. 실제 PCM hash만 같고 container SHA가 다르면 이 계약은 거부한다.

stdout의 필수 구조는 최상위 `input_sha256`, `result.status="inferred-prototype"`, `result.seconds`, **`result.source_offset_applied_us=0`**, `result.segments`, `result.overlap_segments`다. `seconds`는 WAV frames/16000과 같은 finite f64 값이어야 한다. 이전 source offset107초의 stdout이나 metadata가 없는 구형 결과는 거부한다. 다른 모델 metadata 등 추가 필드는 보존된 raw stdout SHA로 연결하며 임의 프로그램/셸 실행에 사용하지 않는다.

`segments`의 각 span은 unsigned integer `in_us < out_us <= duration_us`와 명시적 string/null `speaker_id`를 가져야 한다. `overlap_segments`도 모든 span을 검증하고 이번 v0.1 계약은 **명시적 `global_speakers:null`만** 받는다. 미래 resolved overlap 형식을 조용히 무시하지 않고 새 계약이 필요하다는 오류를 낸다.

system hypothesis는 실제 non-null `speaker_id`만으로 구성한다. null short span과 unresolved overlap은 별도 union duration 및 UEM 내 duration으로 보고한다. 이 시간들을 **reference나 UEM에서 빼지 않으며**, 그 시간에 known-speaker hypothesis가 없으면 MISS 등 오류로 남는다. null을 dummy global speaker로 묶거나 local powerset class를 window 간 같은 인물로 추측하지 않는다.

### reference/crop 검증

RTTM은 정확히 10필드, SPEAKER, 지정 file ID/channel1, `<NA>` optional field만 받는다. onset/duration은 음수·NaN/inf·지수 표기·빈 소수·6자리를 넘는 소수·precision/시간 overflow를 거부한다. 1us보다 작은 원본 정밀도를 조용히 반올림하지 않는다. duration은 end가 아니며 checked addition으로 end를 계산한다. 동일 화자의 겹치거나 접하는 interval은 union하고 다른 화자의 overlap은 유지한다.

원본 `[s,e)`와 source crop `[a,b)`를 교차한 뒤 a를 빼고, 원본 UEM도 같은 규칙으로 crop한다. crop 후 reference와 hypothesis 양쪽에 명시 UEM을 적용한다. 자동으로 speech union을 UEM으로 삼지 않는다. WAV가 원본에서 올바르게 crop됐다는 사실 자체는 원본 audio/가공 manifest를 소유한 root가 검증해야 한다. 이 CLI는 supplied source offset을 원본 WAV 없이 독립 입증할 수 없다.

## 계산하는 점수와 한계

DER은 collar0·overlap included다. continuous decimal-microsecond 경계의 원자 구간에서 R/H/C에 대해 `MISS=max(R-H,0)`, `FA=max(H-R,0)`, `CONF=min(R,H)-C`에 구간 길이를 곱한다. 분모는 reference speaker-time이다. 전역 co-occurrence matrix의 합을 최대화하는 one-to-one Hungarian mapping을 crop 전체에 한 번 적용한다. 보고에는 세 오류와 분모의 정수 microseconds·실제 mapping·reference/hypothesis 화자 ID 목록을 남긴다. 점수가 100%를 초과해도 clamp하지 않는다.

JER은 별도의 `1-IoU` 비용 matrix를 최소화한다. dummy/unmatched reference는 error1이고 reference 화자별 macro 평균을 낸다. DER mapping을 재사용하지 않는다. 0.010*k f64 grid의 left-searchsorted onset/offset 범위와 명시 UEM을 쓰며 midpoint/nearest sampling이 아니다. no collar/min_ref_dur0/overlap included다. ref/hyp 화자별 frame count와 scored frame 수를 보고한다.

**원본 dscore와 수치 parity는 미검증**이며 report에도 `external_scorer_parity_verified=false`를 고정한다. 이 구현은 RTTM decimal을 정확한 microsecond로 정규화한 뒤 f64 grid를 만든다. 원본 scorer가 binary64 onset+duration을 더한 값과 정확한 grid 경계에서 차이가 날 가능성이 있다. 이를 숨기고 bit-identical이라고 부르지 않는다. continuous interval JER, collar 보조 점수, overlap-excluded 점수, NIST lexical/metadata no-score 등은 구현하지 않았다.

reference speaker-time이 없으면 전체 평가를 오류로 종료하며 DER0으로 성공 처리하지 않는다. continuous speech는 있지만 어떤 reference 화자도 10ms grid에 남지 않는 경우도 조용히 제외하거나 NaN을 Hungarian에 전달하지 않고 JER 오류로 종료한다. 출력 report는 검증/두 점수가 모두 성공한 뒤 `create_new`로 한 번만 생성한다. 기존 출력과 경합해도 덮어쓰지 않는다. 실제 prototype frame 균등 투영의 timestamp 근사는 이 도구가 보정하지 않으며 gold를 prediction에 맞춰 이동시키지 않는다.

## 합성 검증 결과

최종 **19개 integration test 통과**(CLI5 / 입력3 / 지표11). unit/doc test의 0개를 추가 테스트처럼 세지 않았다. 모든 WAV·RTTM·UEM·stdout fixture는 Rust 테스트가 생성한 합성이며 AMI나 모델 추론 결과가 아니다.

| 독립 기대값/실패 예 | 확인 결과 |
|---|---|
| 동일/화자 이름 permutation | DER0/JER0 |
| empty hypothesis | DER100/JER100 및 전체 MISS |
| 1초 overlap의 두 reference 중 한 화자만 예측 | 분모2초, DER50/JER50 |
| ref1초 + hypothesis4초 | FA3초, **DER300/JER75**, clamp 없음 |
| ref A[0,1),B[1,2) → hyp X[0,2) | DER50/JER75 |
| ref 한 화자를 두 cluster로 과분할 | DER50/JER50 |
| 같은 화자 중복 interval | 중복 speaker-time 없이 DER0 |
| 명시 UEM crop | 양쪽 같은 영역만 점수화; UEM/scored frame 확인 |
| ref[.001,.011), hyp[.010,.020) | continuous DER180 / left-grid JER0, grid와 continuous 시간 구별 |
| 최대 cooccurrence `[[9,8],[8,0]]` | 탐욕9 대신 전역16 선택; 실현된 turn 예의 DER36 |
| 독립 3×3 matrix | 최적 합325; 2×2 0~4 값625개 matrix도 대각/교차 두 독립 합의 최대와 대조 |
| DER/JER가 다른 mapping을 선호하는 실제 합성 interval 구성 | DER A→X, JER A→Y를 별도로 선택 |
| invalid RTTM/UEM·time precision/overflow·WAV format·hash/duration/offset·all span bounds | 오류이며 새 report 생성 안 함 |
| unresolved null/overlap만 있는 hypothesis | reference를 제외하지 않고 DER100 |
| CLI subprocess + 반복 output | 합성 CLI 실행 성공, 두 번째 동일 출력 거부 |
| snapshot tamper·unsafe relative path·INCOMPLETE marker | verifier가 거부 |

최종 실행한 검증 명령은 모두 이 **독립 manifest**에 한정했다.

```text
cargo fmt --manifest-path tools/diarization-evaluation/Cargo.toml --check
cargo test --manifest-path tools/diarization-evaluation/Cargo.toml --target-dir runtime/parallel-build/diarization-evaluation -j 2 --locked
cargo clippy --manifest-path tools/diarization-evaluation/Cargo.toml --target-dir runtime/parallel-build/diarization-evaluation -j 2 --locked --all-targets -- -D warnings
cargo build --manifest-path tools/diarization-evaluation/Cargo.toml --target-dir runtime/parallel-build/diarization-evaluation -j 2 --locked
```

fmt/test/clippy/build 모두 exit0이다. root 제품 테스트·기존 model-spike 테스트·사용자 GUI 실행은 수행하지 않았다.

## 동결 build/source/EXE

build.rs가 Cargo.toml/Cargo.lock/build.rs와 src/tests 전체 **21개 입력 파일**의 path/bytes/SHA256을 컴파일 시 `compiled-source.json`에 넣는다. preservation 시 현재 소스 inventory와 각 byte hash가 내장 값과 일치해야 한다. 불일치하면 rebuild를 요구한다. EXE와 source/lock·내장 metadata 사본을 새 content-addressed 디렉터리에만 생성하며 기존 경로는 거부한다. 실패한 보존 작업은 INCOMPLETE marker로 구별한다. 이 CLI를 통한 보존은 create-new 방식이며 OS 수준 WORM 또는 제삼자 서명 attestation은 아니다.

- bundle: [.spec/evidence/ai/diarization-evaluator-builds/5202d56c…](../../../evidence/ai/diarization-evaluator-builds/5202d56c5d91afece9a344a9b6d2572868707ecf46904b425ae5fa73a10aca6b/build-manifest.json)
- frozen EXE: **`5202d56c5d91afece9a344a9b6d2572868707ecf46904b425ae5fa73a10aca6b`**, 1,593,344 bytes
- manifest SHA256: `74a79fdd2d27b08cfce78aadd8ce64d610d79dd9875ad9af446ab258136e3f38`
- Cargo.lock SHA256: `925a07a4eedbf3b2026995263849715b4455c1c434a4cf096949ef55b34a7d07`
- compiled-source.json SHA256: `0d2dbc8c015bd62c3fa2400e1b858c488388513cb2883ed31db36880c4b22fb2`
- manifest listed files: **23개**(source21 + EXE + compiled-source.json). manifest 자신은 self-hash 목록 밖이며 위 별도 SHA로 식별한다.
- toolchain: rustc1.99.0 `b940084d7eb6a299eb4bfeb8e34901bc051e7ac4`, x86_64-pc-windows-msvc, debug profile, LLVM23.1.1.

mutable target EXE로 preserve-build를 실행한 뒤 **저장된 frozen EXE의 `verify-build ABS_BUNDLE`**를 실행해 23개 파일 hash 확인 exit0을 받았다. 원본 코드·소스/lock 대응의 보존 근거이며 Windows linker/SDK/전역 Cargo 설정/registry cache 전체를 동결하거나 bit-reproducible build를 증명한 것은 아니다. dependency artifact identity는 Cargo.lock checksum으로 추적하되 제삼자 source archive 전부를 bundle에 넣지는 않았다.

root는 위 frozen EXE를 실제 local ref/UEM/새 stdout/fixture와 **새 output 경로**로 실행하면 된다. 원본 scorer parity와 실제 human reference DER/JER 결과는 그 후 별도 관측이며 이 준비 완료 문서에 실측 숫자를 미리 넣지 않는다.
