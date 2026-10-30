---
id: evidence:kinekino/20261029113719
title: AI 스파이크 실행 증거 보존 감사
status: active
created: 2026-10-29
updated: 2026-10-29
---
# AI 스파이크 실행 증거 보존 감사

2026-10-18 독립 읽기 전용 감사. 관측 창의 기준 시각은 **13:43:33 KST(04:43:33 UTC)**이며 이어지는 source/EXE 확인은 같은 감사 중 수행했다. root가 보존 기능을 개발 중이므로 디렉토리 전체를 한 시점에 원자적으로 동결한 감사는 아니다. 측정 결과·source·manifest·EXE를 읽고 기본 파일 SHA256 도구로 대조했으며 다운로드·빌드·추론·재실행·기존 증거 변경은 하지 않았다. 기록 쓰기는 이 문서와 `runtime/coordination/provenance-audit.md`뿐이다.

## 핵심 판정

**실행 성공/품질 수치와 재현 증거의 완전성은 별개다.** 기존 수치를 뒤집지 않는다. 확인한 `runtime/ai-spike/results/*.json` 중 top-level `executable_sha256`을 가진 측정 label은79개, 서로 다른 EXE hash는25개였다. 동일 label+EXE hash 쌍79개가 `.spec/evidence/ai/model-spikes-2026-10-18/results`에도 있으며 차이를 찾지 못했다. 이것은 측정 identity 대조이며 모든 stdout/stderr/결과 파일의 전체 byte 무결성을 재감사한 것은 아니다.

일반 model-tool27개 측정/16개 과거 EXE hash에는 해당 binary와 당시 source를 label별로 묶은 보존 증거를 찾지 못했다. 최신 `tool-source`는 과거 모든 source의 정본이 아니다. Whisper 이전18개 측정/2개 hash와 최초 실패 Qwen1개 hash도 확인한 보존 경로에서 EXE를 찾지 못했다. frozen Qwen/aligner는 더 강한 증거가 있으나 Qwen0.1.0 source는 명시적 재구성본이고 aligner0.1.0은 selected fingerprints만 있다.

범위 한계: 시스템 전체·모든 백업/삭제 파일을 탐색하지 않았다. 아래 missing은 조사 경로에서 미확인이라는 뜻이며 영구 복구 불가능을 입증한 것이 아니다. 재구성 source로 원래 binary가 재생성된다고 주장하지 않는다. 보존 binary가 있어도 같은 환경/모델/사전/프로세스 조건에서 동일 출력이 보장된다고 하지 않는다.

## 감사 기준과 immutable/최신 구분

| 상태 | 의미 |
|---|---|
| `frozen-present` | 버전 경로에 EXE/source/lock/archive 또는 명시 manifest가 있고 측정 hash와 저장 EXE 실제 hash가 맞음. bit-reproducible build 증명은 아님 |
| `limited-reconstructed-source` | 측정 EXE는 보존됐지만 당시 source는 재구성으로 명시돼 과거 byte identity를 증명 못함 |
| `limited-live-binary` | 측정 hash에 맞는 현재 runtime binary는 있으나 역사별 frozen source/EXE 패키지가 없음 |
| `missing-historical-binary/source` | 측정 hash·결과는 있으나 확인한 경로에서 해당 EXE/당시 source 연결을 찾지 못함. 최신 source가 대신하지 못함 |
| `prospective/incomplete` | 앞으로의 보존 기능/작성 중 directory. 과거 측정의 missing을 소급 해소하지 않음 |

기준 [snapshot-manifest.json](../../../evidence/ai/model-spikes-2026-10-18/snapshot-manifest.json)은 조회 시210,807 bytes, SHA256 **`98f7f409fa7e073f051c388349c5cda0fe6356d674b29acc35e4c4f32aad7885`**, LastWriteTimeUtc04:39:44.0371997이었다. 이후 root report가 이 경로를 갱신할 수 있으므로 현재 파일이 달라져도 이 digest는 감사 당시 기준이다. manifest에는 tools path/hash/bytes를 기록하는 `tool_provenance`와 사본 source/target/hash를 기록하는 `snapshots`가 구분되어 있다.

