---
id: evidence:kinekino/20261029114729
title: Trusted standalone SDK·동일 view skip union12 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Trusted standalone SDK·동일 view skip union12 검증

2026-10-21. 기준 root `82f605d1d2ed73d8c4143c77f0d1e079f08add23`. SDK 담당의 단일 heavy/shared 배정으로 실행했고 운영 검증 직후 PM에 명시 반환했다. SDK 담당은 root 소스·index·commit을 변경하지 않았다. 이후 PM이 원 source/before/길이를 재대조해 SDK7 `07251ea`, skip5 `b89682ab44f9bb31dbf263890a8c95cb5d53a480`로 분리 채택했다고 회신했다. 새 [원64 export 생산자 계획](../../../plans/player/native-seek-row-export-producer.md)은 이 gate 밖의 문서 전용 후속이다.

## 입력과 보호

동결 SDK7 `83C2F05F9BE65DE4714218D81191A11ABFF2741574707C4C3A52739EF7FC62A1`과 skip5 `ED3A961B41B50B2A0A77BC6CF70D626574E18560A6C4F59AFDFD0479EB75C692`를 합류했다. 원 source·before와 부문별 수정 책임을 유지했다. 경로 중복0, 기존10/신규2, 수리0이다.

합류 manifest는 `runtime/coordination/sdk-native-skip-union12-gate-plan-r1/union12.json`, SHA `7FC0F4DFFA341C0103C5AECA265A2D3C29D7309A3A6248AEA7C4802D33DC93B5`다. 계획 root17d 관측과 별도로 최신 root/snapshot의 before12를 확인했다. UI protected-snapshot→최종 integrated21-r3→library408 순으로 path-key를 덮었다. library 상대 경로에는 `packages/meari/`를 한 번 붙였다. 원 UI 보호 manifest의 이전 Meari 값과 최종408의 차이는 입력 우선순위 문제였으며 최종 union 보호 대조는0이다.

증거 디렉토리는 `runtime/quality-review/sdk-native-skip-union12-r1/`다. `before12.json`, `protected-root.json`, `protected-snapshot.json`, additive check/apply한 `union12.patch`, 원 frozen bytes와 일치하는 `integrated12.json`을 보존한다. Cargo/lock·UI·Meari408·비입력 파일을 후보 checkout 값으로 대체하지 않았다.

## 실제 결과

fixed resource runner의 conservative2, affinity3/2 logical CPU, BelowNormal, descendant Job을 사용했다. Cargo는 기존 snapshot/target에서 offline/locked/-j1, 시험은 thread1이었다.

| 검증 | 관측 |
|---|---|
| default 첫 compile+lifecycle | 97,704ms, exit0, 실제19 |
| 같은 default binary의 나머지 SDK | 실제106 |
| 같은 binary의 skip 전체 필터 | 실제26, 중복 분할 실행0 |
| 고유 시험 합계 | SDK125+skip26=151, helper2 포함/행동149 |
| default all-target Clippy `-D warnings` | 31,881ms, exit0 |
| native-proposal-window all-target Clippy `-D warnings` | 60,747ms, exit0, 타입 검사 |
| workspace fmt check | 4,939ms, exit0 |

실제 이름은 `actual-tests.json`과 각 stdout에 있다. lifecycle19에는 신규 factory6, skip26에는 신규 consumer10이 포함된다. 추가 소스 수리·재컴파일0이다. native 실제 시험을 실행하거나 default 결과를 native runtime 통과로 승계하지 않았다.

실제 stderr가 가리킨 default libtest는 `kinekino_player-06c910fefd49f93f.exe`, SHA `B8AA9C331964A49C75CCE234A1846AFF53FBC3C0793E907BEE68BD40CCC783A1`다. 경로·크기·mtime/hash는 `default-binary.json`, `default-binary-sha.json`에 있다.

## 반환과 한계

`closeout.json`: 16 receipt 모두 terminal/exit0, 기록32 PID 부재, 본인 실행 handle0, root/snapshot 보호 delta0, root input12 원 before 유지, snapshot input12 원 source 일치, binary 불변, root index0. 이 운영 확인 직후 상세 문서 작성에 앞서 heavy/shared를 명시 반환했다.

생산 registry false·legacy dispatch·지원 flag·CLI wire는 유지한다. 실제 mpv·provider/browser·GUI·positive workspace·CLI ready/EOF·미디어 재생/skip 종단은 미검증이다. 신규 factory의 명시 권한 경계와 consumer의 동일 view/guard/submission 회귀가 실제 미디어 품질 승인을 뜻하지 않는다. 장기 ticket export/wire·원64 row 보존 계약은 별도 후속 범위다.
