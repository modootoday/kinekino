---
id: sot:kinekino/cli-2
title: 모델·엔진 요청과 파일 입출력의 CLI 인계 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 모델·엔진 요청과 파일 입출력의 CLI 인계 계약

2026-10-19 사용자 지시를 PM 경로로 인계받은 정적 계약 검토다. 모든 제품 기능은 공통 SDK/application/domain과 실제 CLI에서 먼저 검증한다. GUI 자동화·입력·스크린샷·모델 결과 재생은 모델 기능 완료 gate가 아니다. 신규 추론·다운로드·worker 빌드는 코어 준비와 중앙 자원 배정 전까지 보류한다. 기존 R4 vendor bundle은 자연terminal0으로 완료해 EXE와 원문을 보존했다.

이 문서의 DTO/포트 요구는 SDK에 전달하는 설계 조건이다. 새 제품 schema·모델 설치·inference executor·cancel 구현이 존재한다고 주장하지 않는다. [공통 포트 경계](inference-package-sdk-port-boundary.md), [기존9 결과의 source·오류 계약](text-r3-import-error-contract.md), [도구 승격 범위](diarization-metrics-extraction-review.md)와 함께 사용한다.

후속 현행 source 감사에서 media-jobs의 JobExecutor/ExecutionControl·실행 host·CAS publication/cancel·Interrupted 복구가 구현된 것을 확인했다. 아래의 초기 jobs 준비 제안은 그 이전 조사 맥락이며 현재 jobs 포트 부재로 해석하지 않는다. AI adapter는 기존 계약을 재사용한다. 특히 submit/cancel에 expected_revision을 임의 추가하지 않고 retry의 revision과 cancel의 expected_attempt를 구분하며 UTF8 source blob을 binary 저장소로 간주하지 않는다. 현재 접합 세부는 [현행 jobs와 AI executor](ai-executor-current-jobs-integration.md)를 따른다.

## 현재 소스로 확인한 범위

| 경로 | 실제 존재 | 제품 공통 CLI의 남은 일 |
|---|---|---|
| Qwen ASR spike | TranscribeRequest와 AsrPort, 모델/tokenizer 검증, strict WAV 입력·Transcript/raw output | application이 infrastructure를 직접 사용하며 request에 로컬 path가 있음. 제품 job/cancel/deadline·model registry·안정 artifact 입력으로 연결되지 않음 |
| Gemma R3 | 고정 engine/worker·3경계 실제 HTTP 요청·raw trace·owned stop/outer exit 근거 | 연구 cohort이며 일반 제품 text-generation 지원이나 품질 승인이 아님 |
| Gemma R4 | controls/commit/ACK source와 worker source, vendor check/test/Clippy/CLI build 통과·정확 EXE/원문 archive | worker compile/gate·실제 R4 요청·SDK source/archive-profile admission 미완료. pending worker pin은 실행 거부 |
| 기존9 text results | 원본 package/source와 실제 SDK/CLI import·proposal 경로의 별도 근거 | 결과 import는 새 inference 실행 증거가 아님. typed mismatch는 raw 보존/needs_repair와 명시 edit 경로로 처리 |
| DER/JER | 순수 계산 API 및 별도 reference 도구 | product port 미연결. 원래 protocol/legacy grid·oracle 한계·source pins 유지 |

Qwen 현재 입력은 Japanese/Korean/English의 명시 enum, mono16kHz PCM16 WAV,400..480000samples(25ms..30s), max tokens1..256, threads1..4로 제한된다. 자동 resample/channel conversion은 없다. raw/upstream text를 보존하고 정확한 leading `<asr_text>` 한 번만 분리하며 body/whitespace는 수정하지 않는다. unexpected protocol token은 requires_review이며 evaluation_text는 None이다. word timestamps와 token-limit/EOS 관측은 미지원/None이고, upstream 내부 interrupt API도 없어 전체 owned process deadline이 필요하다. 이를 일반 ASR·streaming·정렬 성공이나 무음 판정으로 확대하지 않는다.

## SDK 요청·응답 요구

