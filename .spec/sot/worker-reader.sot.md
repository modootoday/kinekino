---
id: sot:kinekino/worker-reader
title: 검토 worker의 원본 결합 상태 reader
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 검토 worker의 원본 결합 상태 reader

2026-10-27 중앙 PM. Unverified association의 playback_ready=false 및 RD10/11 경계를 유지한다. public local-open을 이 상태로 우회 등록하지 않는다. 본 단계는 기존 playback.status 공통 SDK/CLI에 정확한 source/instance의 engine-time 관측을 연결한다.

ReviewPlaybackWorker는 입장한 원 ReviewInputIdentity를 snapshot에 보유하고 status_reader(case, media, engine_instance_id)를 제공한다. reader는 원 State의 Weak만 보유하며 정확한 case(package/item/request)/media ID·SHA/양수 engine instance를 요구한다. 다른 source/instance는 RevisionConflict, 관측 전·closing은 NeedsInput, physical close·worker thread 종료·원 owner 해제는 NotFound다. 새 source/다음 instance를 자동 선택하지 않는다.

관측은 기존 PlaybackStatusPort와 project_native_status를 소비한다. engine seconds→checked integer microseconds 투영을 공통 구현과 공유하며 이는 source PTS/정확 frame이 아니다. published_age_ms는 worker의 마지막 상태 publication 나이이고 원 native event의 신선도를 입증하지 않는다. unknown pause/idle/position/duration/video/audio는 기존 optional 관측으로 유지한다.

worker publication 수명 guard는 thread 종료 때 reader를 만료시킨다. thread panic도 관측 권위를 끝내지만 physical exit를 추정해 true로 바꾸지 않는다. 실제 panic 뒤 process/source 정리 전체 인수는 별도 잔여다. reader가 source 세션이나 tool/Job을 보유하거나 수명을 연장하지 않는다.

공개 command 목록93 및 기존 schema는 바꾸지 않는다. 기본 SDK는 계속 provider 미주입이고, trusted Rust host가 reader를 주입한 경우에만 이 source-bound 관측을 제공한다. 반환 DTO에 caller Verified/모델 품질 승인/재생 준비 권한을 추가하지 않는다. 이후 local-open/status/close service와 opaque proof·Verified 연결은 전체 요구로 유지한다.

기존 SOURCE F7A32EFE…의 선택190/정상 bins/CLI 회귀는 [완료 checkpoint](../../verification/application/source375-review-worker-regression-20261027.md)로 보존한다. 새 변경은 별도 source를 고정하고 [상태 검증](../../verification/application/review-worker-status-20261027.md)에서 actual/fixture·성공/실패를 구분한다.
