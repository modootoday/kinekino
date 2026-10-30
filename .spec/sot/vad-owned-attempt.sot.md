---
id: sot:kinekino/vad-owned-attempt
title: VAD 공용 요청과 원 owned attempt의 연결
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# VAD 공용 요청과 원 owned attempt의 연결

현재 상태(2026-10-21 갱신): controlled-reader r5와 SDK durable envelope/from_prepared·구체 executor의 [실제 admission43](../../../verification/application/vad-job-admission-2026-10-21.md)이 root `34be4b3dc679de7cf5c66eaa18319e8ebc4efd97`에 채택됐다. Media P0 13·product guards22·SDK envelope4·원 retaining JobService caller4가 실제 통과했다. 원 음원 SHA 영속화와 원 scope, 파일 저장소 재개방·attempt2 재시도, 실제 reader EOF 직후 취소를 포함한다. 최종 source18 `161E5A8E1DEC5ACA1F0643220F22A9D4887B2915AAE744D12194D05795A0F55C`, 생성 lock 포함 인계19 `44E4CCCF0E4432E75433E10F9C7C460A80FD08C4F28D454002114CC1D06791F4`이며 원 실패·최소 수리·최초 source18을 보존했다.

후속 [실제 CLI consumer](../../../verification/application/vad-cli-consumer-2026-10-21.md)는 별도 source4 `BEC5C7EDD6DA5EDA909288252FC8C623609524E6ED59AB7D2B8D7DC6693F104B`에서 exact ignored 시험1과 실제 CLI child5를 검증했다. 주입한 동일 retaining host에서 submit/get·같은 키 dedup, 명시 종료·저장소 재개방 후 retry/get을 관측했다. 원음원 변경은 typed needs_input/source_expired로 거절됐고 원 prepared SHA/음원 pin을 보존했다. output=null/publication_started=false다. 1ms 만료는 authorize_calls=1, 두 번째 admission 전 failed/timeout이며 중간 admission 만료로 확대하지 않는다.

기본 registry/공개 operation·실제 AI worker/runtime load·모델 추론·제품 지원·배포 권리·품질 승인은 여전히 미완료다. profile/runtime/notice 변조 전체 회귀, 실제 worker·강제 panic/cleanup 실패도 위 통과로 승계하지 않는다. root Cargo와 SDK 최종 합성/실행은 SDK, root 채택은 PM 단일 소유다. 아래 source-only·제안·과거 root 상태는 접합 이전 조사 원문으로 보존하며 현재 지원 판정은 이 상태절과 연결한 실제 검증 정본을 따른다.

접합 전 계약 원문: SDK 담당이 공용 요청/JobExecutor를, Media가 manifest DTO·구체 profile/audio/destination·native 자원 조립을 소유한다. 다음 절은 source-only 후보 당시의 누락과 구현 방향을 기록하며 최신 실제 검증 범위는 위 상태절로 갱신했다. 실행 권한은 중앙의 현재 배정만 따른다.

## 현재 실제 호출 경로와 누락

`media-jobs/application/service.rs::submit`은 normalize 후 SubmitJob 전체를 fingerprint하므로 원 expected_source_sha256도 submission identity에 들어간다. 그 후 executor.prepare에 해당 expected를 전달하고 반환 PreparedJobInput에 원 execution_timeout을 설정해 저장한다. generic JobService는 expected를 descriptor나 parameters로 자동 복사하지 않는다.

공용 request4의 parameters6에는 원음원 hash 중복 필드가 없다. r9 descriptor는 profile와 파생 WAV/PCM·range를 담고 원 imported source hash는 PreparedVadAdmission.audio_source에만 별도로 존재한다. 따라서 공개 parameters6과 descriptor만 영속하면 최초 admission이 drop된 뒤 재시작/새 attempt에서 원 imported source pin을 복원할 수 없다. descriptor SHA를 원음원 SHA로 바꾸어 이 누락을 덮지 않는다.

SDK가 소유하는 내부 prepared parameter envelope에 canonical 공용 parameters와 original_source_sha256를 함께 보존하는 방향을 제안했다. 이는 public request 필드를 추가하는 것이 아니며, original source SHA를 생략하거나 현재 파일의 hash로 재추정하지 않는다. 버전/unknown-field/정확 정수 규칙을 적용한 단일 typed decoder를 SDK가 소유한다. 전체 prepared input SHA에는 envelope·descriptor ref·원 timeout이 포함돼 원 attempt identity와 결속된다.

## 안정된 host adapter와 매 attempt의 원 자원

기존 JobExecutor::owned_executor는 operation별 안정된 참조를 반환하고 OwnedExecutorPort는 Box<dyn OwnedOutputCandidate> 후보를 고정한다. r9 assemble_admitted_attempt_executor는 한 prepared input에 고정된 비활성 executor이므로 host 전체에 그것을 단 하나 공유하지 않는다.

SDK의 operation adapter가 JobExecutor와 OwnedAttemptExecutor를 함께 구현하고 owned_executor에서 자신을 반환한다. 매 prepare_owned(input, blobs, 원scope)에서 영속된 원 request/pin을 decode하고 신뢰된 ID 매핑·구체 입력 admission을 수행한다. 재구성 admission의 descriptor SHA/bytes와 원 input.source가 일치해야 한다. 그 입력과 admission으로 local r9 executor를 조립해 같은 원 scope로 prepare_owned를 위임하고 반환 session Box를 core에 넘기는 방향이다. 새 scope/deadline/permit 또는 별도 process owner를 생성하지 않는다.