요청 envelope는 기존 RequestContext의 request_id/api_version/expected_revision/idempotency_key/timeout_ms를 재사용한다. 작업 payload는 task 종류, source revision과 artifact SHA/bytes/media type, model profile ID와 고정 revision/precision/variant, task별 bounded options를 갖는다. 임의 shell/argv/engine URL·웹 데이터에서 받은 executable path를 공통 요청으로 받지 않는다. local CLI path 입력은 신뢰 경계의 file adapter가 검증·읽고 artifact로 고정하며, SDK의 순수 계약에 PathBuf/OS handle을 퍼뜨리지 않는다.

실행 접수 응답은 job/attempt ID와 revision·현재 상태를 반환하고 completed로 표시하지 않는다. 완료 결과는 immutable result artifact reference, raw request/response artifact reference, typed decode 결과·부분 결과 여부·불확실한 관측을 분리한다. source PTS/cue ID/word time이 원본에 없으면 만들지 않고 None/미지원으로 보존한다. actual token count·EOS/finish reason·pool size가 관측되지 않으면 설정값이나 텍스트 길이로 추정하지 않는다. 사람이 품질을 승인하지 않은 결과에 usable을 자동 기록하지 않는다.

각 engine adapter가 HTTP status·transport error·프로세스 exit·typed schema error를 각각 보존한다. HTTP200은 typed 결과 유효·품질 승인과 별개다. 실제 count request와 generation request의 정확한 bytes/params/model pin을 묶고, 변환/수정은 원문을 덮어쓰지 않는 명시 adapter/proposal로 남긴다. JSON 문자열·literal backslash-n·실제 LF를 추가로 decode하거나 임의 trim하지 않는다.

## 오류와 취소

기존 media-contracts ApiError를 재사용한다. 잘못된 DTO/옵션은 invalid_argument(2), adapter/variant 미지원은 unsupported(3), 추가 입력은 needs_input(3), artifact/model 식별 부재는 not_found(4), stale source/revision은 revision_conflict(5), 권한 실패는 permission_denied(6), 자원 부족은 resource_exhausted(7), source 만료는 source_expired(8), engine 실패는 engine_failed(8), 취소 확정은 cancelled(9), deadline은 timeout(10), 내부 결함은 internal(1)이다. 이 매핑은 제품 player-cli의 exit 의미이며 외부 engine/연구 CLI의 exit를 그대로 재사용하지 않는다. 구체 원인·retryable·remediation은 adapter 증거와 정책에 맞게 확정한다.

취소 요청의 접수와 engine 종료 확정은 다르다. queued 취소, running cancel 요청, 완료와 취소의 경합, 이미 종료된 job의 재요청을 job revision으로 판정한다. stop/wait는 해당 attempt가 소유한 process/job에 한정한다. timeout으로 caller가 반환했어도 engine이 종료되지 않았으면 종료 확인·cleanup 실패를 별도 상태/근거로 남긴다. 내부 취소 API가 없는 engine은 전체 process adapter로 bound하고, 작업 중 강제 checkpoint/resume 가능성을 가정하지 않는다. retry는 새 attempt와 원인 연결을 유지하며 이전 raw와 output을 덮어쓰지 않는다.

## 파일·provenance 요구

input/output는 bounded 크기·strict UTF8/미디어 형식·SHA/bytes·source revision을 검증한다. output은 임시 산출→검증→nooverwrite publication 후에만 completed 결과로 연결한다. path traversal/기존 파일 overwrite·symlink/재연결 경계는 기존 file publication 계약을 재사용한다. 다른 request/attempt의 output이나 늦은 응답을 채택하지 않는다. 재시작 후 상태와 output reference를 다시 읽어 검증하고 단순 파일 존재로 completed를 복원하지 않는다.

provenance는 model repository/revision/license/정밀도/정확 variant·artifact SHA/bytes, tokenizer/processor pin, engine executable SHA/bytes와 source/build manifest, adapter version, exact request와 source artifact, 실제 child/outer exit, runtime controls의 configured/observed/unavailable 구분을 포함한다. env threads4는 실제 OS thread cap이 아니며 HF offline은 OS 네트워크 격리가 아니다. source-hash env 선언도 자체적인 실행 source 인증이 아니다. R4 post-sync ACK는 별도 원문 파일/commit/raw identity를 검증해야 하고 ACK 자체의 전원 장애 durability를 자기 필드로 증명하지 않는다.

