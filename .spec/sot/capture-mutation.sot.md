---
id: sot:kinekino/capture-mutation
title: 프로젝트 capture의 명시 제안 mutation
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 프로젝트 capture의 명시 제안 mutation

상태: 2026-10-21 [일반 파일·실제 CLI gate](../../../verification/application/capture-mutation-cli-2026-10-21.md)를 통과하고 PM root541b4c8에 채택된 SDK 계약이다. UI 활성화·전체 제품 완료와 구별한다. [origin/v1-v2 ADR](../../../../adrs/player/adr-capture-origin-and-project-v2.md), [프로젝트 저장](projects.md)을 따른다.

`proposals.edit_capture`는 `EditCapturedProposal { anchor, expected, draft }`, `proposals.accept_capture`는 `AcceptCapturedProposal { anchor, expected, target_source_revision }`를 받는다. 타입은 `application::proposals::guarded`, 공통 typed client는 `ApplicationClient::edit_captured_proposal/accept_captured_proposal`이다. GUI와 CLI는 같은 application 유스케이스를 사용한다. 기존 proposals.edit/accept 계약은 유지한다.

anchor는 `project_id`, `project_revision`, `project_revision_sha256`, `proposal_id`, `capture_sha256`다. 구성된 ProjectStore의 정확한 불변 revision을 실제 읽고 해시·capture를 검증한다. caller가 전달한 capture DTO나 origin 문자열로 authority를 조립하지 않는다. 새 origin은 명시 live capture producer가 원 root directory handle의 physical identity와 결속해 발급한다. 과거 origin 없는 capture는 bytes/해시/읽기 전용을 유지하며 소급 승인하지 않는다.

최초 expected 생략/null은 원 저장 capture 전체의 revision/hash를 기대한다. 후속 명시 동작은 마지막 성공 응답의 `continuation { draft_revision, state_sha256 }`를 전달한다. 이 값은 비밀 아닌 CAS 기대값이며 권한 토큰이 아니다. 같은 원 writer 안에서 실제 origin·완전한 source/generated/historical anchor·기대 StoredProposal 전체 해시/revision을 대조한 뒤 공통 domain transition·private locked publication을 사용한다. UI의 사전 inspect·hash 비교로 대체하지 않는다.

응답은 `CaptureMutationResult { state, continuation, acceptance, reused_acceptance }`다. 성공만 새 기대값을 제공하며 자동 inspect-refresh·프로젝트 capture/working draft 덮어쓰기·자동 project save는 없다. 오류 후 caller가 임의로 최신 상태를 재조회해 원 intent를 갱신하지 않는다. 새 명시 selection/capture는 별도 사용자 동작이다.

정확 accept 재시도는 원 pre-accept payload를 그대로 반복한다. 원 expected revision의 불변 상태를 같은 writer 아래 읽고 전체 해시를 확인한 뒤 공통 도메인이 계산한 post-state 전체가 current와 같은 경우에만 기존 receipt를 반환한다. 이후 변경이 있으면 거절한다. 기존 acceptance receipt만으로 guard를 우회하지 않는다. 반환 post-state continuation은 이후 의도적인 동작을 위한 기대값이며 exact retry의 원 payload를 대체하지 않는다.

| 조건 | 오류 |
|---|---|
| 잘못된 selector/부분 expected/unknown field/context expected_revision·idempotency_key·timeout_ms | InvalidArgument |
| backend/optional writer session 미지원 | Unsupported |
| origin 없는 legacy capture | NeedsInput, remediation=create_explicit_live_capture |
| 실제 backend/source/current full state/revision 또는 project hash 불일치 | RevisionConflict |
| 저장 손상·origin metadata 복구 필요 | 기존 adapter의 NeedsInput·remediation 보존 |

CLI는 `--input - --project-root ABS_DIR --proposal-root ABS_DIR`로 같은 SDK를 구성한다. remote endpoint와 local roots를 혼합하지 않는다. capabilities/schema는 adapter 지원 여부를 명시하지만 개별 capture 허가를 뜻하지 않는다. source 자산 bytes SHA·typed source 해시·capture 전체 canonical SHA·raw 저장 파일 SHA·project revision SHA를 혼합하지 않는다.

프로젝트 current latest와 제안 저장소의 cross-root 원자 transaction은 보장하지 않는다. origin은 비밀/보안 토큰이 아니다. 기술적 채택은 모델 품질 승인·자막/카드 렌더링·실제 미디어 적용이 아니며 automatic_application_enabled=false다. UI readonly 해제는 실제 SDK 인수 뒤 별도 원 host·presentation·시각 검토를 거친다.