확인한 [report.rs의 증거 사본](../../../evidence/ai/model-spikes-2026-10-18/tool-source/src/application/report.rs)은 SHA256 `9fda6f91fb4a70aaa29bcf808ea9e6ba94bda43c8280fd58d2c15a3beb759973`이었다. 감사 중 해당 workspace report.rs도 같은 hash였다. `snapshot`은 같은 target으로 copy하고 `source_snapshot`은 현재 .rs들을 `tool-source/src`에 쓴다. tools 목록은 path/hash를 JSON에 **기록할 뿐 EXE를 evidence로 복사하지 않는 경로**다. 이 소스로 '모든 이전 tool EXE/source가 별도 보존됨'을 주장할 수 없다. 최신 snapshot lock은 `69eb18586d35f1e4bc788c1fb979db9eb61f65989548bc9e7533d49c75d2d45f`였다.

측정 JSON 자체에는 args와 executable_sha256이 있지만 조사한 schema에는 별도 executable path 필드가 없었다. 아래 도구군은 args·현재 runner source·별도 보존 manifest로 연결했다. source가 mutable인 경우 그 연결을 과거 EXE source identity 증명으로 쓰지 않는다. `structured-text`/`production-export`처럼 measured process가 다른 engine/server/media tool을 호출할 수 있는 label은 top-level hash가 그 자식 engine의 identity까지 증명하지 않는다.

## 측정 label 전체 목록과 EXE identity

다음 H번호는 이 문서의 분류용이다. EXE SHA256은 원본 측정 JSON에 기록된 값이다. `r1/r2/r3` 표기는 해당3개 개별 label 모두를 뜻하며, `base-fp32/tiny-q8`는 각 조합을 명시한다. aggregate suite/evaluation/preflight JSON은 독립 EXE 측정 label로 중복 세지 않았다.

### A. mutable model-tool 계열 —27개 측정,16개 hash

전부 `missing-historical-binary/source`다. label별 hash/args/결과는 보존되어 있으나 당시 binary·source bundle 연결을 찾지 못했다. 감사 초기에 현재 `player-model-spike.exe`의 hash는 `e18e3092ffdb56b89950e91541d664291c64b64deabc808f0ac290fa006d95bb`였고 아래 어떤 hash와도 같지 않았다. debug/deps의 현재 .exe들도 아래 hash와 일치하지 않았다. latest tool-source·현재 Cargo.lock을 각 과거 hash에 소급 배정하지 않는다.

| H | 측정 label | EXE SHA256 |
|---|---|---|
|01|depth-fp32; ocr-ko-wide; vad-dialogue; yamnet-music|`08dd1964069272c7ab8416b202d5082328333226767ed4d27a6e908b29a74c36`|
|02|diarize-f32|`0af324ddda204b9350b42b1756ed9dd5bbbac2c2973515ba2f53f90b852224d8`|
|03|text-structured-balanced-v4|`192ad33c578c2c5dc4fab1de3dd930d14342d4dd4866dd90c23d901f14cbd73c`|
|04|production-manual-export-refined|`1b21f89a26ed96f64e6b0a043b080368244a760253d7224b5d2ab190af37618f`|
|05|depth-q8; ocr-ko-tight|`1c2c8a0847bb5b50e8c859de4992b26d645b4d54fa398affe036f2422d3871ee`|
|06|ocr-jp-v6|`3c83c77985ca42ce7822bca2a478b6af2566bc1d2cf230e3207360de0a188077`|
|07|diarize-rust-fbank|`7b3f3f813f2158633b5884cc0fead2501c43747d1456f1b2e12dfd40a1ad40f8`|
|08|depth-asset-preview|`7ffa161170debc7428ac72fc736c3870a9d554537276e74c360b32f3fc5cb6de`|
|09|align-vad-ja1516-r1|`9109447eb2a515351697c26c2537f20a565c749a669335fe04d2519ad76788d4`|
|10|depth-asset-export|`929b2d2404de16ed50fc1923209b7e8352f9dfd32d4e43ca7794ca32783d7ebd`|
|11|depth-fp32-r1/r2/r3; depth-quantized-r1/r2/r3|`9f47f294840e7dad20154196298b49dc70c1f1f535fe5c51b314538fe07c83b5`|
|12|ocr-jp-v6-bgr|`a36b2691c80291c4852f679b9adf2b2db0c1b512c9ec39fb64a25ec6cea3f8bd`|
|13|text-structured-balanced|`ad7a49000e37813a5aa4d956814f16e67464fe042da7e17bf4a51c9cf732f914`|
|14|detector-nano; diarize-int8|`b5ccc62c4793716f34af11d08c7121489c05e53df7bd7b63ee58cdbd0e61db36`|
|15|assets-export; fingerprint-control|`b7980256a6940d0122b3f16376f50ffbd65536c73bccee0e857dfec1c34e5793`|
|16|production-manual-export|`fd70cdc7b2a4c07bedee572f3fc8d637ae904e8ac52a9a442d79017a5c4c8132`|

