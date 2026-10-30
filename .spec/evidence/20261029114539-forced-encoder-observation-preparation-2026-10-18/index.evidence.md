---
id: evidence:kinekino/20261029114539
title: 고정 NumPy mel 입력의 Rust encoder stage 관측 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 고정 NumPy mel 입력의 Rust encoder stage 관측 준비

2026-10-18. [공식 encoder 참조 준비](../../player/infrastructure/inference/research-followups/forced-encoder-pytorch-readiness.md)와 [실제 분절 A/B](../ai/forced-segmentation-ab-actual.md)를 읽고 별도 Rust 관측 어댑터를 구현했다. **컴파일·비추론 검증·선보존만 완료했다. 모델 load/encoder forward/새 신경망 실행은0회이며 실제 stage 값과 Torch 수치 일치는 미검증이다.** 기존 전체 품질 목표를 축소하거나 timestamp0 문제를 해결한 것으로 표시하지 않는다.

## 소유 범위와 현재 API

새 adapter는 `tools/forced-encoder-observation`, 새 engine fork는 `runtime/ai-spike/source/lumos-encoder-observation-7984967`다. 원본 b069 segmentation source/tool/EXE·frozen evidence·root tools/model-spike·root 문서는 변경하지 않았다. 제품 SDK/CLI 능력은 계속 unsupported이며 이 실행기는 진단 도구다.

기존 public `AudioTower::load`와 `forward/forward_with_lens`는 최종 encoder Tensor만 제공한다. intermediate stage는 private forward_one 안에 있어 기존 public API만으로 모두 dump할 수 없었다. 새 fork에 passive EncoderObserver와 CPU F32 batch1/eager용 `forward_observed`를 추가하고 기존 body를 공유했다. 일반 forward는 같은 internal body에 observer=None으로 들어간다. conv의 중첩 `gelu(conv.forward)` 표현은 같은 순서의 두 let binding으로만 분리하여 preactivation도 읽는다. callback은 tensor를 교체하지 않는다.

encoder 로드는 기존 Candle VarBuilder/AudioTower constructor와 `thinker.audio_tower` prefix를 그대로 사용한다. full ASR/forced-aligner·text/head/tokenizer·오디오 전처리를 실행하지 않고 고정 full mel tensor를 직접 넣는다. 기존 sinusoid f64 계산 후 F32 cast, conv/GELU/LayerNorm/GEMM, chunking, eager attention을 바꾸지 않았다. eager는 실제 cu_seqlens가 `[0,packed_length]`이고 attention에서 이를 mask로 사용하지 않는 기존 구현이다. 공식 eager도 mask를 사용하지 않는다는 root 조사와 flash varlen을 구분하며, 이를 이미 확정된 불일치로 주장하지 않는다.

## 고정 입력과 모델

CLI는 case ID3개와 새 출력 경로만 받는다. 임의 model/input 경로·HF ID·URL·대체 tokenizer/mel·shard/다운로드 fallback은 없다. 위치는 compiled workspace의 기존 고정 경로다. relocation을 위한 자동 탐색도 추가하지 않았다.

| case | 입력 F32 shape | bytes | 전체 mel SHA256 | 예상 최종 shape |
|---|---|---:|---|---|
| ko1883 | 1×128×1248 | 638976 | `a11be61a84c23db6c9e0fe261b14b99142b76760bc21c2d5cd1e9a1bb58b4949` | 162×1024 |
| ja1516 | 1×128×1674 | 857088 | `5c760eb80f244f9ea99db57387de1fe024435c16f63e4e5eed359b9995fb92d8` | 218×1024 |
| ja1634 | 1×128×1212 | 620544 | `74510774b41a55ff689ae43a273b2e42629b87dd0aac444cbdf722dc84acca59` | 158×1024 |

원본은 `.spec/artifacts/spikes/2026-10-18/acquisitions/mel-numpy-functions-r1/<case>-official-numpy-f32.bin`의 **전체 bytes**다. hash/정확한 길이/finite 입력을 검사하고 little-endian으로 decode한다. engine 입장 직전에도 pinned case/hash/config와 decoded f32 bit가 준비된 values와 같은지 확인한다. 실제 input Tensor를 dump한 SHA도 원본과 같아야 한다. 예상 최종 shape는 config/기존 길이 함수에 근거한 실행 전 계약이며 이번에 관측한 출력 shape가 아니다.

모델 revision은 `c7cbfc2048c462b0d63a45797104fc9db3ad62b7`, local fixed config SHA는 `d616c65d46c4b90bdc651b0a0963ea932732241140f337f9bb6b0335a9c8ef09`다. config의 d_model/output_dim1024·encoder_layers24·n_window50·conv_chunksize500을 기존 타입으로 읽는다. root의 encode 실행 시에만 single safetensors1835544544B와 SHA `47831d0e82f96b20e9034dba01a075ee06436654719f6a68289e49f1b65ce0e7`를 확인하고 read guard를 유지한 채 CPU/F32로 encoder를 로드한다. 모델/loader의 F32 요청 및 dump dtype 검사는 실제 로드/수치 성공을 미리 증명하지 않는다. Rayon4 요청은 CPU 사용률이나 hard resource cap을 뜻하지 않는다.

