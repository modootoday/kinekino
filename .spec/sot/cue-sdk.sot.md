---
id: sot:kinekino/cue-sdk
title: Cue 교정 초안의 SDK 연결
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# Cue 교정 초안의 SDK 연결

`subtitles.edit_cues`는 명시 proposal store의 기존 cue snapshot 교정 초안을 교체한다. GUI/CLI typed client는 같은 application 경로를 사용한다. 모델 실행·음성 정렬·시각/분할/병합 편집·임의 SRT 파일 직접 덮어쓰기는 이 경로의 지원 범위가 아니다. 전체 자막 기능의 미완료 범위는 유지한다.

입력은 `{proposal_id, edits:[{cue_id,text}]}`와 필수 `context.expected_revision`이다. `EditCues`는 기존 Correction 값과 `proposals.edit` 유스케이스에 위임한다. 원 source·generated/raw·cue ID·시각은 유지하고, 새 immutable proposal draft revision을 발행한다. 중복/없는 cue·빈 교정/텍스트·다른 task/source 종류·unknown 필드는 기존 typed 검증으로 거절한다. stale revision은 revision_conflict이고 재시도 전에 명시적으로 재조회한다.

edits는 **전체 교정 초안 교체**다. 이전 수정에 대한 patch 누적이 아니며 이번 edits에서 빠진 cue는 원 source 텍스트를 사용한다. 이 의미를 schema/capability와 클라이언트에 노출한다. 기존 acceptance는 새 edit로 해제되며 새 명시 accept 전 자동 적용·내보내기·사용자 품질 승인으로 취급하지 않는다. 문장 의미/원음 정확도는 구조 검증만으로 승인하지 않는다.

저장 정책·원 source 검증·revision 충돌/발행은 [proposal 계약](../api/proposals.md)이 소유한다. 별도 cue 저장소나 두 번째 revision 정책을 만들지 않는다. `proposals.inspect`로 재열고 `proposals.accept`로 선택한 초안을 채택한다. 기존 accepted subtitle projection/export가 수정하지 않은 cue와 원 시각을 보존한다.

포트 미주입은 Unsupported다. 명시 `--proposal-root`는 기존 ProposalFiles를 주입하며 새 CLI 우회 정책을 두지 않는다. request schema와 capability는 해당 store 가용성에 따른다. 기존 proposal의 `idempotency_key/timeout_ms` 비지원은 유지하고 명시 거절한다. 이 동기 store 경로의 in-flight cancellation은 지원하지 않으며 시간초과/취소 성공을 가장하지 않는다. SDK 실행 전 queued cancellation과 실행 중 cancellation은 별개다. GUI·실제 미디어·crash/IO fault 전수와 시각 편집은 후속 인수다.
