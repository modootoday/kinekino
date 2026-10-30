---
id: evidence:kinekino/20261029114702
title: LocalHost 명시 종료 집중 검증 · 2026-10-21
status: active
created: 2026-10-29
updated: 2026-10-29
---
# LocalHost 명시 종료 집중 검증 · 2026-10-21

상태: 중앙 배정의 집중 gate를 마치고 heavy/shared snapshot을 PM에 명시 반환했다. 고유17pass·양 all-target strict/fmt0이다. 기존 caller의 명시 Pending 연결·CLI carrier 배선·전체 제품 완료를 뜻하지 않는다. 이후 추가 Cargo는 수행하지 않는다.

중앙 제품 인수 commit은 **324c3e0b41ae30aad8349bd161f80a14d45e0144**다. PM이 before6와 최종 bytes를 직접 대조하고 정확6(+517/-59)를 커밋했으며 index empty로 반환했다. 이후 UI가 단일 heavy를 소유한다. commit은 아래 LocalHost 계약/회귀 인수이며 CLI4 또는 carrier가 합류했다는 의미가 아니다.

## 입력과 보호

root fc22d8c88a6dd1b07d6196f5d1e42d2cb1f3ad5b / Meari98f6a5c4c72ad07a40ac0eae7f7f113926609fae를 유지했다. 원 LocalHost source6 CBF11D50925E7C5FFC18D0CE5EB73711E6705C21A0622F432235055A9916F690은 별도 보존했다. 기존3/신규3의 before6, A7 반환 snapshot1300 대조 delta0 후 비중첩1297·root1977을 동결했다. source6 정확6만 shared snapshot에 현재 mtime으로 합류했다.

최종 시험 fixture 수리 source6 r2는 **B86C28B53A2E085D9043C80F0AA8355BD72BAB4A3DAE3B33E66E20539481DBE1**, integrated6-r2는 **75CA764CC679FC47B9D7CBEC7E343074DE07F2D12C3688B77C6FFA0196BB5EC1**다. r1→r2는 shutdown_tests.rs exact1의 endpoint→실제 pipe_path 변환만 변경했다. 나머지5는 hash 불변이다. carrier4는 합류하지 않았다.

원 근거 디렉터리는 `runtime/verification/local-host-shutdown-r1/`이다. Cargo cwd와 절대 manifest는 shared snapshot/public-p3-parent-r1, target-dir은 기존 project-9dc296c4/native다. 지정 runner EEA9DEE6E8BCE8950E74A297098044EFF1154CB334D7E6FA2C4C622B4005488D에 conservative2/affinity3/BelowNormal/descendant 예산, offline/locked/-j1/testthreads1을 적용했다.

## 실제 결과

| 최종 범위 | 실제 결과 | 원 receipt 경과 |
| --- | --- | --- |
| executor 신규3 | 3pass | executor-final 1047ms/exit0 |
| server/shutdown 신규5 | 5pass | server-r2 32751ms/exit0 |
| 기존 tests/local_host | 9pass | local-host 86871ms/exit0 |
| default all-target strict | 실제 Checking kinekino-player, exit0 | 26319ms |
| native-proposal-window all-target strict | 실제 Checking kinekino-player, exit0 | 43027ms |
| fmt --all -- --check | exit0 | 3003ms |

최종 고유17이며 구성/반복·metadata0selected는 더하지 않는다. 최종 r2의 실제 Compiling 및 정확 시험 이름을 stdout/stderr와 연결했다. 원 executor-corrected3/77021ms0도 보존하지만 최종3에 중복 합산하지 않는다. 해당 호출이 default-members의 다른 lib에 남긴0selected는 pass0이다. 이후 -p kinekino-player와 직접 --lib 필터/--test local_host를 사용했으며 broad test --list를 하지 않았다.

신규 회귀는 accepted 작업 barrier→close→새 request not_dispatched→원 SDK/client/Pending 유지·pipe만 join→release→양 실제 join, 응답 대기 timeout 뒤 accepted 작업 유지, partial header pipe 종료, pipe/SDK worker panic 독립 처리, close/admission 경쟁·poison·SDK execute panic을 확인했다. 순서 경합의 모든 가능한 스케줄을 탐색한 것은 아니다.

## 보존한 실패와 정정

