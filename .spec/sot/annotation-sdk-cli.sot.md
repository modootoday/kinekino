---
id: sot:kinekino/annotation-sdk-cli
title: 수동 화자 annotation SDK/CLI 계약
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 수동 화자 annotation SDK/CLI 계약

2026-10-27. [화자 값 정책](../../domain/subtitles/speaker-contract.md)의 모델 없는 수동 편집 부분을 공통 Rust domain/application과 ordinary local JSON lease로 구현한다. `subtitles.speakers`와 `subtitles.edit_speakers`만 해당한다. diarize/align·음원 분리·사람/캐릭터 식별·단어 정렬이나 실제 모델/미디어 귀속을 제공한 것으로 표시하지 않는다.

## 입력과 원본

speaker-annotations/1 JSON은 document_id/revision/parent_sha256,선언된 source,original state,append-only edits를 가진다. source는 media_id/media_sha256/audio_track_id/source_revision·원본 기준 정수 microseconds 범위·선언 cue source SHA·선택 model refs다. model refs는 speaker_segmentation/embedding의 fixed revision/SHA만 선언 가능하며 현재 검증된 엔진 provenance가 아니다. 파일 SHA를 source media SHA로 바꾸지 않는다.

original은 Speaker(id,label),SpeakerSpan(id,in_us,out_us,speaker_ids,origin,evidence,review_state),CueSpeakerLink(cue_id,span_ids,speaker_ids,assignment_origin)를 보존한다. review_state는 선언된 unreviewed/needs_review/reviewed이며 미제공은 undeclared로 유지하고 만들어 채우지 않는다. 원 reviewed 선언도 실제 사용자 AI 품질 승인이 아니다. 빈 speaker_ids는 unknown이고 복수 ID/겹친 span은 overlap이다. 모든 ID/범위/참조를 검증한다. cue link가 있으면 선언 cue source SHA를 요구하지만 그 cue 원문이나 word alignment를 검증했다는 뜻은 아니다. source binding/parent chain/품질 승인은 false다. confidence를 만들어 채우지 않는다.

512KiB 파일·128speaker/1024span/1024cue·128history/요청32edit 한도는 수동 편집/저장 자원 한도이며 모델의 최대 인원수/지원 capability가 아니다. 원본/source/history와 현재 상태를 함께 표시할 수 있도록 read가 document와 derived current를 반환한다. raw 원본을 현재 값으로 덮어쓰지 않는다.

## 편집과 이력

add/rename/merge/split/assign_cues/delete는 명시 선택만 수정한다. merge는 선택한 기존 화자 중 target으로 refs를 결합한다. split는 기존 화자의 선택 span/cue membership만 새 ID로 옮기며 다른 overlap ID와 모든 시각을 유지한다. cue 배정만 바꿔도 원 span은 바뀌지 않는다. 사용 중인 화자 삭제는 reassign을 명시해야 하며 []는 unknown 선택이다. 실명/성별/역할을 추론하지 않는다.

undo/redo는 append-only event 이력으로 현재 상태를 재생한다. undo 뒤 새 edit는 redo branch를 해제하되 과거 log/원본 파일은 보존한다. 과거 add/split에서 사용된 ID는 삭제/undo 뒤에도 새로운 화자에 재사용할 수 없다. 원래 ID를 복원하는 undo/redo와 새 ID 생성은 구분한다. 원본의 evidence/origin/review_state는 그대로 두고 영향을 받은 현재 membership만 user_edit로 표시한다. 이름/현재 span 배정 변경은 해당 span을 needs_review로 돌리고, cue만 배정할 때 원 span 상태를 변경하지 않는다.

## API와 저장

`subtitles.speakers`: input_path,optional input_sha256. 최초 read가 실제 JSON SHA를 반환한다. optional pin 불일치는 RevisionConflict다. `subtitles.edit_speakers`: input_path/input_sha256/output_path/edits와 양수 context.expected_revision이 필수다. idempotency_key는 미지원,timeout_ms1..120000(기본30000)이다. edit는 n+1 revision·실제 입력 parent SHA를 가진 self-contained 새 JSON만 발행한다. file-path branch이며 전역 latest head/CAS나 실제 source rebind가 아니다.

원 ordinary source/file/root guard를 parse·edit·publish 동안 보유한다. 원 handle을 복제해 실제 content SHA도 재검사하며 source 경로를 재개방하지 않는다. 출력은 별도 ordinary parent의 새 파일이어야 하며 write/sync 후 원 source hash·최종 cancellation 순서로 재검사하고 create-only link 한 번으로 commit한다. commit 전 취소는 unpublished,commit 뒤 추가 fallible 검사/롤백으로 성공을 뒤집지 않는다. 응답 객체/출력 SHA는 commit 전에 구성한다. 기존 출력/원본/이전 revision을 덮어쓰지 않는다. 동일 요청/같은 output 재전송은 충돌이며,응답 유실 때 read로 output SHA·parent/history/revision을 확인한다. 자동 재시도/최신값 추정/프로젝트 저장·모델 승인 없음.

공통 SDK는 with_speaker_documents로 주입하며 미주입은 Unsupported다. Windows normal file CLI만 해당 adapter를 주입한다. GUI는 아직 미연결이고 수동 시각 인수도 미실행이다. SRT/VTT/ASS speaker export·실제 diarization/align·model 결과 편집 연결/DER/JER은 별도 미완료다. 모델·미디어 다운로드/추론·수동GUI/WARP/push 보류는 유지한다.

## SDK execution의 취소 경계

같은 프로세스의 원 PlayerSdk/ExecutionOwner는 `subtitles.speakers/edit_speakers`에도 execute_controlled의 원 operation 신호를 전달한다. Deadline은 기존 Instant 기한과 외부 checkpoint를 합성한다. 기존 파일 lease/read/write/sync·원 content hash 재검사·create-only commit 정책은 변경하지 않는다. 임시 파일을 기록한 뒤 최종 checkpoint에서 취소되면 output/임시 파일을 남기지 않고 source lease를 반환 시 해제한다.

commit 뒤의 늦은 취소에는 추가 checkpoint나 롤백을 두지 않는다. 이미 구성된 실제 성공 응답과 immutable output을 보존하고, execution terminal의 cancellation_requested와 actual response를 구분한다. same-process 원 SDK execution의 기능이며 다른 wrapper client·CLI 원격 cancel ACK·강제 선점은 미지원이다. GUI 연결·사용자 품질·모델 추론 지원을 뜻하지 않는다.