---
id: sot:kinekino/modoo-today
title: "modoo.today 스킬 적용: 기존 모델 근거·후속 평가 프로토콜 검토"
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# modoo.today 스킬 적용: 기존 모델 근거·후속 평가 프로토콜 검토

2026-10-19 사용자 직접 지시를 PM 경로로 받아 수행했다. **기존 근거를 읽고 판정 경계를 검토한 작업**이며 새 추론·다운로드·worker compile·GUI·평가 실행은 없다. Rust-only·CLI/SDK 우선·중앙 자원 배정·사용자 품질 검토 마지막 순서를 유지한다.

## 실제 탐색·본문·참고자료와 적용

출처는 modoo.today skills MCP의 `mcp__modoo_today_skills__skill_find → skill_call → skill_read`다. 실행 근거·양자화·출력 품질 검토라는 문장으로 검색한 뒤 설명을 읽어 아래3개를 채택했다. 학습/훈련 preflight·catalog scaling·judge calibration 등의 다른 검색 결과는 이번 과제와 달라 채택하지 않았다. 세 스킬은 모델 실행기가 아니라 프로토콜/제공 결과 감사이며 이번 읽기 전용 범위에 적합하다.

| 정확한 skill 이름 | 실제 읽은 자료 | 이번 적용 판단 |
|---|---|---|
| modootoday-free/ai-quality-plus/inference-benchmark-workload-protocol | skill_call 본문, references/protocol-tables.md. reference SHA256 89f6ff32617d660be61433aac491db8f4e1fc1c9bb7b07b8a0c0cf80c1f0c4fd | R3 전체 소요를 decode 속도로 계산하지 않음. 실제 길이/캐시/동시성·prefill/decode·실패 분모와 단일 장비 범위를 구분 |
| modootoday-free/ai-quality-plus/quantization-and-vision-input-parity-check | skill_call 본문, references/worked-example.md. reference SHA256 e8d120408618dd25b39cd4e814630a472322a177313b83b9eae263b1d23ef7b0 | Q4_K_M 이름·저장 tensor·실행 weight/KV dtype을 분리. extraction/잘림을 먼저 확인하고 1회 표본의 모델/정밀도 순위 금지 |
| modootoday-free/ai-quality-plus/llm-output-eval-suite | skill_call 본문. 추가 reference 목록 없음 | correction/cards/highlights별 기준·실패 표본·paired 비교·반복/분산을 구분. schema/SDK 성공을 의미 품질로 승격하지 않음 |

참고자료의 표·숫자는 synthetic 예제이며 프로젝트 실행 결과로 복사하지 않았다. 스킬의 20..50 case·3회 반복·CI 제안은 향후 workload/비용에 맞출 설계 지침이지 지금 새 실행·과금·CI·GUI 승인 gate를 도입하는 지시가 아니다. 다른 언어 예제나 script는 실행하지 않았다.

## 기존 근거에 적용한 판정

| 대상 | 관측 사실 | 가능한 판정과 금지할 확대 |
|---|---|---|
| R3 startup lifecycle actual | source 고정 engine/worker 1회, count2047/2048/2049 각각HTTP200, generation200/400/400. accepted는completion1/total2048. startup dummy 별도. owned/outer exit0. 전체65.8439968s, peak private1,331,810,304B/working1,564,905,472B | 경계 수용/거절과 lifecycle/종료 근거. 2건의 예상 admission 거절을 숨기거나 3건 생성 성공으로 표시하지 않음. count 요청·startup·모델 load를 완료 텍스트 성능 분모에 섞지 않음. performance_comparison_eligible=false 유지 |
| R4 | source1,893+fork53 재검증, check/test1388pass5ignored/strictClippy/CLI build exit0, EXE/원문 archive. worker pin pending, 실제 R4 모델 실행0 | producer source와 엔진 빌드 검증. 구현된 controls9/commit/ACK가 실제 실행 관측됐다고 하지 않음. R3 대비 속도/메모리/품질·noninterference/strictjoin 통과 미확정 |
| 실제9 text package | Gemma270/Llama1B/Qwen0.6 × correction/cards/highlights. exact source request와 candidate content asset, 원문 매핑·SDK 수입/명시 수정·채택의 별도 검증 근거 | 9개는 3task×3model의 고정 표본이며 대표 workload·반복 평가·모델 순위가 아님. schema/import/사람 수정 결과가 모델 원출력의 품질 성공을 뜻하지 않음 |

