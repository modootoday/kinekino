---
id: plan:kinekino/20261029113651
title: VAD profile admission·artifact source 후보 계약
status: active
created: 2026-10-29
updated: 2026-10-29
---
# VAD profile admission·artifact source 후보 계약

status: active. 2026-10-21 중앙 source-only 배정에 따른 [제품 수직기능](cached-vad-product-slice-2026-10-21.md)의 P0 상세다. 구현/컴파일/등록/제품 AI 지원은 아직 없다.

후속 갱신: 중앙 exact10 배정의 독립 overlay P0 후보는 작성했다. `runtime/coordination/media-vad-p0-r1/HANDOFF.md`와 source10 SHA `B79C4C47616BB4432A2C079452D36C0FF6EEA04F88E418354B4AEAB98674D880`를 따른다. 아래8개 projected module은 초기 책임분해이며 실제10경로에서는 artifact/runtime을 profile.rs, admission을 private analyze.rs carrier로 통합했다. 직접fmt0·회귀9작성/실행0·컴파일0·제품등록0이다. 아래 미구현 서명 설명은 초기 제안이며 현재 후보의 실제 API는 핀 source가 정본이다.

## Outcome

검토 기록과 실행 admission을 구별하고, 허용된 고정 모델·runtime·입력만 원 jobs attempt에서 실행하도록 최소 Rust 계약을 만든다. 원 모델·입력·출력 provenance를 보존하며 부분 취소 결과는 성공 결과 타입으로 생성하지 않는다.

## Authority

| 책임 | 소유·경계 |
|---|---|
| profile/input/result 값·실행 전 artifact 대조 | 미디어 담당 신규 media-vad package 후보 |
| 원 job/attempt/request/input SHA·cleanup 진단 지속성 | SDK 담당 공통 jobs 계약, 별도 합의 전 편집0 |
| 실제 모델 사용허가·runtime 배포·공개 source closure | PM 인수. DTO의 license 문자열은 승인 아님 |
| 사람 품질 판단 | 기존 model_review/proposals 경로. 실행 admission과 분리 |

## Evidence

2026-10-21 `Get-Content src/domain/model_review/package.rs`에서 EvidenceAsset(asset_id/SHA/bytes/media_type), ModelCondition(model_id/revision/precision/engine_id/engine_revision/input_conditions), ReviewResult/ReviewPackage를 확인했다. 이들은 사람이 검토할 package이며 실행 가능한 registry entry·binary locator·artifact reader·admission 증명은 없다. 새 package에서 ModelCondition이나 ReviewPackage를 복사하지 않는다.

원 private r3 validation.rs를 읽어 고정 model/input SHA, CPU2/1, actual loaded DLL5, 유한0..1 score, normal782와 cancel1frame/원 signal proof 검증을 확인했다. source 위치는 `runtime/development/ai-resume-vad-r3/tools/model-spike/src/application/bounded_vad/validation.rs`다. root tools/model-spike에는 새 bounded 파일이 없으며 r3 overlay를 root 구현으로 오인하지 않는다. [실제 실행 정본](../../../sots/verification/ai/cached-vad-resource-adapter-2026-10-21.md)이 정상1/취소1·HF/입력/runtime pin의 근거다.

## Unverified

아래 서명은 제안이며 Rust 소스 구현·컴파일 검증0이다. license 문서 전체 closure, ORT/DirectML 배포 방식, product artifact store, 새 package 공개 경계는 미인수다. 신규 profile을 승인하는 권한·영구 manifest schema version은 PM/SDK가 확정해야 한다. 고정25초 자료만으로 모든 입력 길이의 VAD 품질이나 실제 waveform semantic 검증을 보장하지 않는다.

## Phases

### P0 — 독립 값과 신뢰 경계

최소 후보 module과 공개 항목은 다음으로 한정한다. 아래 타입 이름은 미래 Rust 서명으로, 현재 제품 API가 아니다.

