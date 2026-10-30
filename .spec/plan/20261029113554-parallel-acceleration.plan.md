---
id: plan:kinekino/20261029113554
title: 병렬 가속 운영안
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 병렬 가속 운영안

2026-10-20. PM 및 읽기 전용 조사 에이전트 2명과 UI·SDK·Meari 담당의 조사 결과다. 구현·실행 완료를 뜻하지 않는다.

## 판단과 경계

기존 [자원 정책](resource-budget.md)은 저자원 소스 작업 병렬·heavy 실행 직렬을 허용한다. 소비 검증에 필요한 의존 소스 동결을 담당 전체 대기로 확대하지 않는다. 현재 UI가 단일 heavy를 유지하고, 다른 담당은 충돌 없는 계약·반례·영향 검토를 병렬 수행한다. 제품 구현은 기존 담당 세션, 서브에이전트는 조사·검토만 맡는다.

root Cargo는 Meari Core/Native/Paint를 live path dependency로 읽는다. Cargo.lock만으로 working source를 고정하지 못한다. 현재 root의 `git ls-files -- src Cargo.toml Cargo.lock AGENTS.md .gitmodules .spec`에는 `.gitmodules`만 나오며 Meari submodule도 dirty다. 이는 ignore 여부와 별개인 추적 상태다. HEAD 기반 worktree만으로 현 체크포인트를 재현했다고 주장하지 않는다.

## 병렬 작업표

| 담당 | 즉시 가능한 작업 | 구현 경계·검증 선행 |
|---|---|---|
| UI | 현재 P1a SDK/frame 소비 마감; Inspector 상태·캡션 host port·13화면 coverage 검토를 분리 | 현재 proposals/helper/CSS는 본 담당 단일 writer. P1b/P2 공개 계약 이전에 지원 완료로 표시하지 않음 |
| Meari | Boolean typed payload의 enum/parser/schema/Native/Web 영향과 Dialog controlled/focus 반례 검토 | Core 공통 계약을 먼저 고정. Boolean과 scalar capture 공통 parser는 같은 writer. Native/Web는 계약 고정 뒤 분리 가능 |
| SDK | S2 cleanup/ACK 보류, historical runner 취소, parent-ready/revoke broker 계약 검토 | 각각 media-jobs, historical_evidence, 신규 mpv parent 모듈로 구분. 공통 trait·mod 등록은 본 담당 통합. SDK13/main 동결 유지 |
| PM | 파일 소유표·의존 계약·검증 슬롯·인계 체크포인트 통합 | 같은 감사·로그 재집계를 각 세션에 중복 배정하지 않음 |
| AI | 보류 유지 | 추론·모델 다운로드·새 실측 없음 |

각 담당의 읽기 전용 서브에이전트는 질문 하나와 정확 경로, 변경 금지 범위, 근거·반례·권고 출력으로 제한한다. 호스트 전체 활성 조사 2명부터 운영하며 모든 세션에 동시에 최대 에이전트를 열지 않는다. 본 조사 2명이 끝난 뒤 다음 조사에 슬롯을 재사용한다. 동일 파일의 두 구현 writer는 만들지 않는다.

## 구현 병렬화를 여는 체크포인트

후속 관측: 사용자 Git 커밋 지시로 Meari 현재 기준선 `2c0883262a9eaf2d0120732f4f5cbd6603bd7178`이 커밋되고 clean 상태가 됐다. source344 재해시 delta[]이며 root gitlink `1d4fac9`로 인수했다. 따라서 최초 조사 당시의 dirty HEAD 제약은 Meari에 한해 해소됐다. PM은 기존 consumer checkout을 동결한 채 `runtime/development/meari-p1b`의 별도 worktree에서 담당이 Core P1b를 작성하도록 배정했다. 실제 생성/구현 완료는 아직 인수 전이다. UI heavy와 병렬인 소스 작성만 허용하고 추가 Cargo 실행은 금지한다. root 미추적 소스의 재현성 문제는 별도로 남아 있다. 새 submodule 커밋은 미push이므로 원격 재현은 완료가 아니다.