R3의 2047/2048/2049는 **고정 경계 시험 입력**이지 실제 사용자 길이의 median/p90/max가 아니다. 같은 프로세스에서 순서대로 실행됐어도 warm/prefix hit를 증명하지 않는다. R3에 없는 actual scheduler/prefix/speculative getter를 실행 인자로 대신 확정하지 않는다. R4 source는 그 부족을 보완하도록 설계됐지만 아직 actual 수집이 없다. threads4는 설정값이며 실제 pool/OS cap이 아니다.

R3 전체시간은 load/count/생성/수집/정리 등을 포함하므로 delivered1token÷65.84초를 모델 decode tok/s로 보고하지 않는다. TTFT/prefill/decode/queue·per-request latency와 aggregate throughput은 실제 근거가 있는 칸만 채운다. 현재 동시성·cold/warm 조건별 반복·median/p90가 없어 일반 성능 비교를 지지하지 않는다. R4 Cargo 소요 시간은 모델 성능이 아니다.

## 실제 적용 정밀도와 원문 처리

실제9 package는 세 모델 모두 `GGUF Q4_K_M (mixed stored tensors)`와 max_tokens256/temperature0/seed42/thinking=false/constrained JSON 조건을 기록한다. 이는 보존 package의 선언이다. 이번 검토가 각 런타임의 weight 연산 dtype·K/V cache dtype·적용 context/truncation·fallback을 새로 계측한 것은 아니다. R3 model-inspection은 pin된 Gemma253,115,424B와 SHA/revision 검증을 기록하지만 conversion_lineage_complete=false·tokenizer_template_parity_verified=false다. 모델 파일 이름·의도한 인자만으로 전체 실제 실행 정밀도/계보를 승인하지 않는다.

세 모델 비교는 크기·모델 계열도 다르므로 양자화 한 요인 실험이 아니다. FP16 baseline/KV dtype별 arm과 반복이 없어 Q4가 더 좋거나 손해 없다는 결론도 없다. 이 범위는 text-only이므로 image token budget/tiles/crop은 적용 대상 아님으로 기록한다. 다른 vision 역할에서 확인됐다고 전이하지 않는다.

source asset의 outer JSON→messages[0].content→정확한 실제 LF 경계→suffix 배열 경로를 유지한다. literal backslash-n과 실제 LF, 원본 content와 typed edit를 구별한다. Gemma/Llama correction의 미완성 JSON·반복·문장 누락을 자동 수선해 모델 성공으로 바꾸지 않는다. cards의 cue ID 존재와 실제 문장 근거 일치는 별개고, 원문 없는 사건/깨진 문자·한문장/중복 위반은 task별 내용 감사로 남긴다. correction에는 원래 cue/time이 없으며 cards/highlights의 원본 시간 존재와 product proposal의 명시적 null 정책을 혼동하지 않는다.

## 분모·품질·시간·메모리 인계

후속 평가는 시작/완료/transport error/timeout/blank/잘림/typed 불일치/내용 실패를 case별 분리 보존한다. blank·timeout·잘림을 분모에서 제거하지 않으며 survivor 평균은 명시한다. 현재9개에 없는 runtime timeout 수를 추정해0으로 넣지 않는다. 구조가 잘린 것은 관측 가능하지만 finish_reason/token count가 없으면 원인을 token cap으로 단정하지 않는다. R3 경계 거절은 생성 품질 accuracy가 아니라 admission 정책 case의 expected outcome으로 채점한다.

후속 보고의 한 행은 model+정확 engine/profile/정밀도·task/source pin·길이·반복·cache/concurrency·품질 기준 통과수/전체수·실패 종류·peak memory·전체/단계시간을 함께 갖는다. 9표본의 memory/time이 해당 원본 run에 연결되지 않은 경우 unknown으로 두며 R3 memory/time을 text9에 붙이지 않는다. 품질은 원문 보존/교정, 문구의 근거·가독성, 하이라이트의 근거·선별성으로 slice를 먼저 고정하고 전체 평균만으로 덮지 않는다.

현재 판정은 **실행/근거/typed 처리의 제한된 성공, 제품 모델 품질 승인 없음, 성능·정밀도 우열 미확정**이다. 개발자가 수정/채택한 성공은 product workflow 검증으로 분리한다. LLM judge를 지금 도입하지 않고 사람 정답/도메인 기준 없는 점수를 만들지 않는다. 사용자 품질 검토는 코어 준비 뒤 마지막 별도 단계이며 GUI 실행이 선행 조건이 아니다.

남은 일은 기존 CLI/SDK 계약에 case별 provenance·안전한 raw/output·오류/취소·관측 None을 연결하는 저비용 작업부터다. 실제 R4 worker/controls 수집, off/on 비간섭·SDK admission/strictjoin, workload 길이/캐시/동시성 반복, 동일질문 paired precision/품질·메모리·시간 평가는 중앙 조율 이후의 **미실행 후속 항목**이다. 이번 스킬 적용으로 실행 예약하거나 source freeze·기존 artifact를 변경하지 않았다.