| projected source | 후보 항목 | 불변 조건 |
|---|---|---|
| media-vad/src/domain/artifact.rs | ArtifactPin { id, sha256, bytes }, Sha256Digest | lower-case64hex, positive bytes, bounded opaque ID. 경로·URL·실행권한 없음 |
| media-vad/src/domain/profile.rs | VadProfileSpec { profile_id, model_repo, hf_revision, weights, license_document, license_id, precision, runtime, input_contract } | 첫 후보 Silero repo/revision 고정. precision은 Fp32 enum만. license 문서는 ArtifactPin으로 따로 묶음 |
| media-vad/src/domain/runtime.rs | RuntimeSpec { engine_id, engine_revision, worker, modules, provider, intra_threads, inter_threads } | worker와 각 module은 별도 ArtifactPin, 중복 module ID 거부. 첫 CPU2/1. 모듈 이름만으로 허용하지 않음 |
| media-vad/src/domain/input.rs | VadInput { audio, pcm_sha256, sample_rate, channels, sample_count, source_range } | 원 WAV artifact hash와 decoded PCM hash 구별. mono16kPCM16만. range는 정수 sample 구간, checked arithmetic. source PTS 부재는 None |
| media-vad/src/domain/result.rs | CompleteVadResult; PartialVadObservation | 생성자·필드는 private. complete는 입력 sample_count에 대응한 ceil(samples/512)개의 유한0..1 score를 검증. 마지막 padded chunk의 유효 sample 수 보존. partial→complete 변환 API 없음 |
| media-vad/src/application/admission.rs | ProfileAdmission::admit(spec) -> Result<AdmittedVadProfile, ApiError> | AdmittedVadProfile 생성자는 비공개, Deserialize 미제공. 사용자 JSON의 admitted=true 또는 license 문자열을 신뢰하지 않음 |
| media-vad/src/application/artifacts.rs | ArtifactReader::open_verified(pin) -> Result<VerifiedArtifact, ApiError> | 검증된 bytes/읽기 lease를 반환. 검증한 path를 닫고 엔진이 같은 path를 다시 열도록 하는 TOCTOU는 미해결로 남기지 않음 |
| media-vad/src/application/execution.rs | VadEngine::analyze(admitted, verified_input, control) -> engine outcome | control은 기존 SDK deadline/cancel 경계와 연결. worker response의 self-reported profile만으로 결과 생성 불가 |

ArtifactPin은 실행 artifact 전용이다. EvidenceAsset과 구조가 일부 같아도 검토 package를 실행 credential로 변환하지 않는다. 검토 화면에 보낼 때 root application의 단방향 명시 mapping에서 기존 ModelCondition/EvidenceAsset을 생성한다. 새 media-vad가 root crate를 역으로 의존하지 않는다. 단순 전시 mapping은 admission이나 품질 승인을 부여하지 않는다.

첫 구현은 최소 공통 serde/media-contracts 계약을 사용하되 새 package가 raw OS handle/ORT 타입/PathBuf를 domain으로 노출하지 않는다. serialized profile spec와 runtime의 verified observation을 별도 타입으로 둔다. `observed_precision`을 runtime이 실제 검증하지 못하면 None으로 남기며 requested Fp32를 observed 값으로 복사하지 않는다.

완료 조건: SDK/PM이 위 타입 책임·기존 검토와의 단방향 mapping·admission 권한을 인수하고 exact source 후보 목록을 고정한다.

### P1 — artifact admission과 실행 lease

검증 순서는 profile schema/지원 계약→repo/revision 및 weight/license/runtime digest→local reader의 실제 bytes 대조→원 읽기 lease 또는 attempt 전용 immutable 복사→engine 실행이다. registry는 opaque artifact ID를 로컬 실제 경로로 해석하고 허용 root/reparse·no-overwrite 정책을 infrastructure에 둔다. caller가 외부 URL/worker path/argv를 지정하지 않는다. hash 확인 후 다른 파일로 치환 가능한 path-only 전달은 거부한다.

