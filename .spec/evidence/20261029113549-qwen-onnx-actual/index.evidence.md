---
id: evidence:kinekino/20261029113549
title: Qwen ASR 혼합 INT4 실제 추론
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Qwen ASR 혼합 INT4 실제 추론

2026-10-18 초기5회의 기록이다. [준비 검증](qwen-onnx-preparation.md)의 고정 community ONNX 모델을 Rust adapter와 native ONNX Runtime CPU 경로로 실제 실행했다. 실행 성공·prompt 계약 통과와 사람 검토가 필요한 자막 품질을 구별한다. 한국어 초회와 opt2 재실행, 일본어1516·1634·1578의 실제5회가 모두 exit0·EOS도달·timeout=false로 종료했다. 중복 한국어를 독립 품질 표본으로 세지 않는다. 후속 [같은 ONNX FP32/INT4 4쌍8회](qwen-onnx-paired.md)는 별도 cohort다.

## 추론 전에 차단하고 수정한 prompt 결함

고정 Rust 엔진의 `prompt.rs`는 system=9125, user=882를 사용했다. 실제 고정 tokenizer.json의 공식 tokenizers 인코딩은 system=8948, user=872다. 최초 `check-tokenizer`는 이 불일치로 모델 로드 전에 실패했다. 이 실패를 ASR 품질 실패나 신경망 실행으로 계산하지 않는다.

별도 fork의 두 상수를 실제 값으로 고쳤다. 단어별 검사 외에 전체 system/user/audio/assistant/language prompt를 공식 tokenizer로 encode한 배열과 실제 엔진의 prompt 생성 함수 반환값을 대조한다. 3audio-pad 진단에서 한국어·일본어·영어 모두 전체 배열이 일치했다. 언어 ID도 각각16134/10769/6364로 일치했다. 이는 이 고정 tokenizer·template에 대한 계약 검사이며 전체 processor tensor parity가 아니다. 잘못된 prompt로 신경망을 돌려 개선 수치를 만들어내지 않았다.

## adapter의 실행 경계

Rust `tools/qwen-onnx-spike`는 application 조정·domain 검토 상태·infrastructure 파일/모델/tokenizer/engine/보존을 분리한다. 원본 엔진의 CPU 세션은 intra4/inter1·순차 실행·intra spinning off로 고정했다. GPU와 decoder GPU를 명시적으로 끄고 FP32 encoder/INT4 decoder/FP16 저장 임베딩의 F32 캐시 경로를 사용했다.

loader 호출 전에7개 모델 파일의 고정 SHA를 모두 확인하므로 누락된 INT4 파일의 FP32 fallback을 허용하지 않는다. mono16k PCM16·25ms~30초·128token 한도를 적용한다. 생성 token IDs·실제 prompt·EOS와 raw decoded text를 보존하며 임의 prefix 삭제·문장 교정을 하지 않는다. empty·markup·token limit 결과는 검토 상태로 유지하고 정상 종료도 품질 승인으로 올리지 않는다.

## 실제 한국어 결과

| 항목 | qwen-onnx-ko-r1 |
|---|---|
| 원음 | 기존 FLEURS KO 12.48초·199680samples |
| WAV SHA256 | ab8cd52bf831d09862cd8eddf24050a2f279811409f11c435ed72888f67622d6 |
| 프로세스 | exit0·success=true·timeout=false |
| EOS·token 수 | true·52개 |
| normalized CER | 3/53 = 5.660377358490566% |
| load / inference 함수 | 16338ms / 5820ms |
| 전체 worker wall | 183.8254719초 |
| sampled peak private / working set | 3,031,281,664 / 2,830,438,400bytes |
| admission CPU / free RAM | 27.83203125% / 33,611,591,680bytes |
| 원시 출력 | 다리미 수직 간격은 15m이며 공사는 2011년 8월에 마무리되었으며 해당 다리의 통행금지는 2017년 3월까지이다. |

CER는 기존 evaluator와 같은 Unicode alphanumeric/lowercase·공백/문장부호 제외 규칙이다. protocol 문자를 지우지 않았다. 원문의 `다리 밑`, `15미터` 등에 차이가 남으며 동일 입력의 [기존 Candle F32](qwen-asr.md)와 CER가 같다는 제한 관측이다. engine·export·mel·prompt·정밀도·실행 조건이 달라 순수 양자화 효과 또는 속도 우열로 표시하지 않는다. `performance_comparison_eligible=false`를 유지한다.

최초 adapter는 비최적화 Rust dev build였다. 함수 계측 밖 약161.7초에는 무결성 검사·tokenizer 검사·시작 등 다른 adapter 비용이 포함되며 hash 단독 시간을 따로 계측하지 않았다. 실행 중 낮은 private memory와 CPU 누적을 관측했다. 후속 빌드는 Rust opt-level2와 단계별 stderr 시간을 추가해 이 비용을 분리한다. 첫 실행 값을 후속 최적화 빌드의 값으로 소급하지 않는다.

## 취득과 실행 증거

큰 가중치3개는 `fetch-record qwen-onnx-weights-r1`에서 순차 취득·모두 LFS SHA 대조를 마쳤다. admission CPU44.62890625%, free36,427,841,536bytes, neural count0이다. 작은 계약7파일과 동일 HF revision을 유지했다. 사용자의 ChatGPT/Terminal과 기존 player를 종료하지 않았다.

