---
id: plan:kinekino/20261029113951
title: 제품 CLI review host의 승인 입력 공급 공백
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 제품 CLI review host의 승인 입력 공급 공백

최신 SOURCEA2FF8328/2226에서 명시 `--serve-review`가 실제 producer와 fresh token을 생성해 동일 SDK를 소비하도록 연결했다. 정상2host의 저장/재생/EOF/재시작·이전 scope 거절/저장 보존, 별도 broken ready_write/os232/exit8·미디어 미생성을 인수했다. 선택190/정상bins/strict·CLI3/옵션2/fmt/전체입력 동일 통과. [근거](../../sots/verification/application/review-product-startup-20261027.md), [시작 계약](../../sots/player/application/review-startup.md). **남은 것은 준비 중 취소·강제crash·cleanup 미완료/IO 전수 및 전체CLI/AI/GUI**다. 아래 startup 미연결 설명은 각 과거 epoch의 기록이다. caller JSON·과거 Verified를 권위로 재입장하지 않는다.

최종 후속: SOURCE49C38F20/2221은 기존 저장CLI3/fmt/전후전체입력 동일까지 통과했다(539~541). 아래 ‘최종 입력 재확인 진행 중’은 종료됐으며 다음 작업은 제품 startup/producer 공급과 재시작 입장이다. [근거](../../sots/verification/application/timeline-live-sdk-20261027.md).

2026-10-27 SOURCE49C38F20 후속: 새 `assemble_sdk_with_evidence`가 동일 live resolver Arc를 저장과 playback worker에 공급했다. 실제CLI40으로 fresh user Verified2/replay/inspect와 같은 연결 재생/제어/종료를 검증했다. [근거](../../sots/verification/application/timeline-live-sdk-20261027.md). archive 설정/진입점은 유지한다. **다음 공백은 제품 startup의 명시 입력/producer 공급 및 재시작 입장**이며 기존 archive나 caller JSON을 새 권위로 승격하지 않는다. 최신 선택190/정상bins/all-targets strict 통과, 최종 입력 재확인 진행 중이다.

2026-10-27 SOURCE1E2B3767 후속: [fresh publisher token 기반 live_2_1 reader](../../sots/verification/application/timeline-live-reader-20261027.md)가 실제 case/media/application evidence 입장까지 통과했다. 명시 revision/역할 계약·Rust producer pin·원 trace/PCM/PNG와 원 모델 결과 보존을 검증했다. [190/정상bins 회귀](../../sots/verification/application/source375-timeline-live-reader-regression-20261027.md)도 통과. 다음은 새 resolver를 같은 SDK/정상CLI 저장·재생 조립에 연결하는 것이며 제품 startup/재시작·전체CLI/AI/GUI는 남는다. initial live_2와 첫 ID 실패는 소급 인수하지 않는다.

2026-10-27 DA8C42F7 후속: [로컬 package 새 버전](../../sots/verification/application/timeline-package-publication-20261027.md)의 실제 발행/재열기·원본/판정 보존·취소 미출판·동일 package replay를 검증했다. PublishedTimelinePackage는 commit 기록이며 아직 TimelineEvidenceLease/Verified 권위가 아니다. v2의 명시 mapping_revision/역할별 실행 계약과 reader/token 입장·SDK/CLI 연결은 남고, 초기 live_2를 default 보충해 소급 인수하지 않는다.

2026-10-27 SOURCE53105D42 후속: [live 결합 기록](../../sots/verification/application/timeline-live-bundle-20261027.md)이 모든 실제 관측을 동시에 보유하고 mapping을 검증했다. custody/Verified 입장은 아직 없으며 [기존 package 새 버전 발행 목표](../../sots/player/application/live-timeline-custody-publication.md)로 연결한다. 원 package/모델 결과/판정과 case identity를 자동 변경하지 않는다. caller JSON·기존 archive reference로 새 record의 권위를 우회하지 않는다.

2026-10-27 B1B46C92 후속: [실제 PCM 재생·clip 계보](../../sots/verification/application/timeline-pcm-lineage-20261027.md)를 공통 producer에 연결하고 [선택190/정상bins](../../sots/verification/application/source375-timeline-lineage-regression-20261027.md) 회귀를 통과했다. 아래 PCM 결합 미완료 설명은 원본/재생/clip 부분이 진전했으며 origin/capture·결합 opaque proof/custody·admission/startup은 계속 미완료다. 부분 계보를 제품 권위로 공급하지 않는다.

2026-10-27 후속: 공통 Rust producer의 영상·오디오 원 실행/clock 관측과 생성 후 취소 정리를 실제 검증했다. [근거](../../sots/verification/application/timeline-audio-production-20261027.md). archive reader만 존재하던 공백에서 partial component 구현으로 진전했지만 PCM/캡처 결합 issuer·custody/admission·startup은 여전히 남는다. partial observation을 권위로 공급하지 않는다.

중앙 PM 조율 계획. 기존 담당 구현 소유권과 WIP는 변경하지 않는다. [세션 계약](../../sots/player/application/review-playback-sessions.md)의 trusted Rust nomination 결정을 유지한다. 설치 의존성 문제가 아니라 제품 조립과 실제 producer 권위 연결의 잔여다.

[현재 후조건 감사](../../sots/verification/application/review-host-postconditions-20261027.md)는 archive execution reader와 새 실행 증거 issuer를 구분했다. 동적 issuer/custody 귀속·제품 startup 소비는 미완료이며 현재 field/JSON parser를 새 실행 권위로 승격하지 않는다. 공통 I/O driver는 실제 EOF/BrokenPipe까지 별도 인수했다.

| 요구사항 | 현재 근거 | 남은 실제 구현·검증 |
|---|---|---|
| startup 설정과 SDK 요청 분리 | LocalReviewSessionConfig는 Rust-only/no Default/no Deserialize, 요청은Selection만 | 제품 CLI startup 경로의 root/tool/budget 공급·중복/누락/혼합 모드 거절 |
| 실제 provenance 권위 | ReviewTimelineEvidence는 trusted nomination의 case/media/evidence와 producer pins를 검증 | 제품이 소유한 실제 producer의 검증된 결과에서 trusted Rust admission을 공급하는 연결. caller manifest·DTO·저장 Verified만으로 권위를 만들지 않음 |
| 동일 SDK 조립 | assemble_sdk가 review reader/source 검증 API/playback을 같은 root/admission으로 결합, CLI40의 새 Verified 저장→재생 실측 | 제품 CLI의 명시 serve startup 경로에서 이 조립을 소비 |
| 정상 종료 | 공통 serve_stdio→원 pending/join/물리완료; 별도host EOF/재시작·실제BrokenPipe 최초오류/exit8/미디어비강제정리 실측 | 제품 CLI entrypoint 소비와 stdin/flush OS오류·crash 전수 |
| 재시작 | fixture의 별도OS2host·ordinal/engine1재사용·이전참조거절 | 제품 entrypoint에서 동일 인수·실패/강제 crash 행렬 |

고정 Sprite fixture를 일반 제품 기본값으로 하드코딩하거나 임의 nomination JSON을 읽는 옵션으로 이 공백을 닫지 않는다. shared assembly가 존재한다는 이유로 기본 CLI 지원을 true로 바꾸지 않는다. 다음 구현은 제품 producer/권위 공급 책임을 계약에 맞게 연결한 뒤 serve 경로를 소비시키는 것이다. GUI·AI 사용자 품질·추론/다운로드/수동GUI 보류를 대체하지 않는다.