- 원 server는 4pass/1fail, terminal101/1137ms였다. partial-pipe 시험이 제품 endpoint 문자열을 Windows pipe API에 바로 전달하여 NotFound가 발생했다. r2는 기존 LocalEndpoint::parse(...).pipe_path 변환만 적용했다. 원 실패/stdout/stderr/핀을 보존했다. panic fixture의 의도된 panic 출력은 이 실패와 구별한다.
- 최초2회는 Cargo 입력의 운영 실패다. 첫 실행은 잘못된 root cwd/새 snapshot target, 둘째는 snapshot cwd지만 같은 잘못된 target이었다. 첫 자손이 남은 상태에서 둘째가 겹쳤다. 원 runner receipt2는 false로 보존하고 exec31573/87697의 exit1 및 자손 identity별 정리 sidecar를 남겼다. 이를 정상 terminal이나 source6 시험 실패로 바꾸지 않았다. `operational-rejected.md`에 원 시각·한계·중첩을 기록했다.
- runner의 descendant Job association은 CPU/priority 예산이며 자손 종료 보장이 아니다. no-kill-on-close 소스 읽기와 EEA9 바이너리 완전 귀속의 한계를 구별했다. 잘못 생성한 snapshot/target 캐시는 삭제하지 않고 정리 대상으로 남겼다.

## 산출물·종료·보호

실제 integration helper가 실행한 default debug player-cli는29123584bytes/SHA **EF4B0931A9FFC24FC9ECD40542C8318B260AE819CB914D7BB1DE279D6514D6E7**다. 원 cache와 gate/artifact/player-cli.exe의 hash가 같다. release/native GUI 산출물이 아니다.

final-receipts.json은10개다. 8개는 terminal(7개 exit0, 원 fixture 실패1개101)이며 운영 중단2개는 false 그대로다. 정상 실행은 각 원 exec handle의 terminal 회수 뒤 다음 실행으로 진행했다. 운영 실패2의 중첩은 앞 절처럼 별도 기록한다.

기록한 숫자 PID119개에는 CLI93개와 runner/관찰한 하위가 포함된다. initial final-owned 조회에서73700은 새 conhost.exe였다. 원 CLI는 stderr:28의 code0/53ms로 종료했고 전체 local-host 로그 종료 UTC16:26:11보다 새 conhost 생성 KST01:28:35가 늦으며 부모도 새 pwsh였다. 원 CLI의 절대 생성시각/부모는 포착하지 않았다는 한계가 있다. raw·pid-reuse-73700.json을 보존하고 이 새 프로세스를 kill하지 않았다. 최종 별도 CIM은 current candidates[]/owned[]다. 숫자 PID가 재사용되지 않았다고 주장하지 않는다.

final-source6/snapshot/root/A7 differences는 모두[]다. snapshot1297은 기존 Meari405/UI·SDK 비중첩 및 A7의 바이트를 포함해 보호하며 A7 exact7은 별도 재대조도 delta0이다. root index에 stage하지 않았다. root live 코드/모듈 등록·SDK B15·Meari·carrier 소스는 변경하지 않았다.

## 남은 범위

LocalHost Drop이 pipe join에서 신호만 보내는 방식으로 바뀌는 호환 차이는 남는다. [CLI 범위 계획](../../../plans/player/local-host-cli-shutdown-scope.md)과 caller-audit.md의 CLI4/main/audio/fixture owner 보존 배선은 별도다. 기존 stale/drop 시험 통과가 이 새 종료 통합을 대신하지 않는다. Pending을 join 전에 버리면 proof를 잃고, startup ready.send 실패의 accepted0/SDK handle detach도 실제 join proof가 아니다. 이 gate는 동기 SDK 취소 성공·하위 dependency 종료·GUI/viewerH 수명 전체·실제 미디어·추론을 인수하지 않는다.

modoo.today `modootoday/software-qa/code-change-review-verification` 본문/참조의 경로·원 주장·회귀 대응을 적용했다. Rust 소유권/RAII와 기존 SDK worker 재사용을 지켰다. 계획은 [LocalHost gate](../../../plans/player/local-host-shutdown-gate.md), 원 source 인계와 r2 fixture diff는 각각 runtime/coordination/local-host-shutdown-r1 및 -r2다. root 통합·commit은 PM 소유이고 push0이다.