| 보존 범위 | 식별 |
|---|---|
| 초기 prompt guard 실패 build·62파일 | [055d3608](../../../evidence/player/qwen-onnx-builds/055d3608f6fe50d98dc6fa280f3c0fa5694cffe0af9e99e31a634613fbee761a/) |
| 실제 한국어 worker·62파일 | [daaeb929](../../../evidence/player/qwen-onnx-builds/daaeb929b3bcd498b22155ebc6b862e1b2ed487c5585bafef196b769744d2dbd/) |
| 실제 worker SHA256 | daaeb929b3bcd498b22155ebc6b862e1b2ed487c5585bafef196b769744d2dbd |
| orchestrator·89파일 | [c54dbd7d](../../../evidence/ai/model-spike-builds/c54dbd7d1a7587282e4d222d9674c417ddb5960ac40047466d168ed40a11f1b4/) |
| orchestrator manifest SHA256 | 1874e46982d973ce6a78bcd7b025f964941b90d5b82fc963c18de940431abf9a |

## 최적화 빌드 후속 결과

Cargo dev opt-level2와 단계별 stderr 계측을 적용한 worker는 `dfdfe8aa6e99df76ff0affd05110b4b6555ae394be384b1a8a9ee0e57a06e3d7`이다. [동시 보존62파일](../../../evidence/player/qwen-onnx-builds/dfdfe8aa6e99df76ff0affd05110b4b6555ae394be384b1a8a9ee0e57a06e3d7/)의 manifest SHA는 `a5dee580af09b8394e8f64308e58bc19916a9a1d6f1e2b99e3956a59933bcc06`이다. orchestrator는 [4b2e4e8e·89파일](../../../evidence/ai/model-spike-builds/4b2e4e8e7f64d503af21e107a8b7be17945045df74b0e6e252e1b2a5c2afa224/)이며 manifest SHA `bd2d4416882c2dee2c6f190e49d5b66d103040a4aaae320a21b2ee23e3bbfa06`를 각 측정에 기록했다. 두 도구의 fmt·각2/32tests·Clippy all-targets Dwarnings를 다시 통과했다.

| label | CER | 기존 Candle F32 CER | EOS | hash / load / inference ms | 전체 wall 초 | peak private bytes |
|---|---:|---:|---|---|---:|---:|
| qwen-onnx-ko-r2 | 3/53 = 5.66% | 5.66% | true | 22473 / 3478 / 3612 | 31.5199426 | 3,030,765,568 |
| qwen-onnx-ja1516-r1 | 8/61 = 13.11% | 24.59% | true | 7377 / 3727 / 3851 | 16.9380191 | 3,444,641,792 |
| qwen-onnx-ja1634-r1 | 0/43 = 0% | 0% | true | 8304 / 14783 / 9136 | 35.240825 | 2,753,638,400 |
| qwen-onnx-ja1578-r1 | 0/25 = 0% | 12% | true | 56207 / 3716 / 1747 | 66.4638335 | 2,820,554,752 |

초회와 한국어 재실행의 출력 문구·CER는 같았다. 일본어1516의 오류 수는 기존 F32보다 적었지만 `バニ`, `ホロコースス`, `ナティス` 등 오인식이 남았다. 일본어1634의0%는 한 짧은 발화의 제한 관측이다. 이 표는 정밀도만 바꾼 paired benchmark가 아니며 실행별 CPU 부하와 OS cache도 통제하지 않았다. 최초 wall183.83초를 모델 자체 inference 시간으로 사용하거나 opt2의31.52초를 일반 속도 향상 비율로 홍보하지 않는다.

실제 headroom은 한국어재실행 CPU11.8164%/여유36,753,682,432bytes, JA1516 CPU15.9069%/37,193,789,440bytes, JA1634 CPU19.0430%/36,806,762,496bytes였다. JA1634 후 호스트 CPU97.35%, 이후 별도 read-only 표본77.12%가 관측돼 추가 추론을 즉시 시작하지 않았다. 나중의 read-only 표본46.09%를 확인한 뒤에도 실제 gate를 다시 통과해야 실행하도록 했고, JA1578 admission은 CPU29.3372%/33,800,859,648bytes였다. admission은 OS 전체 hard cap이 아니며 기존 앱을 종료해 수치를 맞추지 않았다. 마지막 입력의 hash56.21초와 inference1.75초도 이 변동을 분리해 보존한다.

JA1578 출력은 `その後 ラッカシンが先頭に立って バジャンを歌いました。`였으며 문자 오류0은 공백·문장부호 제외 규칙 아래의 결과다. 실제 단어 timestamp나 화자 분리 결과가 아니다. 최적화 build의4입력 기준 최대 private 관측 범위는2,753,638,400~3,444,641,792bytes다. 원본 F32의 다른 구현·시점과 이를 직접 비교해 양자화의 메모리 절감률로 계산하지 않는다.

원본은 runtime/ai-spike/results의 qwen-onnx-ko-r1.{json,stdout,stderr,preflight.json}, evaluation.json과 tokenizer-r1/r2에 보존한다. 공통 report로 evidence에도 snapshot한다. adapter2tests·fmtcheck·Clippy all-targets Dwarnings와 orchestrator32tests·fmtcheck·Clippy를 통과했다. 외부 crate를 workspace 의존성으로 -p 지정한 unit test는 workspace 외부 dev 의존성 때문에 실행되지 않았으며 upstream unit test 통과로 집계하지 않는다.

## 남은 게이트

같은 ONNX FP32/INT4 대조는 [후속4쌍](qwen-onnx-paired.md)에서 완료했다. 독립 애니메이션 음성·여러 화자, native 의존성 재배포와 원본 checkpoint revision 변환 계보는 미해소다. 자동 자막·화자/시간 정보 생성이나 제품 SDK 통합을 완료로 표시하지 않는다. 특히 JA1516의 실질 오류가 남아 있어 제한 입력의0%를 전체 일본어 품질로 일반화하지 않는다.
