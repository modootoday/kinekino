---
id: sot:kinekino/ai-executor-jobs
title: AI executor의 현행 jobs 접합 경계
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# AI executor의 현행 jobs 접합 경계

## 2026-10-21 VAD 인수 뒤 재대조

### P0 실제 계약 인수와 SDK identity 후보의 접합 대응

후속 [media-vad P0 실제 검증](../../../../verification/ai/media-vad-p0-contract-2026-10-21.md)에서 r4 source424B04C6의 metadata/fmt/all-target strict와 실제 고유fixture11이 통과했다. 이는 순수 candidate/partial 계약 인수다. jobs 등록·실ORT·원 process cleanup·제품 CLI·사람 품질은 별도다.

SDK source5 `298EE76BD9592E68EC4F9AB0775A4092DAF8A2A3C8446F027F2C87D4C5604226`의 execution_identity.rs/worker.rs/ports.rs를 읽었다. 최초에는 별도 overlay의 미컴파일 후보였으나, 후속 실제 package31·strict/fmt·기본 CLI 소비자 compile 인수 뒤 root `1cec249344e5678cd62c1fbcd7e4e69796e201a4`에 채택됐다. [SDK 실제 검증](../../../../verification/application/job-attempt-identity-p0-2026-10-21.md)과 원closeout을 읽었으며 새identity5는31에 포함되고 외부worker/AI실행0이다. `ExecutionControl::execution_identity() -> Option<&JobAttemptIdentityV1>` 기본None, 원 worker는 start CAS 반환record에서 private 생성한 Some을 보유한다. getter는 job_id/attempt/input_sha256/source_blob_sha256/submission_request_sha256다. 마지막은 Option<&str>이며 원 API requestID는 만들지 않는다. 이 타입/생성자를 VAD에서 복제하지 않는다.

| 원 identity/입력 | VAD에서 대응할 바이트·값 | 조용히 대체하면 안 되는 값 |
|---|---|---|
| identity.input_sha256 | 원 PreparedJobInput::sha256(operation,parameters,source,timeout) | WAV hash·모델 hash·idempotency key가 아님 |
| identity.source_blob_sha256 | 원 SourceBlobRef의 UTF8 descriptor bytes SHA | descriptor가 가리키는 WAV/PCM의 SHA와 다름 |
| descriptor의 WAV/PCM pin | P0 reader가 실제 WAV 전체 및 decoded PCM bytes를 각각 검증 | file bytes만 같은 metadata 또는 selected PCM만의 hash로 전체 PCM을 대체하지 않음 |
| descriptor의 range/profile | P0 VadInput 전체sample·start/end와 admitted profile의 불변spec | partial에서 range/profile을 잃거나 expected runtime을 actual 관측으로 복사하지 않음 |
| submission_request_sha256 | 원 normalized SubmitJob fingerprint의 Some/None | None은 legacy absence. control None은 전체 binding 부재이며 서로 다른 상태 |

미래 AI executor는 launch 전에 identity 존재→원 input fingerprint/sourceblob SHA 대응→verified descriptor bytes decode→model/input/profile admission을 검사한다. mismatch면 외부 worker를 시작하지 않는다. `JobFiles::read_utf8`는 기존 source byte수/SHA를 확인하는 실제 storage 구현이고, 다른 SourceBlobStore 구현도 같은 신뢰 경계에서 대조돼야 한다. worker의 echo identity는 parent 원 binding과 대조할 데이터일 뿐 authority가 아니다. 원 core publication의 attempt/CAS 검사는 그대로 필요하다.

fixture 후보는 missing control identity, descriptor bytes와 sourceblob SHA 불일치, 다른 preparedinput/timeout, WAV/PCM hash 교환, optional submission absence, partial nonzero range/profile 보존, stale retry의 old identity/result 거부다. 이 접합 fixture는 아직 작성/실행0이며 P0 pure11을 중복 집계하지 않는다. SDK readonly getter 인수와 별도 배정 뒤 실제 adapter에서 구현한다.