1. 현재 UI 소비를 마감하고 source pin·검증 receipt·owned 종료·heavy 반환을 인수한다.
2. 다음 생산자와 소비자의 입력을 분리한다. 현재 상태에서는 개발 담당을 검증된 별도 소스 복사본으로 옮기고 기존 소비 경로를 동결하는 방법이 변경량이 작다. Rust/Cargo/TSX뿐 아니라 include 자산·폰트·SVG·상대 workspace 의존 전체를 inventory로 검증해야 한다. runtime/cache/model/.git 전수 복사는 하지 않는다.
3. 추적된 로컬 체크포인트가 별도 승인되어 존재하면 worktree를 선택할 수 있다. 자동 commit·stage·현재 checkout 이동은 본 계획에 포함하지 않는다.
4. 소유 담당이 새 계약과 변경을 통합한 뒤 단일 heavy에서 집중 시험과 필수 fmt/test/clippy를 수행하고 소비 입력을 한 번 전환한다. 신규 격리/큐 자동화가 필요하면 Rust로 작성한다.

## 권장 순서

UI P1a 마감과 동시에 Meari P1b 계약 검토·SDK S2/parent 계약 검토를 병렬 진행한다. 이후 Meari P1b 구현과 SDK 독립 모듈 구현 준비를 서로 다른 입력에서 병렬화한다. Core Boolean 계약 뒤 Native Dialog와 Web listener 준비를 분리하되 공통 파일·manifest는 단일 통합 담당이 소유한다. UI는 안정된 공개 계약을 받아 Inspector와 route/transport 표시를 연결한다. 실제 mpv child 결합은 parent/revoke/cleanup과 geometry/focus 포트가 모두 준비된 후다.

## 자원과 측정

기존 UI first receipt는329494ms 중 Cargo Finished318초, frame-r3는51182ms 중39.11초가 컴파일이었다. 최초 실패에는 hidden callback 오인과 exhaustive match 누락이 있어 실행 전 계약·변경 영향 검토가 유효하다. 이 수치를 CPU time이나 전체 대기 비율로 해석하지 않는다. 원 근거는 `runtime/quality-review/studio-frame-sdk-r1/`와 Meari P1a 검증 디렉터리다.

현재 resource-runner는 개별 Job의 affinity/priority/자식 대기와 receipt를 제공하지만 호스트 전역 heavy lease/queue는 없다. 두 실행은 같은 첫 두 CPU(mask3)를 공유할 수 있으므로 heavy2를 즉시 켜지 않는다. 전역 lease·명시 CPU 예산·queue timestamp·peak 측정과 고정 warm-cache 비교가 마련되어야 별도 제한 실험을 검토할 수 있다. 가속률은 아직 측정하지 않았다.

같은 workspace/feature/crate-type/target 문맥의 캐시를 재사용하고 집중 검사를 묶는다. target 삭제·전역 compiler flag 변경·불필요 cold build는 하지 않는다. source pin이 같고 영향이 없는 전체 회귀를 반복하지 않되 프로젝트 필수 검사는 최종 통합 입력에 수행한다. timeout/terminal=false는 슬롯 반환이 아니다.

## 후속 작업

- [ ] 담당별 계약/반례 조사 결과를 각 소유 정본에 반영하고 단일 writer 경계를 확정한다.
- [ ] UI 최종 소비 반환 후 별도 개발 입력의 전체 의존·자산 inventory 및 소비 전환 절차를 준비한다.
- [ ] SDK S2와 parent broker 중 먼저 고정할 계약을 담당이 제안하고 구현을 배정한다.
- [ ] 필요 시 기존 Rust resource-runner에 전역 lease/queue와 자원 측정 기능을 별도 업무로 배정한다.

공식 참고: [Cargo build cache](https://doc.rust-lang.org/cargo/reference/build-cache.html), [Git worktree](https://git-scm.com/docs/git-worktree). 새 캐시 도구 도입이나 전역 설정 변경은 수행하지 않았다.
