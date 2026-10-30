---
id: evidence:kinekino/20261029114426
title: F1 실제 프로세스 중단·예약 경합 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# F1 실제 프로세스 중단·예약 경합 검증

2026-10-26. 원 실제9나 정상GUI를 실행하지 않고 독립 synthetic user store와 현재 Cargo libtest의 내부 child helper를 사용했다. production fault switch를 추가하지 않았다. helper는 parent가 준 유일 root/mode만 사용하며 부모가 직접 만든 Child handle만 종료한다. 기존 readonly/CAS/publication 정책과 원 실패·증거는 유지한다.

## 실제 관측

runtime/verification/source375-datalab-b0-r1-logs/126-review-source-process-tests: lib review_source 20pass/0fail/1ignored/1412filtered, exit0/65661ms. 종전17에 실제 프로세스 시험3을 추가했다. ignored1은 parent가 --exact/--ignored로 명시 실행하는 내부 helper이며 별도 완료1건으로 계산하지 않는다. 부모는 ready 관측·try_wait로 child 생존을 확인한 뒤 종료하고 wait한다. startup deadline15s와 RAII 종료/wait를 둔다.

| 중단 지점 | 실제 fixture / child PID | 결과 |
|---|---|---|
| 출판 완료·readback 뒤 테스트 응답 전 | runtime/build-cache/tmp/review-source-files-2132-0 /14040 | parent 강제종료 exit1. child-receipt.json과 committed envelope를 보존. 새 store에서 동일 receipt replay 및 같은 body publish 재호출이 원 receipt 반환, envelope1개 유지 |
| 완성 temp sync 뒤 최종 link 직전 | .../review-source-files-2132-1 /22480 | parent 강제종료 exit1. .export-22480-0.tmp를 보존. 새 store는 NeedsInput으로 거절하고 원 temp bytes 동일. 빈 history로 초기화/덮어쓰기/자동삭제하지 않음 |
| case write reservation 보유 중 | .../review-source-files-2132-2 /10344 | child 생존 중 다른 프로세스의 reader/writer 모두 RevisionConflict. 종료·wait 뒤 예약 재획득, 기존 replay없음 확인 후 publication 및 새 read에서 replay 성공 |

각 fixture의 child.stdout/stderr/ready/parent-termination.json과 transaction 파일을 보존한다. 위 mapping은 실제 디렉터리의 child receipt/고유 tmp/parent 종료 PID와 시험 결과를 함께 확인했다. 마지막 두 행의 ...는 같은 <원 호스트 저장소>/runtime/build-cache/tmp 부모다.

127 fmt --check exit0/2457ms.128 root --all-targets --features native-proposal-window -- -D warnings exit0/18892ms. 기존 paint deprecation warning을 원로그에 보존했다. locked/offline/jobs1/CPU2/BelowNormal/C native/tmp와 기존 toolchain 유지.

## 범위와 미검증

이번에는 물리 OS process 종료로 파일 reservation 해제와 임시/committed 파일 재열기를 검증했다. application/CLI transport의 실제 ACK 유실 주입이나 OS전원손실·NTFSdirectory flush 보장은 아니다. 출판 완료 뒤 응답 전 테스트 phase와 실제 네트워크 응답 유실을 혼동하지 않는다. disk full/권한 변경/root 교체와 다른 media custody/실제9 연결·Verified PTS·AI/GUI는 여전히 별도 미완료다.

스킬: modootoday/software-qa/test-fixture-and-mock-build SKILL.md SHA a393f15faf68fee96c187509d0f22ed72987d8d1929afaf782d43bca80f3f100의 실제 자원 보유/해제/두 번째 획득과 mock 한계 구분을 이어 적용했다. 새 Rust helper는 cfg(test)이며 일반 cargo test에는 ignored이고 parent가 소유한 실행에만 사용한다. 모든 child 종료 후 원 fixture를 삭제하지 않는다.

| 코드 | SHA256 |
|---|---|
| src/infrastructure/review_source/process_tests.rs |8D9ADD068F361F2154EA2563BAD314811608D9D5C7E27CAEEE57CF35DCE6A42A|
| src/infrastructure/review_source/tests.rs |97E80D4D8F06B427F13E598B85218EECDB4C85E5420962BB19D321D1B2356D92|
| src/infrastructure/review_source/mod.rs |0C738DDD8FC0F8085A8D47ADCE4466D1AC227C50E63297BF92C2A8A67ACAA311|

다음은 RD별 잔여 정합성 점검과 최신 source/producer pin의 전체190 회귀다. 이전8A 기준190과 정상125 bins의 근거를 현재 전체시험 완료로 승계하지 않는다. 원 WIP/소유권·seal·실제9·추론/다운로드/수동GUI/WARP/push 보류를 유지한다.
