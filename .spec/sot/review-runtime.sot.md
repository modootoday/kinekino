---
id: sot:kinekino/review-runtime
title: 로컬 review 재생의 입력·runtime 입장
status: active
created: 2026-10-29
updated: 2026-10-29
verify: manual
---
# 로컬 review 재생의 입력·runtime 입장

2026-10-26 중앙 PM. 기존 episode SDK native 경로는 reverified resolution artifact의 HLS URL을 소비하며 local MP4 경로가 아니다. review source를 그 타입으로 위장하거나 GUI 전용 LocalVideo/Mpv::launch_local로 SDK 검증을 우회하지 않는다.

## 입력 B0

ReviewCaseEvidence가 원 package/item/request와 모든 asset의 full SHA/bytes·immutable guard를 검증·보유한다. 내부 prefix 읽기는 같은 asset ID/SHA의 최대64KiB만 반환하며 full-content hash 검증을 대체하지 않는다. source102MB도 전체를 새메모리buffer로읽지않고동일세션보유하에분류할수있다.타provider의prefix기본구현은Unsupported다.

inspect_review_input은선택item의media identity/bytes/선언MIME를확인하고독립raw prefix분류를요구한다.현재지원은video/mp4와구조가맞는ISO BMFF ftyp header다.이값은IsoBmffHeader이며codec/track/전체container/실decode인수가아니다.다른format지원은추가구현대상으로유지하고추측분류하지않는다.파생PNG/PCM동일성으로원source identity나선언MIME를대신하지않는다.

## Runtime B0

후속 구현: ReviewInputWithRuntime<L>/GuardedAdmittedReviewMedia<L>가 원 source/case/path guard와 concrete runtime permit을 같은 owner로 유지한다. ReviewRuntimePool은 명시 root의 review 전용 독점 lock·TrustedMpvExecutable·실제 빈 PreparedFamilyJob을 발행한다. [예약 근거](../../verification/application/review-runtime-reservation-20261027.md)를 따른다. 이 pool lock을 전체 producer의 중앙 heavy scheduler로 오인하지 않으며 PM의 heavy1 배정을 유지한다. 실 child 시작 이후의 running owner 전환/IPC/seek/clock은 아직 구현 대상이다.

2026-10-27 공통 owned-process에 PreparedFamilyJob/JobBudget와 spawn_prepared를 구현했다. 실제 빈 Job의 memory/affinity 제한을 OS에서 조회한 뒤 동일 owner를 suspended child에 넘긴다. [실제 시험](../../verification/application/review-prepared-job-20261027.md) 참조. 이 memory는 committed-memory 제한이며 RSS나 물리 RAM 예약을 의미하지 않는다. application 필드명은 max_job_committed_bytes다. 고정 tool/source 및 중앙 heavy permit과 결합하는 production admission은 계속 구현 대상이다.

reserve_review_runtime은이미확인한입력과필수ReviewRuntimeAdmission port를받는다.선택backend·engine SHA·source/RAM/CPU/startup예산·OwnedJobUntilPhysicalExit 소유권종류가명시된실제lease를요구하고,descriptor만으로허용하지않는다.필드에Default/Deserialize를제공하지않는다.lease의check_current와profile snapshot일치를재확인하며source가예산을초과하면거절하고lease를해제한다.

현재production concrete local runtime은미연결이며UnavailableReviewRuntime은명시Unsupported다.이를없어진dependency나사용자승인대기로표시하지않고구현미완료로추적한다.이경로는engine를실행하지않고playback_ready를반환하지않는다.공개CLI command나기존episode source정책도바꾸지않는다.

## 실제 backend의 다음 계약

2026-10-27 구현: ReviewFiles::bind_playback_media는 ReviewMediaInput을 소비하고 configured root/objects/<선택 SHA>만 연다. 전체 bytes/SHA를 다시 대조한 뒤 원 case session·root/상위 디렉터리 guard·열린 file을 GuardedReviewMedia 하나에 함께 보유한다. caller/wire path를 받지 않는다. 이는 동일 스레드의 겹치는 custody handoff이며 runtime 허가·process 시작은 아니다. 소유 worker가 이 값을 physical child exit까지 유지해야 한다.

조사 결과 기존 mpv::owned_startup은 Child/reader/IPC 회수 carrier를 제공하지만 Windows Job 할당 자체는 하지 않는다. HostScope도 논리적 issuer 수명이며 Job 증거가 아니다. 따라서 이를 그대로 OwnedJobUntilPhysicalExit 구현으로 인정하지 않는다. packages/owned-process의 suspended create→Job assign→resume와 family-empty cleanup 계약을 재사용하고, mpv IPC/reader 경계를 연결해야 한다. 기존 episode/C 담당 경로를 임의 개조하지 않는다.

1. 기존guarded local input과timeline proof의case/media identity를결합한다.
2. 고정Mpv tool/라이선스·resource permit·owned Windows Job을실제보유한뒤에만local file전용고정argv로원child를시작한다.원HLS source type이나GUI fallback을쓰지않는다.
3. startup부분실패와종료지연에서도원source/tool/resource/child owner를물리종료까지보유한다.비Send review session을unsafe Send로바꾸지않고소유worker안에서생성·보유하거나검증된handoff를구현한다.
4. 같은engine instance의seek/clock·해당source binding·EOF/자식/IPC정리를실제로관측한다.그전에는Verified나UI/품질완료로승격하지않는다.

운영resource선택과원source입장상태는명시생성자로만만들고편의Default로대체하지않는다.C NVMe context와heavy1/CPU2현재배정·WARP/추론/다운로드/수동GUI/push보류는유지한다.
