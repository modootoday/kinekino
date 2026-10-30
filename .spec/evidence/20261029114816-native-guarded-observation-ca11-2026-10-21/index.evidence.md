---
id: evidence:kinekino/20261029114816
title: Native guarded Observe/Submit A11 Rust gate · 2026-10-21
status: active
created: 2026-10-29
updated: 2026-10-29
---
# Native guarded Observe/Submit A11 Rust gate · 2026-10-21

root35d257c7d9db1d055afd6f2341b84df74127c956/Meari98f6a5c4c72ad07a40ac0eae7f7f113926609fae와 UI13 r2/자막8/harness6/CLI11/B15 반환 snapshot에서 **A11만** 검증했다. B7/C7·skip 소비·production 활성화는 포함하지 않았다. 실제 IPC server/mpv/media/browser/GUI 실행0이다. 근거 폴더는 `runtime/quality-review/sdk-guarded-ca11-r2/`다.

## 입력·합류·수리

| manifest | SHA256 |
|---|---|
| 원 A11 r2 source11 | D1C2FCE4DF09CEB29CBDC75394FE95AACA4709AE3725F83458914E9C602CF18B |
| 최종 r3 source11 | E3BB2D8D194F4ECDF5AD507D8BF1DF80424D26BA28F0B620EB41C5EEACA1C34F |
| r2→r3 delta2 | 636E42C64E879502FCEED26D09FBFB567FD1DEB0762889335CD811EA12AA484F |
| integrated11-r3 | CC5B95A4B9FD4B886D65EFE0E090118F90F5E20620F4F6AAE425C6020CB20B44 |

before기존9/부재2를 보존했다. 기존9는 원 B15 base와 EOL 정규화 차이0을 확인하고 additive patch check/apply0 뒤 frozen bytes/현재 mtime으로 고정했다. source11 외 protected root1376/snapshot1336은 최종 hash delta0이다. UI13/자막8/CLI11/B15 비중첩/Meari 소스도 보호에 포함된다.

최신 UI protected1340과 integrated13에는 기존8 경로가 중복됐다. 최초 단순 연결 대조의 차이8은 구버전 중복이며 원 기록을 보존했다. 최종 UI13 우선 path union 고유1345를 **합류 전에** snapshot과 대조해0을 확인했다. A11 기존9를 제외한 보호는1336이다.

최초 실제 compile/test는69807ms/exit101,34pass/1fail 및 unused import 경고였다. fixture가 `scoped(scope,8)`로 마지막 HostScope Arc를 넘겨 소멸시킨 것이 원인이었다. r2 pin/실패 로그·receipt를 보존했다. r3는 scope.clone으로 fixture 소유를 유지하고 명시적 drop 후 SourceExpired assertion을 추가했으며 queue.rs의 unused import만 제거했다. 제품 정책·assertion 완화·지원 변경0, 다른9 byte 불변이다.

## 실제 실행

| 검증 | 결과 |
|---|---|
| default guarded protocol/fence | 실제 재컴파일 후35pass,28011ms0 |
| native guarded protocol/fence | 실제 재컴파일 후35pass,112448ms0 |
| 양 구성 공통 영향 | application guard6/projection3/control 기본5+옵션1/수신 관측9/closed IPC worker join1 |
| native 추가 영향 | router12/binding6 |
| default all-target strict Clippy | 실제 Checking,28997ms0 |
| native all-target strict Clippy | 실제 Checking,42895ms0 |
| cargo fmt --all -- --check | 2998ms0 |

actual-test-names.json의 실제 성공 이름 기준 default60/native78, 중복60을 제외한 **고유78pass/0fail**다. guarded35=기존B19 이관+신규12+기존fence4다. 최초 실패34나 두 구성의 같은35를 신규 성공으로 더하지 않는다. 0selected/목록 성공0이며 전체 suite는 반복하지 않았다.

runner EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D/conservative2/affinity3/BelowNormal/descendant Job을 사용했다. 절대 manifest/기존 native target/snapshot cwd/offline/locked/-j1/thread1을 고정하고, 매 원 handle terminal 회수 뒤 다음 own 실행을 시작했다.

## 종료·바이너리

receipt20은 전부 terminal이며19개 exit0/원 실패1개 exit101이다. 기록40 launcher/child PID는 모두 부재, 최종 own/evidence process0, index empty다. 최초 최종 시스템 조회에서 기록 PID와 다른 cargo17700/rustc114472(parent94312,생성03:18:45KST)를 일시 관측했다. command line 재조회 전에 사라져 출처/부모 identity는 미확인이다. 임의 종료·타 프로젝트 확정·own 실행 귀속0이며 transient-process-observation.json에 보존했다. 다음 조회 final-owned.json은[]다. 시스템 전체 Cargo0를 별도 반환 조건으로 추가하지 않는다.

default libtest94761984bytes/SHA4F955D3892F8231B1ECB8BA1B6FCC8D5480F269D06C36E72CC8EECF0B8A4190D, native156298752bytes/SHA3C1D2BB0BBFBA84E2163EEAE4CDCC9DE7AAE75C5BDC03DD73AF6A4AAC047C8D3다. cached CLI29416960bytes/SHA96B2A2A99F592FBFA2DDFC7D91E627C48B06332C30D9B4051CF8EB9EFFDBB03E는 기존 캐시 식별이며 이번 CLI 실행 증거가 아니다.

## 판정

Observe8→private view/opaque guard→application domain 평가→같은 guard Submit1, 공유64slot/conditional generation/final deadline을 Rust protocol seam에서 검증했다. deadline 회귀의 시작 신호는 실제 mutex 대기 위치까지 증명하지 않는다. 실제 서버/미디어의 sampled entry/landing/품질은 미검증이다.

B7 host/foreignscope와 C7 parent/read-context는 별도 후보이며 이번 gate 미포함이다. production issuer=false, default constructor 진단false/guardNone, skip 소비/CLI 결과 wire 미완료를 유지한다. 중앙 인수·commit은 PM이 수행한다.
