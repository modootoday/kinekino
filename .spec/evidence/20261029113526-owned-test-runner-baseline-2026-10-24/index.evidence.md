---
id: evidence:kinekino/20261029113526
title: baseline outer Rust driver — 타입·기본5 실제 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# baseline outer Rust driver — 타입·기본5 실제 검증

중앙 배정 root `dd27fd6c1bf33cab9f52cac949a180a3b3f309ef`, Meari `b532fe0eed06d8526b755b11bc36416070244ef7`, root lock `8F2C1D56C8A2C22D36D9E57A285D727C7EE9C7D2D1160902B66CC6630B042C67`에서 진행했다. 입력은 private r2 source8 `5FCE21F0017D869D7984ED07558B85169123E06175C0A16773933C518E7C9946`, before8 `8628FE7A8C3006A2290BF1B5FDECAF86F20DB9A5672ECB625C9915ACC065B0CF`다. 원 r1 CAEE와 B6B16/runtime13은 불변이다. canonical 도구 채택·commit·push0이다.

## 실행·의존성

원 S/T/runner conservative2·CPU2/mask3/BelowNormal/descendant Job/offline/j1/thread1을 사용했다. 단계별 fresh 자원·writer0를 확인했다. 01 metadata는 terminal0이지만 생성 lock373A의 외부33 중 syn2.0.119가 root 범위 밖이라 뒤 단계를 중지했다. 원 metadata와 lock을 보존했다. PM 추가 배정에 따라 cached archive checksum을 확인하고 standalone에만 offline update exact1(syn2.0.119→2.0.117)을 수행했다. root lock·source 손편집0/download0이다.

정합 lock SHA256 `0B2562E8D295AEE5602407228C3A5A21DBD5E478299A685CE94780FADE49226B`, 외부33의 name/version/source/checksum 모두 root lock 범위 안이다. 원373A와 정합0B는 별도로 보존한다. 원 metadata는 정합 전 graph이며 최종 graph로 혼동하지 않는다. 최종 lock·05/06 Cargo JSON의 baseline owned-process path/빈 feature 및 driver producer를 함께 사용한다.

03 cargo fmt --check, 04 offline/locked all-target Clippy -D warnings, 05 정상 build, 06 unit no-run 모두 terminal0이다. Cargo JSON의 profile.test=false/true를 구별하여 고정했다. 07 전체 목록13, 08 ignored 목록8을 작성13과 정확 대조했다. 09~13 exact 순수5 각각1pass/0fail/0ignored다. receipt13 모두 terminal0이며 lock 입장이탈은 별도 검증 실패로 보존한다.

| 고정 artifact | SHA256 | bytes |
|---|---|---:|
| 정상 driver |58690FF6585458DE84FF469009F92A8A576B9AF1BD3BB33B6B462E219EF4836F|914944|
| unit BIN |0057C9E980CF679890489227A645BAA7108F5CFACA58FA4156761878CFE8608D|1573888|

순수5는 고정 typed 입력/중복·비로컬·예산 거절, 정확 출력·should-panic 구분, 빈 선택·틀린 이름·중복 summary·ignored 거절, 첫 sink write 실패 latch/재시도 성공 금지다. OS 인수5와 child 자극3은 등록만 확인했고 실행0이다. 정상 driver 자체의 process/OS 동작과 late verdict/후속 원 owner 관측은 이번 통과로 입증하지 않는다.

## 보호·협업 예외

사전 root 추적1371/library493, 기존 S5622/T exe576, fixed104/alias4/known2를 보존했다. root/index/lock4·library493·S기존5622·T기존576·fixed104·alias4·known2는 검사 범위의 bytes/UTC mtime 차0이며 alias4 FileID도 같다. S 신규9는 승인 source8+정합 generated lock1이다. 새 T exe5는 정상 root/deps alias2+unit1+serde_core build-script alias2이며 Cargo producer와 동일 bytes 대응을 보존한다. 새 고정 BIN2도 producer SHA와 일치한다.

root 감사 중 SDK canonical 초안3의 일시 변경을 실제 관측했다(viewer_callbacks/codec.rs, mod.rs, viewer_read_actions/mod.rs). PM이 SDK 작성/복원에 귀속했고 후속 재조회에서 원 before SHA/mtime로 복원됨을 확인했다. Media 복구/mtime 변경0이다. 중간 before/after/delta와 최종 재조회를 보존하며 이를 전체 기간 root 불변으로 표시하지 않는다. 나머지1368과 협업 예외3을 구별한다.

known2의 이번 현재 SHA는10B560이며 원 bytes 사본2를 보존했다. 과거 cache4709 producer 미귀속2는 계속 미해결/all_cache_protected=false다. 이번 기존 exe576 감사와 전체 캐시 인증은 다르다. clean/새 target/광범위 touch/임의 복구0이다.

## 의미와 잔여

baseline 입력 파일 pin은 공급된 bytes 검증이며 실제 컴파일 출처의 자체 인증이 아니다. 외부 sourcegraph23+source8/최종 lock/Cargo JSON/고정 BIN을 별도 결속한다. child alias guard는 실제 호출 증명이 아니다. durable verdict는 저장 전 관측이고 저장 뒤 cutoff의 late error는 최종 Outcome/passed=false·exit1에 반영된다. 이 차이와 first-error 보존의 새 회귀는 미실행이다. 미종료 원 carrier의 leak/같은 owner 후속 관측은 전체 driver hard deadline이나 leak-free cleanup 보장이 아니다. 동기 file/setup/sync를 선점하지 않는다.

modoo.today `modootoday/software-qa/test-fixture-and-mock-build` 본문 SHA256 `a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100`(추가 필수 참조 없음)을 실제 fixture/인수 분리·원 자원 보존·실측 없는 성능 주장 금지에 적용했다. Rust/SRP/SDK·CLI 우선을 유지했다. 원자료는 `runtime/verification/owned-test-runner-r2/`의 admission/lock-admission-r2/cargo-artifacts/fixed-bins/registration/runner-receipts/protection-summary/root-collaboration-exceptions3/slot-return이다.

후속은 [별도 OS·회귀 제안](../../../plans/player/production/owned-test-runner-os-followup-2026-10-24.md)이다. 신규 추론·다운로드·native worker 빌드·B overlay·GUI/OS 시험·새 회귀 구현은 별도 배정 전0이다.
