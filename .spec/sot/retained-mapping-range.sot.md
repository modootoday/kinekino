---
id: sot:kinekino/retained-mapping-range
title: 근거 보유 범위와 기록 데이터의 분리
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 근거 보유 범위와 기록 데이터의 분리

2026-10-27 중앙 PM. 기존 source/case/media custody와 playback_ready=false·사용자 품질 승인 경계를 유지한다. 이 범위 guard 자체는 association 저장 상태를 변경하지 않으며, 저장 후계는 별도 [Verified 계약](api/review-source-draft-contract.md)이 소유한다.

TimelineMappingRecord는 case/media/evidence·mapping revision·정규화 time base·signed video origin·원 origin 관측 SHA·source coverage·경계 frame PTS를 가진 직렬화 가능한 기록 데이터다. 비율/u128와 signed i64 tick은 canonical decimal 문자열이며 숫자 JSON·비정규 표현·추가 field를 허용하지 않는다. 원 validator와 같이 origin/video time base는 동일해야 한다. validate는 구조만 검사하며 현재 권한이 아니다.

RetainedTimelineEvidence.mapping_record는 기존 원 case/media/evidence lease를 확인한 뒤 immutable snapshot으로부터 기록을 만든다. check_record는 그 원 proof의 현 상태와 기록 전체를 비교한다. record를 JSON으로 재열어도 원 proof가 없으면 권한이 생기지 않는다.

retain_range는 nonempty user range가 실제 관측 coverage 안에 있을 때만 RetainedMappingRange를 만든다. coverage 밖은 NeedsInput(추가 근거 필요), 다른/변조 record는 RevisionConflict, 잘못된 shape는 InvalidArgument다. 범위를 자동 확장·잘라내기하거나 전체 영상 duration으로 승격하지 않는다. 현재 Sprite proof coverage는17~25.5초다.

opaque range는 원 proof를 borrow하므로 먼저 proof를 drop할 수 없고 Deserialize/Default/공개 field constructor가 없다. plan은 raw video frame table에서 각 source boundary 이상인 첫 frame을 정수 time base로 선택한 **기대 frame 계획**이다. 실제 seek/capture/decoded frame 관측이나 완료 receipt가 아니다. publication/실행 직전에 check_current를 호출해야 하며 late revocation은 실패한다.

후속 저장 Verified 전이는 원 proof와 CAS/control/원자 publication을 결합해 구현됐다. 저장 metadata를 재열었다는 이유로 live 권한이나 playback_ready를 만들지 않는다. association의 archived Verified 출력과 원 proof 기반 서버 전이는 별도 계약을 따르며 caller 요약은 권한이 아니다. [이 guard의 최초 검증 근거](../../verification/application/retained-mapping-range-20261027.md)는 당시의 stored_verified_transition=false를 그대로 보존하고 fixture/실제·기록/권한·계획/실행을 구분한다.