## 최소 CLI 수직 인수 묶음

1. model 없는 file conversion/frame 작업으로 공통 jobs 접수·조회·대기·취소·오류·result publication·재시작 정책을 먼저 검증한다. 실제 SDK 구현이 제공하는 명령/schema를 사용하며 미구현 jobs/watch 등을 가정하지 않는다.
2. 기존9 결과 import로 raw→typed mismatch→명시 edit/accept→재시작 readback을 검증한다. 새로운 모델 실행 증거와 구별한다.
3. engine/model 준비 후 고정 입력 하나의 실제 CLI/SDK 요청·응답·raw/output·exit를 묶고 잘못된 artifact/model profile/timeout/cancel/late output을 대조한다. 구체 실제 실행은 중앙 자원 배정 이후다.
4. 모델별 variant·정밀도·license·정확 input/output 정책과 품질 검토를 별도 확정한다. all-feature 지원, 성능 비교 적격, capture 인증, noninterference를 기본 true로 만들지 않는다.

이번 인계에서 새 실행·테스트는 수행하지 않았다. 현재 연구 source에 있는 trait이나 test 이름은 제품 CLI 지원 또는 이번 회귀 통과 근거가 아니다. SDK 담당은 실제 지원/미지원과 명령별 고정 입출력 근거를 기록하고 제품 CLI 기능 검증이 끝나기 전 GUI 단계로 넘어가지 않는다.

## 저장된 실제 출력 import의 provenance 인수 항목

새 모델 실행 없이 저장된 output을 사용하는 다음 SDK acceptance 범위다. **아래 검사를 이번에 실행한 것은 아니다.** task별 변환/후속적용 잔여는 [교정·카드·하이라이트 접합](text-r3-import-error-contract.md), 현행 jobs/CAS는 [AI executor 접합](ai-executor-current-jobs-integration.md)을 따른다.

현 GeneratedResult는task/raw_text만이고 VerifiedJobOutput은path/SHA/bytes/receipt다. 따라서 모델 identity/정밀도/job attempt가 proposal에 자동 보존된다고 가정할 수 없다. 기존 deny_unknown_fields wire를 무단 확장하거나 historical artifact를 다시 쓰지 않고, 담당 SDK가 명시 계약으로 연결할 불변 provenance reference/receipt를 확정해야 한다.

1. **입구 identity**: 실제9 package/각candidate raw asset SHA·bytes와 source request asset SHA·task·model repo/revision·engine SHA·precision 선언·원 source manifest를 읽고 하나라도 불일치하면다른자료로대체하지않는다. historical자료에실제job/attempt가없으면unknown/absent·historicalimport로남긴다. local snapshot revision1을원래project revision이라고하지않고새job ID를과거실행ID로발명하지 않는다.
2. **실제값/선언 구분**: stored tensor의Q4_K_M mixed와runtime weight/KV dtype·seed/thinking 실제적용·cache/concurrency관측은분리한다. 미관측값은None이고기존license evidence 종류/revision/readscope·lineagefalse를유지한다. intendedflag를actual로승격하지 않는다.
3. **create/readback**: 원본raw를그대로수입한뒤create/inspect와fresh CLI에서rawSHA·sourceSHA/revision/pointer·명시provenance reference의동일성을확인한다. 과거schema차이의needs_repair는성공적인보존상태이며모델성공/자동typed수선과다르다. task/model/source를서로바꾼fixture와부족/손상reference를거부하는범위를확정한다.
4. **edit/accept**: 명시edit은draft/revision만변경하고original raw/source·model/engine/profile·historicaljob상태·lineage참조는불변이어야한다. accept는그draft와target_source_revision의CAS를따르고,receipt또는연결된불변reference에서같은provenance를끝까지재조회할수있어야한다. typed검증은의미품질이나자동적용승인이아니다. stale revision·다른attempt/lateoutput은채택하지 않는다.
5. **재시작/프로젝트 capture**: accept후새CLI프로세스/별도SDKadapter·프로젝트저장capture에서source/raw/modelrevision/precision/evidencekind와receipt연결을재확인한다. companion artifact를사용한다면그파일의SHA/bytes까지검증하고누락/손상은정직한오류로반환한다. 경로문자열이나파일존재만으로same provenance를선언하지 않는다. workingdraft를새모델원출력으로기록하지 않는다.
6. **기능과품질 분리**: 세task별원본·기술edit·accepted출력·사용자verdict는각각보존한다. technical fixture의user_decisions_written0/automatic_applicationfalse와품질미승인을유지한다. 자막파일·카드export·highlight range/skip/render는각각별도유스케이스/인수근거이며proposal receipt로완료처리하지 않는다. CLI이검증을위해모델을재실행하거나GUI를열필요는없다.