읽은 프로젝트 근거: `.spec/sots/verification/ai/generation-gemma270-startup-lifecycle-actual.md`, `generation-gemma270-runtime-controls-r4.md`, 이 디렉터리의 `text-r3-source-mapping.md`·`model-quality-final-review-readiness.md`, package659b3cb6…0197 원문 및 R3 run/model-inspection.json. 상세 pin/관측은 각 정본과 보존 원문을 따른다.

## 후속 원문 연결 감사: 실제9의 정밀도·시간·메모리

같은날 goal continuation에서 현재 package와 각 smoke model-inspection, `generation-gguf-header-actual.md` 및 세 smoke 정본을 추가로 읽었다. 새 실행이나 모델 파일 전체 재hash는 하지 않았다. 앞 절의 unknown 원칙을 유지하면서 **이미 저장돼 있던 관측**을 다음처럼 구체화한다. 이 값들은 R3 startup lifecycle이나 R4 build의 값이 아니다.

| 실제9의 arm | 고정 HF revision·파일 SHA256 | bytes | 기존 헤더의 저장 tensor type 수 |
|---|---|---:|---|
| Gemma270 | c90975dbd40c0c7b275fefaae758c3415c906238 / b1baabd6b729e4041822220d3e648e00d99cac5df86b10dffb77bcccf0688e39 |253115424|F32 109,Q4_K 27,Q5_0 81,Q6_K 9,Q8_0 10|
| Llama1B |067b946cf014b7c697f3654f621d577a3e3afd1c / 6f85a640a97cf2bf5b8e764087b1e83da0fdb51d7c9fab7d0fece9385611df83|807694464|F32 34,Q4_K 96,Q6_K 17|
| Qwen0.6 |50968a4468ef4233ed78cd7c3de230dd1d61a56b / ac2d97712095a558e31573f62f466a3f9d93990898b0ec79d7c974c1780d524a|396705472|F32 113,Q4_K 168,Q6_K 29|

위 표의 tensor 수는 기존 Rust header 검사 결과이며 runtime 연산/KV dtype을 뜻하지 않는다. 특히 Gemma embedding은 Q8_0이다. Q4_K_M 파일명만으로 모든 weight를4bit로 표시하지 않는다. 기존 inspector에는 모두 conversion_lineage_complete=false와 tokenizer_template_parity_verified=false가 있고, inspector의 neural_runs0/runtime_executed=false는 **inspector 자체가 추론하지 않았다는 뜻**이다. 이를 별도 smoke의 실제3요청이 없었다는 뜻으로 읽지 않는다.

실제9 Gemma의 engine은 e862c74e…4be229/runtime-cap-r1이고 source manifest는66b1657a…c18이다. 초기 실패한 `text-structured-gemma270-smoke-r1`의59ee engine 검사와 구별하여 `text-structured-gemma270-smoke-cap-r1` inspection을 연결했다. R3 startup의b7ec engine이나 R4 ec3d engine으로 소급 바꾸지 않는다. Llama/Qwen smoke는59ee8ff0…38a9d73 engine을 사용했다. package의 engine_revision은 엔진 EXE SHA identity이며 HF model revision과 다른 필드다.

| arm/각3task 1회 | HTTP/strict 구조·알려진 내용 실패 | batch seconds | peak private B / working B |
|---|---|---:|---:|
| Gemma270 |HTTP3/3,strict2/3. correction length/256tokens·반복/잘림,카드 근거 과대·highlight 원문 없는 서사|36.0653998|1231151104 /1451917312|
| Llama1B |HTTP3/3,strict2/3. correction length/256tokens·반복/잘림,카드 문장과cue ID 불일치|27.29919|2122715136 /2767605760|
| Qwen0.6 |HTTP3/3,strict3/3. correction의 draft 보존≠음성 오인식 교정,카드 문구 반복·부자연스러움,highlight 전체7cue|50.3019265|1270865920 /1578708992|

이 배치는 모두 당시 저장된 exit0/timeout=false이며 load/소유 tree·행별 저장 등의 범위와100ms 표본 메모리다. request_elapsed는 각 HTTP call 구간이고 TTFT/prefill/decode 분해가 아니다. 동일 batch 수치를 세 item에 복사해 독립 메모리/시간 표본3회로 세지 않는다. 현재 재실행·반복분산·다른 호스트 적격성이 없으므로 위 시간의 정렬은 모델 속도 순위가 아니다. strict 총7/9도 의미 품질 accuracy가 아니다.

