---
id: evidence:kinekino/20261029114755
title: 공통 SDK parent router 검증
status: active
created: 2026-10-29
updated: 2026-10-29
---
# 공통 SDK parent router 검증

2026-10-20. [조립 계획](../../../plans/player/native-playback-child-assembly.md). 검증 산출물은 `runtime/quality-review/sdk-parent-router-r2/`다. 실제 OS parent/main/미디어 검증은 아니다.

| 단계 | 목표와 직접 관측 | 결과 |
| --- | --- | --- |
| 입력 | 동결 r2 source4, 기존 snapshot 보호551 해시와 before4 원문/absent | overlay 전 일치 |
| 최초 router | router.stdout.log 실제 실행 개수 | 0개/391 filtered, 성공으로 인수하지 않음 |
| 입력 갱신 | byte 해시 유지, exact4 snapshot mtime만 갱신 뒤 같은 명령/filter | 재컴파일 및 실제12개 실행 |
| router r2 | router-fresh.stdout.log | 12 passed, receipt0/43256ms |
| r3 binding | binding-r3.stdout.log | 6 passed, receipt0/14274ms |
| r3 dispatcher | dispatcher-r3.stdout.log | 4 passed, receipt0/1064ms |
| r3 default strict | strict-default-r3.stderr.log | unused import2, receipt101/7168ms 보존 |
| r4 default/native strict | 각 stderr와 receipt | 모두0, 24746/26981ms |
| r4 fmt | fmt-r4 receipt | 0/2687ms |
| 마감 | final-source4/protected-differences/receipts/owned | live/frozen/snapshot4 일치, 보호551 delta0, 8 receipt terminal, launcher/child16 PID 부재 |

## 입력 수정과 근거

r1 source4 pin DDF233C75B1C6E9E12FE40494F32AED8C7E3FC6A38AC5ED1AFAE00866FE4E21D는 보존했다. r1 두 테스트의 Full 전제 오류는 소스 반례이며 실행 실패가 아니다. 실행 중 worker/old callback은 큐에서 이미 소비됐으므로 별도 대기job을 enqueue해야 Full이다. r2는 tests.rs만 고쳐 capacity1 실제Full/거절wake 제거, capacity2 FIFO 수락을 구분하고 accepted ticket을 정리한다. 핀495C153F7BF2BE7371BF039CE04978FFAD5091B502EB3EECF9CD37F83B35A10E.

r2 실제12개는 통과했으나 완료 helper가 반환한 must-use handle을 버리는 경고10개가 있었다. r3는 helper 반환을 unit으로 바꾸고 기존 actual joined Complete 관측 직후 같은 handle을 명시 drop했다. 단언/정책/콜사이트는 불변이다. 핀0F30E5346937A7D847C192FF5326BFDBBD918F451E901BE93CD7135C77BF4236.

r3 strict에서 테스트에서만 사용하는 RetirementFailure/RetirementProgress의 무조건 재수출이 unused import였다. r4는 이2개를 cfg(test) use로 옮겼다. 타입 정의/생산 동작/단언은 불변이다. 최종 frozen4는 `runtime/coordination/sdk-parent-router4-r4/`, pins.json SHA256 FE219612964C28256AFDA5237CFF32FC39538D214928108F4B2A293C3279EFD7. r3/r4는 행위 동등한 경고 수정이므로 router12를 반복하지 않았고 최종 all-target strict가 테스트 문맥도 컴파일했다.

최초0개는 Cargo가 A4 기존 binary를 재사용한 관측이다. 동일 byte의 snapshot exact4 mtime만 갱신한 뒤 컴파일42.56s/12개가 실행되어 입력 freshness 문제를 재현·교정했다. 광범위 clean/filter 완화/byte 조작은 하지 않았다. 기존 A4 바이너리는 기존 `test-binary.json` 기록의 identity만 보존한다(SHA A03ACBD376B7FF00DCCCA958DB0AA83CE9DC96B0319A97B6AB9F67B9B7074D01/91880448bytes). 재채집 시 linker lock이 발생했으므로 옛 binary bytes를 별도 보존했다고 주장하지 않는다.

링크 종료 뒤 채집한 실제 r3 libtest binary 경로는 `%USERPROFILE%/AppData/Local/CodexBuildCache/project-9dc296c4/native/debug/deps/kinekino_player-b7490a190745947c.exe`, SHA256 0CA935FC834DAE0BB6010DB59DA5D3ED17D26848B344247FF18C2C46EA4A0976, 92240384bytes. binding6/dispatcher4가 이 binary에서 실행됐다. r2 실행 binary와 r4 strict 산출물을 이 identity로 대체하지 않는다.

## 실행과 적용 스킬

snapshot `public-p3-parent-r1` 절대 cwd/manifest, 공유 native cache, canonical resource runner SHA692C22751FAC7F9999F2D833571DAB6605AFD58E66711370494E62DD8A62C463, offline/locked/j1/testthreads1/affinity3/BelowNormal/descendant job으로 직렬 실행했다. 명령 계획은 r2/gate-plan.md, 실제 명령과 원 handle은 채팅 tool trace에 보존된다. SDKwire/UI thin6/등록A4/기존 Meari·자산을 보호했고 미검증 live UI/Meari 혼합은 없다. stage/commit/push는 하지 않았다.

modoo.today에서 `modootoday-free/ai-quality-plus/agent-run-postcondition-and-replay-check` 본문과 `references/worked-example.md`(SHA256 80c9fe8a8a79dd8e08d84b36483d7742f273e55fb2769d2e6156a1111fdd5c11)를 읽었다. 종료코드만 믿지 않고 실행 개수·소스/보호 해시·PID의 직접 상태를 대조하는 절차를 적용했다. 스킬 이름만으로 실제 입력을 알 수 없으므로 revision별 source/pin·명령·receipt·로그를 연결했다. 모델 호출은 없어 prompt/model trace는 해당하지 않는다. replay는 동일 동결 입력에서 mtime 한 조건만 변경했고 fake engine을 사용해 외부 재생 부작용을 만들지 않았다. OS 스케줄·경과시간/PID는 재현 가능한 고정값으로 취급하지 않는다. 로컬 lint-and-validate 절차도 Rust/Cargo 규칙으로 적용했다.

## 한계

원 cleanup task 소유/actual join, admission revocation, 세대별 SDK port, FIFO wake 분리의 fake 회귀 증거다. NativeChild shared owner 주입·실제 surface Destroy·main 조립·OS parent/미디어 성공은 미구현/미검증이다. non-test parent_router 모듈 한 곳 expect(dead_code) 유예는 trusted assembly 연결 때 제거·재검토한다. 모델/DPI/VM 보류는 유지한다.