현재 source를읽어확인한기능과위미실행인수조건을분리한다. 이접합확인에맞춰모델실측/workerbuild보류를해제하지않고,필요한구현/자동검사는SDK소유와중앙검사순서로진행한다.

## 독립 tool companion 검증과 SDK 인계

[도구 README](../../../../../../tools/model-review-bundle/README.md), [application reader](../../../../../../tools/model-review-bundle/src/application/historical_companion.rs), [tool-only domain](../../../../../../tools/model-review-bundle/src/domain/historical_evidence.rs), [bounded file adapter](../../../../../../tools/model-review-bundle/src/infrastructure/historical_files.rs)를 참조한다. 고정659b package·원manifest3개·source/candidate 객체9쌍의 bytes/SHA와 원 request/candidate UTF8·model/revision/engine 연결을 검증한다. 제품 DTO/root Cargo는 변경하지 않았다.

companion은 absent/declared/observed를 구분한다. runtime weight/KV dtype·과거job/attempt/project revision·license acceptance 미관측은 absent이며, mixed stored precision/options/lineagefalse는 declared다. HTTP200은 과거 manifest의 기록으로서 새 관측이 아니다. modoo.today의 기존 3스킬을 조건 고정·선언/실측 구분·원문/실패/품질 분리에 적용했다. 이 draft는 SDK admission·전체 license·실제 정밀도 인증이 아니다.

게시 adapter는 기존 media-file-publication을 재사용한다. RevisionConflict만 동일 bytes 재조회로 처리하고 다른 오류는 실패한다. fmtcheck, 신규6 포함 전체11 test, strict all-target Clippy, export binary build를 중앙 슬롯에서 locked/offline으로 통과했다. 최초 absent 추가필드 허용 실패와 Absent {} 수정 후 통과 로그는 모두 보존했다.

고정 actual9 생성/verify/동일bytes 재게시 exit0이며 companion SHA256은 7a7cd05eafa6294e846c8080c976600ea135aa91e9da315ac9f770e45f624b10이다. 별도 scratch의 baseline은 exit0, 객체 누락·손상·declared→observed 승격·기존변조companion 덮어쓰기·malformed JSON은 각각 CLI exit1로 거부했다. 원본 object/package 목록 해시는 전후 동일했다. [검증 로그](../../../../../evidence/verification/historical-companion-2026-10-19/)와 [실행 이력/남은 목표](../../../../../plans/model-evidence-cli-current-goal.md)를 참조한다.

SDK 담당은 도구 근거를 backlog로 인수했다. 제품 provenance adapter/admission은 미구현이며, 위 6단계 application 인수와 사람 품질 검토는 남아 있다. 현재 중앙 우선순위는 회차→episode DOM→resolve/native다. 새로운 추론/download/worker build는 보류한다.

후속 read_verified/inspect-import tool 경계는 중앙 슬롯에서 fmt·일반12시험·strictClippy·binbuild와 고정CLI/Rust actual9 부정인수1시험을 통과했다. 선택 source/candidate 원문과 companion reference를 함께 반환하며 선택 후 bytes도 재hash한다. 원문/원본 불변·missing item 및 scratch source/candidate 누락·동일크기변조·companion 승격/누락 거부가 실제 근거다. 고정 EXE SHA e00c1726603ef7d0167c6d58deb79e694071044c20a2723de8abe8a504f45e41. 상세 [gate](../../../../../plans/player/historical-import-tool-gate.md). 제품 application/ai_provenance port·MemoryStore 회귀는 여전히 root 미등록/미컴파일이며 이 검사에 포함되지 않는다. 제품 원자 저장과 CLI6단계 인수·품질승인은 별도다.
