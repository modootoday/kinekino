---
id: evidence:kinekino/20261029113601
title: E5 의미 검색 스파이크의 준비·취득 상태
status: active
created: 2026-10-29
updated: 2026-10-29
---
# E5 의미 검색 스파이크의 준비·취득 상태

2026-10-18. 공식 `intfloat/multilingual-e5-small` FP32 ONNX의 준비·취득 후 실제33입력 추론1회를 완료했다. 준비 당시 추론0회와 아래 후속 실제 실행을 구별한다. 실행 호환성·출력 구조와 검색 품질 승인은 별개다. [모델별 상태표](model-status.md), [후보 연구](../../player/infrastructure/inference/research-followups/semantic-retrieval.md)를 따른다.

## 고정 자산과 실제 취득

새 Rust `catalog-fixed`는 기존 metadata를 덮어쓰지 않고 고정 revision API 응답의 repo·sha를 확인해 별도 key `retrieval-e5`로 저장했다. revision은 `614241f622f53c4eeff9890bdc4f31cfecc418b3`이며 metadata SHA256은 `9f3aa989077af590a10ee175332e77ac9842cf96c2fffba26351c2224ddbefd1`다. 기존 global catalog의 historical28개 기록은 수정하지 않았다. 코드의 후보 목록에 새 key를 추가한 것과 원래 catalog의 조회 시점을 구별한다.

`e5-fp32-r1` 취득에서9파일을 순차 스트리밍했다. 실제 `onnx/model.onnx`470,268,510bytes/SHA256 `ca456c06b3a9505ddfd9131408916dd79290368331e7d76bb621f1cba6bc8665`, `tokenizer.json`17,082,730bytes/`0b44a9d7b51c3c62626640cda0e2c2f70fdacdc25bbbd68038369d14ebdf4c39`가 공식 LFS와 일치했다. 나머지 config·pooling·special tokens·modules·sentence config·README7파일은 실제 로컬 hash를 기록했으며 LFS 확인으로 표현하지 않는다. [취득 결과](../../../evidence/ai/model-spikes-2026-10-18/acquisitions/e5-fp32-r1/result.json)와 [고정 metadata](../../../evidence/ai/model-spikes-2026-10-18/catalog/retrieval-e5.json)를 보존한다.

실제 사전 검사 CPU46.58203125%·여유32,668,569,600bytes에서 ADR-012 기준을 통과했다. UI 데모 때문에 ready=false인 원문 기록을 유지했으며 사용자 앱·서비스·메모리 캐시·OS 설정을 변경하지 않았다. root가 한 취득 worker로 실행했고 다른 모델 추론을 겹치지 않았다. 다운로드 wall time을 신경망 속도로 제시하지 않는다.

취득 실행 파일 SHA256은 `cf3c76c71afb49d3c55d533b3ca5dfa208a4a7313c5a3de6b16c3de1b6fcb5cd`이며 source73파일과 manifest를 먼저 동결했다. 해당 시점 fmt/test17개/Clippy-Dwarnings/locked build가 통과했다. 모델 카드의 MIT metadata와 외부 ORT·tokenizer 의존성 및 변환 계보 검토는 별개다.

## 입력·계산 계약