SDK가 추가 확인한 stop_owned 최종 unpublished ACK sweep도 P1 owner 설계 범위다. run_queued 한곳만 바꾸거나 readonly identity를 도입해도 retained cleanup/host lease/retry/진단 지속성은 해결되지 않는다. 현재 P0 binding 자료를 cleanup proof로 승격하지 않는다. 정확 후보 인계는 `runtime/coordination/job-attempt-identity-p0-r1/HANDOFF.md`와 `runtime/coordination/ai-jobs-execution-contract-r1/proposal.md`다.

현재 중앙 배정은 source-only다. [VAD 검증 정본](../../../../verification/ai/cached-vad-resource-adapter-2026-10-21.md)의 private normal1/cancel1은 통과했으나 제품 AI operation 등록·품질 승인은 아니다. 공통 owned-process resource4만 root `e93a0b7b778e2b524d607acffd5737aa27a04305`에 채택됐고 private tool9는 공개되지 않았다. 추가 Cargo·추론·다운로드·worker build는 수행하지 않았다.

### 현재 소스에서 확인한 식별 경계

2026-10-21 `Get-Content`로 media-jobs의 domain/{identity,state}, application/{ports,worker,mutations,service,session,executor_registry}, infrastructure/subtitle_executor와 root infrastructure/jobs, domain/model_review/package를 읽었다. 아래는 실행 시험 결과가 아닌 소스 관측이다.

| 식별 | 현재 관측 | 연결 시 주의 |
|---|---|---|
| API request / submission | service는 정규화된 SubmitJob 직렬화 SHA와 idempotency key SHA를 별도로 저장한다. 같은 key·다른 request는 RevisionConflict다 | request ID를 job ID 또는 input SHA로 대체하지 않는다. 모델의 mutable 별칭을 normalize 때 조용히 새 revision으로 바꾸면 재전송이 달라질 수 있으므로 명시 불변 profile pin이 필요하다 |
| job / attempt / revision | worker는 job ID와 attempt를 받고 read_attempt/change_attempt가 stale attempt와 revision CAS를 검사한다 | retry는 기존 정책의 명시 attempt 증가다. 새 모델·입력 선택은 기존 immutable input을 바꾸는 retry로 구현하지 않는다 |
| PreparedJobInput | operation·parameters·source descriptor·timeout이 input SHA에 포함된다 | descriptor SHA와 WAV 자체 SHA를 구별한다. 8MiB UTF8 blob은 바이너리 저장소가 아니다 |
| execute 문맥 | JobExecutor::execute에는 input, blobs, ExecutionControl만 전달된다. control은 deadline/cancel/publication을 제공하지만 job ID·attempt getter는 없다 | private spike run ID를 제품 attempt로 위조하거나 submit parameters에 미래 job ID를 넣지 않는다. 아래 context 확장은 제안이며 현재 API가 아니다 |
| model review | ModelCondition은 model/revision/precision/engine/engine_revision/input_conditions를 가진 검토 기록이다 | 실행 admission·파일 license closure·등록된 runtime profile과 동일하다고 간주하지 않는다 |

### SDK 협의가 필요한 최소 확장 제안 — 미구현

기존 worker가 이미 소유한 job ID·attempt·input SHA의 읽기 전용 execution identity를 adapter에 전달하는 최소 공통 계약을 SDK 담당이 결정한다. 기존 ExecutionControl의 기본 optional accessor 또는 별도 versioned execution context를 비교하되, 다른 executor를 무의미하게 파괴하는 필수 trait 변경은 피한다. None인 executor 경로는 유지하더라도 AI adapter는 identity 부재를 명시 거부한다. submission request SHA 전달도 필요하면 같은 원 저장 record에서 가져오며 adapter가 역산하거나 임의 생성하지 않는다. 운영 ID·OS PID·model profile·원 API request ID는 별도 필드다. AI용 새 scheduler/state machine은 만들지 않는다.