local executor가 drop돼도 r9의 원 transport/publication이 admission·원 파일 guards·attempt/parent writer lease를 보유한다. 실패/취소/Pending 및 original error는 기존 session/core 경로로 전달한다. job별 mutable map이나 새 registry로 메모리에만 admission을 저장하지 않는다. queued 동안 원 bytes가 바뀌면 다음 attempt admission에서 원 pin 불일치로 실패해야 한다. 기존 persisted descriptor를 새 bytes로 덮어쓰지 않는다.

## 구체 profile producer 후속

후속 r2는 `runtime/coordination/vad-release-profile-r2/` source55 `CFDCA028E07FAEFBEF436F20B59D28F55C7AE1827CE12073F025CAFDC6A5F1AF`다. 원 r1을 보존하고 제품3파일만 바꿨다. SDK와 resolve_audio(public_id, &original_source_digest), resolve_destination(public_id)를 합의해 profile 매핑과 함께 구현했다. 별도 FixedVadInputMapping은 실제 고정 producer의 내부 audio/destination IDs와 일치해야 한다. audio pin 조건은 원 lookup과 같은 helper를 호출하며 실제 bytes는 admission lookup에서 다시 검증한다. SDK ID validator나 wire DTO는 복제하지 않았다. 실제 파일 fixture가 public 점ID3개에서 기존 typed prepare/JobFiles/원 lease까지 이어지도록 확장됐으나 작성8/실행0이다.

현재 root03a107에서 부모 production-before55 및 SDK 합성58 before가 일치하고 원 Media51도 그대로임을 독립 대조했다. 이 release r2는 현재 합성58 CPU gate에 들어가지 않는다. 메서드 pub(super)는 infrastructure 형제의 concrete operation adapter가 소비할 범위이며 application→infrastructure 역의존을 도입하지 않는다. 원 scope 전후 확인·원 cutoff 유지와, 기존 bounded 파일 읽기의 chunk 간 취소 polling 미구현은 구별해야 한다. 이 helper만으로 준비 단계의 즉시 취소를 지원한다고 표시하지 않는다.

`runtime/coordination/vad-release-profile-r1/`의 source55 `A91F0C8D19676C2FE9D29902DA44C3DFB2ED480EE849F32797E23926A1FA19EE`는 별도 후보이며 public51을 수정하지 않았다. decoder는 vad.release/1 원bytes/64KiB/SHA·P0 profile·명시 model/runtime/license/dependency/notice pins를 검증한다. runtime metadata·conversion provenance·evaluation/memory는 선언으로만 반환한다. 파일 존재/hash와 배포 권리/실 runtime 로드/품질은 별개다.

구체 GuardedReleasedVadInputs는 호스트의 public ID→private profile ID 매핑을 사용하고 기존 ProfileLookup에서 release manifest와 private snapshot의 별개 pin·전체 profile 일치를 확인한다. 모든 고정 원 asset을 bounded read/hash 검증하고 composite lease를 반환한다. audio/destination은 기존 구체 producer를 위임하며 public ID trusted mapping은 후속 r2에 구현돼 actual23 범위에서 검증됐다. 실제 생산 ProfileAdmission/SDK operation 소비는 제품 조립 잔여다. release 선언만으로 license/native closure를 승인하지 않는다.

controlled-reader r4는 기존 P0 AnalysisControl/읽기 루프를 재사용하고 원8KiB 읽기 사이 취소/deadline을 확인하는 source 후속이다. returned admission은 원 File/ancestor guards를 소유하며 짧은 control view를 borrow하지 않는다. 초기 submit에는 원 Running scope가 없으므로 bounded uncontrolled 경로를 명시한다. shared test helper `tests/support/vad_release_fixture.rs`는 원 제품 fixture의 단일 파일/고지/입력/ExactProfile 구현을 재사용한다. SDK 실제 JobService caller가 이 helper와 원 scope를 사용해 취소·expiry·재시도를 검증해야 한다. current r4의 작성13과 형식/모듈 경로 확인을 컴파일·실행 결과로 표시하지 않는다.

## 실행 인수에 필요한 반증 회귀

- 실제 JobService.submit→queue→원 owned attempt에서 canonical public request·원음원 SHA·descriptor SHA의 서로 다른 identity를 보존하고 저장/재조회한다.
- queued 후 원 profile/audio/model/runtime/notice bytes 변조·누락을 원 hash로 거절하며 worker0을 유지한다.
- 두 서로 다른 요청과 같은 job 재시도에서 새 원 attempt directory/cutoff를 사용하고 이전 session/candidate/lease를 공유하지 않는다.
- 재시작 뒤 메모리 map 없이 원 prepared input으로 재검증한다. 같은 idempotency key의 동일 요청은 같은 job, 다른 원 source/release pin은 conflict다.
- 원 run/poll/publication/verification 실패와 취소/CAS/EOF/정리·재열기를 SDK 공통 정책으로 확인한다. 실제 native worker와 품질 게이트는 별도 배정이다.

이 표는 작성/합의 중인 후속 조건이며 실행 결과가 아니다. 기존 executor46이나 retaining CLI3을 이 접합의 성공으로 사용하지 않는다. 적용 스킬: modootoday/software-qa/code-change-review-verification과 verdict-table 참조(9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9). 실제 필드 읽기/저장 경로를 추적해 누락을 구체화하고 선언·작성·실등록·통과의 범위를 구별했다.
