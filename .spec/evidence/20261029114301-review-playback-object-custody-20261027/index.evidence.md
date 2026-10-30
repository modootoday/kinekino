---
id: evidence:kinekino/20261029114301
title: 검토 원본의 엔진 경로 custody
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 검토 원본의 엔진 경로 custody

후속 검증: 221 기존 review_source 저장/복구 영향25시험 통과(1 helper ignored), fmt --all --check 통과, 222 all-targets clippy -D warnings terminal=true/exit0. dependency blitz-paint의 기존 deprecated 경고는 남아 있으며 담당 WIP를 수정하지 않았다. 이번 입력의 전체190/정상 bins는 아직 재실행하지 않았다.

중앙 PM, 2026-10-27. 기존 소유권·WIP·C NVMe 캐시·heavy1/CPU2와 보류를 유지했다. 새 미디어 프로세스·추론·다운로드·GUI 시험은 실행하지 않았다.

ReviewFiles::bind_playback_media는 application의 비공개 생성자 기반 ReviewMediaInput을 소비한다. 선택 asset SHA로만 구성한 objects 경로를 immutable open하고 전체 bytes/SHA를 재확인한다. 원 case session과 별도 configured root/상위 경로 및 file guard를 GuardedReviewMedia가 함께 소유한다. 외부 요청의 path를 받지 않으며 Send를 새로 구현하지 않았다. runtime 입장이나 Verified를 부여하지 않는다.

219 actual1 통과: reader/store drop 후에도 source 검사와 쓰기 open 거절(Windows sharing violation32)을 확인했다. 220 actual1 통과: 다른 root에 같은 SHA 이름으로 잘못된 bytes를 놓은 경우 RevisionConflict가 발생하고 실제 root의 전체 source 연결은 성공한다. 부정 fixture는 runtime/build-cache/tmp에 보존하며 경로는 stdout에 기록했다. 원 NOMINATION은 B1A47D17… pin 및 전후 guard로 확인했다. 선택 package의 모든 원 asset 검증은 기존 ReviewFiles/ReviewCaseEvidence로 수행한다. 원 봉인 bundle은 재발행하지 않았다.

이는 실파일과 case를 보유한 경로 연결의 근거이며 실제 mpv 시작·decode·seek·clock 또는 제품 전체 인수 근거가 아니다. SOURCE396 전체190/정상 bins와 E710 bundle publisher 입력은 각 과거 checkpoint로 보존하고 이번 변경의 입력으로 승계하지 않는다.

기존 적용 스킬 test-fixture-and-mock-build와 code-change-review-verification의 실제/합성 구분 및 foreign binding 거절 원칙을 적용했다. 출처 연결은 [case custody](review-case-evidence-resolver-20261026.md), [B0](review-playback-b0-20261026.md)에 유지한다.

다음: bound source의 runtime 입장 연결, 실제 resource permit 및 owned-process suspended create/Job assign/resume, 고정 local argv와 동일 인스턴스 IPC/seek/clock, 물리 종료 전 source/tool/resource owner 유지. 기존 mpv owned_startup/HostScope는 Job 자체가 아님을 [설계](../../player/application/review-playback-runtime-admission.md)에 반영했다.
