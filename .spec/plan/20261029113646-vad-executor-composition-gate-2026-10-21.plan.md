---
id: plan:kinekino/20261029113646
title: VAD executor 조립·검증 준비
status: active
created: 2026-10-29
updated: 2026-10-29
---
# VAD executor 조립·검증 준비

## 2026-10-21 현재 실행 목표

이 단계 CPU gate는 고유46 pass/ignored0으로 PM 인수·반환됐다. [실제 검증과 실패 수리 정본](../../../sots/verification/ai/media-vad-executor-r9-cpu-2026-10-21.md), [공개 편입 후속 제안](vad-public-package-integration-2026-10-21.md)을 따른다. 현재 source-only이며 아래 실행 계획은 완료된 gate 기록이다. 전체 goal과 공개 기능 지원은 여전히 미완료다.

중앙 PM의 execution-leases 현재 배정이 아래 과거 준비 기록보다 우선한다. 채택 root `2fd3e94eea4d32a4189b2dfdb331f8af05d0bf7f`·Meari `0b79d57d7a59b3146b7ec72d02e2651b21b166fd`와 r9 source25 `BE3B631A07734F57EA8BCCB2B88DF6B7313F178FFF874AEE07816688E4886E31`를 사용한다. SDK 최종 P1b는 이미 root에 채택됐으므로 WIP나 추가 회신을 기다리지 않는다. P0r4 원10·protocolr2 원13·common9·현재 저장/프로세스 어댑터와 원 immutable guard를 상속한다.

현재 단계 목표는 별도 snapshot의 입력/provenance/reference lock 고정, offline metadata와 registry drift 확인, locked fmt·all-target strict·fresh lib/product_guards BIN·실제 목록·고유 시험 결과의 인계다. 기대 Windows46(lib31/product15)은 실제 등록과 통과 전까지 작성 수다. owned-process/test-support는 활성화하지 않는다. 단독 CPU 슬롯만 배정됐으며 실제 worker 빌드/실행·추론·다운로드·GUI·VM·DPI는0이다. 첫 실패는 원본을 보존하고 별도 최소 수리로 진행한다.

공용 VAD request는 SDK 담당의 별도 source-only 후보로 조율하며 현재 gate에 혼합하지 않는다. original source SHA·정식 release manifest·private profile snapshot을 구별하고 기존 ID에서 내부 opaque key로 trusted mapping한다. 전체 AI 품질·공개 VAD 지원·전체 제품 목표는 미완료다.

goal 도구 관측: 사용자는 reset을 알렸으나 get_goal은 기존 active를 반환했고 create_goal은 unfinished goal 때문에 거절했다. 가짜 완료로 우회하지 않고 이 현재 목표와 중앙 배정을 실행 정본으로 삼는다. 적용 스킬은 기존에 읽은 `modootoday/software-qa/code-change-review-verification` 및 `references/verdict-table.md`(SHA256 `9d9619a829ae24d73645ef5f7c8126c8257caeddac80800632866257008ad2a9`)다. 작성·실등록·통과·지원 주장을 구별하고 최초 실패/원 입력을 보존한다.

2026-10-21. source-only 후속이며 실행 권한이 아니다. 현재 중앙 실행 배정을 따른다. 기존 실제 VAD probe/순수 P0 11/protocol19·세카테고리 CLI 성공은 이 executor의 통과로 재사용하지 않는다.

## 필요한 원 입력

- runtime/coordination/media-vad-executor-r7/source17.json과 frozen raw17. 기존r6 code16 불변+integration target1이며 production root 등록0.
- 실제 인수할 SDK P1b package source pin. 현재 WIP를 임의 최종본으로 동결하거나 축약 stub로 대체하지 않는다. NotStarted/원scope/driver/원error·retained 및 per-attempt factory의 실제 서명이 필요하다.
- Supervisor additivecleanup3 FB29E8612CC708970FCE1E96B0B46AAF3CB4DE2650C01E9BF8BECF2B1B581736/frozen raw. 기존 finish 본문 및 원 SDK/common과 차이를 확인한다.
- P0r4 source10 424B04C6·protocolr2 source13 81DDC830·공통 contracts9의 원bytes. 변형하거나 purepolicy를 executor에 복제하지 않는다.
- required-baseline-guard.json의 기존 immutable_storage_paths source1을 snapshot root의 같은 경로로 보존한다. root Cargo/mod에 새 production 등록을 할 필요는 없다.
- 기존 media-file-publication/owned-process 및 해당 SDK dependencies의 전체 상속 기준·reference lock을 기록한다. offline 생성 lock의 registry version/source/checksum drift는 인수 전 실패로 분리한다.

## 배정 후 검증

현 자원 admission/원 EEA9 conservative2·mask3/BelowNormal·offline/locked/-j1·기존 model-spike target·thread1을 유지한다. 새 coldtarget/모델/다운로드/worker build/실제spawn/GUI/OS프로세스 gate0이다.

후보 내부에서 metadata/필요lock 차이 보존→fmt→media-vad-executor package all-target strict -D warnings→최종 lib와 product_guards BIN no-run/원compiler artifact/SHA/고정사본→각 actual list·고유이름대조→정확 library19/product10 회귀를 수행하는 계획이다. 현재 **작성29/실등록0/실행0**이며 목록 차이가 나면 분모를 숨기지 않는다. 실제 FS/Windows symlink/lease 회귀 권한과 capability는 중앙 배정에 명시한다. 권한 부족을 자동 skip/pass로 바꾸지 않는다.

SDK/owned-process 영향 검사는 해당 담당 gate를 근거로 필요한 범위만 확정하고 전체 SDK/GUI/CLI·P0/protocol 반복을 자동 추가하지 않는다. 실제 SDK core 통합은 중간 poll Err 뒤 원 owner를 계속 관측하고 최초오류가 최종 Exited에 지워지지 않는지 별도 회귀로 입증한다. fake physical closure를 실제 Windows cleanup으로 확대하지 않는다.

원before/상속·비입력·원lock·BIN·index·원handle/receipt를 보호한다. 첫 실패는 원로그/코드를 보존하고 최소수리별도revision으로 구분한다. 자동 실제추론/예산증가/전체suite확대0. 마지막 raw receipt/원owner/PID·보호 대조 뒤 문서보다 먼저 heavy/shared를 명시 반환한다.

남은 제품 인수는 공개 operation/schema 및 profile/audio/destination admission, 실제 worker wire·native runtime/license/precision, 원 취소·강제정리/저장/재열기와 사용자 품질 검토다. 이 계획의 존재나 source helper 연결을 전체 goal 완료로 표시하지 않는다.
