---
id: evidence:kinekino/20261029114606
title: CLI host 종료 배선 검증 · 2026-10-21
status: active
created: 2026-10-29
updated: 2026-10-29
---
# CLI host 종료 배선 검증 · 2026-10-21

상태: 중앙 단일 gate를 마치고 heavy/shared snapshot을 PM에 명시 반환했다. 인수 근거는 **r6 실제26 + r5 격리 parent1 = 고유27**이며 반복과 child3case를 중복 합산하지 않는다. r6에서 실제30초 subprocess를 다시 실행한 것은 아니다. root 통합/commit은 PM 소유다.

PM 제품 인수 commit은 **f286d813ac23cbf7710a18741d4fa59045ef4e0e**다. before11/최종 bytes·보호·종료 근거를 직접 대조하고 정확11(+1583/-172)을 커밋했으며 index empty다. 이후 UI Viewer9r2가 heavy를 소유한다. 추가 Cargo0이고 원 native 스킵 소비는 SDK C1 공통 계약 인수 후 연결할 별도 범위다.

## 입력과 격리

base root4568aecac967a52dc2c6a9a8db755d42f844ecae/Meari98f6a5c4c72ad07a40ac0eae7f7f113926609fae다. B15/UI7/LocalHost6/A7 인수 source를 포함한 반환 expected1321을 대조해 delta0을 확인하고, CLI 비중첩 snapshot1315/root1991·before11(기존6/신규5)을 보존했다. shared snapshot에 정확11만 현재 mtime으로 합류했다. root live/index는 변경하지 않았다.

- r5 source11: E0AA336BA8E43C131B97B4CE71A198715AAE2A40A3BFFE587A4A38D51420C310. integrated11:47671D7ED82C27831493E2CB96044525CFDCB6C8B46B59865C379A63E4F040F7.
- 최종 r6 source11: **247F238EA0C0129EA8BF0268C1BAE9E6E3BCD417BA7163512F0555DBA6A67339**. integrated11-r6: **257FB0E2CC80E706D248605B0A388F8FDF3DE40753DA323B50CEA80C18990EAC**.
- r6는 carrier/driver/tests3의 Readiness enum 및 match만 변경했고 나머지8 SHA는 불변이다. heap/allow/정책/DTO 추가0이다. 하지만 격리 경로도 advance/try_ready를 통과하므로 '실행 경로 전체가 불변'이라고 주장하지 않는다.

원 파일은 `runtime/verification/local-host-cli-wiring-r5/`에 보존했다. 지정 EEA9 runner/conservative2·2CPU affinity3·BelowNormal·descendant 예산을 사용했다. Cargo는 snapshot cwd와 절대 manifest 및 기존 project-9dc296c4/native target, offline/locked/-j1/-p kinekino-player/thread1을 명시했다. broad test --list와 모델/브라우저/미디어/GUI 실행0이다.

## 실제 결과와 pin 범위

| 범위 | source / 실제 결과 | 원 receipt |
| --- | --- | --- |
| 공통 종료+runner 일반 | r6 16pass/2ignored | regular-r6 9842ms0 |
| 기존 LocalHost | r6 9pass | local-host-r6 15918ms0 |
| 실제 제품 CLI job lifecycle/EOF | r6 1pass | job-cli-r6 9226ms0 |
| 격리 boundary parent | r5 1pass, child3case 직렬 | isolated 31093ms0 |
| default all-target strict | r6 실제 Checking/exit0 | 2296ms |
| native-proposal-window all-target strict | r6 실제 Checking/exit0 | 2858ms |
| fmt --all -- --check | 최종 exit0 | 2925ms |

r5의 공통11/59488ms·runner5/1045ms·LocalHost9/48234ms·제품 job1/19703ms도 성공했으나 r6와 합산하지 않는다. 실제 Compiling/Checking와 정확 시험 이름을 source SHA/mtime 및 stdout에 연결했다. r6 일반 회귀에는 실제 원 Pending 관측·가짜 view 회수 거부·D 보존·BrokenPipe 첫 오류 보존이 포함된다. 타입 수리의 기계적 동등성/이 회귀와 r5 실제 격리 근거로30초 반복을 생략한 중앙 판정을 기록하며, r6 격리 미재실행 한계는 유지한다.

## 실제 비정상 경계와 EOF

r5 격리 binary는 실행 당시 SHA31C55497CF5D0AE533DBD4BB63DECB46F18780123C08F48F059240325B18981C/11010560bytes로 식별했다. current_exe+ignored exact child/전용 phase·parent marker로 선택하고 다음을 한 parent에서 순차 인수했다. 이 binary는 제품 player-cli.exe와 다르며, 당시 SHA metadata를 보존했다. 이후 같은 cache 파일명은 r6 재빌드 대상이므로 현재 경로가 원 r5 bytes라고 주장하지 않는다.

| child case | 원 PID / 종료 | 확인한 사실 |
| --- | --- | --- |
| timeout |56652 / exit8,elapsed30045ms,관측30005ms | 실제 production30초·pipe joined/sdk pending, D Drop0·cleanup/cancel false, stopped/lifetime 성공0 |
| panic |616 / exit6,47ms | 실제 accepted SDK owner panic, 원 PermissionDenied/details 유지·sdk panicked, D Drop0·성공 출력0 |
| job EOF |37860 / exit0,55ms | test binary 안의 production job handler·원 pipe/sdk joined/idle·성공 stop |

