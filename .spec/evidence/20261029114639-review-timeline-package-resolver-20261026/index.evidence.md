---
id: evidence:kinekino/20261029114639
title: Package timeline resolver 본체 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Package timeline resolver 본체 검증

2026-10-26 중앙 PM. [producer bundle v1](../../player/application/timeline-producer-bundle.md)의 ReviewTimelineEvidence가 TimelineEvidenceResolver를 구현한다. 신뢰된 Rust 조립의 단일 case/media/manifest 및3종 tool SHA 지명을 요구하고 다른 입력은 package 조회 전에 거절한다. wire 요청으로 입장을 생성하지 않는다.

원 ReviewCaseEvidence 세션을 보유한 채 item별 source/tool metadata·manifest·실행 기록·raw audio/video·PCM 전체/clip/replay·완전 PNG capture를 교차 확인해 TimelineObservations를 계산한다. 공통 domain 검증을 통과해야 lease를 반환한다. 상위 application admission은 원 case/media/evidence lease와 snapshot을 유지한다.

## 실행

- 199 timeline_evidence include-ignored:15pass/0fail,시험1.88초.기존12에새resolver3시험을더했다.기존3actual시험은보존audio/video/PCM읽기이며새resolver3은실ReviewFiles의합성producer bundle이다.
- 새양성은실파일package→resolver→공통application admission→17초/25.5초mapping→원media writer거절→lease해제후writer재획득을확인했다.playback_ready=false 유지.
- 새음성은wrong source/incomplete/nonzero exit/async correction/wrong stream/unapproved tool/audio clock gap/clip mismatch/replay mismatch/bad PNG/wrong capture의11조건을거절하고실패후원guard해제를확인했다.별도foreign case/manifest nomination은파일조회전RevisionConflict다.
- fmt --all -- --check 종료0.200 all-targets strict 종료0/24230ms.기존blitz-paint 의존deprecation원로그유지.
- 201 review_source영향25pass/1내부process helper ignored,저장/복구/capacity·foreign item입력회귀통과.

## source pins

| 경로 | SHA256 |
|---|---|
|timeline_evidence/package/mod.rs|FB3927CE385A8BABCB8A011BE5E4805824648AD9FF0A6E0DC4D57FE76F49DE32|
|package/manifest.rs|766AD058B489C4AA2397D3EB8763C060833DA2FA92122318380349EE61F750FD|
|package/tests.rs|CF8814D3BBE1F398BE452BE0E0E1C37135184E2AEB10ACE33D6200324091D294|
|application/review_source/review_evidence.rs|1C7A97EFB41A191B3A0675AD61E5B3C3256A41FE542202949461399391F054FF|

포맷 이후199/200/201동안코드변경없음.새fixture는runtime/build-cache/tmp/timeline-package-{PID}-{index}에원파일을보존한다.실제source375/AI9/역사실패와원자료는변경하지않았다.index0/push0.

## 인수 한계와 다음 작업

resolver본체와합성교차검증은완료했다.임의JSON의hash가producer출처를인증하는것은아니다.TrustedTimelineAdmission은이미검토한producer bundle을선택하는host신뢰경계이며실제producer조립/입장gate가아직남는다.새manifest/execution JSON계약을과거plain-text receipts의새실행결과로소급계상하지않는다.

다음은기존실제Sprite관측의원source/tool/process·PCM/capture핀을보존한별도증거묶음발행과명시입장·실resolver인수다.이를actual9(Sintel)에붙이지않는다.SDK재생owner/clock·Verified저장전이·CLI지원/GUI/사용자품질은별도미완료다.현재wire Unverified/readinessfalse유지.이후계는396기준190/정상bins이후변경이며새전체회귀성공으로승계하지않는다.

기존modoo.today test-fixture-and-mock-build/code-change-review-verification의실파일/합성·출처권한/byte검증구분을적용했다.기존WIP·소유권·원증거·heavy1/C캐시·추론/다운로드/수동GUI/WARP/push보류유지.