AI adapter는 원 attempt의 identity, 고정 profile(모델 repo/revision·weight SHA/bytes·precision·engine/runtime hash·license/admission 상태), binary input SHA/bytes/range, 실행 옵션을 immutable manifest로 결합한다. 현재 Silero 근거의 HF revision은 `e71cae966052b992a7eca6b17738916ce0eca4ec`, FP32 CPU·실측 intra2/inter1이며 weight/input/runtime의 정확 pins는 위 VAD 정본을 참조한다. arbitrary path/argv/URL은 DTO에서 실행하지 않는다. 공개 adapter 추출은 private 도구 부분 복사가 아니라 source/dependency/license closure를 확인한 별도 인수다.

### 출력·취소와 원 caller의 책임

1. 원 supervisor를 유지한 채 실행 중 control cancellation/deadline을 확인한다. 취소 시 원 child/family의 stop·wait·EOF·cleanup 결과를 보존하고 반환한다. resource probe query는 cleanup이 아니다. 취소 관측만으로 자원을 반환했다고 표시하지 않는다.
2. worker의 cancelled1frame은 원 취소 근거·부분 결과다. 성공 VAD artifact로 publish하거나 Completed의 output에 넣지 않는다. 현재 worker는 Cancelling이면 executor 오류 뒤 Cancelled를 기록하므로 cleanup 실패가 ApiError만으로 사라질 수 있다. **cleanup 불명/실패의 진단 보존 계약을 먼저 합의하기 전 제품 취소를 지원 표시하지 않는다.** 이 항목은 기존 jobs 결함 판정이나 이번 수정이 아니다.
3. 성공 결과는 private staging에서 shape/range/source/model pin을 검증하고 원 child 종료 근거를 확보한 뒤 공통 reserve_publication으로 취소와 경합한다. 취소 승리 시 staging을 정리하고 완료 출력은 없다. publication 승리 뒤 no-overwrite 게시→begin_verification→실제 파일 hash/bytes 재검증→VerifiedJobOutput 반환을 따른다. publication 뒤 실패는 recovery_required이며 자동 retry·덮어쓰기하지 않는다.
4. JobRecord::complete는 output 형태/state를 확인하지만 파일 내용 검증은 executor 책임이다. receipt 32KiB 밖의 raw/peak/closeout은 immutable diagnostic artifact로 분리한다. failed/cancelled record의 output 필드를 채우지 않고 bounded failure details의 안전한 진단 ref 또는 별도 attempt evidence 저장을 SDK 담당과 합의한다.
5. read_attempt 거부 뒤 늦은 결과는 현재 attempt를 완료시키지 못한다. 저장 CAS 외에도 no-overwrite artifact 경로와 원 attempt identity의 결합을 Rust fixture로 검증해야 한다. host 재시작은 기존 exclusive owner/Interrupted 경로를 따르며 orphan 결과를 자동 성공 채택하지 않는다.

### 인계 순서와 아직 없는 근거

SDK가 identity 전달·cleanup 진단 보존·binary artifact reader·profile admission의 소유 경계를 확정한 후 최소 Rust adapter source를 배정한다. 첫 검증은 모델 실행 없는 고정 fixture로 stale attempt, 같은 key 다른 profile, pin 변경, 취소 선승/게시 선승, 부분 결과 거부, cleanup 실패 보존, 게시 후 recovery, 실제 출력 hash 대조를 확인한다. fmt/test/strict 및 실제 CLI normal/cancel은 중앙 슬롯을 별도 배정받아 수행한다. 현재 이 제품 AI fixture/등록/CLI 인수는 0이며 private VAD의 고유7·normal1·cancel1을 중복 집계하지 않는다.

문서 방법에는 modoo.today `modootoday/spec-authoring/plan` 및 `modootoday/spec-authoring/plan-authoring` 본문을 읽고 적용했다. 기존 접합 문서를 유지하고 소스 관측·미구현 제안·미검증 인수를 구별했다. backend job 검색의 frontend-first-journey/portable-export/fan-out-script-design은 범위가 맞지 않아 채택하지 않았다. 로컬 architecture의 trade-off-analysis와 api-patterns/response를 적용해 기존 envelope/오류/CAS를 유지하고 context 확장은 승인된 계약이 아닌 협의안으로 남겼다.

