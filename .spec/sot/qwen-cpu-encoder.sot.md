---
id: sot:kinekino/qwen-cpu-encoder
title: 공식 Qwen CPU encoder 참조 실행 준비
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 공식 Qwen CPU encoder 참조 실행 준비

2026-10-18 독립 source/배포 metadata 조사. **실행 가능성은 source 수준에서 확인했으나 Windows import·weight load·신경망 실행은 미검증**이다. 취득·설치·빌드·추론 및 기존 private 환경 변경은 하지 않았다. 이 문서는 root의 다음 실행을 위한 계약이며 완료된 수치 일치 증거가 아니다.

## 현행 관측과 범위

[segmentation 실제 A/B](../../../../verification/ai/forced-segmentation-ab-actual.md)는 누적 호출 17회, 구조 통과 3/실패 14, 품질 통과 0이다. 같은 입력 재호출을 17개의 독립 정확도 표본으로 해석하지 않는다. 원본 nagisa 분절에서도 JA1516 zero 2개, JA1634 zero 3개가 남았다. 다음 우선순위는 encoder/수치 경로다.

[원본 NumPy mel 실제 비교](../../../../verification/ai/forced-mel-numpy-actual.md)는 Transformers 4.57.3/NumPy 2.3.2 원본 함수와 Rust의 전체 529152개 원소에서 최대 절대차 1.1920928955078125e-7을 관측했다. 이는 encoder 일치 증거가 아니다. 다음 비교는 두 엔진에 **같은 하나의 full mel 바이트열**을 제공한다. Rust mel과 NumPy mel을 서로 다르게 넣어 변수를 늘리지 않는다.

| 입력 | mel 형상 | 원본 NumPy F32 bin SHA256 | 예상 encoder 길이 |
|---|---|---|---:|
| ko1883 | 1×128×1248 | a11be61a84c23db6c9e0fe261b14b99142b76760bc21c2d5cd1e9a1bb58b4949 | 162 |
| ja1516 | 1×128×1674 | 5c760eb80f244f9ea99db57387de1fe024435c16f63e4e5eed359b9995fb92d8 | 218 |
| ja1634 | 1×128×1212 | 74510774b41a55ff689ae43a273b2e42629b87dd0aac444cbdf722dc84acca59 | 158 |

bin 위치는 `.spec/artifacts/spikes/2026-10-18/acquisitions/mel-numpy-functions-r1/{입력}-official-numpy-f32.bin`이다. 길이는 원본 `_get_feat_extract_output_lengths`의 100프레임당 13개 및 나머지 3회 stride-2 계산에서 도출한 **예측**이며 실행 관측이 아니다. 출력 차원은 실제 pinned audio_config에서 읽는다.

## 원본과 모델 고정

