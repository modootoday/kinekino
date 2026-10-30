---
id: evidence:kinekino/20261029113513
title: 별도 CPU 구역 상속 진단
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 별도 CPU 구역 상속 진단

2026-10-21 PM의 한정 실행 진단이다. 제품 기능·컴파일 속도·heavy 병렬 안정성 시험이 아니다. [현재 권한](../../plans/player/execution-leases.md)과 [자원 계획](../../plans/player/resource-budget.md)을 따른다.

## 실제 관측

원 근거는 `runtime/verification/pm-affinity-inheritance-r1/{ram.json,disk.json,vram.txt,launcher.json,receipt.json,stdout.log,stderr.log}`다. live free RAM27,816,820KiB·선택 RTX2070SUPER VRAM5,705MiB·C173,055,598,592bytes를 관측했다. 진단 launcher100160의 원 mask65535에서 자기 프로세스만 mask12로 한정하고 기존 canonical runner EEA9의 `rustc --version`을 실행했다. 종료 시 자기 mask를 원 값으로 복원했다. 다른 사용자/컴파일러 프로세스 affinity는 변경하지 않았다.

runner85184/원child103344 receipt는 conservative2·selected_mask12·actual affinity12·논리CPU2·priority16384(BelowNormal)·descendant Job·terminal true·exit0·elapsed973ms를 기록했다. 원 shell 실행도 exit0으로 회수했고 stdout은 rustc1.99.0, stderr는 비었다. 이 elapsed는 버전 질의이며 빌드 속도 비교값이 아니다.

## 확인한 범위와 미완료

기존 runner가 항상 mask3으로 고정되는 것은 아니며 자기 launcher의 허용 mask12를 상속하면 실제 child에도12로 제한할 수 있음을 확인했다. 기존 Meari의 mask3 실행과 구역이 다르지만 무거운 부하를 동시에 실행한 것은 아니다. 이 진단은 RAM 입장 검사를 runner가 수행했다는 근거도 아니다.

현재 heavy 단일 슬롯은 유지한다. 두 CPU2 작업을 병렬 배정하려면 source/target/protection 집합의 교집합·실제 결합 메모리 피크·page pressure·사용자 반응·원 cleanup/receipt를 별도 좁은 시험으로 확인해야 한다. 특히 다른 gate가 보호 중인 S/native T를 조용히 수정하지 않는다. 새 mask 프로파일/도구·Rust 구현·Cargo/build/test·모델·GUI·전역 설정은 추가하지 않았다.

## 별도 읽기 조사: Native 빌드 선택 범위

후속 기록 조사에서 record-radio r6의 정확 `40-r6-native-build.receipt.json`은76359ms/exit0이었다.595131ms는 이전 HookModule gate이며341초는 이번 exact r6 근거로 확정하지 않는다. direction10은 관측 시점 terminal=false여서 총 시간을 아직 비교하지 않는다. r6 compiler-artifact329중 fresh322/재생성7, direction 관측328중 fresh321/재생성7이며 양쪽 canonical 경로는 같고 blitz-paint는 재사용됐다. direction에서 Core/Native가 재생성된 차이는 있지만 전체 캐시 무효화·경로 변경으로 지연을 단정하지 않는다.

integration 선택 외 일반 BIN2(`meari-native`, `meari-kitchen-sink`)가 test=false/fresh=false로 생성되는 비용 후보를 확인했다. [Cargo integration target 규칙](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#integration-tests)에 따라 --test 또는 test=false만으로 일반 BIN 빌드를 제거하지 못한다. 이후 Native library와 executable을 책임별 별도 workspace package로 나누는 후보를 제공자에 인계했다. workspace/배포/CLI 회귀와 기존 public API 소비 시험의 동등성 검증이 필요하며 현재 gate를 변경하거나 BIN/시험을 비활성화하지 않는다. 정량 개선율은 미측정이다.