2026-10-19 goal 후속 읽기 감사다. 현재 media-jobs source·root API dispatcher·기존 jobs 검증 정본을 읽었다. 새 제품 코드·컴파일·네트워크·추론은 없다. 이전 모델 포트 인계의 ‘jobs 실행 포트 부재’는 **그 문서 최초 조사 시점의 상태**이며 현재 상태로 반복하지 않는다.

## 현재 구현과 재사용

`src/application/jobs.rs`는 media-jobs의 application/domain을 re-export한다. 현재 JobExecutor는 operations/supports/normalize_parameters/timeout limit/prepare/execute를 제공하고 ExecutorRegistry는 operation의 명시적·유일 등록을 검증한다. 실행 가능한 operation 목록을 capability에서 읽어야 하며 jobs 지원만으로 models.install/transcribe/diarize 등 모든 모델 작업을 지원한다고 하지 않는다. 모델 registry·variant/license admission은 별도 잔여다.

ExecutionControl은 cancellation_requested/deadline/reserve_publication/begin_verification을 제공한다. worker는 durable job ID+attempt를 확인하고 queued→running으로 진행하며, 취소/출력 publication은 같은 저장소 CAS gate를 사용한다. ExecutionSession의 배타적 owner lease와 시작 시 Interrupted 복구·무자동 retry를 재사용한다. 별도 AI 전용 job 상태 머신·generic engine supervisor를 제품 포트 밖에 새로 만들지 않는다.

기존 `.spec/sots/verification/application/jobs-2026-10-19.md`에는 고정 실제 player-cli의 자막 변환/취소 전후 경합·watch 재조회·idempotency·retry·소유 host 강제 종료 후 Interrupted·EOF 근거가 있다. 이는 **기존 담당 검증을 읽은 것**이며 이번 재실행이 아니다. SRT executor 취소 성공은 외부 모델 child 종료/HTTP 취소/OS 강제 I/O interrupt의 통과 근거가 아니다. 현 browser provider jobs는 별도 DOM checkpoint와 actual NeedsInput 근거를 가지므로 AI 성공으로 전이하지 않는다.

## AI adapter에서 정확히 이어야 할 계약

| 현행 타입/규칙 | AI 연결 시 적용 | 남은 제한 |
|---|---|---|
| SubmitJob(operation,parameters,expected_source_sha256,execution_timeout_ms) | model/profile·입력 artifact/revision·task별 bounded options를 normalize/prepare에서 고정 | parameters Value는 모든 모델 policy의 자동검증이 아님. task DTO의 deny_unknown_fields·정밀도/variant/license 확인 필요 |
| PreparedJobInput(operation,parameters,SourceBlobRef,timeout) | immutable normalized 요청·source descriptor를 같은 input SHA에 결합 | source는1..8MiB UTF8 blob,parameters64KiB 제한. 전체 WAV/영상/텐서·base64를 넣는 계약이 아님 |
| SourceBlobStore put_utf8/read_utf8 | 텍스트·안전한 input manifest를 검증/보존 | 바이너리는 별도 bounded artifact/file adapter가 필요. descriptor SHA와 실제 binary SHA를 혼동하지 않음 |
| VerifiedJobOutput(path,sha256,bytes,receipt) | 검증·publication 후 actual result artifact reference와 bounded receipt 반환 | validate는 형태 검증이며 engine 의미 품질·파일 진정성 전체를 증명하지 않음. receipt32KiB이므로 raw trace/큰응답은 별도 artifact |
| ExecutionControl deadline/cancel | 모델 load/전처리/HTTP/생성·검증 단계마다 확인,남은deadline을외부adapter에전달 | deadline은 worker start 기준이며 queue 대기/RequestContext 호출 timeout과 구별. 내부interrupt없는Qwen engine은 owned process 종료/wait 계약 필요 |
| reserve_publication→write→begin_verification | 취소 경합을 공통 durable gate로 결정하고 기존 nooverwrite writer 재사용 | reserve 이후 실패/host종료는 recovery_required 경로. 새 자동 retry나 기존 output 덮어쓰기 금지 |

