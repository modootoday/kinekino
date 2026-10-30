---
id: evidence:kinekino/20261029113624
title: Gemma270 R2 실제 진단과 미등록 초기 sequence
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Gemma270 R2 실제 진단과 미등록 초기 sequence

2026-10-19. 고정 engine `97a365ae4349d58fc56831ba94fe314ca47ba4f5252a20e87fd8e402d607ccc6`와 선보존 worker `d5b0daecb63131eb7d4a6ede8a8193feea87998360b01b3618a61291bd490195`로 같은 r3 입력3개를 실제 실행했다. [소스·엔진·워커 gate와 자원 거부 이력](generation-gemma270-internal-trace-implementation.md)을 따른다. 실행 label은 `gemma270-boundary-internal-diagnostic-r3`이며 소스 R2와 입력 fixture r3를 구별한다. 앞선 label 끝 r2는 CPU admission 거부로 모델을 실행하지 않았다.

## 관측

count3회는2047/2048/2049를 확인했다. generation2047은HTTP200/usage2047+1=2048,2048·2049는HTTP400/usage없음이었다. 정상 요청 이전부터 진단이 기록됐으며 rawschema2의 최종 상태는 invalid2/dropped0/completefalse, row_batches2/mutation_revision49/flush_attempts7/flush_failures0/first_storage_failurenull이었다.

| 순서 | 실제 journal 관측 |
| --- | --- |
| ordinal1 rows_prepared | origin1,sequence_ids[0],registered[false],is_prompttrue,request/case=null |
| ordinal2 invalid | reason=row_arity_or_registration,같은 origin1/sequence0/미등록/프롬프트 phase |
| ordinal3 invalid | reason=unregistered_forward,origin1/sequence0/input_shape[1,3],request/case=null |
| ordinal4 첫 request | internal request2/case tokens-2047 |
| ordinal8 rows_prepared | origin2/sequence1/registeredtrue/is_prompttrue |
| ordinal33 sampling_final | sequence1/raw2048/prompt2047/generated1/sampled236743/ModelLength value2048 |

이는 두 invalid의 실제 발생 지점이 정상 요청 이전의 미등록 sequence0 경로였음을 보여 준다. 저장 실패가 이번 invalid 원인이라는 근거는 없고 flushfailure0이다. sequence0을 준비한 호출자의 목적·실제 lifecycle은 별도 source/phase 조사 전에는 단정하지 않는다. startup/warmup이라고 추정해 이벤트를 무시하거나 invalid를0으로 바꾸지 않는다. 미등록 경로는 성공한 forward_enter/return의 정상 key가 없으므로 저장된 정상 forward1을 전체 실제 호출1로 확대하지 않는다.

후속 [소스 연결 조사](../../player/infrastructure/inference/research-followups/gemma270-unregistered-sequence-zero.md)는 `MistralRs::new`의 기존 startup dummy 요청을 확인했다. 이 경로는 Completion 입력hello/id0/max_len1/seedNone을 보내고 응답 채널을 drain하며, Chat만 등록하는 현재 recorder에는 등록되지 않는다. sourcechain과 이번 요청 전sequence0/그다음cohortsequence1이 일치한다. 다만 실제dummy의 IDs/return/response가 이번journal에서 완결 수집된 것은 아니므로 이source설명을 runtime lifecycle marker로 대체하지 않는다. 다음 수정은 원래dummy를 유지하고 trusted 생성지점에서 별도 lifecycle과 request 소유권을 연결해 전후연산·응답·종료를 관측하는 것이다. 숫자0/shape3/문자열hello 패턴 면제는 사용하지 않는다.

worker는exit0/timed_outfalse,67.5595603초,100ms표본 peakprivate1,330,266,112B/working1,563,222,016B였다. 실행 직전CPU47.75390625%/free31,794,167,808B로correctness admission을 통과했다. 성능비교eligiblefalse다. 소유 서버는Job/worker수명으로 해제한 뒤 별도CIM에서PID116764가 없음을 관측했다. 이PID 부재를다른프로세스종료로확대하지않는다. 품질max256/3task나역할23묶음수는변경하지않았다.

## 보존과 판정

- [실제 manifest](../../../evidence/ai/model-spikes-2026-10-18/results/gemma270-boundary-internal-diagnostic-r3/manifest.json) SHA256 `604b29db538121d383db4543c6da92cbbc42929d6a28bce4e84ab86a19bdd466`.
- [raw journal](../../../evidence/ai/model-spikes-2026-10-18/results/gemma270-boundary-internal-diagnostic-r3/internal-trace/journal.json) SHA256 `46db1ce4ff620b9abdd7e18b67e319124a94cbabfc2ed4df73944dc02b35fbde`.
- [측정](../../../evidence/ai/model-spikes-2026-10-18/results/gemma270-boundary-internal-diagnostic-r3.json) SHA256 `3bc56c4f8d39d68276e06e172756ebc52ba35d9fb17e1927f9e2245c6a6db0a2`.

고정 [예비 검사](generation-boundary-observation-contract.md) EXE로 이rawschema2를 실제 읽었다. JSON decoder는수락했고accounting은exit1/incomplete raw recorder accounting으로거부했다. 응답은 [원문](../../../evidence/ai/boundary-observation/preflight-r1/raw-journal-r2-actual-rejection.stdout)에보존했다. 이는필드별DTO변환이나공통계약통과가아니다. shared_observation_contract_verified/noninterference_verified/full_boundary_gate_passed는모두false를유지한다.

다음은실제미등록경로의source와phase를연결하고전체초기화/요청forward accounting을보존하는최소수정·회귀·새source/EXE/worker선보존·같은3case재실행이다. [매핑 계약](../../player/infrastructure/inference/research-followups/gemma270-raw-observation-mapping.md)의sidecar·외부HTTP/workercontrols·trustedpin검증과terminalwitness/off-on도남아있다. [R1 실패](generation-gemma270-internal-trace-actual.md)를소급통과시키지않는다.