[공식 고정 모델 카드](https://huggingface.co/intfloat/multilingual-e5-small/blob/614241f622f53c4eeff9890bdc4f31cfecc418b3/README.md)는 비영어에도 retrieval 역할에 맞는 접두어와 mask mean/L2를 요구한다. 원래 `semantic-search-ko-v1.json`의 cue ID·text·role·시간·임시 relevance를 그대로 보존한다. Rust domain의 `e5_input`은 corpus에 `passage: `, query에 `query: `를 추가하고 원문 trim/교정/치환을 하지 않는다. 원문 자체가 prefix처럼 시작하더라도 원문은 제거하지 않는다. 모르는 role·빈 문장은 거부한다.

Rust `retrieval-fixture`가 [별도 입력 계약](../../../evidence/ai/model-spikes-2026-10-18/results/e5-input-contract-v1.json)에33개 원본 cue와 별도 model_input을 저장했다. source SHA256 `b20ea540f1784e21acc5d78b6eb355ca465d39687b271a33ac8facff7cc237b8`는 생성 전후 동일했고, 계약 SHA256은 `dc4d3259282e3fee1ea05d174558f5a09b0cefc8ac5173639cf263f5bc5437a0`다. 임시 relevance는 assistant의 판단으로 사람 정답이 아니다.

이번 비교의 제한은 batch1·최대128tokens로 명시한다. 공식 최대512tokens와 동일하다고 표시하지 않는다. 로컬 config는384차원·12층 BertModel·XLMRobertaTokenizer이며 modules는 Transformer→Pooling→Normalize, pooling은 mean only, sentence config는max512/lowercase=false였다. 이 설정 확인만으로 ONNX 입력 이름·dtype·출력 형상을 추정해 실행 성공으로 처리하지 않는다. 다음 실제 adapter가 graph signature와 finite384벡터를 검증해야 한다.

## 남은 검증

이하 준비 당시의 실행 게이트 중 graph signature·CPU 실행·384유한정규벡터 검사는 아래 실제 관측으로 통과했다. 독립 변환 parity·사람 relevance·no-match 및 held-out 품질은 남는다.

- 실제 ORT graph signature와 CPU kernel 지원, 동일 token·mask·type 처리, 실제 최대 token 수·truncation을 관측한다.
- 원본33개 cue와 별도 model_input의 대응을 보존하며 의미 평가기는 원본 cue 기준으로 비교한다. prefix를 원문 relevance에 섞지 않는다.
- MiniLM과 임시 질의별 순위·실패 사례를 비교한다. 모델 간 절대 cosine을 동일한 확률이나 rejection threshold로 취급하지 않는다.
- 사람 기준·held-out 질의와 no-match 보정 전 품질 승인을 하지 않는다. 공식 AVX512-VNNI 파일을 현재 AVX2 호스트용 양자화 결과로 부르지 않는다.
- 실제 worker는 별도 검토된 observation 계약·frozen build·자원 사전 검사 후 root에서 하나씩 실행한다.

## 실제 FP32 33입력 관측1회

[ADR-014](../../../adrs/player/adr-014-e5-retrieval-correctness-observation.md)에 따라 label `e5-fp32-observed-r1`을 root 단일 CPU worker로 실행했다. worker/orchestrator SHA256은 `e984bd3cf688091c143b6a4e0366f835bc77d0e5c27bea57749fffb4817db6a6`, 선보존73파일 manifest SHA256은 `5d6e3eb09ec345ecbf57ac53c29306d354d1326b16fa2eaf315d3a50505476de`다. 해당 변경의17tests/fmt/Clippy-Dwarnings/locked build가 통과했다.

실제 graph는 int64 dynamic[batch,sequence]의 input_ids/attention_mask/token_type_ids와 float32 last_hidden_state[batch,sequence,384]였다. 각 row에 원본 cue와 실제 model_input/ids/mask/types를 보존했다.33개 모두 유한384차원·정규화 검사를 통과했고 최대19tokens로128 제한 내였다. 종료0·timeout=false였다.

실제 preflight는 CPU22.722820763956907%·여유37,458,100,224bytes·ready=true·validation_apps=[]였다. 이전 취득의 ready=false 상태를 이 실행으로 덮어쓰거나 앱을 root가 종료했다고 추정하지 않는다. 원래 앱 목록을 결과에 보존했다. wall3.4604066초, sampled private peak1,061,515,264bytes, working set770,420,736bytes, graph load0.7076518초·worker total1.6528158초였다. wall에는 모델/입력 hash·시작/직렬화가 포함된다. 목적은 retrieval_correctness_not_performance이며 performance_comparison_eligible=false다. 서로 다른 시점의 단1회 실행으로 모델 간 성능 우열을 주장하지 않는다.

| 임시 의역 질의 | MiniLM FP32 관련cue순위 | E5 FP32 관련cue순위 |
|---|---:|---:|
| 칼날의 어두운 내력 | 1 | 1 |
| 죄 없는 사람들의 피 | 1 | 1 |
| 고마움 표현 | 3 | 8 |
| 용들이 사는 땅 | 3 | 1 |
| 찾으려던 대상 미발견 | 12 | 1 |

임시 MRR은0.55→0.825, hit@3은0.8→0.8이었다. 일부 개선과 고마움의 악화를 함께 보존한다. 무관 야구 질의의 top은cue16/cosine0.7894168381197564였으며 자동 매칭 미승인이다. exact-copy는 다른 접두어를 사용하므로 자기벡터와 동일한 대조가 아니고 top1/cosine0.9576862580236266이었으며 양성 집계에서 제외했다. assistant 임시 relevance5개로 전체 한국어·장문·다국어 품질이나 모델 채택을 승인하지 않는다.

[원시 출력](../../../evidence/ai/model-spikes-2026-10-18/results/e5-fp32-observed-r1.stdout), [실행 기록](../../../evidence/ai/model-spikes-2026-10-18/results/e5-fp32-observed-r1.json), [평가](../../../evidence/ai/model-spikes-2026-10-18/results/e5-fp32-observed-r1-semantic-evaluation.json)를 보존한다. 의미 평가기는 원본 fixture hash·cue 내용/순서·벡터 shape/norm을 확인했으며 사람의 의미 판정을 대체하지 않는다.