프로세스 분리 때문에 원 file handle을 넘길 수 없다면 adapter가 attempt 전용 불변 복사본의 SHA/bytes를 확인하고 worker도 전달된 pinned descriptor를 재검증한다. 이때 no-reparse/디렉터리 guard/동시 교체 방지까지 충족해야 하며 이 문서만으로 Windows 구현 완료를 주장하지 않는다. 다운로드와 모델 설치는 별도 기능이고 이 첫 후보에 포함하지 않는다.

runtime expected manifest와 parent가 수집한 actual worker/module 관측을 구별한다. OS DLL 전체 closure를 무근거로 요구하거나 현재5개를 전체라고 주장하지 않는다. 기대 모듈 누락·중복·hash mismatch는 실패이고, 관측 실패를 빈 목록으로 성공 처리하지 않는다. static ORT일 때 worker hash와 build provenance를 묶으며 별도 onnxruntime.dll이 있다고 가정하지 않는다. CPU 실행인데 DirectML import가 존재하는 사실만으로 DML provider 사용을 주장하지 않는다.

완료 조건: source fixture로 profile/weight/license/runtime/input 각각 변경·누락·중복·TOCTOU 거부를 확인할 검증 범위를 합의한다. 실제 테스트 실행은 중앙 배정 뒤다.

### P2 — 출력과 원 jobs의 접합

engine outcome은 complete candidate, partial observation, engine failure를 분리한다. 이 결과 enum에는 cleanup Complete 권한을 넣지 않는다. 원 owner/supervisor의 종료·Empty·EOF·Incomplete는 SDK와 합의할 별도 typed owner carrier가 담당한다. worker JSON의 cleanup=true나 ApiError.details 키가 원 ownership을 대신하지 않는다.

성공 출력에는 schema version·원 execution identity·immutable profile manifest SHA·input WAV/PCM pin·valid sample count·frame scores·실제 runtime 관측 ref를 묶는다. 원 API request ID가 job에 지속되지 않았다면 새로 만들어 채우지 않는다. partial/cancelled는 별도 diagnostic이며 Completed output0이다. 검증 성공 뒤 원 cleanup이 확인된 경우에만 common publication CAS에 진입한다. cleanup Unknown/실패 시 원 owner를 보존하고 지원 성공 판정0이다.

immutable artifact write와 JobRecord CAS는 단일 OS transaction으로 가정하지 않는다. reserve 이후 artifact가 남고 record complete가 실패하면 기존 recovery_required와 고정 artifact를 보존한다. retry로 파일을 덮어쓰거나 orphan 결과를 새 attempt에 자동 채택하지 않는다. stale attempt를 거부하는 CAS만으로 파일 경로 충돌까지 방지됐다고 주장하지 않는다.

완료 조건: SDK가 execution identity와 cleanup 진단 지속성 계약을 확정하고 fixture/actual CLI gate를 별도 인수한다. 그 전 registry에 audio.vad를 추가하지 않는다.

## Rollback

이 후보는 기존 profile/review/job schema를 수정하지 않는다. source 구현 시 독립 package부터 준비하고 실패하면 원 후보와 이유를 보존한다. 불변 결과는 rollback 중 삭제·수정하지 않는다. root wiring은 SDK/PM의 별도 변경이며 현재 shared/index/Cargo 변경0이다.

스킬 출처: modoo.today `modootoday-free/ai-quality-plus/inference-benchmark-workload-protocol` 본문 및 `references/protocol-tables.md` SHA256 `89f6ff32617d660be61433aac491db8f4e1fc1c9bb7b07b8a0c0cf80c1f0c4fd`를 읽었다. input 길이·runtime/precision·단일 concurrency·load/infer/wall·실패/부분 분모를 분리하고 cancel handshake를 속도 비교에서 제외했다. 표의 반복3회 제안을 신규 실행 권한으로 사용하지 않았다. 이번 검색의 fine-tune preflight와 llm-output-eval-suite는 VAD admission 설계 범위에 맞지 않아 채택하지 않았다. local rust-pro의 private constructor/ownership/typed outcome을 적용한다. plan/plan-authoring의 여섯 절·관측/제안 분리도 유지한다.