watchdog=false/wait_issue=null3개이며 강제 cleanup은 실행하지 않았다. parent의 원 Child handle wait/exit로 프로세스 종료를 확인한 것이며 timeout/panic의 SDK join/의존 cleanup 성공을 증명하지 않는다. node/browser/mpv 손자 프로세스를 시작하지 않았다. actual product CLI는 아래 별도 파일/회귀로 확인했다.

## 제품 CLI와 원 DTO

최종 debug player-cli.exe는29405696bytes, canonical content_sha256 **5d17ae2dad8f54f89f7bdb442919ce1acfe7c4e741a2d5fa12f9fe5d307e0e6d**다. cache와 `artifact-r6/player-cli.exe`, 기존 Rust fixture가 복사/검증한 `job-evidence-r6/lifecycle/player-cli.exe`의 입력이 동일하다. canonical hash는 cli-artifact-r6.json 한곳에서 환경 입력으로 소비했고 Rust fixture의 source/expected/copy 대조를 통과했다. 원 r5 CLI3aca033c…1eeb5b와 artifact/는 별도 보존했다.

실 ready/stop은 `job-evidence-r6/lifecycle/logs/normal-ready.stdout`, `normal-stop.stdout`이다. stop은 `local_host_shutdown={pipe:joined,sdk:joined,dispatch:completed,dispatch_poisoned:false}`, dependency receipt는 completed_total2/cancelled_total0/interrupted_total0이다. 원 Host::stop의 Child.wait/성공 단언을 통과했다. ready의 PID/start token과 forced/restart 증거는 job-host-identities.json·기존 fixture logs에 연결했다. forced-stop은 시험의 명시 소유 child 강제 종료이며 양 join 성공으로 읽지 않는다. completed는 원 SDK 함수 반환이지 업무 성공 보장이 아니다.

기존 Response를 수정하지 않는 CLI Serialize wrapper와 additive DTO, 원 첫 오류/추가 진단 분리, 최상위 owner 유지 process exit를 소비한다. 공통 Read/Write 주입이 실제 native/provider/playback 명령들의 모든 외부 환경을 실행한 것은 아니다. 전체 종료 bounded를 주장하지 않으며 동기 dependency shutdown은 관측30초 밖이다.

## 원 실패 보존

1. 근거 보존용 r5 job 재실행은 JOB_CLI_SHA256의 대문자 문자열 때문에 fixture58에서1890ms/101이었다. 실제 digest bytes는 같고 CLI 실행 전 설정 실패다. 원 최초 제품 job 성공은 유지했다. 소문자 input JSON·새 evidence root로 정정한 job-cli-evidence-r2는8776ms0이며 원 r5 stop JSON을 남겼다. 이 반복1pass는 고유 합계에 더하지 않는다. 기존 Rust fixture 요구를 재사용했고 별도 검증 스크립트나 제품 수리는 없다.
2. r5 default strict는 carrier Result Err 크기 경고로1998ms/101이었다. 원 source/로그를 보존하고 r6는 Ready/Incomplete 전용 enum으로 원 owner를 옮긴다. 불완료는 오류가 아니라 소유 상태이며 추가 heap/경고 allow0이다. r6 독립 소스 재감사·실 strict/일반 회귀로 수리를 인수했다.

이전 source-only r3의 eprintln panic/P2, carrier r1의 caller view를 믿은 Ready/P2 이력도 각 원 pin/handoff에 남아 있다. 이번 r6를 그 원 실패와 섞지 않는다.

## 보호·종료·반환

final-source11/snapshot1315/root1991/B15 differences는 모두[]다. UI7/LocalHost6/A7 비중첩/Meari405는 인수한 snapshot 보호에 포함된다. 원 exec handle 회수 뒤 다음 실행을 시작했으며14 receipt 모두 terminal,12 exit0·위 원 실패2는101이다. carrier/CLI production 코드를 live root에 적용하거나 stage하지 않았다.

기록한 숫자 PID는220개다(CLI 로그186 records, 격리3, persistent host6 및 runner/child 포함). 초기 numeric 후보11은 마지막 gate receipt 종료 UTC17:11:09.2655432 뒤 KST02:12:42+에 생성된 새 cmd/conhost/Codex cua_node와 그 자식이었다. 원 CLI5개 code0와 현재 creation/path/parent를 대조한 pid-reuse-sidecar.json·initial raw를 보존했다. 후속 numeric 후보6이 있어도 원 identity owned[]이며 타 앱을 kill하지 않았다. 모든 숫자 PID가 부재라고 주장하지 않는다. 원 CLI의 절대 creation은 전부 포착하지 않았고 원 종료 근거는 Child receipt/fixture wait다. 격리 temp prefix 조회는0이었으며 잘못된 hash 입력의 빈 persistent root는 원 이력으로 남긴다.

heavy/shared snapshot은 PM에 명시 반환했고 이후 추가 Cargo0이다. 모델/다운로드/worker 빌드·실브라우저·실미디어·GUI/VM/DPI0이다. 원 skip 권한/SDK C native 인가 공백·GUI main/audio owner 연결은 이 gate로 완료되지 않는다. ADR·[wire 정본](../../player/application/local-host-cli-shutdown-wire.md)·[gate 계획](../../../plans/player/local-host-cli-wiring-gate.md)을 따른다. modoo.today 계획/변경 검증 본문·참조를 적용해 관측·정책·미검증을 구분했다.