Gemma/Llama correction은 기존 정본에 실제 finish_reason=length와256token 관측이 있으므로 이 두 case에는 잘림을 명시할 근거가 있다. 다른 case에 finish_reason/token관측이 없으면 원인을 추측하지 않는 앞 절 원칙은 유지한다. 사람이 수정해 완결된 proposal은 이 원래 length 실패의 분모를 제거하지 않는다.

라이선스는 기존 header/acquisition 정본에서 Gemma·Llama HF metadata가 각각 gemma·llama3.2라고 확인한 범위다. 공개 변환본 gated=false는 원본 동의/배포 승인·완전 계보를 뜻하지 않는다. 이 감사에서 세 모델의 고정 LICENSE/NOTICE 전체 및 tokenizer/변환 recipe를 새로 읽은 것은 아니므로 출시 권리 완료로 표시하지 않는다. Qwen을 다른 원본 모델의 Apache 표기만으로 승인하지 않는다. 기존 `provenance-licenses.md`의 MobileSAM/WhisperQ8/OCR/YAMNet/segmentation/WeSpeaker별 원본·변환·사전/고지 잔여도 계속 별도 유지한다.

따라서 다음 저비용 계약 인계에는 model artifact pin·stored tensor precision·runtime observed precision·license evidence 상태를 각각 담아야 한다. current model registry가 모든23역할의 license/variant를 완결했다는 가정을 하지 않는다. 전체 ASR/정렬·화자·OPED·vision/OCR·asset·retrieval 범위는 현재 model-status/coverage 정본의 실행/품질/권리/동등성 차이를 유지하며 이3arm 감사로 대체하지 않는다.

## 후속 로컬 license evidence identity 확인

새 다운로드 없이 보존 acquisition 자료를 실제 읽고 native SHA256을 확인했다. `.spec/evidence/ai/model-spikes-2026-10-18/acquisitions/`의 아래 두 디렉터리다. runtime acquisition에 model-card.txt가 없더라도 보존 evidence 사본이 있으므로 원문이 없다고 단정하지 않는다. 이번에 모델 가중치 전체나 Git blob을 다시 검증한 것은 아니며 **카드/metadata bytes가 기존 result 선언과 같은지**를 확인했다.

| 보존 디렉터리 | metadata.json 실제 SHA256 | model-card.txt 실제 SHA256 | 읽은 선언 |
|---|---|---|---|
| generation-gemma270-acquisition-r2 |7f1ab38e41a47411d5a49af32b0a02bdf1f5f7e00a7cec9d0d8c02568a5e70a8|ba1837a8087f9298adf7835437f9eb3cfc8bd270e5ee904b5f42cf6f819341a3|base_model google/gemma-3-270m-it,license gemma|
| generation-llama1b-acquisition-r1 |79e57290ba32398a2552567ff1f90365f4c4a4baf9721dc474962f0d134cd561|a10d25e26b43c5a697ed311d391e903586104341fe5bfce31ef2c77574ae887f|base_model meta-llama/Llama-3.2-1B-Instruct,license llama3.2,extra_gated_prompt 내 Community License/AUP 문구 존재|

이 네 hash는 기존 acquisition result의 metadata_sha256/card.sha256 선언과 일치했다. 당시 result의 card_git_blob_verified=true·원본접근/수락UI변경없음·conversion_lineage_complete=false를 별도 관측으로 보존한다. 카드에 약관 문구가 있다는 사실을 LICENSE.txt 원본의 별도 byte identity, 원본 모델 수락/계정 상태, 배포 범위 검토 완료로 바꾸지 않는다. 이 문서는 법적 적합성 판단이 아니라 근거 identity의 제한된 확인이다.

Qwen0.6은 기존 `runtime/ai-benchmark/q4-model.json`에서 revision50968a44…a56b·가중치SHAac2d9771…d524a·396705472B를 확인했다. 확인한 고정 hub snapshot의 파일목록은 GGUF1개였고 이번 범위에서 고정 README/LICENSE/NOTICE 사본은 찾지 못했다. 이는 전체저장소·다른evidence에문서가없다는판정이아니며같은계열다른모델의Apache표기로채우지 않는다. 추가취득은현재보류그대로다.

registry/CLI 인계 시 license label·evidence kind(card metadata/card embedded terms/original license)·고정 repository/revision·원문 SHA/bytes·읽은 범위와 lineage 상태를 분리해야 한다. 준비 상태의 missing/unchecked를 legal_denied로 자동 매핑하거나 임의 permissive license로 바꾸지 않는다. 모델 install/기존 로컬 artifact 검증·사용 조건·제품 배포 결정은 별도 정책이며 이번 감사가 새 설치나 배포 승인을 부여하지 않는다.