## stage의 실제 의미

모든 tensor record는 shape/dtype=F32/byte_order=little_endian/layout=logical_row_major/파일명/bytes/전체 SHA/비유한 원소 수를 담는다. 파일은 `.f32le.bin`, sidecar는 같은 이름에 `.json`을 붙인다. float를 JSON 숫자로 줄여 저장하거나 일부 row만 뽑지 않는다. dump는 dtype를 변환하지 않고 실제 CPU F32가 아니면 거부한다. 비유한 출력과 -0/NaN bit는 제거·clamp하지 않는다.

| stage | source 대응과 shape 의미 |
|---|---|
| input_mel | 실제 encoder에 넣은 `[1,128,T]` 전체 tensor |
| conv_input | 기존 chunk split/zero-pad/transpose/unsqueeze 이후 NCHW `[chunks,1,128,100]` |
| conv1/conv2/conv3 | 각각 기존 conv2d1/2/3 모듈 **GELU 전** 출력. instance는 conv group의 시작 chunk index |
| conv1_act/conv2_act/conv3_act | 같은 실제 convolution 뒤 기존 GELU 출력. 추가 Rust 관측이며 Torch 쪽 functional GELU 대응을 자동 추정하지 않음 |
| conv_out | conv3 뒤 기존 transpose/flatten을 거친 linear_no_bias 출력 `[chunks,13,1024]` 예상 |
| positional_embedding | 실제 사용한 narrowed/cast/unsqueeze된 PE `[1,13,1024]` 예상.1500-entry 전체 table dump가 아님 |
| positional_add | broadcast_add 직후의 padded chunk tensor. 마지막 chunk의 padding도 포함 |
| ragged_gather | 원래 index_select가 valid positions만 골라 packed한 `[N,1024]` |
| layer instance0..23 | 각 기존 AudioEncoderLayer의 완료 출력. residual/FFN까지 포함하며 중간 qkv/attention logits/FFN 출력이 아님 |
| ln_post | 기존 최종 layer norm 모듈 출력 |
| proj1 | 기존 첫 projection의 preactivation 출력 |
| act | proj1 뒤 기존 GELU 출력 |
| proj2 | 기존 마지막 projection 출력 |
| encoder_final | 원래 batch cat 이후 최종 encoder 반환 tensor |

chunk_lengths/aftercnn_lengths/cu_seqlens도 실제 local Vec를 callback에 전달하고 별도 JSON sidecar로 저장한다. 같은 cu Vec가 각 layer로 전달되며24개의 독립 재계산을 만들지 않는다. 현재 pinned config·3개 입력은 conv group1개와24layers이므로41 tensor record가 예상되지만 실제 횟수·shape·bytes는 root 실행이 남아 있다. sidecar를 즉시 게시하므로 뒤 stage가 실패해도 완료된 앞 stage의 shape/hash를 잃지 않는다.

요청한 encoder stage는 source상 모두 연결했지만 **실제 모델 로드/모든 kernel 지원·실제 값은 아직 검증하지 않았다**. 내부 qkv/softmax/FFN 각 substage, 전체 positional table, attention mask tensor, decoder/head/logits/word timestamps는 이 도구의 관측 범위 밖이다. Torch와 stage별 transpose/packed/padded shape를 구분해야 하며 서로 다른 layout을 그대로 zip해서 비교하면 안 된다.

## 경계와 실패 처리

기본 내부 수치/repair/시간 보정은 없다. 새 출력 directory만 만들며 기존 파일을 덮어쓰지 않는다. stage당256MiB·전체 tensor1GiB·128records, sequence8개/1024값 상한을 둔다. 단순 ASCII stage label과 generated ordinal 파일명을 사용한다. full tensor materialize/bytes/hash/쓰기와 conv 중간 binding의 수명은 메모리·시간 overhead를 추가하므로 모델 성능 측정으로 해석하지 않는다. hard memory/time cap·kill/reap는 root의 외부 supervision 책임이다.

정상 완료의 요약은 output directory 안 `encoder-result.json`이다. 모델 load/kernel/shape/쓰기 오류라면 nonzero exit이고 부분 tensor/sidecar만 남을 수 있다. partial 파일 존재를 성공이나 품질 통과로 바꾸지 않는다. 예상 shape와 다른 경우도 실패이며 dump를 원래 shape에 맞추어 자르거나0으로 채우지 않는다.

## 검증과 선보존

비추론5tests가 통과했다: 원본 mel3개 및 고정 config bytes/hash, 모델 입장 전 values/config 변조 거부, synthetic Tensor의 full LE bits/strided logical layout/NaN/-0/shape/SHA 및 tensor·sequence sidecar/중복·traversal·overwrite 거부, 실제 check-input CLI/임의 model URL flag 거부, passive taps를 제거하고 conv let split을 원래 표현으로 환원한 수치 소스 동일성. attention/sinusoid/GELU/layer/load의 원본 prefix도 동일하게 대조했다. AudioTower::load/forward나 copied upstream의 neural/zero-weight encoder tests는 실행하지 않았다.