원래 관측(실행·출력·자원·품질의 각 상태)은 그대로다. missing 판정은 예를 들어 depth/OCR 실행이 없었다는 뜻이 아니라 **정확히 그 실행 EXE와 당시 source의 보존 연결이 제한됨**을 뜻한다. 예전 `.rs` 하나를 추정 복원해 다시 빌드하는 것만으로 이 표를 통과로 바꾸지 않는다.

### B. Candle Whisper/SAM·portable text engine —38개 측정

| H | 측정 label | EXE SHA256 | 보존 판정 |
|---|---|---|---|
|17|asr-fp32-r1/r2/r3; asr-quantized-r1/r2/r3; asr-ko-base; asr-ko-fp32; asr-ko-q8; asr-tiny-f32; asr-tiny-q8|`d82ecd2772cfffd6bd146b3c7dc892a91bfef0af53bcb6196645e42d7a40fad7`|11개. 과거 whisper EXE 미확인, historical source 연결 limited |
|18|asr-ko-base-text; asr-ja-1516-base-fp32; asr-ja-1516-tiny-q8; asr-ja-1578-base-fp32; asr-ja-1578-tiny-q8; asr-ja-1634-base-fp32; asr-ja-1634-tiny-q8|`488343c11f0be0a79c0016f2a1c05d729d69253da4ffd5121e953e199a040102`|7개. 같은 제한 |
|19|asr-ja-1516-base-fp32-bounded; asr-ja-1516-tiny-q8-bounded; asr-ja-1578-base-fp32-bounded; asr-ja-1578-tiny-q8-bounded; asr-ja-1634-base-fp32-bounded; asr-ja-1634-tiny-q8-bounded|`e980948855b7a552f85a6f15690486060c9e5a07cd3d5fe59e3c915d9175720a`|6개. 현재 runtime whisper.exe 실제 hash 일치. `limited-live-binary` |
|20|sam-tiny-person; sam-tiny-r1|`dfcfff6f7d6d72de6a75277fe5d53eca21b72def3f0407557dd5324767df9d05`|2개. runtime segment-anything.exe/segment_anything.exe 둘 다 hash 일치. `limited-live-binary` |
|21|text-cards, text-cards-clean, text-cards-next, text-cards-balanced; text-correction, text-correction-clean, text-correction-next, text-correction-balanced; text-highlights, text-highlights-clean, text-highlights-next, text-highlights-balanced|`59ee8ff009a06cee8c39890cf6fd90d74f4617e46e88ac9b46a28a11538a9d73`|12개. runtime/ai-benchmark/mistralrs-v0.9.4-cpu/mistralrs.exe 실제 hash 일치. `limited-live-binary` |

Whisper/SAM은 vendor 예제 소스·Candle audio 원본/수정 파일·일부 source hash가 남아 있다. 그러나 현재 vendor main.rs 및 한 최신 lock이 세 whisper EXE 버전 전체의 당시 source를 증명하지 않는다. `.d`/`.pdb`·빌드 캐시 파일의 존재도 당시 전체 source 사본과 같지 않다. H19/H20/H21은 현재 동일 binary가 확인된 것이며 evidence의 versioned EXE+완전 source manifest로 동결된 build라는 뜻은 아니다. portable mistral engine source/compiler 설정도 이번 범위에서 완전 재구성 검증하지 않았다.

### C. Qwen·ForcedAligner —14개 측정