- [Qwen 원본 저장소](https://github.com/QwenLM/Qwen3-ASR/tree/7c6daf77a2421100f5fb066495372c00129d39ff): commit `7c6daf77a2421100f5fb066495372c00129d39ff`, Apache-2.0.
- [모델](https://huggingface.co/Qwen/Qwen3-ForcedAligner-0.6B/tree/c7cbfc2048c462b0d63a45797104fc9db3ad62b7): revision `c7cbfc2048c462b0d63a45797104fc9db3ad62b7`; weight 1835544544 B, SHA256 `47831d0e82f96b20e9034dba01a075ee06436654719f6a68289e49f1b65ce0e7`.
- [원본 modeling](https://github.com/QwenLM/Qwen3-ASR/blob/7c6daf77a2421100f5fb066495372c00129d39ff/qwen_asr/core/transformers_backend/modeling_qwen3_asr.py): 57850 B, Git blob `75bef860ea8ccf314d67ab9d5821625034f26398`.
- configuration 20164 B/blob `17d7263a6a818404b3c319d7a64e3fb916fae7b4`; processing 8545 B/blob `b63f0c0f64636695981d39f7b4e184c620c599cf`; pyproject 1304 B/blob `97a4ae1fd8ad475e78a1c1bd89460f0aabf02338`。Git blob SHA1과 배포 파일 SHA256은 다른 식별자다.

## 원본 import closure

`qwen_asr.core.transformers_backend.modeling_qwen3_asr`만 import해도 원본 `qwen_asr/__init__.py`가 ASR·forced aligner·utils를 먼저 import한다. backend는 config/model/processor를 모두 import한다. 따라서 torch/transformers 외에도 **nagisa, soynlp, librosa, soundfile 및 그 전이 의존성**이 필요하다. nagisa import는 Tagger/model 초기화를 포함하므로 encoder만 호출해도 이 비용을 계측해야 한다. vllm은 try/except 선택 경로이며 CPU 참조에 필요하지 않다.

[원본 pyproject](https://github.com/QwenLM/Qwen3-ASR/blob/7c6daf77a2421100f5fb066495372c00129d39ff/pyproject.toml)는 transformers==4.57.6, nagisa==0.2.11, soynlp==0.0.493, accelerate==1.12.0을 고정하고 qwen-omni-utils/librosa/soundfile/sox/gradio/flask/pytz도 선언한다. 아래는 원본 모듈 import와 CPU encoder를 위한 profile이다. 사용하지 않는 CLI/web 의존성을 생략한 profile을 qwen-asr 전체 distribution 의존성 충족으로 표현하지 않는다. 런타임에서 추가 import가 드러나면 실패를 기록하고 closure를 갱신한다. stub으로 통과시키지 않는다.

전체 원본 package tree를 보존한다. 원본에 없는 `core/__init__.py`, `inference/__init__.py`를 임의 생성하지 않는다. Rust PyO3/C-API에서 원본 module을 import하고 실제 Python 객체 메서드를 호출한다. Python wrapper/source 추출/eval/가짜 parent module은 허용하지 않는다.

기존 `runtime/ai-spike/source/mel-numpy-runtime-r2`와 `segmentation-runtime-private-r1`는 읽기/검증 후 **새 private 환경에 복사할 후보**다. CPython 3.13.3/NumPy 2.3.2와 기존 hub/tokenizers/safetensors/HTTP closure는 재사용 가능하다. 그러나 Transformers 4.57.3과 nagisa 0.3.0을 공식 pin으로 대체하는 작업은 새 환경에서 수행한다. 기존 환경을 덮어쓰지 않는다. SciPy 1.16.0, scikit-learn 1.9.1, joblib 1.6.0, threadpoolctl 3.7.0, narwhals 2.27.1 및 DyNet38 2.2/six/soynlp 기존 manifest를 함께 보존한다. DyNet 실제 표시 버전 2.1과 artifact 2.2를 혼동하지 않는다. soynlp 원본 라이선스는 GPLv3다.

## Windows CP313 wheel 취득 후보

아래는 2026-10-18 공개 metadata 확인값이다. **파일을 내려받아 검증한 값이 아니다.** root 취득 시 길이와 SHA256을 모두 검사하고 라이선스 원문·native 포함물까지 manifest에 남긴다. PyPI 항목의 근거는 `https://pypi.org/pypi/{package}/{version}/json`이며 wheel filename을 정확히 선택한다.

| wheel | B | SHA256 |
|---|---:|---|
| torch-2.9.1+cpu-cp313-cp313-win_amd64.whl | 110859971 | 404a7ab2fffaf2ca069e662f331eb46313692b2f1630df2720094284f390ccef |
| transformers-4.57.6-py3-none-any.whl | 11993498 | 4c9e9de11333ddfe5114bc872c9f370509198acf0b87a832a0ab9458e2bd0550 |
| nagisa-0.2.11-cp313-cp313-win_amd64.whl | 21352513 | 4ae21df165b31145353c011d78b20658fa8d0e5571ac77da5fbc005c58e0318f |
| accelerate-1.12.0-py3-none-any.whl | 380935 | 3e2091cd341423207e2f084a6654b1efcd250dc326f2a37d6dde446e07cabb11 |
| librosa-0.11.0-py3-none-any.whl | 260749 | 0b6415c4fd68bff4c29288abe67c6d80b587e0e1e2cfb0aad23e4559504a7fa1 |
| numba-0.63.1-cp313-cp313-win_amd64.whl | 2750228 | eb227b07c2ac37b09432a9bda5142047a2d1055646e089d4a240a2643e508102 |
| llvmlite-0.46.0-cp313-cp313-win_amd64.whl | 38138941 | 67438fd30e12349ebb054d86a5a1a57fd5e87d264d2451bcfafbbbaa25b82a35 |
| sympy-1.13.3-py3-none-any.whl | 6189483 | 54612cf55a62755ee71824ce692986f23c88ffa77207b30c1368eda4a7060f73 |
| mpmath-1.3.0-py3-none-any.whl | 536198 | a0b2b9fe80bbcd81a6647ff13108738cfb482d481d826cc0e02f5b35e5c88d2c |
| networkx-3.4.2-py3-none-any.whl | 1723263 | df5d4365b724cf81b8c6a7312509d0c22386097011ad1abe274afd5e9d3bbc5f |
| jinja2-3.1.6-py3-none-any.whl | 134899 | 85ece4451f492d0c13c5dd7c13a64681a86afae63a5f347908daf103ce6d2f67 |
| markupsafe-3.0.3-cp313-cp313-win_amd64.whl | 15113 | 9a1abfdc021a164803f4d485104931fb8f8c1efd55bc6b748d2f5774e78b62c5 |
| setuptools-80.9.0-py3-none-any.whl | 1201486 | 062d34222ad13e0cc312a4c02d73f059e86a4acbfbdea8f8f76b28c99f306922 |
| soundfile-0.13.1-py2.py3-none-win_amd64.whl | 1019162 | 1e70a05a0626524a69e9f0f4dd2ec174b4e9567f4d8b6c11d38b5c289be36ee9 |
| cffi-2.0.0-cp313-cp313-win_amd64.whl | 183402 | 19f705ada2530c1167abacb171925dd886168931e0a7b78f5bffcae5c6b5be75 |
| pycparser-2.23-py3-none-any.whl | 118140 | e5c6e8d3fbad53479cab09ac03729e0a9faf2bee3db8208a550daf5af81a5934 |
| audioread-3.1.0-py3-none-any.whl | 23143 | b30d1df6c5d3de5dcef0fb0e256f6ea17bdcf5f979408df0297d8a408e2971b4 |
| decorator-5.2.1-py3-none-any.whl | 9190 | d316bb415a2d9e2d2b3abcc4084c6502fc09240e292cd76a76afc106a1c8e04a |
| pooch-1.8.2-py3-none-any.whl | 64574 | 3529a57096f7198778a5ceefd5ac3ef0e4d06a6ddaf9fc2d609b806f25302c47 |
| platformdirs-4.5.0-py3-none-any.whl | 18651 | e578a81bb873cbb89a41fcc904c7ef523cc18284b7e3b3ccf06aca1403b7ebd3 |
| soxr-1.0.0-cp312-abi3-win_amd64.whl | 172070 | 2a3b77b115ae7c478eecdbd060ed4f61beda542dfb70639177ac263aceda42a2 |
| lazy_loader-0.4-py3-none-any.whl | 12097 | 342aa8e14d543a154047afb4ba8ef17f5563baad3fc610d7b15b213b0f119efc |
| msgpack-1.1.2-cp313-cp313-win_amd64.whl | 72631 | a465f0dceb8e13a487e54c07d04ae3ba131c7c5b95e2612596eafde1dccf64a9 |
| standard_aifc-3.13.0-py3-none-any.whl | 10492 | f7ae09cc57de1224a0dd8e3eb8f73830be7c3d0bc485de4c1f82b4a7f645ac66 |
| standard_sunau-3.13.0-py3-none-any.whl | 7364 | 53af624a9529c41062f4c2fd33837f297f3baa196b0cfceffea6555654602622 |
| standard_chunk-3.13.0-py3-none-any.whl | 4944 | 17880a26c285189c644bd5bd8f8ed2bdb795d216e3293e6dbe55bbd848e2982c |
| audioop_lts-0.2.2-cp313-abi3-win_amd64.whl | 30503 | a2c2a947fae7d1062ef08c4e369e0ba2086049a5e598fda41122535557012e9e |

Torch 원본은 [공식 CPU index](https://download.pytorch.org/whl/cpu/torch/)의 [고정 wheel](https://download-r2.pytorch.org/whl/cpu/torch-2.9.1%2Bcpu-cp313-cp313-win_amd64.whl)이다. PEP658 metadata SHA256은 `9c1d1d656259704ee03acfbe2684b6dfca07776d1cc3dba4c0de0802dd1d7287`, 버전은 2.9.1+cpu다. PyPI의 +cpu 없는 2.9.1 wheel(110944162 B/SHA `545844cc16b3f91e08ce3b40e9c2d77012dd33a48d505aed34b7740ed627a1b2`)과 섞지 않는다. 최신 버전 비교가 아니라 고정 CPU 참조 후보 선정이다.

Torch 의존성 filelock/typing-extensions/fsspec는 기존 pin을 재사용하고 sympy/mpmath/networkx/jinja2/MarkupSafe/setuptools를 추가한다. librosa closure에는 표의 numba/llvmlite/audioread/decorator/pooch/platformdirs/soxr/lazy_loader/msgpack 및 기존 SciPy/sklearn/joblib가 필요하다. CPython 3.13에서 제거된 aifc/sunau/audioop의 대체 wheel도 포함한다. numba 0.63.1의 NumPy 범위는 >=1.22,<2.4이므로 기존 2.3.2가 들어간다. soxr cp312-abi3는 일반 CP313 후보이며 cp313t wheel로 바꾸지 않는다. soundfile은 libsndfile을 포함하는 Windows wheel을 선택한다.

라이선스 metadata: Torch BSD-3-Clause, Transformers/Accelerate/msgpack Apache-2.0, nagisa/audioread/cffi/setuptools/platformdirs MIT, librosa ISC, numba/sympy/mpmath BSD 계열, llvmlite BSD-2-Clause AND Apache-2.0 WITH LLVM-exception, soxr LGPL-2.1-or-later, decorator BSD-2-Clause, soundfile/MarkupSafe/pycparser/pooch/lazy_loader BSD-3-Clause, standard_* PSF-2.0. networkx/jinja2의 조회한 요약 license 필드는 비어 있어 wheel 원문 확인을 남긴다. audioop-lts 라이선스와 soundfile 번들 libsndfile 조건도 취득 단계 확인 사항이다. 의존성의 라이선스를 Qwen Apache 한 개로 덮지 않는다.

## Rust에서 호출할 실제 API

1. 기존 isolated PyConfig 정책을 유지한다. environment/user-site/site 자동 import/bytecode/signal handler를 끄고 private python zip·stdlib·site-packages·원본 source 경로만 명시한다. CPython DLL SHA `69fd86ea29370697c203f7e12830084f920f490766a8e3045af52c036a9ad529`를 대조한다.
2. `torch.set_default_dtype(torch.float32)`, `set_num_threads(4)`, `set_num_interop_threads(1)`을 실제 작업 전에 한 번 설정한다. get 값, torch build/CPU backend/MKLDNN/autocast 상태를 기록한다. 공식 Windows torch import는 DLL 디렉터리 및 VC runtime에 의존하므로 wheel 존재만으로 import 성공을 선언하지 않는다.
3. 원본 `Qwen3ASRConfig.from_json_file(절대 config 경로)`에서 `thinker_config.audio_config`를 얻고 원본 `Qwen3ASRAudioEncoder(audio_config)`를 생성한다. `encoder.set_attn_implementation("eager")`는 Transformers 4.57.6의 공개 setter다. 실제 effective `_attn_implementation`을 기록한다. CPU `.float().eval()`과 `torch.no_grad()` 또는 `inference_mode` context를 각각 적용한다. eval은 gradient 비활성화를 대신하지 않는다.
4. `safetensors.safe_open(절대 weight 경로, framework="pt", device="cpu")`의 `keys()/get_tensor()`로 필요한 tensor만 읽는다. source 객체 구조에 대응하는 `thinker.audio_tower.` prefix를 Rust에서 제거하여 `encoder.load_state_dict(dict, strict=True)`한다. **실제 weight header의 prefix/누락/초과 key 확인은 실행 전 게이트**다. BF16 원본을 F32 parameter로 로드한 뒤 dtype/device/shape/유한값을 기록한다. 누락 weight를 random 초기값으로 실행하지 않는다.
5. 단일 입력은 `encoder(input_features, feature_lens=...)`: input_features **2D [128,T] F32**, feature_lens **[1] int64 CPU**다. 원본 forward는 aftercnn_lens를 다시 계산한다. 반환은 `BaseModelOutput.last_hidden_state` 2D다. encoder에 없는 `output_hidden_states` 인자를 만들지 않는다.

상위 대안은 전체 모델의 `from_pretrained(local_dir, local_files_only=True, torch_dtype=torch.float32, attn_implementation="eager", device_map=None)`다. text/head까지 올리는 비용이 크다. 상위 batch API는 **`model.thinker.get_audio_features`**이며 input [1,128,T]와 mask 또는 audio_feature_lengths를 받는다. top-level model의 동명 메서드나 standalone encoder의 전체 checkpoint 자동 prefix 처리를 가정하지 않는다.

## Attention과 중간 관측 계약

원본에는 `_prepare_attention_mask` 정의가 있지만 encoder forward는 `encoder_layer(hidden_states, cu_seqlens)`로 호출하여 attention_mask를 전달하지 않는다. **eager 경로는 전역 softmax**이고 cu_seqlens는 flash varlen 경로와 구분된다. 조사한 Rust CPU audio encoder도 flash 외 경로에서 별도 window mask 없이 softmax한다. 따라서 현 근거로 mask 불일치를 원인으로 확정할 수 없으며 참조 실행에 임의 block mask를 추가해서도 안 된다. 현 source 판독은 최신 frozen executable의 실제 tensor attestation을 대신하지 않는다.

공개 관측 지점은 `conv2d1/2/3`, `conv_out`, `positional_embedding.positional_embedding`, `layers`, `ln_post`, `proj1`, `act`, `proj2`다. PyTorch 공개 `register_forward_hook/register_forward_pre_hook`에 **Rust PyO3 native callable**을 연결하면 Python 소스 추가 없이 원본 호출을 관측할 수 있다. Module의 `__call__`을 호출하며 hook은 tensor를 교체하지 않고 None을 반환한다. 각 encoder layer 반환은 tensor 자체가 아니라 `(hidden_states,)`다. pre-hook으로 layer의 실제 cu_seqlens도 보존한다. 이 hook 어댑터와 Rust 대응 stage dump는 아직 구현/실행하지 않았다.

원본 positional sinusoid는 Torch F32 연산, Rust 구현에는 f64 계산 후 cast 가능성이 있어 ULP 차이 후보로 남는다. conv/GELU/LayerNorm/GEMM도 후보이며 zero timestamp의 원인이라는 증거는 없다.

## root의 실행 순서와 중단 게이트

1. 새 환경 manifest·source/weight/input SHA·실제 import origin/version·DLL origin을 먼저 고정한다. Torch CPU 2.9.1+cpu, Transformers 4.57.6, nagisa 0.2.11과 기존 NumPy 2.3.2 조합을 확인한다. host package나 네트워크 fallback이 발견되면 중단한다.
2. weight strict load/모든 parameter CPU F32/eager/eval/no_grad/threads 4 확인 후 ko1883 한 개로 shape와 유한값을 확인한다. import 시간·native/model 초기화·load 시간·실행 peak memory를 분리한다.
3. 세 입력의 최종 encoder 전체 F32 tensor, 형상, SHA, 최대 절대차·상대 RMS·cosine·비유한 수를 기록한다. 중간 지점도 같은 실제 호출에서 비교하여 **첫 차이 지점**을 찾는다. 현재 없는 Rust stage dump를 이미 관측한 것으로 쓰지 않는다.
4. 진단용 수치 게이트 제안은 각 원소 `abs(actual-ref) <= 1e-4 + 1e-4*abs(ref)`다. root가 실행 전에 고정할 제안이며 아직 통과 판정은 없다. mel의 기존 1e-5 게이트를 neural 전체에 자동 전용하지 않는다. 실패 후 허용오차를 넓혀 일치를 선언하지 않는다. tensor 일치와 최종 word/time 품질은 독립 판정이다.
5. F32/eager 기준을 확보한 뒤에만 SDPA 또는 BF16을 **한 변수씩** 바꾼다. CPU 참조에 flash-attn을 설치하지 않는다. encoder가 다르면 전체 alignment 반복보다 첫 stage 차이를 먼저 해결한다.

현 `reference_packages.rs`의 wheel 64 MiB 제한은 Torch CPU 110859971 B를 거부한다. 기존 archive 단일 file 128 MiB/전체 768 MiB 제한도 unpacked Torch DLL이 만족하는지는 미확인이다. root는 전용 typed acquisition/extraction profile에 정확한 bytes/SHA·경로·확장 크기 제한을 준비해야 한다. 전역 제한 완화나 기존 명령으로 즉시 실행 가능하다는 주장은 하지 않는다.

## 검증 상태

원본 API·metadata 조사와 실행 순서 정리만 완료했다. CP313 wheel 존재는 확인했지만 실제 DLL load/import/strict weight load/encoder 결과/수치 원인은 미검증이다. 변경은 이 신규 연구 문서와 research coordination 인계뿐이며 개발 코드 변경이 없어 cargo fmt/test/clippy는 실행하지 않았다.