```text
cargo fmt --manifest-path tools/forced-encoder-observation/Cargo.toml --check
cargo test --locked --offline --release --manifest-path tools/forced-encoder-observation/Cargo.toml --target-dir runtime/parallel-build/lumos-observation -j 2
cargo clippy --locked --offline --release --manifest-path tools/forced-encoder-observation/Cargo.toml --target-dir runtime/parallel-build/lumos-observation -j 2 --all-targets -- -D warnings
cargo clippy --locked --offline --release --manifest-path tools/forced-encoder-observation/Cargo.toml --target-dir runtime/parallel-build/lumos-observation -j 2 -p qwen3_asr_encoder_observation --lib -- -D warnings
cargo build --locked --offline --release --manifest-path tools/forced-encoder-observation/Cargo.toml --target-dir runtime/parallel-build/lumos-observation -j 2 --bins
```

모두 통과했다. 수정 fork3개 파일 rustfmt 및 standalone Rust bootstrap clippy-driver -Dwarnings도 통과했다. 초기 새 chunks_exact lint1개는 as_chunks로 수정했으며 최종 strict clippy에 남은 Rust 경고는 없다. 기존 native MSVC LNK4098 LIBCMT 경고는 억제하지 않고 구별했다. 처음 새 lock은 기존 segmentation lock에서 출발하여 offline으로 해당 새 package만 추가했고 이후 locked/offline을 유지했다. 기존 release cache를 재사용했지만 feature closure 차이로 dependency 재컴파일이 있었으며 다운로드나 다른 세션 target 종료/잠금은 하지 않았다.

[최종 보존 manifest](../../../evidence/player/forced-encoder-observation-builds/r2-final/build-manifest.json)는 compiler input109개를 현재 파일과 hash/bytes 대조 후 복사했다. 원본 segmentation/new encoder src·Cargo·tool/build/test/bootstrap/README, full mel3개와 config를 보존했으며 모델 weights는 복사·로드하지 않았다. 전체 copy tree의 양쪽 hash, package/audio_encoder.rs/new encoder_observation.rs/model/mod.rs의4개 정확 diff patch가 있다. 새 파일 patch의 원본 부재도 명시했다. bootstrap의 symlink/Git/target 제외 및 기존 목적지 거부로 복사를 재현하고 saved compiler-input files로 정확한 새 소스를 복원할 수 있다. parent Git index를 수정하지 않았다.

| identity | SHA256 |
|---|---|
| 최종 saved encoder EXE | `836dcef4e0a81f08c7fb53c74d328ae19f6d412de48a565f4fe61c72b7f0f4ef` |
| build-manifest | `7474d7b6be5de75568722e1461e65478d60729b7e47c4268471e2d232d1a1ce1` |
| embedded compiled-source | `43525224075b531556fea7182c68770c62317978b9d9dc29518e892f7536e92e` |
| Cargo.lock | `9a18a1cec3f0921895fd746ad7859c02b7248db1989cc5a792a297a0a90f6c68` |

saved EXE의 `--build-info`와 보존 helper의 embedded compiler hash도 일치했다. r1-dfc909f3은 초기 후보로 보존만 하며 최종 실행에는 사용하지 않는다. r2는 실패 시에도 completed tensor/sequence metadata가 남도록 sidecar를 추가한 최종판이다. 최종 saved EXE의 실제 `check-input`3회는 `runtime/ai-spike/acquisitions/sdk-encoder-input-<case>-r2.json`에 남겼으며 weights_loaded/model_weights_verified/inference_executed가 모두false다. 이 파일과 source 컴파일 검증은 model load 성공 근거가 아니다.

## root 전용 실행

root가 새 worker 한 개씩 기존 resource supervision 안에서 실행한다. 아래 encode 명령은 이 세션에서 실행하지 않았다.

```powershell
$encoderExe = 'D:/workspace/@myso-kr/com.kinekino.player/.spec/evidence/player/forced-encoder-observation-builds/r2-final/forced-encoder-observation.exe'
& $encoderExe encode --case ko1883 --output 'D:/workspace/@myso-kr/com.kinekino.player/runtime/ai-spike/results/encoder-rust-ko1883-r1'
& $encoderExe encode --case ja1516 --output 'D:/workspace/@myso-kr/com.kinekino.player/runtime/ai-spike/results/encoder-rust-ja1516-r1'
& $encoderExe encode --case ja1634 --output 'D:/workspace/@myso-kr/com.kinekino.player/runtime/ai-spike/results/encoder-rust-ja1634-r1'
```

세 명령은 서로 다른 NEW directory여야 한다. root의 실제 model load/encoder 실행, stage별 Torch tensor 대응/수치 게이트 판정, 첫 차이 지점 분석, word/time 품질 검토는 남아 있다. 새 다운로드·모델 load·신경망 추론·GUI·VM·DPI·endurance·사용자 ChatGPT/Terminal 종료는 수행하지 않았고 제품 SDK 능력도 승격하지 않았다.