| H | 측정 label | EXE SHA256 | 판정·보존 경로 |
|---|---|---|---|
|22|qwen-asr-ko-r1|`e0de67bd1e21b3e6adf6c0a76809dc6705416eb9e84c81c0a9a1e32ab3df5f3a`|1개. exit2/success=false가 원래 기록. 해당 EXE와 당시 adapter source frozen 연결 미확인. `missing-historical-binary/source` |
|23|qwen-asr-en-r1; qwen-asr-ja1516-r1; qwen-asr-ja1578-r1; qwen-asr-ja1634-r1; qwen-asr-ko-r4|`fc9246b1a8bbc477e0cccf858fa24ac38e9a6e599be5b6c0119d672800c960fe`|5개. EXE는 [0.1.0-fc9246b1](../../../evidence/player/qwen-asr-builds/0.1.0-fc9246b1/build-manifest.json)에 보존·실제 hash 일치. source는 `limited-reconstructed-source` |
|24|forced-align-ja1516-r1; forced-align-ja1578-r1; forced-align-ja1634-r1; forced-align-ko-r1; forced-align-ko-shift1000-r1; forced-align-ko-shift1040-r1|`a2e4568bf2a4b1acceaa46b91db46ffd57adb92730bd9dfa31ac3a149744de1b`|6개. [0.1.0-a2e4568b](../../../evidence/player/forced-aligner-builds/0.1.0-a2e4568b/observed-build.json)의 EXE·source·lock·archive 존재. `frozen-present`이나 source fingerprints는 일부만 |
|25|forced-align-ja1634-a-v11-r1; forced-align-ja1634-b7000-v11-r1|`dca7db3164c40b27380d35435bd4750653dd43a1583623b5eb397f12c706be13`|2개. [0.1.1-dca7db31](../../../evidence/player/forced-aligner-builds/0.1.1-dca7db31/build-manifest.json)에 EXE+파일별 source/lock·archive 보존. `frozen-present` |

Qwen [README](../../../evidence/player/qwen-asr-builds/README.md)는0.1.0-fc9246의 source가 contemporaneous snapshot이 아니라 관측에 근거한 **재구성**이며 역사적 per-file byte identity는 미입증이라고 명시한다. 실제 EXE+관측 lock+원본 engine archive가 있다는 점을 보존하되 source 재구성 성공·원래 EXE 재빌드 가능을 주장하지 않는다. source/Cargo.lock 실제 hash `49947e6a18246d6130afb4bd671268aa48af8a64b2512eae690724c2bfd0c684`는 manifest와 맞았다. stored EXE는 `qwen-asr-spike.exe`이며 manifest.executable.name의 원래 basename으로 저장 위치를 추측하면 안 된다([storage supplement](../../../evidence/player/qwen-asr-builds/storage-contract-supplement.md)).

aligner0.1.0 `observed-build.json`의 source_status는 contemporaneous frozen handoff다. 그러나 note가 **selected hashes는 전체 per-file manifest가 아님**을 명시한다. EXE 실제 hash, source lock의 `cb9b5102eb20f1b1823123fb04676743597248238f5e2443e03976bdb5de25f2`, selected engine.rs hash의 일치를 직접 확인했다. 전 source가 compiler에 입력된 당시 bytes 그대로인지 완전 입증했다고 확대하지 않는다. aligner0.1.1-dca7는 manifest-listed coverage이며 source-status 선언·파일 보존이 historical complete build attestation과 동일한 것은 아니다.

## 측정 재실행과 구분할 다른 frozen artifact

- Qwen [0.1.1-d5931cc3](../../../evidence/player/qwen-asr-builds/0.1.1-d5931cc3/build-manifest.json)의 EXE actual hash는 `d5931cc325f4353248865481c3c8721369fd1c4495ff7b0ed49b2ef58a4b258d`, source_status=contemporaneous_source_snapshot이다. 이번79개 측정의 EXE hash에는 없다. `*-adapter-0.1.1-output.json` 등 기존 raw 출력의 정책/평가 버전을 fc neural 측정의 재실행으로 세거나 fc source의 contemporaneous 증거로 대체하지 않는다.
- aligner [0.1.1-d88a6e56](../../../evidence/player/forced-aligner-builds/0.1.1-d88a6e56/build-manifest.json)의 EXE actual hash `d88a6e5673af582975b76c24f1b9efbb0d0d39d8f9bdd9d9f510e42410ffa7ff`도 보존됐다. source/lock/실행파일이 있는 최종 frozen handoff build이며 dca7은 pre-final이다. 다만 이번79개 측정에는 그 hash label이 없다. 같은 adapter_version0.1.1이더라도 dca7 EXE와 같지 않다.
- repair diagnostic [0.1.0-2781d98f](../../../evidence/player/repair-diagnostic-builds/0.1.0-2781d98f/build-manifest.json)의 EXE actual hash `2781d98fc7e1fea6d8d346783663850baa1a36e525adb3aeea0e2df012068c74`는 versioned source/report와 함께 보존됐다. [forced-repair-root-v010-r1](../../../../runtime/ai-spike/results/forced-repair-root-v010-r1.json)은 synthetic ms의 공개 repair 호출이며 model_weights_loaded=false/neural_inference_executed=false로 분리한다. 이 artifact를 추가 정렬 추론으로 세지 않는다.