실제 model weights/tokenizer/processor·engine pins는 정규화 시 고정한 값을 execute 직전 다시 검증하며 arbitrary executable/URL/argv를 외부 payload에서 실행하지 않는다. model/source가 바뀌면 명시 오류로 남기고 새 profile로 조용히 fallback하지 않는다. actual precision·cache/concurrency와 intended options는 각각 provenance에 기록한다. None을false/0으로 바꾸지 않는다.

## CLI 호출 문맥의 주의점

현 dispatcher는 jobs.submit에 idempotency_key를 요구하고, expected_revision은 jobs.retry에만 허용한다. cancel은 expected_attempt를 쓴다. 앞선 모델 인계의 일반 RequestContext 재사용 제안을 그대로 복제해 submit/cancel에 expected_revision을 넣으면 현재 계약과 충돌한다. 모델 source revision은 typed parameters/source descriptor와 expected_source_sha256 admission으로 묶고, retry는 현재 positive expected_revision 규칙을 따른다.

watch는 durable event page이지 blocking stream이나 engine token stream이 아니다. token streaming이 필요하면 별도 bounded cursor/event 계약을 설계하고 raw model text를 job event/기본 RPC 응답에 무제한 넣지 않는다. oneshot readonly job root와 explicit execution host의 capability 차이를 유지한다. job submit의 접수는 실제 completion이 아니다.

취소 요청은 Cancelling/requested와 worker의 Cancelled ack를 구분한다. engine child를 소유한 adapter는 cancel/timeout 시 그 child만 stop/wait하고 원문 child/outer exit·cleanup 오류를 보존한 후 결과를 반환해야 한다. process 종료 확인 없이 ‘cancelled’를 근거 없는 성공으로 반환하지 않는다. reserve 이후 publication_in_progress 경합은 기존 정책을 따르며 사용자가 취소했다고 이미 검증·게시된 결과를 삭제하지 않는다. engine 실패·모델 typed mismatch·내용 품질은 별개다. 원본 응답 artifact가 존재하면 failure/needs_input에서도 안전한 diagnostic reference를 보존할 방법을 담당자가 합의해야 하며 completed-only output 필드를 임의 채우지 않는다.

## 저비용 다음 인계와 미실행 gate

SDK 담당에게 현재 JobExecutor/ExecutionControl/DTO를 그대로 재사용하는 최소 task adapter를 제안한다. 최초에는 모델 실행 없는 기존 response artifact decode·source/model pin 검증·typed 오류/취소·결과 publication fixture로 접합을 확인할 수 있다. 제품에서 실제 채택할 operation명/schema·artifact reader·license admission·diagnostic storage는 담당/PM이 확정하며 이 문서로 미등록 명령을 지원 표시하지 않는다.

그 이후 외부 engine의 실제 요청·응답/파일·deadline·owned cleanup·late result 거부·재시작 회귀는 제품 CLI 준비와 중앙 자원 배정 이후에 수행한다. R4 worker/SDK admission·strictjoin·비간섭과 전체ASR/화자/OPED/vision/assets/retrieval/text 품질 잔여는 유지한다. 새로운 AI 실행이나 worker build·GUI 인수·대형 통합 검사를 이 읽기 감사 때문에 시작하지 않는다.

판정에 적용한 modoo.today 스킬은 앞서 실제 읽은 inference-benchmark-workload-protocol·quantization-and-vision-input-parity-check·llm-output-eval-suite다. 구체 source/모델/조건별 immutable input·시간 경계·실패 분모·raw/typed/품질 구분에 적용했으며 정확한 이름·출처·reference SHA는 `modoo-existing-evidence-protocol-review.md`를 따른다.
