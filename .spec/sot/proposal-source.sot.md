---
id: sot:kinekino/proposal-source
title: Proposal source 컨텍스트 조회 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Proposal source 컨텍스트 조회 계약

중앙 PM, 2026-10-28. 명령은 `proposals.inspect_source_context`, 공통 typed client는 `inspect_proposal_source_context`다. 기존 inspect/저장 schema는 변경하지 않는다. 이 기능은 원 요청의 선언 시간 정보를 읽는 경로이며 미디어 재결합·검증된 source clock·품질 승인을 만들지 않는다.

입력은 `{proposal_id}`만 허용한다. expected_revision/idempotency_key/timeout_ms와 caller source bytes/path/context는 받지 않는다. historical verifier를 명시한 ProposalFiles의 opt-in 포트만 지원하며 기본/일반 store/legacy 포트는 Unsupported다. 발견 명령은98개로 늘지만 지원수나 완료율은 구성별로 별도 판단한다. 과거97개 발견 기록은 원입력의 역사다.

어댑터는 원 WriterReservation 안에서 같은 canonical state를 읽고, pinned HistoricalEvidenceReader의 source/candidate/reference를 기존 matches_source/matches_input으로 검증한다. 그 검증된 source bytes에서 R65 별도 컨텍스트를 계산하고 writer/root를 재확인한 뒤 current state를 다시 읽어 hash가 달라지면 RevisionConflict로 거절한다. 읽기 결과를 새 revision/판정/프로젝트/원본에 게시하지 않는다.

응답 `proposal-source-context/1`은 `state`와 `context`를 같은 조회 결과로 반환한다. context.source와 state.document.source가 같아야 하며 cue ID/순서/개수가 선언 시간 목록과 대응한다. canonical start_us/end_us는None, document 입력은 선언 시간 목록이 비어있어야 한다. timing_authority=declared_request_only/media_binding=not_verified/quality_approved=false를 유지한다. quality flag는 이 조회가 승인을 부여하지 않음을 뜻하며 실제 사용자 판정 이력은 기존 별도 조회를 소비한다.

typed client는 응답 schema/상태/원source/선언시간 불변조건과 요청proposal ID 일치를 재검사한다. 한 응답의state로 원문/후보/초안 비교를 구성하고 서로 다른 revision 조회를 합치지 않는다. response schema는 context의 canonical cue 시간도null로 제한한다.

응답 전체JSON envelope 상한은2MiB다. 먼저 bounded byte counter로 value를 측정한 뒤 실제 envelope도 측정하며 초과는 ResourceExhausted다. 더작은 host/consumer budget은 별도 적용된다. SDK queued 취소는 기존 실행 계층을 사용하지만 in-flight 취소는 지원으로 표시하지 않는다. 실제 미디어 검증·lease·프로젝트 적용·새모델실행은 수행하지 않는다.

새 actual-nine 시험은9 import후CLI/typed조회9쌍·6개cue/3개document·default미지원·잘못된payload/context·evidence부재·동일저장소무변경을 확인한다. 기존103 CLI회귀와 discovery98 schema/capability 구성대조는 별도 검증한다. 구현/실행의 실제 결과는 후속 근거에 기록하며 설계만으로 완료 판정하지 않는다. 원WIP/소유권/보류와 source375 private overlay를 유지한다.
