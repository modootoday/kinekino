---
id: sot:kinekino/speaker-contract
title: 자막 화자 분리 값·변환 정책
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 자막 화자 분리 값·변환 정책

상태: 사용자 요구 확정, 설계 정본. 추론·파일 변환 구현은 미완료. 화자 분리는 누가 언제 말하는지에 대한 로컬 화자 군집이며 실명·캐릭터·성별 식별이나 목소리 음원 분리 기능으로 간주하지 않는다.

## 원본과 편집 값

`SpeakerId`는 해당 MediaId 안의 불투명 ID, `SpeakerLabel`은 사용자 표시 이름. 화자1/화자2 등 임시 라벨을 부여하며 다른 회차의 같은 번호를 같은 사람으로 보지 않는다. 회차 간 연결이나 피사체/캐릭터 연결은 명시적 사용자 작업과 근거를 요구한다.

`DiarizationRevision(id,media_id,audio_track_id,source_revision,range,model_refs,speakers,spans,edits)`를 CueDocument와 별도로 저장한다. `SpeakerSpan(id,in_us,out_us,speaker_ids,origin,evidence,review_state)`은 겹침과 unknown을 보존한다. `CueSpeakerLink(cue_id,span_ids,speaker_ids,assignment_origin)`는 정렬 결과이며 단어 정렬이 없으면 화자 경계로 단어 시간을 꾸며 넣지 않는다. 모든 시각은 원본 PTS 기준 정수 마이크로초·[in,out)다.

겹친 발화는 복수 speaker_ids 또는 분리된 겹침 span으로 보존하며 transcript 표시용 단일 화자 투영과 구별한다. unknown은 빈 추론 오류나 새 화자로 강제 배정하지 않는다. OCR 자막만으로 화자를 알 수 없으며 오디오/사용자 근거 없는 cue는 미지정이다. 병합/분할/이름 변경/일괄 배정은 새 revision과 undo/redo 수정 이력을 만들고 원본 추론을 덮어쓰지 않는다. 참조 중 화자 삭제는 재배정 또는 unknown 선택을 요구한다.

## 출력과 손실

| 출력 | 정책 |
|---|---|
| 프로젝트/JSON sidecar | stable IDs, 원본 span·overlap·정렬/수정 근거·revision을 보존하는 완전 데이터 |
| SRT | 표준 화자 metadata가 없으므로 선택한 `[화자] 대사` 접두어 방식 또는 라벨 생략. 겹침·구조 정보 손실을 미리 알림 |
| WebVTT | voice annotation으로 표시 가능한 범위를 내보내되 대상 플레이어 실제 호환 시험 필요. 전체 군집/편집 이력은 sidecar |
| ASS | Name 필드 및 선택 스타일로 표시하는 매핑을 검증. 이름·스타일과 실제 identity는 구별하며 전체 span 데이터는 sidecar |

사용자가 선택한 출력 프로필에 따라 label 포함 여부·unknown 표현·겹침 줄 처리·sidecar 저장을 결정한다. 자막 구간 자르기/0 기준 재배치/오프셋은 cue와 span/link에 같은 변환을 적용한다. 기존 형식에서 없는 화자를 복원한 것으로 표시하지 않는다. 라벨 내 특수문자·마크업 escaping과 파일명 정책은 출력 어댑터가 형식별로 처리한다.

인수: 한 화자/다수/유사 목소리/짧은 발화/노래/BGM/무음/동시발화·화자 변경 cue·단어 정렬 없음·VFR·잘라내기·merge/split undo·변환 손실·sidecar 왕복. 정답 라벨과의 DER/JER 및 overlap 포함 여부·collar를 별도 명시하고 자체 점수를 정확도로 표시하지 않는다.
