---
id: evidence:kinekino/20261029113646
title: 실제 자막 의미 벡터·양자화 관측
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 실제 자막 의미 벡터·양자화 관측

## 원본 Candle 참조 대조의 취득 상태

이 절은 취득 시점의 이력이다. 후속 원본 Candle 실제 추론1회와 ONNX FP32 수치 대조는 [독립 참조 관측](embedding-reference.md)에서 완료했으며 자원 게이트 차이·과거 토큰 재현 한계도 함께 기록한다.

ONNX 변환/전처리 차이와 모델 한계를 구별하기 위해 별도 Rust Candle BERT 참조 어댑터를 준비했다. `minilm-original-r1` 취득은 실제 완료했다. 고정 revision `e8f8c211226b894fcb81acc59f3b34ba3efd5f42`의 `model.safetensors`는470,641,600bytes이고 SHA256 `eaa086f0ffee582aeb45b36e34cdd1fe2d6de2bef61f8a559a1bbc9bd955917b`가 공식 LFS 기대값과 일치했다. `special_tokens_map.json`239bytes와 `modules.json`229bytes도 실제 취득했다. 일반 Git 파일의 로컬 SHA와 LFS hash 확인은 구별한다.

CPU41.015625%·여유36,277,530,624bytes를 확인한 뒤 ADR-012에 따라 순차 스트리밍했다. 기존 ready=false와 사용자 앱을 보존했다. [취득 기록](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/minilm-original-r1/result.json)에 파일별 identity를 남기며 기존 download-embedding.json은 수정하지 않았다. 이 취득의 실행 도구는 SHA256 `d8d6a157a56edc739b5b1f03e30ecc5d57c92e340c0361654a3f128046ae4e22`, source71파일을 동결한 build다. 별도 참조 경로의 모델 로드·추론은 아직0회이며 위 ONNX2회에 추가 계산하지 않는다.

2026-10-18. 준비·실행 전 대기·실제 추론을 분리한다. 의미 검색은 기존 텍스트 생성과 다른 역할이며 문장 벡터만으로 하이라이트의 재미나 정답을 판정하지 않는다.

## 입력과 모델 계약

기존 실제 Sintel 한국어 배포 자막에서 파생한 `runtime/ai-spike/assets/cues-ko.json`의 cue ID·원문·in/out us를 유지한다. 이 자막은 번역이며 ASR 정답으로 사용하지 않는다. [공식 모델 카드](https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2)는384차원·attention-mask mean pooling·max sequence128을 제시한다. 후속 cosine 비교를 위해 별도 L2 정규화를 명시하며 모델 출력에 이미 정규화가 적용됐다고 가정하지 않는다.

기존 catalog 고정 revision은 `e8f8c211226b894fcb81acc59f3b34ba3efd5f42`다. 예정 FP32 `onnx/model.onnx`는470,301,610 bytes/SHA256 `10f7a088420252b26caf819236ca2c9d2987afd0fc06fec7553b542a5655a05a`, AVX2 양자화 `onnx/model_quint8_avx2.onnx`는118,453,870 bytes/`98a01d88b7de996cdea58c32ca71208c09968d143798814b2ea09d3439dc334f`다. 이는 Hub metadata의 선언값이며 실제 파일 취득·해시 일치 결과와 다르다. 파일 크기로 실행 메모리·속도를 예측하지 않는다.

## Rust 준비와 실행 상태

독립 model-spike 도구의 domain은 attention-mask mean/L2와 shape·유한값·zero norm 검사를 담당한다. infrastructure는 tokenizer JSON·ORT CPU token graph 실행·pooling config 확인을 담당한다. cue별 원문·좌표·token count·truncation 여부·384 vector를 보존하며 검색 품질은 false로 유지한다. Tokenizers0.22.2에는 native Oniguruma 종속성, ORT에는 native ONNX Runtime이 있다. 새 개발 코드는 Rust지만 전체 외부 계산 라이브러리가 순수 Rust라고 표현하지 않는다.

`embedding-fetch-r1`은 자원 사전 검사에서 실행 중인 player를 발견해 대기했다. 다운로드/모델 로드/추론은 시작하지 않았다. 사용자 지시로 열린 UI 데모를 root가 종료하지 않는다. preflight JSON을 보존하고 재시도에는 새 label을 사용한다. 도구 컴파일은 모델 실행이 아니다.

후속 Rust 단위 검사14개(신규 pooling의 padding 제외·zero norm·NaN·shape 거부 포함)와 fmt/Clippy·locked 실행 파일 빌드가 통과했다. graph에 실제 입력을 넣은 검사가 아니며 tokenizer/ONNX parity나 검색 정확도 증거가 아니다. Cargo.lock에 tokenizers0.22.2를 고정했다. 실행 파일 SHA256은 `994b7f3d80c8b4ae4d3b73887e9405ae1a44e1a78a30a76c806d7211fd5061b4`이며540개 snapshot에 새 source/lock·취득 대기 preflight를 반영했다.