원본 engine source archive actual hash도 직접 읽었다. Qwen 두 build의 archive는 `27aed60a749cb82787a24c0b44d481bfb2e89f9835298897f52b487a5e0fc0cf`, aligner/repair builds의 archive는 `22ae59e1c784c025200701e2891c9ef0d6bfc248b0facdad3e3f138669904600`로 기록과 일치했다. 이는 archive byte identity이지 archive를 풀어 현재 Windows CRLF checkout·모든 compiled inputs와 전체 대응시킨 검증이 아니다. 기존 [integrity-check](../../../evidence/player/forced-aligner-builds/integrity-check-2026-10-18.json)도 source_coverage와 reconstruction 미입증 한계를 명시한다. 이번 감사는 그 전체 파일 검사를 새로 실행하지 않고 EXE/archive·선택 lock/지문을 직접 확인했다.

## 감사 중 추가되는 model-tool 보존 기능

root가 향후 실행용으로 `.spec/evidence/ai/model-spike-builds/<full-EXE-SHA>/`에 별도 source/EXE 저장 기능을 구현 중이라고 전달했다. 감사 중 `0e0d3375545402fb6b13d97807f11968279eee4a55738b3c41d96606420f0b1b` directory에서 actual EXE hash 일치·source 사본과 **INCOMPLETE.md**를 읽었고, 그 확인 시점에는 build manifest가 없었다. 이 작성 중 상태를 완료된 frozen bundle이라고 표시하지 않는다. root가 이후 완료하면 그 시점의 새 manifest/hash로 현재 build에 한해 별도 확인할 수 있다.

새 `build.rs`는 src/Cargo.toml/Cargo.lock/build.rs와 shared validation processes.rs의 지문을 compiled-source.json에 넣는 source를 확인했다. 이것은 과거 H01~H16의 당시 source를 복구하는 기능이 아니며 과거 missing 판정을 소급 삭제하지 않는다. 새 EXE hash도 위79개 label의 hash와 다르다. 이 감사는 root의 새 기능을 실행하거나 상태를 변경하지 않았다.

## 후속 보존 요구와 해석 한계

앞으로의 측정은 실행 전 frozen EXE와 source manifest를 만들고, label record가 그 build identity를 참조하면 좋다. orchestrator와 자식 engine·미디어 adapter·DLL·모델/사전·입력·실행 옵션을 각각 식별하고 source snapshot의 contemporaneous/reconstructed를 구분한다. historical missing을 보완할 자료가 새로 발견되면 원본 해시와 비교해 별도 supplement로 추가하며 원래 관측/실패 분모를 지우지 않는다.

이 감사는 label identity·artifact 보존 범위를 한정한 결과다. 모델 다운로드·runtime build·재추론 없이 기존 품질 수치와 실행 사실을 그대로 보존했다. Windows toolchain/dependency cache·native 라이브러리·환경 전체의 재현이나 bit-identical build는 이번 감사에서 증명하지 않았다.

후속 인계: root는 새 build `342202cad0c869fd7ab07ab7bbf4d467aee7fc1237387fa1c04ab555f62d6da7`의 manifest 66개 파일 보존 완료를 알렸다. 감사자는 해당 build-manifest.json의 존재와 executable_sha256 선언을 읽었으나 66개 파일 전체 hash 검증은 수행하지 않았다. root의 별도 검증 결과와 구분한다. 0e0d bundle은 INCOMPLETE로 유지되며 새 build도 과거 79개 측정의 missing을 소급 해소하지 않는다.