취득·실제 자막 추론·FP32/AVX2 동일 입력 비교는 아래 후속 관측에 기록한다. 남은 게이트는 독립적인 사람의 relevance 검토, 더 넓은 질의/음성 대조, no-match 보정과 기본 모델 선정이다. 원문 복사 자기 유사도를 의미 일반화 품질로 제시하지 않는다. 공통 SDK/GUI의 의미 검색 기능은 아직 구현된 상태가 아니다.

## 평가 입력·순위 집계 준비

후속 취득: [스트리밍 취득 정책](../../../adrs/player/adr-012-streaming-model-acquisition-headroom.md)에 따라 CPU14.174%·여유36,143,280,128bytes와 열린player3개를 확인한 뒤 Rust fetch를 순차 실행했다. preflight.ready=false는 숨기지 않았다. 고정 revision의8파일을 실제 취득했고 FP32470,301,610bytes/AVX2양자화118,453,870bytes/tokenizer9,081,518bytes의 LFS SHA가 모두 일치했다. 일반5파일에는 LFS hash를 만들어 넣지 않고 실제 로컬 SHA만 기록했다. 모델 로드·추론은 아직0회이며 다운로드 시간/peak를 신경망 지표로 제시하지 않는다.

Rust `semantic-fixture`가 실제26cue를 복사해 role만 붙이고 의역5개·무관 질의1개·원문 복사 대조1개를 추가한33개 입력을 저장했다. 원본 source hash가 전후 동일함을 확인했다. fixture SHA256은 `b20ea540f1784e21acc5d78b6eb355ca465d39687b271a33ac8facff7cc237b8`다. 관련 cue ID는 assistant가 문구를 보고 지정한 임시 기준이며 독립적인 사람의 relevance 정답이 아니다.

`semantic-evaluate LABEL`은 실제 graph stdout의 fixture hash·모든 cue 내용/순서·384차원 유한값·정규화 norm을 검사한 뒤 cosine 전체 순위·첫 관련 rank·임시 MRR/hit@3를 기록한다. 의역5개만 양성 집계에 넣고 원문 복사·무관 질의는 제외한다. 무관 질의의 최고 점수를 자동 매칭으로 승인하지 않으며 rejection threshold는 별도다. 준비 시점에는 실제 순위를 생성하지 않았고 후속 실제 결과는 아래와 같다. evaluator 구현/컴파일 자체를 검색 정확도 통과로 해석하지 않는다.

## FP32·AVX2 실제33입력 관측

[관측 정책](../../../adrs/player/adr-013-embedding-correctness-observation.md)에 따라 source/EXE0e2a88f84fb32ce3c2607b968ff71d0e7faaa60a8ecca10d1f64d91099892478를69파일 동결한 뒤 두 모델을 순차 실행했다. worker/orchestrator EXE와 manifest SHA가 측정에 함께 기록됐다. CPU4/interop1·BelowNormal·offline이고 player3개를 보존했다. 두 실행 모두 exit0/timeout없음/33행·384차원 유한 정규화 벡터 검사를 통과했다. 최대 관측 token17개로128토큰 truncation 경계의 실검증은 아니다.

| label | wall 초 | 표본 최대 private bytes | 임시 MRR | hit@3 |
|---|---:|---:|---:|---:|
| embedding-fp32-observed-r1 |3.7116|762,150,912|0.5500|0.8|
| embedding-avx2-observed-r1 |1.8573|419,835,904|0.5533|0.8|

wall에는 로드·토크나이저·SHA 검사·직렬화가 포함되며 library row-loop 시간은 각각0.2618/0.1607초였다. 한 쌍의 관측이며 UI 부하/캐시를 통제한3회 반복 성능 비교가 아니므로 일반적인 속도/메모리 우열을 승인하지 않는다. 측정의 performance_comparison_eligible=false를 유지한다.

실제 임시 관련 순위는 칼날/무고한피1위씩, 고마움/용의땅3위씩, 찾지못했다12위(FP32)/10위(AVX2)였다. 무관 야구 질의도 '용이요' cue10에0.4254/0.4012 점수를 냈다. 높은 점수만으로 자동 관련 판정을 하지 않으며5개 assistant 기준을 독립 사람의 정답으로 부르지 않는다. exact-copy는1위였지만 양성 집계에서 제외했다.

[독립 수치 비교](../../../evidence/ai/model-spikes-2026-10-18/results/embedding-fp32-avx2-comparison-r1.json)는 동일 도구·입력·token/pooling 매핑을 확인했다.33벡터의 FP32↔AVX2 평균 cosine0.992899/최소0.936871, 최대 성분 절대 차이0.062137이었다.7질의의 top1은 모두 같았으나 exact/no-match도 포함하는 일치도이며 검색 정확도100%라는 뜻이 아니다. 관련 순위가 낮은 질의의 실패를 숨기지 않고 semantic_quality_approved=false를 유지한다. 두 실제 실행/평가/벡터/비교와 신규 비교 도구70파일 동결본은575개 snapshot에 보존했다. 후속 후보 조사는 품질 미달을 통과로 바꾸지 않는 별도 비교다.
